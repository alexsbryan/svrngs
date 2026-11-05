"""Recorded-run driver; measurements and scratch experiments, never a product gate."""
import importlib.util
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]


def run(name, args, cwd=ROOT):
    result = subprocess.run(args, cwd=cwd, text=True, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT)
    output = "\n".join(line.rstrip() for line in result.stdout.splitlines())
    text = f"$ {' '.join(map(str, args))}\n{output}\nexit {result.returncode}\n"
    (HERE / f"{name}.txt").write_text(text)
    print(text, end="")
    return result.returncode


def combine(name, parts):
    text = "".join((HERE / f"{part}.txt").read_text() for part in parts)
    (HERE / f"{name}.txt").write_text(text)
    print(f"wrote {name}.txt from {', '.join(parts)}")


if sys.argv[1] == "survey":
    run("lean-before", ["python3", "scripts/lean_check.py"])
    spec = importlib.util.spec_from_file_location("lean", ROOT / "scripts/lean_check.py")
    lean = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(lean)
    lean.ROOT = ROOT
    paths = [p for p in lean.files(ROOT) if p.is_file()]
    selected = [p for p in paths if p.suffix == ".rs" and "/src/" in str(p)
                and (p.parent == ROOT / "crates/svrngs-node/src"
                     or "/door/" in str(p) or "/oidc/" in str(p)
                     or p.name in ("cli.rs", "share.rs"))
                and not lean.TEST_FILE.search(str(p))]
    run("windows", ["python3", "demo/runs/economize/2026-10-25-give/scripts/windows.py",
                    "4", *[str(p.relative_to(ROOT)) for p in selected]])
    sites = {}
    for p in selected:
        sites[str(p.relative_to(ROOT))] = sum(bool(s := line.strip()) and
            not s.startswith(("//", "#", "/*", "*")) for line in p.read_text().splitlines())
    give = [p for p in paths if p.suffix == ".rs" and
            p.is_relative_to(ROOT / "apps/give/src") and not lean.TEST_FILE.search(str(p))]
    sites["Give src total (same line rule; inline cfg bodies included)"] = sum(
        sum(bool(s := line.strip()) and not s.startswith(("//", "#", "/*", "*"))
            for line in p.read_text().splitlines()) for p in give)
    (HERE / "counts.json").write_text(json.dumps(sites, indent=2) + "\n")
    matches = []
    for p in paths:
        rel = p.relative_to(ROOT)
        if rel.parts[0] in ("demo", "ralph") or p.suffix not in (".rs", ".sh", ".md", ".mjs", ".js"):
            continue
        for n, line in enumerate(p.read_text(errors="replace").splitlines(), 1):
            if re.search(r'\bshare\b|lend::(?:declare|limit|withdraw)', line):
                if "share" in line and not re.search(r'"share"|\bshare\s+(?:\$|[0-9<])|\.share\(|\bshare\(.*\)', line):
                    continue
                matches.append(f"{rel}:{n}: {line.strip()}")
    (HERE / "callers.txt").write_text("\n".join(matches) + "\n")
elif sys.argv[1] == "prototype":
    with tempfile.TemporaryDirectory(prefix="lend-scope-", dir="/var/folders/qf/4ntssn_10d598hw_5rjhx3740000gp/T/opencode") as scratch:
        scratch = Path(scratch)
        shutil.copy(HERE / "prototype.toml", scratch / "Cargo.toml")
        shutil.copy(HERE / "prototype.rs", scratch / "prototype.rs")
        for plant in ("body", "splice", "raw", "udp", "none"):
            code = run(f"prototype-{plant}", ["cargo", "run", "--offline", "--quiet",
                "--manifest-path", str(scratch / "Cargo.toml"), "--", plant])
            if code != (0 if plant == "none" else 101):
                sys.exit(f"unexpected prototype exit for {plant}: {code}")
