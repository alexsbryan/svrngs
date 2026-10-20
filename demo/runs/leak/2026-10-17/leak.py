#!/usr/bin/env python3
"""The founder's memory under the wall's reads (PREDICTIONS.md here),
on one machine: a group of N computers brought up by the 25-computer kit's
fleet.py (`demo/runs/twentyfive/2026-10-17/`, `found`, then `start`), then
the founder's loopback API driven in timed phases while its memory is read
each second: /proc on Linux; ps, footprint and lsof on macOS. Stdlib only.

    leak.py run --cond NAME [--n 25] [--log FILTER] [--b-secs 60] [--b-every S]
                [--bin PATH] [--env K=V ...] [--probe]

The binary is the kit's V1.1 for this machine (`bin/<os>-<arch>/svrngs`
there, fetched as its README says) unless `--bin` names another.

Each condition runs in w-NAME (a fleet work dir, removed at the end) and
leaves data-NAME/: samples.jsonl, one line a second, and summary.json.
Phases: A idle 120 s; B /v1/lends back to back; C idle 60 s;
D /v1/rail/people back to back, 60 s; E idle 60 s. Every wait has a
ceiling (AGENTS.md 6): a condition that can't bring its group up ends as
never-ran, said in summary.json.
"""
import argparse
import json
import os
import platform
import re
import subprocess
import sys
import threading
import time
import urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
KIT = HERE.parent.parent / "twentyfive" / "2026-10-17"
DARWIN = sys.platform == "darwin"
MACHINE = "mac" if DARWIN else "linux"  # the plan's label for this machine (fleet.py `start --machine`)
BIN = KIT / "bin" / f"{'darwin' if DARWIN else 'linux'}-{platform.machine().replace('arm64', 'aarch64')}" / "svrngs"
NAMES = ["Priya", "Aiko", "Bo", "Cy", "Dev", "Femi", "Gus", "Hana", "Ife", "Jun", "Kit", "Lena", "Luz",
         "Mara", "Max", "Nell", "Noor", "Oren", "Pia", "Ravi", "Rosa", "Sam", "Saul", "Theo", "Wren"]
UP_WITHIN = 150  # s for every joiner to have shared, or never-ran
TICK = os.sysconf("SC_CLK_TCK")  # CPU ticks a second; macOS's ps seconds are read in them too
PAGE_KIB = os.sysconf("SC_PAGE_SIZE") // 1024


def kv(path, keys):
    out = {}
    try:
        for line in Path(path).read_text().splitlines():
            k, _, v = line.partition(":")
            if k in keys:
                out[k] = int(v.split()[0])
    except OSError:
        pass
    return out


def seconds(cpu):
    """ps's CPU time as macOS writes it (`m:ss.cc`, `h:mm:ss.cc`), in seconds."""
    s = 0.0
    for x in cpu.split(":"):
        s = s * 60 + float(x)
    return s


def proc_darwin(pid):
    """One reading of `pid` on macOS. `rss` is ps's resident size; `anon`
    is footprint's, the dirty memory the process owns, which is what
    macOS's malloc gives back when it frees (its MADV_FREE'd pages can
    stay resident in `rss` until the machine needs them)."""
    run = lambda *argv: subprocess.run(argv, capture_output=True, text=True, timeout=10).stdout  # noqa: E731
    bits = run("ps", "-o", "rss=,time=", "-p", str(pid)).split()
    m = re.search(r"Footprint: ([\d.]+) (KB|MB|GB)", run("footprint", "-p", str(pid)))
    foot = round(float(m[1]) * {"KB": 1, "MB": 1024, "GB": 1024 * 1024}[m[2]]) if m else None
    lines = run("ps", "-M", "-p", str(pid)).splitlines()
    fds = sum(1 for l in run("lsof", "-p", str(pid), "-Ff").splitlines() if l.startswith("f"))
    return {"rss": int(bits[0]) if bits else None, "hwm": None, "anon": foot,
            "threads": len(lines) - 1 if len(lines) > 1 else None, "fds": fds or None,
            "ticks": round(seconds(bits[1]) * TICK) if len(bits) > 1 else None}


