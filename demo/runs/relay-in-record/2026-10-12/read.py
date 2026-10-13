#!/usr/bin/env python3
"""What the README of relay-n0-off's run reads from its data (run.py's):
each computer's dials that had no address, by whom they dialled; the
folder's fetches, by holder; the door's asks of the ring; any n0 name in
any trace; and every remote address lsof saw, against n0's.

    W=<the run's scratch> python3 demo/runs/n0-off/2026-10-12/read.py
                                     writes data/end.json (from W), data/read.json"""
import glob, gzip, json, os, re
from collections import Counter
from pathlib import Path

DATA = Path(__file__).resolve().parent / "data"
summary = json.loads((DATA / "summary.json").read_text())
people = {p.get("key") or p.get("person_key"): p for p in (summary.get("people") or {}).get("people", [])}


def who(text):
    """The person a key or a key's start in `text` belongs to."""
    for key, p in people.items():
        if key and any(k and (k in text) for k in (key, key[:10], key[:5])):
            return p.get("person") or p.get("name")
    return None


W = os.environ.get("W")
if W:
    end = {}
    for w in ("sam", "ana", "alex"):
        log = glob.glob(f"{W}/{w}/.svrngs/rings/*/ring_oplog.jsonl")[0]
        end[w] = {"acts": [json.loads(l)["id"][:8] for l in open(log) if l.strip()],
                  "folder": sorted(os.listdir(f"{W}/{w}/svrngs/house"))}
    (DATA / "end.json").write_text(json.dumps(end, indent=1))
out = {"people": {k: (p.get("person") or p.get("name")) for k, p in people.items()}}
for name in ("sam", "ana", "alex", "relay"):
    f = DATA / f"{name}.log.gz"
    if not f.exists(): continue
    lines = gzip.open(f, "rt", errors="replace").read().splitlines()
    noaddr = [l for l in lines if "No addressing information" in l or "no dialable path" in l]
    fetched = [l for l in lines if "folder: fetched, verified" in l]
    gave_nothing = [l for l in lines if "gave nothing of what the store lacks" in l]
    asks = [l for l in lines if re.search(r"AskRound|ask(ed)? the ring|asks? the group", l)]
    rounds = Counter(re.search(r"peers_reached=\d+ peers_unreachable=\d+", l)[0] for l in lines
                     if "ring sync: round namespaces" in l)
    inbound = Counter(who(l) for l in lines if "the asker stands in the ring" in l)
    n0 = [l for l in lines if re.search(r"iroh\.link|pkarr|n0_dns|relay\.n0", l)]
    out[name] = {
        "lines": len(lines),
        "no_address": len(noaddr),
        "no_address_first": noaddr[:3],
        "no_address_to": Counter(who(l) for l in noaddr),
        "rounds": rounds, "asked_by": inbound,
        "fetched": [(who(l), l[-260:]) for l in fetched],
        "gave_nothing": [l[-300:] for l in gave_nothing[:5]],
        "asks": [l[-300:] for l in asks[:8]],
        "n0_lines": n0[:5], "n0_count": len(n0),
    }

n0_ips = {ip for ips in summary.get("n0", {}).values() for ip in ips}
seen = [json.loads(l) for l in (DATA / "lsof.jsonl").read_text().splitlines()] if (DATA / "lsof.jsonl").exists() else []
remotes = Counter((s["who"], s["remote"].rsplit(":", 1)[0].strip("[]")) for s in seen if s["remote"])
out["lsof"] = {
    "samples_kept": len(seen),
    "remote_hosts_by_process": sorted([list(k) + [n] for k, n in remotes.items()]),
    "n0_hits": [s for s in seen if s["remote"] and s["remote"].rsplit(":", 1)[0].strip("[]") in n0_ips],
    "n0_ips": sorted(n0_ips),
}
(DATA / "read.json").write_text(json.dumps(out, indent=1, default=str))
print(json.dumps(out, indent=1, default=str)[:6000])
