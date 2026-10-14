"""demo/runs/onair/2026-10-13/lines_at.py: lines lean_check.py counts, at a commit, read from git's objects with
the rules of scripts/lean_check.py as of HEAD (code_lines)."""
import re, subprocess, sys
CODE = (".rs", ".js", ".mjs", ".ts", ".py", ".sh")
def count(c, base):
    names = subprocess.run(["git", "ls-tree", "-r", "--name-only", c, "--", base], capture_output=True, text=True, check=True).stdout.split("\n")
    n = 0
    for rel in filter(None, names):
        if (not rel.endswith(CODE) or "/tests/" in f"/{rel}" or "/node_modules/" in f"/{rel}"
                or rel.startswith(("apps/", "target/", "demo/runs/"))
                or re.search(r"(^|/)tests?(_\w+)?\.rs$|_tests?\.rs$|\.test\.m?js$", rel)):
            continue
        text = subprocess.run(["git", "show", f"{c}:{rel}"], capture_output=True, check=True).stdout.decode(errors="replace")
        n += sum(1 for l in text.splitlines() if l.strip() and not l.strip().startswith(("//", "#", "/*", "*")))
    return n
for c in sys.argv[1:]:
    s = subprocess.run(["git", "log", "-1", "--format=%h %s", c], capture_output=True, text=True).stdout.strip()[:90]
    print(f"{count(c, 'crates/svrngs-core'):5} {count(c, '.'):6}  {s}")
