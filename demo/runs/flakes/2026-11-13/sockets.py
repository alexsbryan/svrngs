#!/usr/bin/env python3
"""Every socket a process tree opens to an address, sampled: bdd3bd51's probe
(`lsof -i` of what a test started, sampled while it runs), over every
descendant of a root process, until the root ends.

    sockets.py <root pid> <out.tsv> [interval s]

Each connected socket (`lsof -nP -i`, a `->` in its name) is written once per
process and remote, with the process's command line, and its remote judged:
`loopback` (127/8, ::1), `host` (an address of this computer's own
interfaces, `ifconfig`), or `outside`. A process running the scenes' binary as
svrngs (`svrngs_here` in its arguments) is named `svrngs_here`. At the end, a count
by kind and class, and every `outside` row, are printed. UDP sent without a
connect is not seen by `lsof`, as in bdd3bd51's probe.
"""
import ipaddress
import re
import subprocess
import sys
import time

root, out = int(sys.argv[1]), sys.argv[2]
every = float(sys.argv[3]) if len(sys.argv) > 3 else 2.0
own = set(re.findall(r"inet6? ([0-9a-f.:]+)", subprocess.run(["ifconfig"], capture_output=True, text=True).stdout))


def tree():
    rows = subprocess.run(["ps", "-A", "-o", "pid=,ppid="], capture_output=True, text=True).stdout.split()
    kids = {}
    for pid, ppid in zip(rows[::2], rows[1::2]):
        kids.setdefault(int(ppid), []).append(int(pid))
    seen, todo = [], [root]
    while todo:
        p = todo.pop()
        seen.append(p)
        todo += kids.get(p, [])
    return seen


def judged(addr):
    host = addr.rsplit(":", 1)[0].strip("[]").split("%")[0]
    try:
        ip = ipaddress.ip_address(host)
    except ValueError:
        return "outside"
    if ip.is_loopback:
        return "loopback"
    return "host" if host in own else "outside"


names, written = {}, set()
with open(out, "w") as f:
    f.write("time\tpid\tkind\tproto\tlocal\tremote\tclass\tcommand\n")
    while subprocess.run(["kill", "-0", str(root)], capture_output=True).returncode == 0:
        pids = tree()
        for p in pids:
            if p not in names:
                args = subprocess.run(["ps", "-o", "args=", "-p", str(p)], capture_output=True, text=True).stdout.strip()
                names[p] = ("svrngs_here" if "svrngs_here" in args else (args.split() or ["?"])[0].rsplit("/", 1)[-1], args)
        listed = subprocess.run(["lsof", "-nP", "-a", "-i", "-p", ",".join(map(str, pids))], capture_output=True, text=True).stdout
        for line in listed.splitlines()[1:]:
            cols = line.split()
            name = next((c for c in cols if "->" in c), None)
            if not name:
                continue
            pid, proto = int(cols[1]), cols[7]
            local, remote = name.split("->")
            if (pid, remote) in written:
                continue
            written.add((pid, remote))
            kind, args = names.get(pid, ("?", ""))
            f.write(f"{time.strftime('%H:%M:%S')}\t{pid}\t{kind}\t{proto}\t{local}\t{remote}\t{judged(remote)}\t{args[:200]}\n")
            f.flush()
        time.sleep(every)

rows = [l.rstrip("\n").split("\t") for l in open(out)][1:]
counts = {}
for r in rows:
    counts[(r[2], r[6])] = counts.get((r[2], r[6]), 0) + 1
print(f"sockets: {len(rows)} (process, remote) pairs over {len(names)} processes")
for (kind, cls), n in sorted(counts.items()):
    print(f"  {kind}\t{cls}\t{n}")
for r in rows:
    if r[6] == "outside":
        print("  outside: " + "\t".join(r[:7]))
