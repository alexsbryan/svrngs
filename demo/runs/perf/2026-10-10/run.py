#!/usr/bin/env python3
"""perf-1's run, rerunnable (README §3-§6), derived from sy-1's run.py
(demo/runs/sy/2026-10-09/run.py): its say/phase/summary, its directory
polled for each file's size and BLAKE3, its nodes started and stopped. Each
member is crates/svrngs-node/tests/perf.rs's perf_node, a node of its own in
a process of its own, built in release, on the path asked (loopback, relay,
n0). Each number keeps the host's load, cores and swap at its start and end,
its path as iroh selected it, and its phases from the nodes' debug traces.

  run.py breakdown [path ...]    small scales, whatever the load
  run.py targets [--wait SECS]   the targets; first waits up to SECS for the
                                 1-minute load under 1.5x cores and 20 GB free

Data in run/<pass>/ beside this file; W is the scratch dir (else
/tmp/perf-run), removed after each group. scripts/check.sh runs none of
this."""
import json, os, re, signal, statistics, subprocess, sys, time, urllib.request
from datetime import datetime
from pathlib import Path
import blake3

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
W = Path(os.environ.get("W", "/tmp/perf-run"))
CORES = int(subprocess.run(["sysctl", "-n", "hw.ncpu"], capture_output=True, text=True).stdout)
BUSY, FREE_GB, SCRATCH_GB = 1.5 * CORES, 20, 6
LOG = "folder=debug,rails=debug,svrngs_node::ring_sync=debug,warn"
KiB, MiB = 1 << 10, 1 << 20
OUT, BIN, PHASE, summary, procs = [None], [None], ["setup"], {}, {}


def say(event, **kw):
    line = {"t": round(time.time(), 3), "phase": PHASE[0], "event": event, **kw}
    print(json.dumps(line), flush=True)
    with open(OUT[0] / "events.jsonl", "a") as f: f.write(json.dumps(line) + "\n")


def phase(name): PHASE[0] = name; say("phase")


def host():
    """The load, swap in use and free disk now (README §6)."""
    load = subprocess.run(["sysctl", "-n", "vm.loadavg"], capture_output=True, text=True).stdout.split()[1:4]
    swap = subprocess.run(["sysctl", "-n", "vm.swapusage"], capture_output=True, text=True).stdout
    st = os.statvfs(W.parent if not W.exists() else W)
    return {"load": [float(x) for x in load], "cores": CORES,
            "swap_used_mb": float(re.search(r"used = ([\d.]+)M", swap)[1]),
            "free_gb": round(st.f_bavail * st.f_frsize / 1e9, 1)}


def judged(before, after):
    """None when the load lets a number be judged, else why it cannot be."""
    if max(before["load"][0], after["load"][0]) > BUSY:
        return f"could-not-judge: load {before['load'][0]}-{after['load'][0]} past {BUSY}"
    if after["swap_used_mb"] > before["swap_used_mb"]:
        return f"could-not-judge: swap in use grew {before['swap_used_mb']}-{after['swap_used_mb']} MB"
    return None


def record(name, value, target=None, meets=None, before=None, **more):
    """One number, its target's verdict, with its load (AGENTS.md 6)."""
    after = host(); why = judged(before, after) if before else None
    if target is None: verdict = "reported"
    elif value is None: verdict = "failed: the bound passed" if not why else why
    else: verdict = why or ("passed" if meets(value) else "failed")
    summary[name] = {"value": value, "target": target, "verdict": verdict, "start": before, "end": after, **more}
    say("number", name=name, **summary[name])


