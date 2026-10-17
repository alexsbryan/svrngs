#!/usr/bin/env python3
"""arch-seal (c)'s cross-build run: a computer built at the row's start and
one built at its close, in one group on this host, exchange and agree on
every act each holds, a seal among them (the row's check; fence 3).

Bo runs the close's build and makes the group; Ana runs the start's build
(c9e4c12) and joins by Bo's link. Each writes two records through its
record API; then Ana's app seals, which only the start's build still takes
(its route sealed then), and Bo's is refused (bar 10, on the real binary).
Each writes one more record, and the run waits for both copies to hold
every act. What each holds is read from its own journal on disk and from
its record API's log.

  mixed.py START_BIN CLOSE_BIN

Data in run/ beside this file; W is the scratch dir (else
/tmp/arch-seal-run), removed at the start. Every computer is on this host;
the two dial each other as svrngs does by default (fence 8)."""
import json, os, shutil, signal, subprocess, sys, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
OUT = HERE / "run"
W = Path(os.environ.get("W", "/tmp/arch-seal-run"))
START, CLOSE = sys.argv[1], sys.argv[2]
BUILD = {"ana": START, "bo": CLOSE}
LOG = "svrngs_core::admit=debug,svrngs_node::ring_sync=debug,rails=debug,warn"
procs, events = {}, []


def say(event, **kw):
    line = {"t": round(time.time(), 3), "event": event, **kw}
    print(json.dumps(line), flush=True)
    events.append(line)


def data(who): return W / who / ".svrngs"


def env(who):
    e = {k: v for k, v in os.environ.items() if k != "SVRNGS_DIR"}
    return {**e, "HOME": str(W / who), "USER": who, "PATH": f"{W / 'stub'}:{e['PATH']}",
            "NO_COLOR": "1", "SVRNGS_DIR": str(data(who)), "RUST_LOG": LOG}