def proc(pid):
    """One reading of `pid`: memory in KiB, threads, fds, CPU ticks."""
    if DARWIN:
        return proc_darwin(pid)
    s = kv(f"/proc/{pid}/status", {"VmRSS", "VmHWM", "Threads", "RssAnon"})
    r = kv(f"/proc/{pid}/smaps_rollup", {"Rss", "Anonymous"})
    try:
        fds = len(os.listdir(f"/proc/{pid}/fd"))
    except OSError:
        fds = None
    try:
        f = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
        ticks = int(f[11]) + int(f[12])  # utime + stime, fields 14 and 15
    except (OSError, IndexError, ValueError):
        ticks = None
    return {"rss": s.get("VmRSS"), "hwm": s.get("VmHWM"), "anon": r.get("Anonymous"),
            "threads": s.get("Threads"), "fds": fds, "ticks": ticks}


class Reads:
    def __init__(self):
        self.n, self.errors, self.phase = 0, 0, None
        self.lock = threading.Lock()


def drive(base, path, until, reads, every=0.0):
    """Reads of `path` until `until`: back to back, or one each `every` s
    from one read's start to the next."""
    while time.time() < until:
        started = time.time()
        try:
            with urllib.request.urlopen(base + path, timeout=15) as r:
                r.read()
                ok = r.status == 200
        except Exception:
            ok = False
        with reads.lock:
            reads.n += 1
            reads.errors += 0 if ok else 1
        if every:
            time.sleep(max(0.0, started + every - time.time()))


def sampler(pids, w, out, reads, stop):
    with open(out, "a", buffering=1) as f:
        while not stop.is_set():
            t = time.time()
            with reads.lock:
                n, errs, phase = reads.n, reads.errors, reads.phase
            line = {"t": round(t, 3), "phase": phase, "reads": n, "errors": errs}
            for who, pid in pids.items():
                line[who] = proc(pid)
            try:
                line["log_bytes"] = (Path(w) / f"{NAMES[0]}.log").stat().st_size
            except OSError:
                line["log_bytes"] = None
            f.write(json.dumps(line) + "\n")
            stop.wait(max(0.0, 1.0 - (time.time() - t)))


def fleet(*args, env, wait=True):
    argv = [sys.executable, str(KIT / "fleet.py"), *[str(a) for a in args]]
    if wait:
        return subprocess.run(argv, env=env, capture_output=True, text=True, timeout=120)
    return subprocess.Popen(argv, env=env, stdin=subprocess.DEVNULL,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, start_new_session=True)