class Group:
    """`n` members on `path`, each a perf_node; `tag` names their dirs."""

    def __init__(self, tag, n, path, relay=None):
        self.tag, self.n, self.path, self.relay = tag, n, path, relay
        self.root = W / tag; subprocess.run(["rm", "-rf", str(self.root)])
        for i in range(n): self.data(i).mkdir(parents=True); self.disk(i).mkdir(parents=True)
        self.addrs, self.shared = {}, {}

    def data(self, i): return self.root / "data" / f"p{i}"
    def disk(self, i): return self.root / "disk" / f"p{i}" / "Shared"
    def log(self, i): return OUT[0] / f"{self.tag}-p{i}.log"

    def up(self, i):
        env = {**os.environ, "PERF_DIR": str(self.data(i)), "PERF_ME": str(i), "PERF_MEMBERS": str(self.n),
               "PERF_PATH": self.path, "PERF_RELAY": self.relay or "", "RUST_LOG": LOG, "NO_COLOR": "1"}
        (self.data(i) / "addr.json").unlink(missing_ok=True)
        procs[(self.tag, i)] = subprocess.Popen([BIN[0], "--ignored", "--exact", "perf_node", "--nocapture"],
                                                env=env, stdout=open(self.data(i) / "out", "a"),
                                                stderr=open(self.log(i), "a"))
        if wait(lambda: (self.data(i) / "addr.json").exists(), 60) is None: sys.exit(f"p{i} did not come up")
        self.addrs[i] = json.loads((self.data(i) / "addr.json").read_text())
        say("up", group=self.tag, who=i, path=self.path)

    def tell(self):
        """Every member running told of every other (perf.rs reads peers.json)."""
        for i in self.addrs:
            (self.data(i) / "peers.tmp").write_text(json.dumps(list(self.addrs.values())))
            (self.data(i) / "peers.tmp").rename(self.data(i) / "peers.json")
        wait(lambda: all("PERF told" in (self.data(i) / "out").read_text() for i in self.addrs), 30)

    def share(self, i):
        self.shared[i] = time.time()
        body = json.dumps({"bothways": str(self.disk(i))}).encode()
        req = urllib.request.Request(f"http://{self.addrs[i]['api']}/v1/folders", data=body,
                                     headers={"content-type": "application/json"})
        say("shared", group=self.tag, who=i, answer=json.load(urllib.request.urlopen(req, timeout=10)))

    def start(self, members=None, share=True):
        for i in members if members is not None else range(self.n): self.up(i)
        self.tell()
        held = lambda: all(any(self.data(i).rglob("ring_oplog.jsonl")) for i in self.addrs)
        say("converged", group=self.tag, s=wait(held, 120, every=0.25)); time.sleep(1)
        if share: [self.share(i) for i in self.addrs]
        time.sleep(1.5)

    def down(self, i):
        p = procs.pop((self.tag, i), None)
        if p: p.send_signal(signal.SIGTERM); p.wait()

    def close(self):
        [self.down(i) for i in list(self.addrs)]
        du = subprocess.run(["du", "-sk", str(self.root)], capture_output=True, text=True).stdout.split()
        say("scratch", group=self.tag, kib=int(du[0]) if du else None)
        subprocess.run(["rm", "-rf", str(self.root)])

    def cpu_s(self, i):
        t = subprocess.run(["ps", "-o", "cputime=", "-p", str(procs[(self.tag, i)].pid)],
                           capture_output=True, text=True).stdout.strip()
        parts = [float(x) for x in re.split(r"[:-]", t)]
        return sum(v * m for v, m in zip(reversed(parts), [1, 60, 3600, 86400]))

    def journal(self, i): return max(self.data(i).rglob("ring_oplog.jsonl"), key=lambda p: p.stat().st_size)


def wait(cond, bound, every=0.02):
    t0 = time.time()
    while time.time() - t0 < bound:
        if cond(): return round(time.time() - t0, 3)
        time.sleep(every)
    return None


def holds(dirs, want):
    """Every directory holds each path of `want` at its size and BLAKE3."""
    def check():
        for d in dirs:
            for path, (size, digest) in want.items():
                p = d / path
                try:
                    if p.stat().st_size != size or blake3.blake3(p.read_bytes()).hexdigest() != digest: return False
                except FileNotFoundError: return False
        return True
    return check


def count(d):
    return sum(len(fs) for _, _, fs in os.walk(d))


def write(d, path, blob):
    p = d / path; p.parent.mkdir(parents=True, exist_ok=True); p.write_bytes(blob)
    return {path: (len(blob), blake3.blake3(blob).hexdigest())}


# ---- the phases, from the nodes' logs (README §3) ----

LINE = re.compile(r"^(\S+Z)\s+(\w+)\s+([\w:]+): (.*)$")
FIELD = re.compile(r'(\w+)=("(?:[^"\\]|\\.)*"|\S+)')


