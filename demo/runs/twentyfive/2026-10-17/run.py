#!/usr/bin/env python3
"""The 25-computer run: one group, 25 svrngs computers on two machines (or
all on the Linux one), joining by one link and each lending a greeter, as
the room would on the evening ("Go!"). Run from the Mac; the Linux
machine is driven over SSH. Every computer is a process with a HOME of its
own (fleet.py), on the V1.1 release binaries in bin/, as install.sh
fetches them.

    python3 run.py --mode burst   --layout split --run-dir runs/<date>-split-burst
    python3 run.py --mode stagger --layout linux --run-dir runs/<date>-linux-stagger

Layouts: `split`, the founder and 11 on the Mac, 13 on Linux; `linux`, all
25 on Linux, founder included, with the wall seen from the Mac through
`ssh -L`. `--mac N --linux M --founder mac|linux` overrides either.
Modes: `burst`, every join started within 10 s; `stagger`, one every 5 s.

Writes <run-dir>/data/: summary.json (every reading, every time on the
Mac's clock), logs-<machine>.tgz, then runs judge.py over it. Refuses to
put load on the Mac while a ralph loop runs there unless --mac-load-ok.
"""
import argparse
import hashlib
import json
import platform
import shlex
import subprocess
import sys
import threading
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
NAMES = ["Priya", "Aiko", "Bo", "Cy", "Dev", "Femi", "Gus", "Hana", "Ife", "Jun", "Kit", "Lena", "Luz",
         "Mara", "Max", "Nell", "Noor", "Oren", "Pia", "Ravi", "Rosa", "Sam", "Saul", "Theo", "Wren"]
GROUP = "House"
N0 = ["use1-1", "usw1-1", "euc1-1", "aps1-1"]  # iroh-1.0.2 src/defaults.rs:27-33
LEAD = 8          # s from planning to the first join: the keepers' SSH start fits in it
SPREAD = 10       # s: burst, every join started within it
EVERY = 5         # s: stagger, one join each
AFTER_LAST = 240  # s: the wall's ceiling past the last planned join
LINGER = 20       # s the wall keeps reading once it first reads all settled
MATRIX_WITHIN = 180  # s: every computer listing every lend, or could-not-judge
FOUND_WITHIN = 90    # s: founded.json written, or never-ran


class Machine:
    def __init__(self, name, host, kit, w, binary):
        self.name, self.host, self.kit, self.w, self.bin = name, host, kit, w, binary
        self.offset, self.rtt = 0.0, 0.0  # this machine's clock minus the Mac's

    def sh(self, cmd, timeout=120, binary=False):
        """`cmd`, a shell line, in the kit dir, here or over SSH."""
        line = f"cd {shlex.quote(str(self.kit))} && {cmd}"
        argv = ["sh", "-c", line] if not self.host else ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", self.host, line]
        r = subprocess.run(argv, capture_output=True, timeout=timeout, text=not binary)
        return r

    def fleet(self, *args, timeout=120):
        r = self.sh("python3 fleet.py " + shlex.join([str(a) for a in args]), timeout=timeout)
        if r.returncode != 0:
            raise RuntimeError(f"{self.name}: fleet.py {args[0]} exited {r.returncode}: {r.stderr[-600:]}")
        return json.loads(r.stdout.strip().splitlines()[-1])

    def keeper(self, role, *args):
        """A fleet.py keeper started detached: it outlives this SSH session."""
        # Braced, so `&` backgrounds the keeper alone: a bare `a && b &` puts
        # the whole list in a subshell holding SSH's stdout, and SSH waits
        # for it (the first linux-burst run hung here 30 s and died).
        cmd = (f"mkdir -p {shlex.quote(str(self.w))} && {{ nohup python3 fleet.py {role} "
               + shlex.join([str(a) for a in args])
               + f" > {shlex.quote(str(self.w))}/keeper-{role}.out 2>&1 < /dev/null & }}")
        r = self.sh(cmd, timeout=30)
        if r.returncode != 0:
            raise RuntimeError(f"{self.name}: keeper {role} did not start: {r.stderr[-600:]}")

    def read(self, rel, binary=False):
        r = self.sh(f"cat {shlex.quote(str(self.w))}/{rel}", timeout=60, binary=binary)
        return r.stdout if r.returncode == 0 else None


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def released():
    """bin/SHA256SUMS, the release's list as fetched beside the tarballs."""
    sums = {}
    for line in (HERE / "bin" / "SHA256SUMS").read_text().splitlines():
        h, f = line.split()
        sums[f.lstrip("*")] = h
    return sums


