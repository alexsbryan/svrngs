#!/usr/bin/env python3
"""svrngs stays standalone and lean (docs/DESIGN.md §4, §5). Exit 0 holds, 1 broken.

1. Cargo.lock names no commonwealth-ai or sovereign crate.
2. svrngs-core declares no I/O dependency, and is no_std: the compiler refuses
   files, sockets, clocks, threads and HashMap there; no core source outside
   its tests links std back in.
3. RUST_ARCH, where a program can decide it: `anyhow` only in the `svrngs`
   command and the app (typed errors everywhere else); no `Arc<Mutex<` or
   `Rc<RefCell<` in a crate (ownership is a tree); no `async` in svrngs-core
   (the core is synchronous).
4. Lines of code — not blank, not a comment, not a test — stay under budget:
   svrngs-core under 3,150, the whole repository under 16,764 — scripts
   included. The budget counts the product: apps/, installed packages (any
   node_modules/) and demo/runs/ apart. A recorded run's driver is what ran,
   kept byte for byte so its data reads against it (ralph/REVIEW_FINDINGS.md
   #61), and no check runs one; code elsewhere under demo/ counts.
"""
import re
import subprocess
import sys
from pathlib import Path

TEST_FILE = re.compile(r"(^|/)tests?(_\w+)?\.rs$|_tests?\.rs$")
ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent
FOREIGN = re.compile(r'^name = "(commonwealth-[\w-]+|sovereign-[\w-]+|oicp-[\w-]+|kernel-types|oplog(-types)?|corpus-engine|host-kit|mesh-[\w-]+)"$', re.M)
IO_DEPS = {"tokio", "async-std", "smol", "iroh", "reqwest", "hyper", "axum", "tower",
           "rusqlite", "sqlx", "notify", "mio"}
# Each cap one past a count: svrngs-core's where events-economize's cut landed,
# the repository's where events-wait's held dial to a lender landed (2026-10-14),
# so no room is left that the operator did not give. v1ui-economize found nothing
# to cut and left both (demo/runs/economize/2026-10-14-v1ui/SURVEY.md). A raise is
# asked only after an economizing pass (AGENTS.md "Done"; DESIGN §5). After that
# pass the operator gave v1ui +650 over 16,315, core unchanged (2026-10-14), each
# row raising only as far as it needs: the repository's cap is one past where
# v1ui-shelf (b), the carried route answering an app's changes of mind, landed.
BUDGET = {"crates/svrngs-core": 3150, ".": 16764}
CODE = {".rs", ".js", ".mjs", ".ts", ".py", ".sh"}


def files(base):
    """What git calls source, tracked or new and not ignored, so a build's
    output never counts wherever it lands (#74: web/target/ was counted). A
    tree outside git, such as an exported revision, is walked whole, and says so."""
    r = subprocess.run(["git", "-C", str(ROOT), "ls-files", "-co", "--exclude-standard", "-z", "--",
                        base.relative_to(ROOT).as_posix()], capture_output=True)
    if r.returncode != 0:
        print(f"lean_check: {ROOT} is not a git tree, so every file counts, build output included",
              file=sys.stderr)
        return base.rglob("*")
    return (ROOT / p for p in r.stdout.decode().split("\0") if p)


def code_lines(base):
    n = 0
    for f in files(base):
        rel = f.relative_to(ROOT).as_posix()
        if (f.suffix not in CODE or not f.is_file() or "/tests/" in f"/{rel}"
                or "/node_modules/" in f"/{rel}"
                or rel.startswith(("apps/", "target/", "demo/runs/"))
                or re.search(r"(^|/)tests?(_\w+)?\.rs$|_tests?\.rs$|\.test\.m?js$", rel)):
            continue
        for line in f.read_text(errors="replace").splitlines():
            s = line.strip()
            if s and not s.startswith(("//", "#", "/*", "*")):
                n += 1
    return n


def main():
    broken = []
    lock = ROOT / "Cargo.lock"
    if lock.exists():
        for m in FOREIGN.finditer(lock.read_text()):
            broken.append(f"Cargo.lock names {m.group(1)}: svrngs links nothing from commonwealth-ai or sovereign")
    for crate in ("svrngs-core",):
        manifest = ROOT / "crates" / crate / "Cargo.toml"
        if manifest.exists():
            deps = re.findall(r'^([\w-]+)\s*=', manifest.read_text().split("[dependencies]", 1)[-1].split("\n[", 1)[0], re.M)
            for d in deps:
                if d.split("-")[0] in IO_DEPS or d in IO_DEPS:
                    broken.append(f"{crate} depends on {d}: the core takes no I/O dependency")
    for manifest in sorted((ROOT / "crates").glob("*/Cargo.toml")):
        crate = manifest.parent.name
        if crate != "svrngs" and re.search(r'^anyhow\s*=', manifest.read_text(), re.M):
            broken.append(f"{crate} depends on anyhow: crates carry typed errors; anyhow is the command's and the app's")
    for f in sorted((ROOT / "crates").glob("*/src/**/*.rs")):
        rel = f.relative_to(ROOT).as_posix()
        text = f.read_text(errors="replace")
        if re.search(r"Arc<\s*Mutex<|Rc<\s*RefCell<", text):
            broken.append(f"{rel} shares mutable state: one owner and messages instead (RUST_ARCH §3)")
        if (rel.startswith("crates/svrngs-core/") and not TEST_FILE.search(rel)
                and re.search(r"^\s*extern\s+crate\s+std\b", text, re.M)):
            broken.append(f"{rel} links std: the core is no_std, so I/O, clocks and threads stay outside it")
        if rel.startswith("crates/svrngs-core/") and re.search(r"\basync\b|\.await\b", text):
            broken.append(f"{rel} is async: the core stays synchronous (RUST_ARCH §2, §4)")
    core_lib = ROOT / "crates" / "svrngs-core" / "src" / "lib.rs"
    if core_lib.exists() and "#![cfg_attr(not(test), no_std)]" not in core_lib.read_text():
        broken.append("crates/svrngs-core/src/lib.rs lost #![cfg_attr(not(test), no_std)]: the core is no_std")
    for path, cap in BUDGET.items():
        base = ROOT / path
        n = code_lines(base) if base.exists() else 0
        print(f"{path:<20} {n:>6} / {cap} lines")
        if n >= cap:
            broken.append(f"{path} has {n} lines of code, budget {cap}: raised only by the operator, with the reason in the commit")
    for b in broken:
        print(f"BROKEN  {b}")
    return 1 if broken else 0


if __name__ == "__main__":
    sys.exit(main())