def lines(log):
    out = []
    for raw in log.read_text(errors="replace").splitlines():
        m = LINE.match(raw)
        if not m: continue
        rest = m[4]; at = rest.find("=")
        msg = rest if at < 0 else rest[:rest.rfind(" ", 0, at)]
        fields = {k: v.strip('"') for k, v in FIELD.findall(rest[len(msg):])}
        t = datetime.fromisoformat(m[1].replace("Z", "+00:00")).timestamp()
        out.append((t, msg, fields))
    return out


def first(ls, msg, after, **want):
    for t, m, f in ls:
        if t >= after and m.startswith(msg) and all(f.get(k) == v for k, v in want.items()): return t, f
    return None, None


def phases(g, writer, member, path, digest, t_write):
    """One change's phases at one member, in ms (README §3); None where a line is missing."""
    w, m = g.lines(writer), g.lines(member)
    found, _ = first(w, "synced: found on disk", t_write - 1, path=path)
    taken, _ = first(w, "synced: taken into the store", found or t_write, path=path)
    written, _ = first(w, "folder: written to the record", taken or t_write)
    ingested = next((t for t, msg, f in m if t >= (written or t_write) and (
        (msg.startswith("ring sync: exchange") and int(f.get("ingested", 0)) > 0)
        or (msg.startswith("ring sync: chunk") and int(f.get("pulled", 0)) > 0))), None)
    chosen, _ = first(m, "synced: to fetch", written or t_write, path=path)
    held, hf = first(m, "synced: written into the directory", chosen or t_write, path=path)
    gaps = lambda a, b: round((b - a) * 1000, 1) if a and b else None
    fetches = [f for t, msg, f in m if chosen and held and chosen <= t <= held and msg.startswith("folder: the fetch's phases")]
    got = [f for t, msg, f in m if chosen and held and chosen <= t <= held and msg.startswith("folder: fetched, verified")
           and f.get("hash") == digest]
    return {"P1_found": gaps(t_write, found), "P2_stored": gaps(found, taken), "P3_act": gaps(taken, written),
            "P4_carried": gaps(written, ingested), "P5_apply": gaps(ingested, chosen),
            "P6_fetch": float(hf["fetch_ms"]) if hf else None, "P7_write": float(hf["export_ms"]) if hf else None,
            "dial_ms": [float(f["dial_ms"]) for f in fetches],
            "first_ms": [float(f["first_ms"]) for f in fetches if f.get("first_ms", "None") != "None"],
            "last_ms": [float(f["last_ms"]) for f in fetches],
            "gets_ms": round(sum(float(f["last_ms"]) for f in fetches), 1) if fetches else None,
            "paths": sorted({f"{'relay' if f.get('relayed') == 'Some(true)' else 'direct' if f.get('relayed') == 'Some(false)' else 'none'} {f.get('addr', '')}" for f in fetches}),
            "payload": sum(int(f.get("payload", 0)) for f in got)}


def med(xs):
    xs = [x for x in xs if x is not None]
    return round(statistics.median(xs), 1) if xs else None


def shares(ph):
    """Each phase's median over many changes, and its share of their sum."""
    keys = ["P1_found", "P2_stored", "P3_act", "P4_carried", "P5_apply", "P6_fetch", "P7_write"]
    m = {k: med([p[k] for p in ph]) for k in keys}
    total = sum(v for v in m.values() if v) or None
    return {"median_ms": m, "share": {k: round(v / total, 3) if v and total else None for k, v in m.items()},
            "paths": sorted({x for p in ph for x in p["paths"]}),
            "dial_ms": med([x for p in ph for x in p["dial_ms"]]), "first_ms": med([x for p in ph for x in p["first_ms"]]),
            "last_ms": med([x for p in ph for x in p["last_ms"]]), "gets_ms": med([p["gets_ms"] for p in ph]),
            "changes": len(ph)}