elif sys.argv[1] == "fast":
    commands = [
        ("lean-after", ["python3", "scripts/lean_check.py"]),
        ("principles", ["python3", "scripts/principles_check.py"]),
        ("trust", ["python3", "scripts/trust_check.py"]),
        ("advisories", ["python3", "scripts/advisories.py"]),
        ("vocab", ["python3", "scripts/vocab_check.py"]),
        ("rustfmt", ["cargo", "fmt", "--all", "--check"]),
        ("lint", ["web/node_modules/.bin/oxlint"]),
        ("format", ["web/node_modules/.bin/oxfmt", "--check", "**/*.{js,mjs,cjs}"]),
        ("types", ["node", "scripts/types.mjs"]),
        ("typed-lint", ["node", "scripts/typed-lint.mjs"]),
        ("page-laws", ["node", "--test", *[str(p.relative_to(ROOT)) for p in sorted((ROOT / "web/tests").rglob("*.test.mjs"))]]),
        ("workspace", ["cargo", "check", "--workspace", "--offline"]),
        ("unchanged-product", ["git", "diff", "--exit-code", "7ad807f1", "--", "crates", "web", "resolver", "scripts", "apps"]),
    ]
    for name, args in commands:
        if run(name, args):
            sys.exit(1)
elif sys.argv[1] == "close":
    run("unchanged-production", ["git", "diff", "--exit-code", "7ad807f1", "--",
        "crates", "web", "resolver", "scripts", "apps",
        ":!crates/**/tests/**", ":!web/tests/**"])
    run("prior-full-gate", ["git", "show", "da503bac", "--", "ralph/REVIEW_FINDINGS.md"])
elif sys.argv[1] == "resume":
    # Resumed after onboarding-grouped-home-reconcile landed (HEAD b484b2b3).
    # Rechecks the current landed baseline and the lending seams the survey
    # names; the survey/prototype evidence is retained, not recreated.
    commands = [
        ("lean-after", ["python3", "scripts/lean_check.py"]),
        ("principles", ["python3", "scripts/principles_check.py"]),
        ("trust", ["python3", "scripts/trust_check.py"]),
        ("advisories", ["python3", "scripts/advisories.py"]),
        ("vocab", ["python3", "scripts/vocab_check.py"]),
        ("rustfmt", ["cargo", "fmt", "--all", "--check"]),
        ("lint", ["web/node_modules/.bin/oxlint"]),
        ("format", ["web/node_modules/.bin/oxfmt", "--check", "**/*.{js,mjs,cjs}"]),
        ("types", ["node", "scripts/types.mjs"]),
        ("typed-lint", ["node", "scripts/typed-lint.mjs"]),
        ("page-laws", ["node", "--test", *[str(p.relative_to(ROOT))
            for p in sorted((ROOT / "web/tests").rglob("*.test.mjs"))]]),
        ("workspace", ["cargo", "check", "--workspace", "--offline"]),
        # The landed grouped-home handback changed product; this is evidence of
        # the seam files being otherwise untouched since the survey base, not a
        # claim of an unchanged product tree.
        ("seam-diff", ["git", "diff", "--stat", "7ad807f1..HEAD", "--",
            "crates/svrngs-node/src/lend.rs", "crates/svrngs-node/src/door.rs",
            "crates/svrngs-node/src/door/", "crates/svrngs-node/src/home.rs",
            "crates/svrngs-node/src/version.rs", "crates/svrngs-node/src/node.rs",
            "crates/svrngs-node/src/oidc/", "crates/svrngs-node/src/origin.rs",
            "crates/svrngs-node/src/surface.rs", "crates/svrngs/src/cli.rs",
            "apps/give/src/", "crates/svrngs-runtime/src/register.rs"]),
    ]
    for name, args in commands:
        if run(name, args):
            sys.exit(1)
    combine("resume-fast", [name for name, _ in commands])
elif sys.argv[1] == "combine":
    combine(sys.argv[2], sys.argv[3:])
else:
    sys.exit("expected survey, prototype, fast, resume, combine or close")
