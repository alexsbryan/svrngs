#!/usr/bin/env python3
"""docs/LESSONS_FROM_BITTORRENT.md §6's shaped runs, on the Fedora box
(AGENTS 7: a recorded run, one each; README beside this file).

  run.py one LABEL BIN [--fetchers N]
                                 step 1: 1 GB across one 40 Mbit/s uplink to
                                 N (1) fetchers at once, ping beside it
  run.py wave LABEL BIN [SIZE_MB] step 3: SIZE_MB (256) to 7 fetchers, all
                                 uplinks 40 Mbit/s
  run.py slow LABEL BIN [SIZE_MB] [--holder I] [--bound S]
                                 step 4: as wave, one fetcher's place taken
                                 by member I (5) that holds the file and is
                                 throttled to 400 Kbit/s. The plant's build
                                 (today's order, I first, plant.patch) reads
                                 data/pI's plant-first file, written here.
  run.py fast LABEL BIN [SIZE_MB]
                                 step 2: SIZE_MB (100) between two members,
                                 no shaping and no delay
BIN is a build of crates/svrngs-node/tests/perf.rs (`perf_node`). Each member
is its own network namespace: a veth `uplink` to a bridge in the namespace
`hub`, shaped with tbf to its rate over a buffer of 500 ms, and a 20 ms
delay on the hub's side towards each member (netem), so every path has a 40 ms
base RTT. Rootless: re-executes under `unshare -Urnm` and mounts a tmpfs on
/run so `ip netns` works. Writes under W (default /dev/shm/swarm/w/LABEL) and
OUT (/dev/shm/swarm/out/LABEL).
"""
import json
import os
import statistics
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path("/dev/shm/swarm")
LOG = "folder=debug,warn"


def run(*a, **k):
    return subprocess.run(a, check=k.pop("check", True), text=True, capture_output=True, **k)


def sh(cmd, check=True):
    return subprocess.run(cmd, shell=True, check=check, text=True, capture_output=True)


def net_up(n, delay=True):
    run("mount", "-t", "tmpfs", "tmpfs", "/run")
    run("ip", "netns", "add", "hub")
    sh("ip netns exec hub sh -c 'ip link set lo up; ip link add br0 type bridge; ip link set br0 up'")
    for i in range(n):
        m = f"m{i}"
        run("ip", "netns", "add", m)
        sh(f"ip link add h{i} netns hub type veth peer name uplink netns {m}")
        sh(f"ip netns exec hub ip link set h{i} master br0")
        sh(f"ip netns exec hub ip link set h{i} up")
        if delay:
            sh(f"ip netns exec hub tc qdisc add dev h{i} root netem delay 20ms limit 100000")
        sh(f"ip netns exec {m} sh -c 'ip link set lo up; ip addr add 10.99.0.{i + 1}/24 dev uplink; ip link set uplink up'")


def shape(i, rate):
    """Member i's uplink at `rate`, over a 500 ms buffer: tbf's limit in bytes."""
    bps = int(rate.replace("mbit", "000000").replace("kbit", "000"))
    limit = int(bps / 8 * 0.5)
    burst = max(32000, bps / 8 / 100)
    sh(f"ip netns exec m{i} tc qdisc replace dev uplink root tbf rate {rate} burst {int(burst)} limit {limit}")


class Group:
    def __init__(self, label, n, bin_):
        self.label, self.n, self.bin = label, n, bin_
        self.root = ROOT / "w" / label
        self.out = ROOT / "out" / label
        for p in (self.root, self.out):
            sh(f"rm -rf {p}")
            p.mkdir(parents=True)
        for i in range(n):
            self.data(i).mkdir(parents=True)
            self.disk(i).mkdir(parents=True)
        self.addrs, self.procs = {}, {}

    def data(self, i):
        return self.root / f"data/p{i}"

    def disk(self, i):
        return self.root / f"disk/p{i}/Shared"

    def up(self):
        for i in range(self.n):
            env = (f"PERF_DIR={self.data(i)} PERF_ME={i} PERF_MEMBERS={self.n} PERF_PATH=lan "
                   f"PERF_IP=10.99.0.{i + 1} RUST_LOG={LOG} NO_COLOR=1")
            cmd = f"exec ip netns exec m{i} env {env} {self.bin} --ignored --exact perf_node --nocapture"
            self.procs[i] = subprocess.Popen(cmd, shell=True,
                                             stdout=open(self.data(i) / "out", "w"),
                                             stderr=open(self.out / f"p{i}.log", "w"))
        for i in range(self.n):
            at = self.data(i) / "addr.json"
            for _ in range(600):
                if at.exists():
                    break
                time.sleep(0.1)
            self.addrs[i] = json.loads(at.read_text())
        body = json.dumps(list(self.addrs.values()))
        for i in range(self.n):
            (self.data(i) / "peers.tmp").write_text(body)
            (self.data(i) / "peers.tmp").rename(self.data(i) / "peers.json")
        print(f"[{self.label}] {self.n} members up", flush=True)

    def api(self, i, method, path, data=None):
        a = self.addrs[i]
        cmd = f"ip netns exec m{i} curl -sS --noproxy '*' -X {method} -H 'authorization: Bearer {a['token']}'"
        cmd += " -H 'content-type: application/json' --max-time 10"
        if data is not None:
            cmd += f" -d '{json.dumps(data)}'"
        return sh(f"{cmd} -w '\\n%{{http_code}}' http://{a['api']}{path}", check=False)

    def share(self, i, bound=300):
        group = (self.data(i) / "group").read_text().strip()
        t0 = time.time()
        while time.time() - t0 < bound:
            r = self.api(i, "POST", "/v1/folders", {"dir": str(self.disk(i)), "group": group})
            code = r.stdout.strip().rsplit("\n", 1)[-1] if r.stdout else "000"
            if code == "200":
                print(f"[{self.label}] p{i} shares after {time.time() - t0:.1f}s", flush=True)
                return
            time.sleep(0.5)
        sys.exit(f"p{i} never accepted the share: {r.stdout!r} {r.stderr!r}")

    def stop(self):
        for p in self.procs.values():
            p.terminate()
        for p in self.procs.values():
            try:
                p.wait(10)
            except subprocess.TimeoutExpired:
                p.kill()