def summarise(samples, phases):
    """Per phase: KiB grown, reads, KiB per read, threads and fds at each
    end; for a phase of reads, the first and last thirds' KiB per read."""
    out = {}
    for name, (t0, t1) in phases.items():
        rows = [s for s in samples if t0 <= s["t"] <= t1 and s["Priya"]["rss"] is not None]
        if len(rows) < 2:
            out[name] = {"verdict": "could-not-judge", "why": "fewer than two samples"}
            continue
        a, b = rows[0], rows[-1]
        d = b["Priya"]["rss"] - a["Priya"]["rss"]
        anon = [a["Priya"]["anon"], b["Priya"]["anon"]]
        n = b["reads"] - a["reads"]
        cpu = None
        if a["Priya"]["ticks"] is not None and b["Priya"]["ticks"] is not None:
            cpu = round(100 * (b["Priya"]["ticks"] - a["Priya"]["ticks"]) / TICK / (b["t"] - a["t"]), 1)
        row = {"secs": round(b["t"] - a["t"], 1), "rss_kib": [a["Priya"]["rss"], b["Priya"]["rss"]],
               "anon_kib": [a["Priya"]["anon"], b["Priya"]["anon"]], "hwm_kib": b["Priya"]["hwm"],
               "grew_kib": d, "anon_grew_kib": anon[1] - anon[0] if None not in anon else None, "reads": n, "errors": b["errors"] - a["errors"],
               "kib_per_read": round(d / n, 2) if n else None,
               "anon_kib_per_read": round((anon[1] - anon[0]) / n, 2) if n and None not in anon else None, "cpu_pct": cpu,
               "threads": [a["Priya"]["threads"], b["Priya"]["threads"]],
               "fds": [a["Priya"]["fds"], b["Priya"]["fds"]],
               "log_bytes": [a["log_bytes"], b["log_bytes"]]}
        if "Aiko" in a:
            row["aiko_rss_kib"] = [a["Aiko"]["rss"], b["Aiko"]["rss"]]
        if n:
            third = (b["t"] - a["t"]) / 3
            def slope(r0, r1):
                x = [s for s in rows if r0 <= s["t"] <= r1]
                if len(x) < 2 or x[-1]["reads"] == x[0]["reads"]:
                    return None
                return round((x[-1]["Priya"]["rss"] - x[0]["Priya"]["rss"]) / (x[-1]["reads"] - x[0]["reads"]), 2)
            row["kib_per_read_first_third"] = slope(a["t"], a["t"] + third)
            row["kib_per_read_last_third"] = slope(b["t"] - third, b["t"])
        out[name] = row
    return out


