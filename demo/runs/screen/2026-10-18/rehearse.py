#!/usr/bin/env python3
"""The evening's rehearsal, of the screen as it ships (home.js `screen`):
the 25-computer burst of demo/runs/twentyfive/2026-10-17/, all 25 on the
Linux machine, its `run.py` and `fleet.py` as they are, on a svrngs built
from the screen's commit. The founder's screen, `/?screen=<group>`, is
opened on the Mac through `ssh -L` before anyone joins, and Aiko's, a
member's, once Aiko is in. watch.mjs reads both every 250 ms and takes
their pictures. Once everyone is in and lent, the founder's computer is
frozen (SIGSTOP), as a closed lid leaves it, and Aiko's screen is watched
drawing her off; then one of Bo's visits through Aiko's computer says the
apps still open.

    python3 rehearse.py --bin /home/alexbryan/screen-src/target/release/svrngs --run-dir <dir>

Writes <dir>/data/: summary.json (the kit's, plus `screen`), watch.jsonl,
shots/*.png, and times.json, when each person and lend first showed on
each screen against their join and share. Times are on the Mac's clock.
"""
import argparse
import json
import os
import shlex
import socket
import subprocess
import sys
import time
from pathlib import Path
from types import SimpleNamespace

HERE = Path(__file__).resolve().parent
# The checkout this sits in; SVRNGS_REPO names one when it is run from elsewhere.
REPO = Path(os.environ.get("SVRNGS_REPO") or next(p for p in HERE.parents if (p / "AGENTS.md").exists()))
KIT = REPO / "demo" / "runs" / "twentyfive" / "2026-10-17"
sys.path.insert(0, str(KIT))
import run as kit  # noqa: E402

FOUNDER, MEMBER = kit.NAMES[0], kit.NAMES[1]
SETTLED_WITHIN = 60  # s past the wall's settling for both screens to read everyone in and lent
FINALE_WITHIN = 30   # s for Aiko's screen to draw the founder off once frozen


