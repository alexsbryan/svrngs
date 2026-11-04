# Per-file lines (count.py's) between two counts: what grew since the last survey.
import sys
def rd(p):
    d = {}
    for l in open(p):
        a = l.split()
        if len(a) == 2: d[a[1]] = int(a[0])
    return d
o, n = rd(sys.argv[1]), rd(sys.argv[2])
for d, k, a, b in sorted(((n.get(k, 0) - o.get(k, 0), k, o.get(k, 0), n.get(k, 0)) for k in set(o) | set(n)), reverse=True):
    if d: print(f"{d:+5} {a:5} {b:5} {k}")
