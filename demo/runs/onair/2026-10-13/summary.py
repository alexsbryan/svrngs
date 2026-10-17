#!/usr/bin/env python3
"""Every number README.md says of the whole run, read from data/ and
printed, so each can be found in what the harness kept:

    python3 demo/runs/onair/2026-10-13/summary.py > demo/runs/onair/2026-10-13/data/summary.txt
"""
import hashlib, json, re
from pathlib import Path

D = Path(__file__).resolve().parent / "data"
jl = lambda f: [json.loads(l) for l in (D / f).read_text().splitlines() if l.strip()]
v, e, ev = json.loads((D / "verdicts.json").read_text()), json.loads((D / "evidence.json").read_text()), jl("events.jsonl")
st = jl("station.jsonl")
pieces = [r for r in st if r["event"] == "piece"]
ended = next(r for r in st if r["event"] == "ended")

print(f"run: commit {ev[0]['commit']}, {ev[-1]['t'] - ev[0]['t']:.0f} s start to end, load {ev[0]['load']} at start, {ev[-1]['load']} at end")
for k, x in v.items(): print(f"verdict {k}: {x['verdict']}")
cut = next(r for r in st if r["event"] == "cut")
print(f"sound: {e['sound']['made_by']}; {cut['bytes']} bytes, {cut['pieces']} pieces, {cut['seconds']} s")
print(f"pieces: {[r['bytes'] for r in pieces]}, sum {sum(r['bytes'] for r in pieces)}")
print(f"station: {e['station']['s']} s on air; puts {sorted({r['put']['status'] for r in pieces})}, signs {sorted({r['sign']['status'] for r in pieces + [ended]})}, "
      f"lives delivered {sorted({r['live']['delivered'] for r in pieces + [ended]})}, peers {sorted({p for r in pieces for p in r['live']['peers']})}")
held = v["4.1"]["held_ms"]
print(f"4.1: bound {v['4.1']['bound_ms']} ms; Bo held_ms {held}; min {min(held.values())} max {max(held.values())}")
cy = {r["n"]: r["ms"] for r in jl("cy-listen.jsonl") if r["event"] == "held"}
print(f"4.1: Cy held_ms {cy}; min {min(cy.values())} max {max(cy.values())}")
for w in ("bo", "cy"):
    tr = (D / f"{w}.trace").read_text()
    frm = re.findall(r"folder: fetched, verified hash=\w+ holder=(\w{8})", tr)
    print(f"{w}: fetched from {dict((k, frm.count(k)) for k in sorted(set(frm)))}")
names = {p["key"][:8]: n for n, p in e["people_joined"].items()}
print(f"keys: {names}")
for w in ("ana", "bo", "cy"):
    sent = [int(x) for x in re.findall(r"folder: sent .*?done=true .*?payload=(\d+)", (D / f"{w}.trace").read_text())]
    print(f"{w} sent: {len(sent)} gets done, {sum(sent)} bytes")
c = e["carried"]
print(f"carried: Cy's Y {c['cy_get_Y']['status']} {c['cy_get_Y']['length']} bytes in {c['cy_get_Y']['ms']} ms; Z {c['cy_get_Z']['status']} in {c['cy_get_Z']['ms']} ms")
l = e["lapses"]
print(f"lapses: {l['s_after_last_put']} s after the last put; {dict((k, a['status']) for k, a in l.items() if isinstance(a, dict) and 'status' in a)}")
print(f"lapses: stores {dict((k, len(x)) for k, x in l['stores'].items())}; Ana restarted, none left after {l['ana_restart']['s']} s")
print(f"scoped: Dee {e['scoped']['dee_get_Y']['status']} {e['scoped']['dee_get_Y']['said']}")
tab = e["page"]["tabs"][0]
print(f"page: played {[p['n'] for p in tab['air']['played']]}, refused {tab['air']['refused']}, said {tab['stopped']!r} {tab['air']['why']!r}, errors {tab['errors']}")
for t in e["plant_pages"]["tabs"]:
    a = t["air"]; s = re.search(r"show=([\w-]+)", t["link"])[1]
    print(f"plant {s}: played {[p['n'] for p in a['played']]}, refused {[(r['n'], r['why']) for r in a['refused']]}, said {t['stopped']!r} {a['why']!r}, errors {t['errors']}")
said = {w: next(r for r in jl(f"{w}-listen.jsonl") if r["event"] == "ended")["t"] for w in ("bo", "cy")}
print(f"off air: the station's ended at {ended['t']}; Bo +{said['bo'] - ended['t']} ms, Cy +{said['cy'] - ended['t']} ms, page +{tab['said_at'] - ended['t']} ms; one piece is {4000} ms")
for w in ("bo", "cy"):
    hm = D / f"{w}-heard.mp3"
    if hm.exists(): print(f"{w}-heard.mp3: {hm.stat().st_size} bytes, sha256 {hashlib.sha256(hm.read_bytes()).hexdigest()[:16]}")
sent = sum(int(x) for x in re.findall(r"folder: sent .*?done=true .*?payload=(\d+)", (D / "ana.trace").read_text()))
print(f"ana upload: {sent} bytes over {e['station']['s']} s on air = {sent * 8 / e['station']['s'] / 1000:.0f} kbit/s, "
      f"{sent / cut['bytes']:.2f} times the show, for 2 listeners and Y")
rows = [l.split() for l in (D / "lines.txt").read_text().splitlines() if re.match(r"\s*\d+\s+\d+\s+\w{7}", l)]
at = {r[2]: (int(r[0]), int(r[1])) for r in rows}
for name, a, b in (("4.3, onair-live", "38ab822", "b3bfc0f"), ("4.2 core, onair-sign (b)", "f61b252", "57dbade"),
                   ("4.2 route, onair-sign (c)", "57dbade", "c03b6db"), ("4.1, onair-blob (a)", "d0347ae", "db6ba51"),
                   ("the three, e114b96 to db6ba51", "e114b96", "db6ba51")):
    print(f"lines {name}: core {at[b][0] - at[a][0]:+}, repository {at[b][1] - at[a][1]:+}")
print(f"lines left at HEAD: {16287 - at['43724d7'][1]} of the ceiling 16,287, core {3182 - at['43724d7'][0]} of 3,182")
hashes = {r["hash"] for r in pieces}
fetched = [m for w in ("bo", "cy") for m in re.findall(r"folder: fetched, verified hash=(\w+) holder=(\w{8})", (D / f"{w}.trace").read_text())]
of_pieces = [names.get(k) for h, k in fetched if h in hashes]
print(f"piece fetches by Bo and Cy: {len(of_pieces)}, from {dict((n, of_pieces.count(n)) for n in sorted(set(of_pieces)))}")