def run(a):
    w, data = HERE / f"w-{a.cond}", HERE / f"data-{a.cond}"
    if w.exists() or data.exists():
        sys.exit(f"{w} or {data} exists: one fresh setup a condition")
    data.mkdir(parents=True)
    # `kit` is the filter the burst run had (fleet.py LOG); else RUST_LOG's own.
    env = {k: v for k, v in os.environ.items() if k != "TWENTYFIVE_LOG"}
    if a.log != "kit":
        env["TWENTYFIVE_LOG"] = a.log
    for pair in a.env:
        k, _, v = pair.partition("=")
        env[k] = v
    binary = Path(a.bin).resolve() if a.bin else BIN
    summary = {"cond": a.cond, "n": a.n, "log": a.log, "env": a.env, "b_secs": a.b_secs,
               "bin": str(binary), "probe": a.probe, "b_every": a.b_every, "started": time.time(), "cores": os.cpu_count()}
    # The founder alone gets the task probe (a scratch build's SVRNGS_TASKPROBE).
    env_found = {**env, "SVRNGS_TASKPROBE": str(data / "tasks.txt")} if a.probe else env
    save = lambda: (data / "summary.json").write_text(json.dumps(summary, indent=1))  # noqa: E731
    save()
    names = NAMES[: a.n]
    w.mkdir()
    (w / "plan-found.json").write_text(json.dumps({"people": [{"name": names[0], "machine": MACHINE, "founder": True, "at": 0}]}))
    keepers = [fleet("found", "--bin", binary, "--w", w, "--plan", w / "plan-found.json", "--group", "House", env=env_found, wait=False)]
    try:
        end = time.time() + 90
        while time.time() < end and not (w / "founded.json").exists():
            time.sleep(0.2)
        founded = json.loads((w / "founded.json").read_text()) if (w / "founded.json").exists() else {"failed": True}
        if founded.get("failed"):
            summary["verdict"] = "never-ran"
            summary["why"] = "the founder did not found the group within 90 s"
            return
        t0 = time.time() + 3
        people = [{"name": names[0], "machine": MACHINE, "founder": True, "at": 0}]
        for i, name in enumerate(names[1:]):
            people.append({"name": name, "machine": MACHINE, "at": round(t0 + i * 10 / max(1, len(names) - 1), 3)})
        (w / "plan.json").write_text(json.dumps({"people": people}))
        keepers.append(fleet("start", "--bin", binary, "--w", w, "--plan", w / "plan.json", "--machine", MACHINE,
                             "--link", founded["link"], "--gid", founded["gid"], env=env, wait=False))
        end = time.time() + UP_WITHIN
        shared = 0
        while time.time() < end:
            evs = [json.loads(l) for l in (w / "events.jsonl").read_text().splitlines() if l.strip()]
            shared = sum(1 for e in evs if e.get("event") == "shared" and e.get("code") == 0 and e.get("who") != names[0])
            if shared >= a.n - 1:
                break
            time.sleep(0.5)
        summary["shared"] = shared
        if shared < a.n - 1:
            summary["verdict"] = "never-ran"
            summary["why"] = f"{shared} of {a.n - 1} joiners shared within {UP_WITHIN} s"
            return
        pids = {names[0]: json.loads((w / "children-found.json").read_text())[names[0]]}
        if a.n > 1:
            pids[names[1]] = json.loads((w / "children-start.json").read_text())[names[1]]
        base = (w / names[0] / ".svrngs" / "api").read_text().strip().rstrip("/")
        gid = founded["gid"]
        reads, stop = Reads(), threading.Event()
        th = threading.Thread(target=sampler, args=(pids, w, data / "samples.jsonl", reads, stop), daemon=True)
        th.start()
        plan = [("A", None, 120), ("B", f"/v1/lends?namespace={gid}", a.b_secs), ("C", None, 60),
                ("D", f"/v1/rail/people?namespace={gid}", 60), ("E", None, 60)]
        phases = {}
        for name, path, secs in plan:
            with reads.lock:
                reads.phase = name
            t = time.time()
            if path:
                drive(base, path, t + secs, reads, a.b_every if name == "B" else 0.0)
            else:
                time.sleep(secs)
            phases[name] = (t, time.time())
            summary["phases"] = {k: [round(x, 3) for x in v] for k, v in phases.items()}
            save()
        time.sleep(1.5)
        stop.set()
        th.join(5)
        samples = [json.loads(l) for l in (data / "samples.jsonl").read_text().splitlines() if l.strip()]
        summary["result"] = summarise(samples, phases)
        if a.probe and (data / "tasks.txt").exists():
            ticks = [tuple(map(float, l.split())) for l in (data / "tasks.txt").read_text().splitlines() if l.strip()]
            for name, (t0, t1) in phases.items():
                x = [r for r in ticks if t0 <= r[0] <= t1]
                if x and name in summary["result"]:
                    summary["result"][name]["alive_tasks"] = [int(x[0][1]), int(x[-1][1])]
        summary["verdict"] = "ran"
    finally:
        summary["ended"] = time.time()
        save()
        r = fleet("stop", "--w", w, env=env)
        summary["stopped"] = (r.stdout or "").strip()[-400:]
        for k in keepers:
            try:
                k.wait(30)
            except subprocess.TimeoutExpired:
                k.kill()
        try:
            summary["founder_log_bytes"] = (w / f"{NAMES[0]}.log").stat().st_size
        except OSError:
            pass
        r = fleet("clean", "--w", w, env=env)
        summary["cleaned"] = (r.stdout or r.stderr or "").strip()[-200:]
        save()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("verb", choices=["run"])
    ap.add_argument("--cond", required=True)
    ap.add_argument("--n", type=int, default=25)
    ap.add_argument("--log", default="error")
    ap.add_argument("--b-secs", type=float, default=60)
    ap.add_argument("--b-every", type=float, default=0.0, help="B's reads one each N s, as the wall paced at 1 s; 0 is back to back")
    ap.add_argument("--env", action="append", default=[])
    ap.add_argument("--bin", help="a svrngs binary other than the kit's V1.1")
    ap.add_argument("--probe", action="store_true", help="the founder writes its alive tasks each second")
    run(ap.parse_args())


if __name__ == "__main__":
    main()
