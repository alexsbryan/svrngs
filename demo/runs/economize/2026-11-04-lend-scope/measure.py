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
    text = f"$ {' '.join(map(str, args))}\n{result.stdout}\nexit {result.returncode}\n"
    (HERE / f"{name}.txt").write_text(text)
    print(text, end="")
    return result.returncode


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
        for plant in ("body", "splice", "raw", "none"):
            code = run(f"prototype-{plant}", ["cargo", "run", "--offline", "--quiet",
                "--manifest-path", str(scratch / "Cargo.toml"), "--", plant])
            if code != (0 if plant == "none" else 101):
                sys.exit(f"unexpected prototype exit for {plant}: {code}")
else:
    sys.exit("expected survey or prototype")