def step_split(g, i, since):
    """The step's split at member i since `since`: each part's median ms (synced.rs's "a step")."""
    steps = [f for t, m, f in g.lines(i) if t >= since and m.startswith("synced: a step")]
    keys = ["read_ms", "walk_ms", "take_ms", "write_ms", "apply_ms", "kept_ms", "ms"]
    return {k: med([float(f[k]) for f in steps if k in f]) for k in keys} | {"steps": len(steps)}


def record_reads(g, i, since):
    xs = [float(f["record_ms"]) for t, m, f in g.lines(i) if t >= since and m.startswith("folder: serving")]
    return {"gets": len(xs), "median_ms": med(xs), "sum_ms": round(sum(xs), 1)}


Group.lines = lambda g, i: lines(g.log(i))


# ---- the cases ----

def change(g, size, tag, writer=0, repeat=1, name=None, target=None, meets=None):
    """A file of `size` at `writer`, timed until every other member holds it."""
    others = [i for i in range(g.n) if i != writer]
    times, ph, rec = [], [], []
    for k in range(repeat):
        before = host(); jb = g.journal(others[0]).stat().st_size
        t0 = time.time(); want = write(g.disk(writer), f"{tag}-{k}.bin", os.urandom(size))
        t = wait(holds([g.disk(i) for i in others], want), 60)
        times.append(t); time.sleep(0.3)
        (path, (_, digest)), = want.items()
        ph += [phases(g, writer, i, path, digest, t0) for i in others]
        rec.append(g.journal(others[0]).stat().st_size - jb)
        if name: record(f"{name}#{k}", t and t * 1000, target, meets, before, unit="ms", path=g.path,
                        phases=shares([phases(g, writer, i, path, digest, t0) for i in others]))
        time.sleep(1.2)
    return times, ph, rec


def many(g, files, size, tag, name, target=None, meets=None, bound=900):
    before = host(); t0 = time.time(); want = {}
    for k in range(files): want |= write(g.disk(0), f"{tag}/{k:06}.bin", os.urandom(size))
    others = [i for i in range(1, g.n)]
    t = wait(holds([g.disk(i) for i in others], want), bound, every=0.25)
    ph = [phases(g, 0, i, p, d, t0) for i in others for p, (_, d) in want.items()]
    record(name, t and t * 1000, target, meets, before, unit="ms", path=g.path, files=files, size=size,
           phases=shares(ph), holder=record_reads(g, 0, t0), member_steps=step_split(g, others[0], t0))