def mac_machine():
    arch = {"arm64": "aarch64", "x86_64": "x86_64"}[platform.machine()]
    return Machine("mac", None, HERE, HERE / "w-mac", HERE / "bin" / f"darwin-{arch}" / "svrngs")


def linux_machine(host, rdir):
    probe = subprocess.run(["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", host, "echo $HOME; uname -m"],
                           capture_output=True, text=True, timeout=30)
    if probe.returncode != 0:
        return None, f"ssh {host} refused: {probe.stderr.strip()[-300:]}"
    rhome, arch = probe.stdout.split()
    kit = Path(rhome) / rdir
    m = Machine("linux", host, kit, kit / "w", kit / "svrngs")
    m.arch = {"x86_64": "x86_64", "aarch64": "aarch64", "arm64": "aarch64"}[arch]
    return m, None


def put_kit(m):
    """fleet.py, greeter.py and the release binary for its arch, checked
    against the release's list once there."""
    local_bin = HERE / "bin" / f"linux-{m.arch}" / "svrngs"
    subprocess.run(["ssh", "-o", "BatchMode=yes", m.host, f"mkdir -p {shlex.quote(str(m.kit))}"], check=True, timeout=30)
    subprocess.run(["scp", "-q", str(HERE / "fleet.py"), str(HERE / "greeter.py"), str(local_bin),
                    f"{m.host}:{m.kit}/"], check=True, timeout=300)
    got = m.sh("sha256sum svrngs").stdout.split()[0]
    return {"binary": f"linux-{m.arch}", "sha256": got, "same_as_fetched": got == sha256(local_bin)}


def clock(m, tries=7):
    """This machine's clock minus the Mac's, from the reading with the
    least round trip; its error is half that trip."""
    best = None
    for _ in range(tries):
        t0 = time.time()
        r = m.sh("python3 fleet.py clock", timeout=30)
        t1 = time.time()
        there = json.loads(r.stdout)["t"]
        if best is None or t1 - t0 < best[1]:
            best = (there - (t0 + t1) / 2, t1 - t0)
    m.offset, m.rtt = best
    return {"offset_s": round(best[0], 4), "rtt_s": round(best[1], 4)}


def n0_answers(m):
    """Each of n0's relays' /ping from this machine, as relay-answers.sh
    asks it: 200 and empty."""
    said = {}
    for r in N0:
        res = m.sh(f"curl -fsS --max-time 10 https://{r}.relay.n0.iroh.link/ping -o /dev/null -w '%{{http_code}}'", timeout=20)
        said[r] = res.stdout.strip() if res.returncode == 0 else f"no ({res.returncode})"
    return said


def ralph_running():
    r = subprocess.run(["pgrep", "-f", "ralph.py run"], capture_output=True, text=True)
    return r.stdout.split()


def shift(lines, m):
    """Each line's times onto the Mac's clock."""
    out = []
    for line in lines:
        for k in ("t", "t0", "t1", "at", "settled_at", "ended"):
            if isinstance(line.get(k), (int, float)):
                line[k] = round(line[k] - m.offset, 3)
        out.append(line)
    return out


def jsonl(text):
    return [json.loads(x) for x in (text or "").splitlines() if x.strip()]


