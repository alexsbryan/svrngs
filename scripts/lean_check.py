#!/usr/bin/env python3
"""svrngs stays standalone and lean (docs/DESIGN.md D2a, D2b). Exit 0 holds, 1 broken.

1. Cargo.lock names no commonwealth-ai or sovereign crate.
2. svrngs-core and svrngs-engine declare no I/O dependency.
3. RUST_ARCH, where a program can decide it: `anyhow` only in the `svrngs`
   command and the app (typed errors everywhere else); no `Arc<Mutex<` or
   `Rc<RefCell<` in a crate (ownership is a tree); no `async` in svrngs-core or
   svrngs-engine (the core is synchronous).
4. Lines of code — not blank, not a comment, not a test — stay under budget:
   svrngs-core under 2,500, the whole repository under 12,000 — scripts included,
   apps/ bundles apart.
"""
import re
import sys
from pathlib import Path

ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent
FOREIGN = re.compile(r'^name = "(commonwealth-[\w-]+|sovereign-[\w-]+|oicp-[\w-]+|kernel-types|oplog(-types)?|corpus-engine|host-kit|mesh-[\w-]+)"$', re.M)
IO_DEPS = {"tokio", "async-std", "smol", "iroh", "reqwest", "hyper", "axum", "tower",
           "rusqlite", "sqlx", "notify", "mio"}
BUDGET = {"crates/svrngs-core": 2500, ".": 12000}
CODE = {".rs", ".js", ".mjs", ".ts", ".py", ".sh"}


def code_lines(base):
    n = 0
    for f in base.rglob("*"):
        rel = f.relative_to(ROOT).as_posix()
        if (f.suffix not in CODE or not f.is_file() or "/tests/" in f"/{rel}"
                or rel.startswith(("apps/", "target/", "node_modules/"))
                or re.search(r"(^|/)tests?\.rs$|_tests?\.rs$|\.test\.m?js$", rel)):
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
    for crate in ("svrngs-core", "svrngs-engine"):
        manifest = ROOT / "crates" / crate / "Cargo.toml"
        if manifest.exists():
            deps = re.findall(r'^([\w-]+)\s*=', manifest.read_text().split("[dependencies]", 1)[-1].split("\n[", 1)[0], re.M)
            for d in deps:
                if d.split("-")[0] in IO_DEPS or d in IO_DEPS:
                    broken.append(f"{crate} depends on {d}: the core and the engine take no I/O dependency")
    for manifest in sorted((ROOT / "crates").glob("*/Cargo.toml")):
        crate = manifest.parent.name
        if crate != "svrngs" and re.search(r'^anyhow\s*=', manifest.read_text(), re.M):
            broken.append(f"{crate} depends on anyhow: crates carry typed errors; anyhow is the command's and the app's")
    for f in sorted((ROOT / "crates").glob("*/src/**/*.rs")):
        rel = f.relative_to(ROOT).as_posix()
        text = f.read_text(errors="replace")
        if re.search(r"Arc<\s*Mutex<|Rc<\s*RefCell<", text):
            broken.append(f"{rel} shares mutable state: one owner and messages instead (RUST_ARCH §3)")
        if rel.startswith(("crates/svrngs-core/", "crates/svrngs-engine/")) and re.search(r"\basync\b|\.await\b", text):
            broken.append(f"{rel} is async: the core and the engine stay synchronous (RUST_ARCH §2, §4)")
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