def big(g, size, tag, name):
    before = host(); blob = bytearray(os.urandom(size)); t0 = time.time()
    want = write(g.disk(0), f"{tag}.bin", bytes(blob)); others = list(range(1, g.n))
    t = wait(holds([g.disk(i) for i in others], want), 900, every=0.25)
    (path, (_, digest)), = want.items(); ph = [phases(g, 0, i, path, digest, t0) for i in others]
    rate = [round(size / (p["P6_fetch"] / 1000) / 1e6, 1) if p["P6_fetch"] else None for p in ph]
    record(f"{name}-rate", min([r for r in rate if r] or [None]) if all(rate) else None, "100 MB/s over P6",
           lambda v: v >= 100, before, unit="MB/s", path=g.path, end_to_end_ms=t and t * 1000,
           per_member_mb_s=rate, phases=shares(ph), holder=record_reads(g, 0, t0))
    time.sleep(2); before = host()
    blob[size // 2:size // 2 + MiB] = os.urandom(MiB); t0 = time.time()
    want = write(g.disk(0), f"{tag}.bin", bytes(blob)); t = wait(holds([g.disk(i) for i in others], want), 900, every=0.25)
    (path, (_, digest)), = want.items(); time.sleep(0.5); ph = [phases(g, 0, i, path, digest, t0) for i in others]
    moved = [p["payload"] for p in ph]
    record(f"{name}-edit", max(moved) if t else None, "at most 2 MiB moved", lambda v: v <= 2 * MiB, before,
           unit="bytes", path=g.path, edit_bytes=MiB, per_member=moved, end_to_end_ms=t and t * 1000, phases=shares(ph))


def at_once(g, writers, tag, name, target=None, meets=None):
    """Each of `writers` writes one 4 KiB change at once; each change's time at every member."""
    before = host(); wants, t0 = {}, time.time()
    for w in writers: wants[w] = write(g.disk(w), f"{tag}-by-p{w}.bin", os.urandom(4 * KiB))
    every = {}
    def seen():
        for w, want in wants.items():
            if w not in every and holds([g.disk(i) for i in range(g.n) if i != w], want)(): every[w] = time.time() - t0
        return len(every) == len(wants)
    wait(seen, 60)
    ph = [phases(g, w, i, p, d, t0) for w, want in wants.items() for p, (_, d) in want.items() for i in range(g.n) if i != w]
    times = [every.get(w) for w in writers]
    record(name, None if None in times else round(max(times) * 1000, 1), target, meets, before, unit="ms",
           path=g.path, each_ms=[t and round(t * 1000, 1) for t in times], phases=shares(ph))


def folder(n_files, path, name, join_bound):
    """A folder of n_files of 1 KiB pointed at, then idle, then a change, then a member joining."""
    phase(name); g = Group(name, 2, path, RELAY[0])
    for k in range(n_files):
        p = g.disk(0) / f"f{k // 1000:03}" / f"{k:06}.bin"; p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(os.urandom(KiB))
    before = host(); t0 = time.time(); g.start(members=[0])
    sent = lambda: next((t for t, m, f in g.lines(0) if m.startswith("synced: a step")
                         and int(f.get("sent", 0)) >= n_files), None)
    wait(sent, 3600, every=2); at = sent()
    record(f"{name}-taken", at and round((at - g.shared[0]) * 1000, 1), None, None, before, unit="ms", path=path,
           steps=step_split(g, 0, t0))
    time.sleep(10); before = host(); c0 = g.cpu_s(0); time.sleep(60); idle = (g.cpu_s(0) - c0) / 60
    record(f"{name}-idle", round(idle * 100, 2), "under 2% of a core", lambda v: v < 2, before, unit="% of a core",
           steps=step_split(g, 0, time.time() - 60))
    found = []
    for k in range(3):
        before = host(); t0 = time.time(); write(g.disk(0), f"change-{k}.bin", os.urandom(KiB))
        t = wait(lambda: first(g.lines(0), "synced: found on disk", t0 - 1, path=f"change-{k}.bin")[0], 30, every=0.25)
        at = first(g.lines(0), "synced: found on disk", t0 - 1, path=f"change-{k}.bin")[0]
        record(f"{name}-found#{k}", at and round((at - t0) * 1000, 1), "within 1 s", lambda v: v <= 1000, before,
               unit="ms", steps=step_split(g, 0, t0))
        time.sleep(2)
    rec_before = g.journal(0).stat().st_size
    say("journal", group=name, bytes=rec_before)
    before = host(); t0 = time.time(); g.up(1); g.tell(); g.share(1)
    t = wait(lambda: count(g.disk(1)) >= n_files + 3, join_bound, every=5)
    record(f"{name}-join", t and t * 1000, None, None, before, unit="ms", path=path, held=count(g.disk(1)),
           of=n_files + 3, member_steps=step_split(g, 1, t0), holder=record_reads(g, 0, t0),
           journal_bytes=rec_before)
    g.close()


RELAY = [None]


def relay_up():
    url = W / "relay-url"; url.unlink(missing_ok=True); W.mkdir(parents=True, exist_ok=True)
    procs[("relay", 0)] = subprocess.Popen([BIN[0], "--ignored", "--exact", "perf_relay", "--nocapture"],
                                           env={**os.environ, "PERF_RELAY_URL": str(url)},
                                           stdout=subprocess.DEVNULL, stderr=open(OUT[0] / "relay.log", "a"))
    wait(url.exists, 30); RELAY[0] = url.read_text().strip(); say("relay", url=RELAY[0])


def small(path):
    """The breakdown pass's cases on one path (README §6)."""
    phase(f"small-{path}"); g = Group(f"small-{path}", 3, path, RELAY[0]); g.start()
    before = host(); times, ph, rec = change(g, 4 * KiB, "c4k", repeat=5)
    record(f"{path}-change-4k", med([t and t * 1000 for t in times]), None, None, before, unit="ms", each_s=times,
           path=path, phases=shares(ph), record_bytes=rec)
    many(g, 100, 4 * KiB, "m100", f"{path}-100-files")
    before = host(); times, ph, _ = change(g, MiB, "c1m", repeat=2)
    record(f"{path}-change-1m", med([t and t * 1000 for t in times]), None, None, before, unit="ms", each_s=times,
           path=path, phases=shares(ph))
    if path == "loopback": big(g, 10 * MiB, "b10m", f"{path}-10m"); at_once(g, [0, 1], "two", f"{path}-two-writers")
    g.close()


def targets(path):
    phase(f"targets-{path}"); g = Group(f"t-{path}", 3, path, RELAY[0]); g.start()
    judge = path == "loopback"
    before = host()
    times, ph, rec = change(g, 4 * KiB, "t1", repeat=10 if judge else 5, name=f"T1-{path}",
                            target="300 ms" if judge else None, meets=lambda v: v <= 300)
    record(f"T1-{path}-median", med([t and t * 1000 for t in times]), "300 ms" if judge else None,
           lambda v: v <= 300, before, unit="ms", each_s=times, path=path, phases=shares(ph), record_bytes=rec)
    many(g, 1000, 4 * KiB, "t2", f"T2-{path}", "10 s" if judge else None, lambda v: v <= 10_000)
    before = host(); times, ph, _ = change(g, MiB, "s1m", repeat=3)
    record(f"{path}-change-1m", med([t and t * 1000 for t in times]), None, None, before, unit="ms", each_s=times,
           path=path, phases=shares(ph))
    big(g, 100 * MiB, "t3", f"T3T4-{path}")
    if judge:
        for k in range(3): at_once(g, [0, 1], f"two{k}", f"two-writers#{k}")
    g.close()
    if judge:
        phase("T6"); g = Group("t6", 10, path); g.start()
        for k in range(3):
            at_once(g, list(range(10)), f"t6-{k}", f"T6#{k}", "1 s", lambda v: v <= 1000); time.sleep(2)
        g.close()


def main():
    which = sys.argv[1] if len(sys.argv) > 1 else "breakdown"
    OUT[0] = HERE / "run" / which; OUT[0].mkdir(parents=True, exist_ok=True)
    if which == "targets" and "--wait" in sys.argv:
        bound = float(sys.argv[sys.argv.index("--wait") + 1]); t0 = time.time()
        while time.time() - t0 < bound:
            h = host()
            if h["load"][0] < BUSY and h["free_gb"] >= FREE_GB: break
            print(json.dumps({"waiting": h}), flush=True); time.sleep(60)
    start = host(); say("start", pass_=which, host=start, head=subprocess.run(
        ["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip(),
        df=subprocess.run(["df", "-k", str(REPO)], capture_output=True, text=True).stdout)
    if start["free_gb"] < FREE_GB: sys.exit(f"{start['free_gb']} GB free, under {FREE_GB}: not started")
    built = subprocess.run(["cargo", "test", "--release", "--offline", "-p", "svrngs-node", "--test", "perf",
                            "--no-run", "--message-format=json"], cwd=REPO, capture_output=True, text=True, check=True)
    BIN[0] = next(json.loads(l)["executable"] for l in built.stdout.splitlines()
                  if '"executable":"' in l and "/perf-" in l)
    say("built", bin=BIN[0])
    paths = [a for a in sys.argv[2:] if a in ("loopback", "relay", "n0")] or ["loopback", "relay", "n0"]
    try:
        if "relay" in paths: relay_up()
        if which == "breakdown":
            for p in paths: small(p)
        else:
            for p in paths: targets(p)
            folder(10_000, "loopback", "f10k", 1800)
            folder(100_000, "loopback", "f100k", 7200)
    finally:
        for k in list(procs): p = procs.pop(k); p.send_signal(signal.SIGTERM); p.wait()
        (OUT[0] / "summary.json").write_text(json.dumps(summary, indent=1))
        for log in OUT[0].glob("*.log"):
            if log.stat().st_size > 5 * MiB: subprocess.run(["gzip", "-9", str(log)])
        say("end", host=host(), df=subprocess.run(["df", "-k", str(REPO)], capture_output=True, text=True).stdout)
        subprocess.run(["rm", "-rf", str(W)])


if __name__ == "__main__":
    main()