def start(who, *args):
    """who's svrngs, `new` or `join` as a person types it: each runs on as
    the computer's svrngs once it has made or joined the group."""
    (W / who).mkdir(parents=True, exist_ok=True)
    procs[who] = subprocess.Popen([BUILD[who], *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(OUT / f"{who}.out", "w"), stderr=open(OUT / f"{who}.log", "a"))
    say("svrngs", who=who, build=BUILD[who], args=[a if not a.startswith("svrngs://") else "<link>" for a in args])


def said(who): return (OUT / f"{who}.out").read_text(errors="replace")


def wait(ok, within, what):
    t0 = time.time()
    while time.time() - t0 < within:
        if ok(): return round(time.time() - t0, 2)
        time.sleep(0.5)
    raise SystemExit(f"could-not-judge: {what} not within {within} s")


def base(who): return (data(who) / "api").read_text().strip().rstrip("/")


def http(method, url, body=None):
    req = urllib.request.Request(url, method=method, data=None if body is None else json.dumps(body).encode(),
                                 headers={"content-type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=30) as r: return r.status, json.loads(r.read())
    except urllib.error.HTTPError as e: return e.code, json.loads(e.read() or b"null")


def group():
    rings = [p.name for p in (data("bo") / "rings").iterdir() if p.is_dir()]
    assert len(rings) == 1, rings
    return rings[0]


def append(who, act):
    status, out = http("POST", f"{base(who)}/v1/rail/append?namespace={group()}", act)
    say("append", who=who, act=act, status=status, answer=out)
    return status, out


def log(who):
    status, out = http("GET", f"{base(who)}/v1/rail/log?namespace={group()}")
    assert status == 200, (who, status, out)
    return out


def held(who):
    """Each line of who's journal on disk, by id: its actor, seq and act."""
    lines = (data(who) / "rings" / group() / "ring_oplog.jsonl").read_text().splitlines()
    ops = [json.loads(l) for l in lines if l.strip()]
    return {o["id"]: {"actor": o["actor"][:8], "seq": o["seq"], "op": o["op"],
                      "what": (o.get("payload") or {}).get("what")} for o in ops}


def whats(who): return {o["payload"]["what"] for o in log(who)["ops"] if o.get("payload") and not o.get("voided")}


def main():
    shutil.rmtree(W, ignore_errors=True); (W / "stub").mkdir(parents=True)
    (W / "stub/open").write_text("#!/bin/sh\nexit 0\n"); (W / "stub/open").chmod(0o755)
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir()
    head = subprocess.run(["git", "-C", str(HERE), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    say("begin", head=head, start=START, close=CLOSE, load=os.getloadavg())
    try:
        start("bo", "new", "Lab", "--as", "Bo")
        wait(lambda: "svrngs://join/" in said("bo"), 120, "Bo's link")
        link = next(w for w in said("bo").split() if w.startswith("svrngs://join/"))
        start("ana", "join", link, "--as", "Ana")
        wait(lambda: "You're in" in said("ana"), 120, "Ana in")
        for who in ("ana", "bo"):
            wait(lambda: (data(who) / "api").exists(), 60, f"{who}'s record API")
            say("running", who=who, api=base(who))
        for who, words in (("ana", ["a1", "a2"]), ("bo", ["b1", "b2"])):
            for w in words: append(who, {"op": "record", "payload": {"kind": "note", "what": w}})
        both = {"a1", "a2", "b1", "b2"}
        took = wait(lambda: both <= whats("ana") and both <= whats("bo"), 240, "the four records on both")
        say("in step", acts=sorted(both), seconds=took)

        ana_seal = append("ana", {"op": "seal"})
        bo_seal = append("bo", {"op": "seal"})
        for who, w in (("ana", "a3"), ("bo", "b3")):
            append(who, {"op": "record", "payload": {"kind": "note", "what": w}})
        seal_on = lambda who: any(v["op"] == "seal" for v in held(who).values())
        took = wait(lambda: {"a3", "b3"} <= whats("ana") and {"a3", "b3"} <= whats("bo") and seal_on("bo"), 240,
                    "a3, b3 and Ana's seal on both")
        say("in step after the seal", seconds=took)
        time.sleep(75)  # one more round each way, so a prune or a resend shows

        ana, bo = held("ana"), held("bo")
        both_ids = sorted(set(ana) & set(bo))
        differ = [i for i in both_ids if ana[i] != bo[i]]
        only_bo = {i: bo[i] for i in sorted(set(bo) - set(ana))}
        only_ana = {i: ana[i] for i in sorted(set(ana) - set(bo))}
        logs = {who: log(who) for who in ("ana", "bo")}
        counted = {who: {o["id"]: (o.get("payload") or {}).get("what") for o in logs[who]["ops"]
                         if o.get("payload") and not o.get("voided")} for who in logs}
        agree = [i for i in both_ids if counted["ana"].get(i) != counted["bo"].get(i)]
        debug_said = {who: sum("a seal is held and retires nothing" in l for l in open(OUT / f"{who}.log", errors="replace"))
                for who in ("ana", "bo")}
        stalls = {who: sum("exchange stalled" in l for l in open(OUT / f"{who}.log", errors="replace"))
                  for who in ("ana", "bo")}
        summary = {
            "head": head, "start_bin": START, "close_bin": CLOSE,
            "ana_seal": {"status": ana_seal[0], "retired": (ana_seal[1] or {}).get("retired")},
            "bo_seal": {"status": bo_seal[0], "kind": (bo_seal[1] or {}).get("kind")},
            "held": {"ana": len(ana), "bo": len(bo), "both": len(both_ids)},
            "both_hold_differently": differ, "counted_differently": agree,
            "only_bo": only_bo, "only_ana": only_ana,
            "complete": {who: logs[who]["complete"] for who in logs},
            "gaps": {who: logs[who]["gaps"] for who in logs},
            "records_counted": {who: sorted(counted[who].values()) for who in counted},
            "seal_said_at_debug": debug_said, "stalled_exchanges": stalls,
            "journals": {"ana": ana, "bo": bo},
        }
        (OUT / "summary.json").write_text(json.dumps(summary, indent=1))
        say("summary", **{k: v for k, v in summary.items() if k != "journals"})
    finally:
        for who, p in procs.items(): p.send_signal(signal.SIGTERM); p.wait()
        (OUT / "events.jsonl").write_text("".join(json.dumps(e) + "\n" for e in events))


if __name__ == "__main__":
    main()
