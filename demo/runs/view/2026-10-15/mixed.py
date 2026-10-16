#!/usr/bin/env python3
"""arch-view (b)'s cross-build run: a computer built at the row's start and
one built at its close, in one group on this host, each accept the other's
new acts (the row's check; fence 3). Adapted from
demo/runs/arch/2026-10-15-seal/mixed.py.

MAKER makes the group (`new`) and the other joins by its link (`join`); each
writes three records through its record API, and the run waits for both
copies to hold and count every one. What each holds is read from its own
journal on disk and from its record API's log: every act both hold is the
same line, both copies are complete with no gaps, and the close's new lines
carry no view while the start's carry one.

  mixed.py START_BIN CLOSE_BIN MAKER    (MAKER: start or close)

Data in run-maker-<maker>/ beside this file; the scratch dir is
/tmp/arch-view-run-<maker>, removed at the start. Every computer is on this
host; the two dial each other as svrngs does by default (fence 8)."""
import json, os, shutil, signal, subprocess, sys, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
START, CLOSE, MAKER = sys.argv[1], sys.argv[2], sys.argv[3]
assert MAKER in ("start", "close"), MAKER
OUT = HERE / f"run-maker-{MAKER}"
W = Path(f"/tmp/arch-view-run-{MAKER}")
# Ana runs the start's build, Bo the close's; the maker makes the group.
BUILD = {"ana": START, "bo": CLOSE}
BUILT = {"ana": "start", "bo": "close"}
MAKES, JOINS = ("ana", "bo") if MAKER == "start" else ("bo", "ana")
LOG = "svrngs_core::admit=debug,svrngs_node::ring_sync=debug,rail=debug,warn"
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
    (W / who).mkdir(parents=True, exist_ok=True)
    procs[who] = subprocess.Popen([BUILD[who], *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(OUT / f"{who}.out", "w"), stderr=open(OUT / f"{who}.log", "a"))
    say("svrngs", who=who, build=BUILT[who], args=[a if not a.startswith("svrngs://") else "<link>" for a in args])


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
    rings = [p.name for p in (data(MAKES) / "rings").iterdir() if p.is_dir()]
    assert len(rings) == 1, rings
    return rings[0]


def append(who, act):
    status, out = http("POST", f"{base(who)}/v1/rail/append?namespace={group()}", act)
    say("append", who=who, act=act, status=status)
    assert status == 200, (who, status, out)


def log(who):
    status, out = http("GET", f"{base(who)}/v1/rail/log?namespace={group()}")
    assert status == 200, (who, status, out)
    return out


def lines(who):
    """who's journal on disk, line by line, by id."""
    text = (data(who) / "rings" / group() / "ring_oplog.jsonl").read_text().splitlines()
    return {o["id"]: o for o in (json.loads(l) for l in text if l.strip())}


def whats(who): return {o["payload"]["what"] for o in log(who)["ops"] if o.get("payload") and not o.get("voided")}


def main():
    shutil.rmtree(W, ignore_errors=True); (W / "stub").mkdir(parents=True)
    (W / "stub/open").write_text("#!/bin/sh\nexit 0\n"); (W / "stub/open").chmod(0o755)
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir()
    head = subprocess.run(["git", "-C", str(HERE), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    say("begin", head=head, start=START, close=CLOSE, maker=MAKER, load=os.getloadavg())
    try:
        start(MAKES, "new", "Lab", "--as", MAKES.title())
        wait(lambda: "svrngs://join/" in said(MAKES), 120, f"{MAKES}'s link")
        link = next(w for w in said(MAKES).split() if w.startswith("svrngs://join/"))
        start(JOINS, "join", link, "--as", JOINS.title())
        wait(lambda: "You're in" in said(JOINS), 120, f"{JOINS} in")
        for who in ("ana", "bo"):
            wait(lambda: (data(who) / "api").exists(), 60, f"{who}'s record API")
            say("running", who=who, api=base(who))
        words = {"ana": ["a1", "a2", "a3"], "bo": ["b1", "b2", "b3"]}
        for who in ("ana", "bo"):
            for w in words[who]: append(who, {"op": "record", "payload": {"kind": "note", "what": w}})
        every = set(words["ana"]) | set(words["bo"])
        took = wait(lambda: every <= whats("ana") and every <= whats("bo"), 240, "the six records on both")
        say("in step", seconds=took)
        time.sleep(10)  # a round more, so a resend or a refusal shows in the logs

        held = {who: lines(who) for who in ("ana", "bo")}
        both = sorted(set(held["ana"]) & set(held["bo"]))
        differ = [i for i in both if held["ana"][i] != held["bo"][i]]
        logs = {who: log(who) for who in ("ana", "bo")}
        counted = {who: sorted((o.get("payload") or {}).get("what") for o in logs[who]["ops"]
                               if o.get("payload") and not o.get("voided")) for who in logs}
        by_build = {}
        for i in both:
            o = held["ana"][i]
            what = (o.get("payload") or {}).get("what")
            writer = "ana" if what in words["ana"] else "bo" if what in words["bo"] else None
            if writer:
                by_build.setdefault(BUILT[writer], []).append({"what": what, "view": "view" in o, "v": o["v"]})
        summary = {
            "head": head, "maker": MAKER, "start_bin": START, "close_bin": CLOSE,
            "held": {"ana": len(held["ana"]), "bo": len(held["bo"]), "both": len(both)},
            "only_ana": sorted(set(held["ana"]) - set(held["bo"])),
            "only_bo": sorted(set(held["bo"]) - set(held["ana"])),
            "both_hold_differently": differ,
            "records_counted": counted,
            "complete": {who: logs[who]["complete"] for who in logs},
            "gaps": {who: logs[who]["gaps"] for who in logs},
            "records_by_build": by_build,
            "lines_with_a_view": {who: sum("view" in o for o in held[who].values()) for who in held},
            "journals": held,
        }
        (OUT / "summary.json").write_text(json.dumps(summary, indent=1))
        say("summary", **{k: v for k, v in summary.items() if k != "journals"})
    finally:
        for who, p in procs.items(): p.send_signal(signal.SIGTERM); p.wait()
        (OUT / "events.jsonl").write_text("".join(json.dumps(e) + "\n" for e in events))


if __name__ == "__main__":
    main()