def plan_people(counts, founder_on, mode, t0):
    total = sum(counts.values())
    if total > len(NAMES):
        sys.exit(f"{total} people asked, {len(NAMES)} names")
    names = NAMES[:total]
    left = dict(counts)
    left[founder_on] -= 1
    order = []  # interleaved, each machine in proportion to what it has left
    while sum(left.values()):
        m = max(left, key=lambda k: left[k] / max(1, counts[k]))
        order.append(m)
        left[m] -= 1
    n = len(order)
    offs = [i * SPREAD / n for i in range(n)] if mode == "burst" else [i * EVERY for i in range(n)]
    people = [{"name": names[0], "machine": founder_on, "founder": True, "at_mac": round(t0, 3)}]
    for name, m, off in zip(names[1:], order, offs):
        people.append({"name": name, "machine": m, "at_mac": round(t0 + off, 3)})
    return people


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=["burst", "stagger"], required=True)
    ap.add_argument("--layout", choices=["split", "linux"], default="split")
    ap.add_argument("--mac", type=int)
    ap.add_argument("--linux", type=int)
    ap.add_argument("--founder", choices=["mac", "linux"])
    ap.add_argument("--host", default="alexbryan@fedora")
    ap.add_argument("--remote-dir", default="twentyfive")
    ap.add_argument("--run-dir", required=True)
    ap.add_argument("--mac-load-ok", action="store_true")
    ap.add_argument("--wall-every", type=float, default=0.0, help="seconds from one wall read's start to the next; 0 is back to back")
    ap.add_argument("--keep", action="store_true", help="leave each machine's work dir after collecting")
    a = ap.parse_args()
    preset = {"split": (12, 13, "mac"), "linux": (0, 25, "linux")}[a.layout]
    counts = {"mac": a.mac if a.mac is not None else preset[0], "linux": a.linux if a.linux is not None else preset[1]}
    founder_on = a.founder or preset[2]
    counts = {k: v for k, v in counts.items() if v}
    if founder_on not in counts:
        sys.exit(f"the founder is on {founder_on}, which has nobody")
    out = Path(a.run_dir).resolve() / "data"
    out.mkdir(parents=True, exist_ok=True)
    summary = {"mode": a.mode, "layout": a.layout, "counts": counts, "founder_on": founder_on,
               "started": time.time(), "pre": {}, "verdict_inputs_on": "the Mac's clock"}

    def save():
        (out / "summary.json").write_text(json.dumps(summary, indent=1))

    # Preconditions: each absent one ends the run as never-ran, said.
    if "mac" in counts and ralph_running() and not a.mac_load_ok:
        summary["pre"]["never_ran"] = f"a ralph loop runs on this Mac (pids {ralph_running()}): load here would skew it; use --layout linux, or --mac-load-ok"
        save(); sys.exit(summary["pre"]["never_ran"])
    machines = {}
    sums = released()
    if "mac" in counts:
        m = mac_machine()
        machines["mac"] = m
        summary["pre"]["mac_binary"] = {"sha256": sha256(m.bin)}
    if "linux" in counts:
        m, why = linux_machine(a.host, a.remote_dir)
        if not m:
            summary["pre"]["never_ran"] = why
            save(); sys.exit(why)
        machines["linux"] = m
        summary["pre"]["linux_kit"] = put_kit(m)
        summary["pre"]["linux_clock"] = clock(m)
    summary["pre"]["release_sums"] = sums
    for name, m in machines.items():
        summary["pre"][f"n0_from_{name}"] = n0_answers(m)
        stale = m.sh(f"test -e {shlex.quote(str(m.w))}").returncode == 0
        if stale:
            m.fleet("clean", "--w", m.w)  # refuses a dir it did not make
        summary["pre"][f"load_{name}"] = m.sh("cat /proc/loadavg 2>/dev/null || sysctl -n vm.loadavg").stdout.strip()
    if not any(v == "200" for name in machines for v in summary["pre"][f"n0_from_{name}"].values()):
        summary["pre"]["never_ran"] = "none of n0's relays answered /ping"
        save(); sys.exit(summary["pre"]["never_ran"])
    save()

    try:
        drive(a, machines, counts, founder_on, summary, save)
    finally:
        collect(a, machines, founder_on, summary, out, save)
    subprocess.run([sys.executable, str(HERE / "judge.py"), str(out)])


