# capability-economize's line-window pass: each run of W counted lines
# (count.py's rule, whitespace collapsed) said in two places of the files
# named, so a fold the clone report's chunks miss is seen. Read-only.
import re, sys
from collections import defaultdict
W = int(sys.argv[1]); files = sys.argv[2:]
def code(l):
    s = l.strip(); return bool(s) and not s.startswith(("//", "#", "/*", "*"))
seen = defaultdict(list)
for f in files:
    lines = [(i + 1, re.sub(r"\s+", " ", l.strip())) for i, l in enumerate(open(f)) if code(l)]
    for k in range(len(lines) - W + 1):
        win = tuple(t for _, t in lines[k:k + W])
        if sum(len(t) for t in win) < 12 * W: continue
        seen[win].append(f"{f}:{lines[k][0]}")
for win, at in seen.items():
    if len(at) > 1: print(" ".join(at), "|", " ⏎ ".join(win)[:160])