def tunnel(host, port):
    """`ssh -L port:127.0.0.1:port`, the same number here, so the page's
    Host is the one the home page answers; returned once it accepts."""
    p = subprocess.Popen(["ssh", "-N", "-o", "ExitOnForwardFailure=yes", "-L", f"{port}:127.0.0.1:{port}", host])
    end = time.time() + 20
    while time.time() < end:
        try:
            socket.create_connection(("127.0.0.1", port), timeout=1).close()
            return p
        except OSError:
            time.sleep(0.2)
    p.kill()
    raise RuntimeError(f"no tunnel to {port}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bin", required=True, help="svrngs on the Linux machine, built from the screen's commit")
    ap.add_argument("--host", default="alexbryan@fedora")
    ap.add_argument("--run-dir", required=True)
    a = ap.parse_args()
    out = Path(a.run_dir).resolve() / "data"
    (out / "shots").mkdir(parents=True, exist_ok=True)
    control, watched = out / "control.json", out / "watch.jsonl"
    summary = {"mode": "burst", "layout": "linux", "counts": {"linux": 25}, "founder_on": "linux",
               "started": time.time(), "pre": {}, "screen": {}, "verdict_inputs_on": "the Mac's clock"}

    def save():
        (out / "summary.json").write_text(json.dumps(summary, indent=1))

    m, why = kit.linux_machine(a.host, "twentyfive")
    if not m:
        summary["pre"]["never_ran"] = why
        save(); sys.exit(why)
    subprocess.run(["ssh", "-o", "BatchMode=yes", a.host, f"mkdir -p {shlex.quote(str(m.kit))}"], check=True, timeout=30)
    subprocess.run(["scp", "-q", str(KIT / "fleet.py"), str(KIT / "greeter.py"), f"{a.host}:{m.kit}/"], check=True, timeout=120)
    m.bin = a.bin
    summary["pre"]["linux_bin"] = {"path": a.bin, "sha256": m.sh(f"sha256sum {shlex.quote(a.bin)}").stdout.split()[0]}
    summary["pre"]["linux_clock"] = kit.clock(m)
    summary["pre"]["n0_from_linux"] = kit.n0_answers(m)
    if m.sh(f"test -e {shlex.quote(str(m.w))}").returncode == 0:
        m.fleet("clean", "--w", m.w)
    summary["pre"]["load_linux"] = m.sh("cat /proc/loadavg").stdout.strip()
    if not any(v == "200" for v in summary["pre"]["n0_from_linux"].values()):
        summary["pre"]["never_ran"] = "none of n0's relays answered /ping"
        save(); sys.exit(summary["pre"]["never_ran"])
    save()
    machines, tunnels, watcher = {"linux": m}, [], None
    control.write_text(json.dumps({"pages": {}}))
    try:
        m.keeper("sampler", "--bin", m.bin, "--w", m.w, "--secs", 10)
        kit.write(m, "plan-found.json", {"people": [{"name": FOUNDER, "machine": "linux", "founder": True, "at": 0}]})
        m.keeper("found", "--bin", m.bin, "--w", m.w, "--plan", f"{m.w}/plan-found.json", "--group", kit.GROUP)
        founded, end = None, time.time() + kit.FOUND_WITHIN
        while time.time() < end and not founded:
            try:
                founded = json.loads(m.read("founded.json") or "null")
            except ValueError:
                founded = None
            time.sleep(0.5)
        if not founded or founded.get("failed"):
            summary["pre"]["never_ran"] = f"the founder did not make {kit.GROUP}: {founded}"
            return
        summary["founded"] = {k: v for k, v in founded.items() if k != "link"}
        gid, port = founded["gid"], founded["port"]
        # The founder's screen, opened before anyone joins.
        tunnels.append(tunnel(a.host, port))
        watcher = subprocess.Popen(["node", str(HERE / "watch.mjs"), json.dumps({
            "web": str(REPO / "web"), "control": str(control), "out": str(watched), "shots": str(out / "shots"),
            "expect": len(kit.NAMES[:25]), "founder": FOUNDER})])
        pages = {"founder": f"http://svrngs.localhost:{port}/?screen={gid}"}
        control.write_text(json.dumps({"pages": pages}))
        summary["screen"]["founder_url"] = pages["founder"]
        end = time.time() + 30
        while time.time() < end and not (out / "shots" / "founder-opened.png").exists():
            time.sleep(0.25)
        summary["screen"]["founder_opened"] = (out / "shots" / "founder-opened.png").exists()
        # Then the burst, as run.py drives it.
        t0 = time.time() + kit.LEAD
        people = kit.plan_people({"linux": 25}, "linux", "burst", t0)
        for p in people:
            p["at"] = round(p["at_mac"] + m.offset, 3)
        summary["plan"] = people
        save()
        kit.write(m, "plan.json", {"people": people})
        m.keeper("start", "--bin", m.bin, "--w", m.w, "--plan", f"{m.w}/plan.json", "--machine", "linux",
                 "--link", founded["link"], "--gid", gid)
        # A member's screen, once Aiko is in: the finale is read on it.
        end, api = time.time() + 120, None
        held = f"import fleet; print(fleet.holds({str(m.w)!r}, {MEMBER!r}, {gid!r}) and fleet.api_base({str(m.w)!r}, {MEMBER!r}))"
        while time.time() < end and not api:
            said = (m.sh(f"python3 -c {shlex.quote(held)}").stdout or "").strip()
            api = said if said.startswith("http") else None
            time.sleep(0.5)
        if api:
            mport = int(api.rstrip("/").rsplit(":", 1)[1])
            tunnels.append(tunnel(a.host, mport))
            pages["member"] = f"http://svrngs.localhost:{mport}/?screen={gid}"
            control.write_text(json.dumps({"pages": pages}))
            summary["screen"]["member_url"] = pages["member"]
        last = max(p["at_mac"] for p in people)
        ceiling = (last - time.time()) + kit.AFTER_LAST
        wall = m.fleet("wall", "--w", m.w, "--gid", gid, "--who", FOUNDER, "--expect", len(people),
                       "--ceiling", round(ceiling), "--linger", kit.LINGER, "--every", 1, timeout=ceiling + kit.LINGER + 120)
        summary["wall_end"] = kit.shift([wall], m)[0]
        save()
        end = time.time() + SETTLED_WITHIN
        both = [out / "shots" / f"{n}-settled.png" for n in pages]
        while time.time() < end and not all(p.exists() for p in both):
            time.sleep(0.5)
        summary["screen"]["settled"] = {p.stem: p.exists() for p in both}
        # The finale: the founder's computer frozen, as a closed lid leaves it.
        kids = json.loads(m.read("children-found.json") or "{}")
        pid = kids.get(FOUNDER)
        m.sh(f"kill -STOP {int(pid)}")
        summary["screen"]["founder_frozen_at"] = round(time.time(), 3)
        end = time.time() + FINALE_WITHIN
        finale = out / "shots" / "member-finale.png"
        while time.time() < end and not finale.exists():
            time.sleep(0.25)
        summary["screen"]["member_finale"] = finale.exists()
        # And the apps still open: Bo's greeter, through Aiko's computer.
        visit = ("import fleet, json; base = fleet.api_base(%r, %r); _, s, body = fleet.get(base, '/v1/lends?namespace=%s', 15); "
                 "l = next(x for x in body['lent'] if x['person'] == 'Bo'); port = int(base.rsplit(':', 1)[1]); "
                 "print(json.dumps({'unanswered': body['unanswered'], 'visit': fleet.visit(port, l['origin'])}))") % (str(m.w), MEMBER, gid)
        r = m.sh(f"python3 -c {shlex.quote(visit)}", timeout=60)
        summary["screen"]["after_frozen"] = r.stdout.strip() or r.stderr[-400:]
        m.sh(f"kill -CONT {int(pid)}")
        save()
    finally:
        control.write_text(json.dumps({"stop": True}))
        if watcher:
            try:
                watcher.wait(timeout=30)
            except subprocess.TimeoutExpired:
                watcher.kill()
        for t in tunnels:
            t.kill()
        kit.collect(SimpleNamespace(keep=False), machines, "linux", summary, out, save)
        times(out, summary)


def times(out, summary):
    """When each person's window and each lend first showed on each screen,
    against the join's start and `share` returning (events, Mac clock)."""
    events = summary.get("events_linux") or []
    start = {e["who"]: e["t"] for e in events if e.get("event") == "join-start"}
    shared = {e["who"]: e["t"] for e in events if e.get("event") == "shared"}
    seen = {}
    for line in (json.loads(x) for x in (out / "watch.jsonl").read_text().splitlines() if x.strip()):
        for w in line.get("windows") or []:
            seen.setdefault((line["page"], "window", w["name"]), line["t"])
            if w.get("lends"):
                seen.setdefault((line["page"], "lend", w["name"]), line["t"])
    rows = {}
    for (page, what, who), t in seen.items():
        base = (start if what == "window" else shared).get(who)
        rows.setdefault(page, {}).setdefault(who, {})[what] = None if base is None else round(t - base, 2)
    (out / "times.json").write_text(json.dumps(rows, indent=1))


if __name__ == "__main__":
    main()