def drive(a, machines, counts, founder_on, summary, save):
    fm = machines[founder_on]
    for m in machines.values():
        m.keeper("sampler", "--bin", m.bin, "--w", m.w, "--secs", 10)
    # The founder first: its link is what everyone else joins by.
    write(fm, "plan-found.json", {"people": [{"name": NAMES[0], "machine": founder_on, "founder": True, "at": 0}]})
    fm.keeper("found", "--bin", fm.bin, "--w", fm.w, "--plan", f"{fm.w}/plan-found.json", "--group", GROUP)
    founded, end = None, time.time() + FOUND_WITHIN
    while time.time() < end:
        text = fm.read("founded.json")
        try:
            founded = json.loads(text) if text else None
        except ValueError:  # read while it was being written
            founded = None
        if founded:
            break
        time.sleep(0.5)
    if not founded or founded.get("failed"):
        summary["pre"]["never_ran"] = f"the founder did not make {GROUP} within {FOUND_WITHIN} s: {founded}"
        return
    summary["founded"] = {k: v for k, v in founded.items() if k != "link"}
    if fm.host:
        port = founded["port"]
        print(f"The wall, from the Mac: ssh -N -L {port}:127.0.0.1:{port} {a.host}\n"
              f"  then open http://svrngs.localhost:{port}/", flush=True)
    t0 = time.time() + LEAD
    people = plan_people(counts, founder_on, a.mode, t0)
    for p in people:
        p["at"] = round(p["at_mac"] + machines[p["machine"]].offset, 3)  # on its own machine's clock
    summary["plan"] = people
    save()
    for name, m in machines.items():
        write(m, "plan.json", {"people": people})
        if any(p["machine"] == name and not p.get("founder") for p in people):
            m.keeper("start", "--bin", m.bin, "--w", m.w, "--plan", f"{m.w}/plan.json", "--machine", name,
                     "--link", founded["link"], "--gid", founded["gid"])
    last = max(p["at_mac"] for p in people)
    ceiling = (last - time.time()) + AFTER_LAST
    wall = fm.fleet("wall", "--w", fm.w, "--gid", founded["gid"], "--who", founded["founder"],
                    "--expect", len(people), "--ceiling", round(ceiling), "--linger", LINGER, "--every", a.wall_every,
                    timeout=ceiling + LINGER + 120)
    summary["wall_end"] = shift([wall], fm)[0]
    save()
    # Every computer's own view, until each lists every lend.
    probe = lambda visits, limit: both(machines, lambda m: m.fleet(  # noqa: E731
        "probe", "--w", m.w, "--gid", founded["gid"], "--plan", f"{m.w}/plan.json", "--visits", visits, timeout=limit))
    rounds, end = [], time.time() + MATRIX_WITHIN
    while time.time() < end:
        rows = [shift([r], machines[n])[0] | {"machine": n} for n, g in probe(0, 300).items() for r in g["rows"]]
        whole = len(rows) == len(people) and all(
            len(r.get("lent") or []) == len(people) and len(r.get("people") or []) == len(people)
            and not any(r.get(k) for k in ("unanswered", "unheard", "refused", "older")) for r in rows)
        rounds.append({"t": round(time.time(), 3), "whole": whole, "rows": rows})
        summary["matrix"] = rounds
        if whole:
            break
    save()
    summary["visits"] = [shift([r], machines[n])[0] | {"machine": n} for n, g in probe(3, 600).items() for r in g["rows"]]
    summary["paths"] = {name: m.fleet("paths", "--w", m.w)["rows"] for name, m in machines.items()}
    save()


def collect(a, machines, founder_on, summary, out, save):
    """Stop every keeper, then bring back each machine's lines and logs,
    then remove its work dir unless --keep."""
    summary["stopped"] = {}
    for name, m in machines.items():
        try:
            summary["stopped"][name] = m.fleet("stop", "--w", m.w, timeout=120)
        except Exception as e:
            summary["stopped"][name] = str(e)
    for name, m in machines.items():
        summary[f"events_{name}"] = shift(jsonl(m.read("events.jsonl")), m)
        summary[f"samples_{name}"] = shift(jsonl(m.read("samples.jsonl")), m)
        if name == founder_on:
            summary["wall"] = shift(jsonl(m.read("wall.jsonl")), m)
        tgz = m.sh("python3 fleet.py logs --w " + shlex.quote(str(m.w)), timeout=300, binary=True)
        if tgz.returncode == 0:
            (out / f"logs-{name}.tgz").write_bytes(tgz.stdout)
        if not a.keep:
            try:
                m.fleet("clean", "--w", m.w)
            except Exception as e:
                summary[f"clean_{name}"] = str(e)
    summary["ended"] = time.time()
    save()
def write(m, rel, obj):
    text = json.dumps(obj)
    line = f"mkdir -p {shlex.quote(str(m.w))} && cat > {shlex.quote(str(m.w))}/{rel}"
    argv = ["sh", "-c", line] if not m.host else ["ssh", "-o", "BatchMode=yes", m.host, line]
    subprocess.run(argv, input=text, text=True, check=True, timeout=30)


def both(machines, f):
    """`f` on every machine at once; each answer by machine."""
    got, errs = {}, {}

    def one(name, m):
        try:
            got[name] = f(m)
        except Exception as e:
            errs[name] = str(e)
    ts = [threading.Thread(target=one, args=(n, m)) for n, m in machines.items()]
    for t in ts:
        t.start()
    for t in ts:
        t.join()
    if errs:
        raise RuntimeError(f"probe refused: {errs}")
    return got


if __name__ == "__main__":
    main()