def write_file(path, size):
    """`size` incompressible bytes at `path`, made outside the folder first."""
    t = path.parent.parent / (path.name + ".tmp")
    with open(t, "wb") as f:
        left = size
        while left:
            n = min(left, 1 << 22)
            f.write(os.urandom(n))
            left -= n
    t.rename(path)


def wait_for(group, who, name, size, bound):
    """Each of `who` holding the file: seconds from now, None if not within bound."""
    t0 = time.time()
    done = {}
    while time.time() - t0 < bound and len(done) < len(who):
        for i in who:
            if i not in done:
                try:
                    if (group.disk(i) / name).stat().st_size == size:
                        done[i] = round(time.time() - t0, 3)
                except FileNotFoundError:
                    pass
        time.sleep(0.25)
    return done


def pings(path):
    rtt = []
    for line in Path(path).read_text().splitlines():
        if "time=" in line:
            rtt.append(float(line.split("time=")[1].split()[0]))
    return rtt


def p95(xs):
    return sorted(xs)[int(len(xs) * 0.95) - 1] if xs else None


def verdict_ping(idle, busy):
    if not idle or not busy:
        return "could-not-judge: no ping samples"
    bar = p95(idle) + 100
    return ("passed" if p95(busy) <= bar else "failed", bar)


def one(label, bin_, size=1 << 30, fetchers=1):
    """p0's uplink shaped; `fetchers` members fetch from it at once."""
    net_up(fetchers + 1)
    shape(0, "40mbit")
    g = Group(label, fetchers + 1, bin_)
    g.up()
    print(sh("ip netns exec m0 tc qdisc show dev uplink").stdout, flush=True)
    # Stand both, then both share; p0 is the writer, its uplink the shaped one.
    time.sleep(1)
    for i in range(fetchers + 1):
        g.share(i)
    idle_out = g.out / "ping-idle.txt"
    p = subprocess.Popen(f"exec ip netns exec m0 ping -i 0.2 -D 10.99.0.2 > {idle_out}", shell=True)
    time.sleep(20)
    p.terminate(); p.wait()
    busy_out = g.out / "ping-busy.txt"
    b = subprocess.Popen(f"exec ip netns exec m0 ping -i 0.2 -D 10.99.0.2 > {busy_out}", shell=True)
    t0 = time.time()
    name = "one.bin"
    write_file(g.disk(0) / name, size)
    done = wait_for(g, list(range(1, fetchers + 1)), name, size, 2400)
    took = time.time() - t0
    b.terminate(); b.wait()
    idle, busy = pings(idle_out), pings(busy_out)
    # The transfer's ping rows only (the first and last 2 s excluded).
    n_ep = max(0, int(len(busy) * (2 / took))) if took else 0
    busy2 = busy[n_ep:len(busy) - n_ep] if n_ep and len(busy) > 2 * n_ep else busy
    summary = {
        "label": label, "size": size, "fetchers": fetchers, "took_s": round(took, 3), "arrived": done,
        "ping_idle_p95_ms": p95(idle), "ping_idle_samples": len(idle),
        "ping_busy_p95_ms": p95(busy2), "ping_busy_samples": len(busy2),
        "ping_busy_max_ms": max(busy2) if busy2 else None,
        "ping_verdict": verdict_ping(idle, busy2),
    }
    (g.out / "summary.json").write_text(json.dumps(summary, indent=1))
    print(json.dumps(summary), flush=True)
    g.stop()


