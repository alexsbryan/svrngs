"""Demo A's three (trio.mjs's JSON) read against demo/a.md's bars, and
each stream's share from the lender's log (the door's `let in, and told
their share` lines, svrngs_node::door).

usage: python3 trio.py <trio json> <the lender's svrngs log> [<from> <until>]
       writes <trio>-summary.json beside it. The log's lines are read from
       the run's `from` to its `until` (trio.mjs keeps both, ISO, UTC), or
       between the two given: one lender's log holds every run of a scratch.

Each bar is passed, failed or could-not-judge; a run with no JSON never
ran. Every seek is kept, a miss never dropped: its time, its moment in
the window and the host's load then.
"""
import json
import re
import statistics
import sys
from pathlib import Path

BAR_MS = 5000

path, log = Path(sys.argv[1]), Path(sys.argv[2])
try:
    d = json.loads(path.read_text())
except (OSError, ValueError) as e:
    print(f"never ran: {e}")
    sys.exit(1)

LINE = re.compile(r'person="([^"]+)" lend=(\S+) people=(\d+) share_bps=(\S+) declared_bps=(\S+) relayed=(\S+) rtt_ms=(\S+)')
since, until = (sys.argv[3], sys.argv[4]) if len(sys.argv) > 4 else (d.get("from"), d.get("until"))
if not (since and until):
    print("could-not-judge  the door's shares: the run keeps no bounds, and none were given")
door = {}
for line in log.read_text(errors="replace").splitlines():
    # tracing's lines start with their time, ISO, UTC, as trio.mjs's bounds.
    if not (since and until and since <= line[:24] <= until):
        continue
    if "told their share of the declared upload" in line and (m := LINE.search(line)):
        who, _, people, share, declared, relayed, rtt = m.groups()
        p = door.setdefault(who, {"let_in": 0, "people": [], "share_bps": [], "declared_bps": None, "relayed": {}, "rtt_ms": []})
        p["let_in"] += 1
        p["people"].append(int(people))
        num = lambda v: int(v[5:-1]) if v.startswith("Some(") else None
        if (s := num(share)) is not None:
            p["share_bps"].append(s)
        p["declared_bps"] = num(declared)
        p["relayed"][relayed] = p["relayed"].get(relayed, 0) + 1
        if (r := num(rtt)) is not None:
            p["rtt_ms"].append(r)
    elif "guessed in this computer's home" in line and (m := re.search(r'person="([^"]+)"', line)):
        door.setdefault(m.group(1), {"let_in": 0}).setdefault("home", 0)
        door[m.group(1)]["home"] += 1
    elif "as many people as the lend's limit" in line and (m := re.search(r'person="([^"]+)"', line)):
        door.setdefault(m.group(1), {"let_in": 0}).setdefault("refused_full", 0)
        door[m.group(1)]["refused_full"] += 1


def spread(xs):
    return {"n": len(xs), "min": min(xs), "median": statistics.median(xs), "max": max(xs)} if xs else None


for p in door.values():
    for k in ("people", "share_bps", "rtt_ms"):
        if k in p:
            p[k] = spread(p[k])

people, verdicts, every = d.get("people", {}), [], []
for user, p in people.items():
    seeks = p.get("seeks") or []
    for s in seeks:
        every.append({"user": user, "way": p.get("way"), "engine": p.get("engine"), **s})
    p["_missed"] = [s for s in seeks if s.get("ms") is None or s["ms"] > BAR_MS]


def verdict(bar, judged, ok, why):
    verdicts.append({"bar": bar, "verdict": ("passed" if ok else "failed") if judged else "could-not-judge", "why": why})


if d.get("stopped"):
    verdict("the run", False, False, f"stopped: {d['stopped']}")
window = d.get("window_s")
for user, p in people.items():
    w = p.get("window")
    if p.get("stopped") or not w:
        verdict(f"{user} plays through the window", True, False, p.get("stopped") or "never played")
        continue
    ev = w["outside_own_seeks"]
    ok = not w["stills_5s"] and ev["ended"] == 0 and ev["error"] == 0 and p.get("signed_in_after") == p.get("account")
    verdict(f"{user} plays through the window", True, ok,
            f"played {w['played_s']} s of {window}; stills of 5 s or more outside their own seeks {w['stills_5s']}; "
            f"events outside their own seeks {ev}; signed in after as {p.get('signed_in_after')}")
    seeks = p.get("seeks") or []
    verdict(f"{user} seeks", len(seeks) >= 3, not p["_missed"],
            f"{len(seeks)} seeks, {len(p['_missed'])} missed {BAR_MS} ms: {[s.get('ms') for s in seeks]}")

all_ms = [s.get("ms") for s in every]
summary = {
    "verdicts": verdicts,
    "seeks": {"every": every, "count": len(all_ms), "missed_5s": sum(1 for m in all_ms if m is None or m > BAR_MS),
              "spread_ms": spread([m for m in all_ms if m is not None])},
    "per_person": {u: {"way": p.get("way"), "engine": p.get("engine"), "seeks": len(p.get("seeks") or []), "missed_5s": len(p["_missed"]),
                       "spread_ms": spread([s["ms"] for s in p.get("seeks") or [] if s.get("ms") is not None])}
                   for u, p in people.items()},
    "door": door,
    "streams_mid": d.get("streams_mid"), "streams_end": d.get("streams_end"),
    "load": {"start": d.get("load_at_start"), "end": d.get("load_at_end"),
             "at_seeks": spread([s["load1"] for s in every if s.get("load1") is not None])},
    "door_lines_from": since, "door_lines_until": until,
}
path.with_name(path.stem + "-summary.json").write_text(json.dumps(summary, indent=1) + "\n")
for v in verdicts:
    print(f"{v['verdict']:<16} {v['bar']}: {v['why']}")
s = summary["seeks"]
print(f"seeks: {s['count']}, {s['missed_5s']} missed {BAR_MS} ms; {s['spread_ms']}")
for who, p in door.items():
    print(f"door, {who}: {p}")
