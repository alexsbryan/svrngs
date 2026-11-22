#!/usr/bin/env python3
"""List every file forked from commonwealth-ai or Corrosion beside its original (DESIGN §4).

A port carries `ported from <commonwealth-ai|superfly/corrosion> <path>@<commit>`. For each,
this prints where the port lives, what it came from, and the original's state:
changed upstream since the port (a fix there may need porting here), unchanged,
or retired (deleted upstream — the pair is closed); and the port's own state:
how many svrngs commits changed it since the commit that added its line, so a
diff against upstream is where to look (DESIGN §4). It is a listing, not a gate:
it exits 0 whatever it finds, and 2 only if commonwealth-ai cannot be read.
Upstreams: $COMMONWEALTH_AI or ~/dev/commonwealth-ai; $CORROSION or ~/dev/corrosion.
"""
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UPSTREAMS = {
    "commonwealth-ai": Path(os.environ.get("COMMONWEALTH_AI", Path.home() / "dev" / "commonwealth-ai")),
    "superfly/corrosion": Path(os.environ.get("CORROSION", Path.home() / "dev" / "corrosion")),
}
UPSTREAM = UPSTREAMS["commonwealth-ai"]
MARK = re.compile(r"ported from (commonwealth-ai|superfly/corrosion) (\S+)@([0-9a-f]{7,40})")
SKIP = ("docs/", "target/", "node_modules/", ".git/", "ralph/", "scripts/ports.py")


def git(*args, cwd=UPSTREAM):
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True)


def here(rel, mark):
    """How many svrngs commits changed `rel` since the one that added `mark`."""
    added = git("log", "--reverse", "--format=%H", "-S", mark, "--", rel, cwd=ROOT).stdout.split()
    if not added:
        return "here: not committed"
    n = int(git("rev-list", "--count", f"{added[0]}..HEAD", "--", rel, cwd=ROOT).stdout.strip() or 0)
    return f"here: changed in {n} commit(s)" if n else "here: as ported"


def main():
    if not (UPSTREAM / ".git").exists():
        print(f"cannot read upstream at {UPSTREAM}", file=sys.stderr)
        return 2
    found = 0
    for f in sorted(ROOT.rglob("*")):
        rel = f.relative_to(ROOT).as_posix()
        if not f.is_file() or rel.startswith(SKIP):
            continue
        for n, line in enumerate(f.read_text(errors="replace").splitlines(), 1):
            m = MARK.search(line)
            if not m:
                continue
            found += 1
            repo, path, commit = m.groups()
            upstream = UPSTREAMS[repo]
            if not (upstream / ".git").exists():
                state = f"{repo} unread at {upstream}"
            elif not (upstream / path).exists():
                state = "retired upstream"
            else:
                since = git("log", "--oneline", f"{commit}..HEAD", "--", path, cwd=upstream).stdout.splitlines()
                state = f"changed upstream in {len(since)} commit(s)" if since else "unchanged"
            print(f"{rel}:{n}  <-  {repo} {path}@{commit}  {state}; {here(rel, m.group(0))}")
    print(f"{found} port(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