def quic(label, bin_, cc, size_mb=200):
    """Step 1's diagnostic: one bare QUIC stream, the sender's cwnd printed, ping beside it."""
    size = size_mb * (1 << 20)
    net_up(2)
    shape(0, "40mbit")
    root = ROOT / "w" / label
    out = ROOT / "out" / label
    for p in (root, out):
        sh(f"rm -rf {p}")
        p.mkdir(parents=True)
    env = f"PERF_SIZE={size} PERF_DIR={root} PERF_CC={cc}"
    srv = subprocess.Popen(f"exec ip netns exec m0 env {env} PERF_ROLE=serve {bin_} --ignored --exact perf_quic --nocapture",
                           shell=True, stdout=open(out / "serve.out", "w"), stderr=subprocess.STDOUT)
    cli = subprocess.Popen(f"exec ip netns exec m1 env {env} PERF_ROLE=fetch PERF_PEER=10.99.0.1 {bin_} --ignored --exact perf_quic --nocapture",
                           shell=True, stdout=open(out / "fetch.out", "w"), stderr=subprocess.STDOUT)
    time.sleep(3)
    b = subprocess.Popen(f"exec ip netns exec m0 ping -i 0.2 -D 10.99.0.2 > {out}/ping-busy.txt", shell=True)
    cli.wait(600)
    b.terminate(); b.wait(); srv.terminate()
    busy = pings(out / "ping-busy.txt")[10:-10]
    print(json.dumps({"label": label, "cc": cc, "ping_p95_ms": p95(busy), "samples": len(busy)}), flush=True)


def fast(label, bin_, size_mb=100):
    size = size_mb * (1 << 20)
    net_up(2, delay=False)
    g = Group(label, 2, bin_)
    g.up()
    time.sleep(1)
    g.share(0)
    g.share(1)
    t0 = time.time()
    write_file(g.disk(0) / "fast.bin", size)
    done = wait_for(g, [1], "fast.bin", size, 300)
    took = round(time.time() - t0, 3)
    summary = {"label": label, "size": size, "took_s": took, "arrived": done,
               "mb_s": round(size / done[1] / 1e6, 1) if done else None}
    (g.out / "summary.json").write_text(json.dumps(summary, indent=1))
    print(json.dumps(summary), flush=True)
    g.stop()


def wave(label, bin_, size_mb=256, slow=None, bound=1200):
    n = 8
    size = size_mb * (1 << 20)
    net_up(n)
    for i in range(n):
        shape(i, "40mbit")
    g = Group(label, n, bin_)
    g.up()
    time.sleep(1)
    if slow is not None:  # the plant's build reads this; a real build ignores it
        for i in range(n):
            (g.data(i) / "plant-first").write_text(str(slow))
    g.share(0)
    name = "wave.bin"
    fetchers = list(range(1, n))
    prior = 0
    if slow is not None:
        g.share(slow)
        t = time.time()
        write_file(g.disk(0) / name, size)
        wait_for(g, [slow], name, size, 600)
        prior = round(time.time() - t, 3)
        shape(slow, "400kbit")
        sh(f"ip netns exec m{slow} tc qdisc show dev uplink")
        fetchers.remove(slow)
        print(f"[{label}] holder p{slow} held the file in {prior} s, now throttled to 400 Kbit/s", flush=True)
    for i in fetchers:
        g.share(i)
    # Held back until every member has shared, so the wave starts together.
    t0 = time.time()
    if slow is None:
        write_file(g.disk(0) / name, size)
    print(f"[{label}] wave start", flush=True)
    done = wait_for(g, fetchers, name, size, bound)
    took = round(time.time() - t0, 3)
    summary = {"label": label, "members": n, "size": size, "fetchers": fetchers, "slow_holder": slow,
               "slow_holder_took_s": prior, "took_s": took, "arrived": done}
    (g.out / "summary.json").write_text(json.dumps(summary, indent=1))
    print(json.dumps(summary), flush=True)
    g.stop()


if __name__ == "__main__":
    if os.environ.get("SWARM_INNER") != "1":
        os.execvpe("unshare", ["unshare", "-Urnm", "--", sys.executable, __file__, *sys.argv[1:]],
                   {**os.environ, "SWARM_INNER": "1"})
    what, label, bin_ = sys.argv[1], sys.argv[2], sys.argv[3]
    args = sys.argv[4:]
    if what == "one":
        one(label, bin_, fetchers=int(args[args.index("--fetchers") + 1]) if "--fetchers" in args else 1)
    elif what == "wave":
        wave(label, bin_, int(args[0]) if args and args[0].isdigit() else 256)
    elif what == "quic":
        quic(label, bin_, args[0], int(args[1]) if len(args) > 1 else 200)
    elif what == "fast":
        fast(label, bin_, int(args[0]) if args else 100)
    elif what == "slow":
        holder = int(args[args.index("--holder") + 1]) if "--holder" in args else 5
        bound = int(args[args.index("--bound") + 1]) if "--bound" in args else 1200
        wave(label, bin_, int(args[0]) if args and args[0].isdigit() else 256, slow=holder, bound=bound)
    else:
        sys.exit("unknown step: " + what)
