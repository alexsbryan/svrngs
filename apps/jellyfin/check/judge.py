"""u5-4's bars (ralph/next/u5/STATE.md, its row's check), each run named
read against them: drive.mjs's pair-<run>.json, and scan.py's scan.json.

1. list to playing — each person, new to Jellyfin, gets from the list (the
   bare command's address on a computer, the page's list on a phone) to a
   playing film, asked to sign in and signing in once with the account the
   lender made.
2. seeks — every seek through jellyfin-web's own seek bar plays again
   within 5 s (Demo A's bar): the first person's five, the second's three.
3. left alone — while one seeks, the other's play is never started over,
   sent back or held (drive.mjs `judge`), and each is still signed in as
   themselves afterwards: no one's sign-in ended another's.
4. no secret — no byte either browser received holds a secret the lend
   keeps, or the lender's, or the other person's (scan.py).

Each bar is passed, failed, or could-not-judge (what it reads never
happened, as seeks when nobody played); a run with no file never ran.

usage: python3 judge.py <out dir> <run>...    exit 0 only if every bar of every run passed
"""
import json
import sys
from pathlib import Path

PASSED, FAILED, COULD_NOT = "passed", "failed", "could-not-judge"
out, runs = Path(sys.argv[1]), sys.argv[2:]
scan = json.loads((out / "scan.json").read_text()) if (out / "scan.json").exists() else {}


def bars(run, d):
    people = d.get("people", {})
    v = {}
    # 1. list to playing
    if not people:
        v["list to playing"] = (FAILED, d.get("stopped", "nobody entered"))
    else:
        def why(p):
            if p.get("play_ms") is None or p.get("signed_in_ms") is None:
                return p.get("stopped", "no play")
            return None if p.get("asked") else "never asked to sign in: a stored sign-in, not one typed"
        missed = [f"{u}: {why(p)}" for u, p in people.items() if why(p)]
        v["list to playing"] = (FAILED, "; ".join(missed)[:300]) if missed else (
            PASSED, ", ".join(f"{u} open {p['open_ms']} + sign in {p['signed_in_ms']} + play {p['play_ms']} ms" for u, p in people.items()))
    # 2. seeks
    phases = [d[k] for k in ("first_seeks", "then_seeks") if d.get(k)]
    if len(phases) < 2:
        v["seeks"] = (COULD_NOT, "the two did not both play, so nobody seeked")
    else:
        all_ms = {ph["seeker"]: [s["ms"] for s in ph["seeks"]] for ph in phases}
        slow = {u: ms for u, ms in all_ms.items() if any(m is None or m > 5000 for m in ms)}
        v["seeks"] = (FAILED if slow else PASSED, f"{all_ms} ms")
    # 3. left alone
    after = {u: p.get("signed_in_after") for u, p in people.items()}
    ended = {u: a for u, a in after.items() if a != people[u].get("account", u)}
    watched = {ph["watched"]: ph["watch"]["left_alone"] for ph in phases if ph.get("watch")}
    if ended or not all(watched.values()):
        v["left alone"] = (FAILED, f"signed in after: {after}; left alone while the other seeked: {watched}")
    elif len(watched) < 2:
        v["left alone"] = (COULD_NOT, f"signed in after: {after}; nobody seeked while the other played")
    else:
        v["left alone"] = (PASSED, f"signed in after: {after}; left alone while the other seeked: {watched}")
    # 4. no secret
    seen = scan.get("pairs", {}).get(run)
    if not seen:
        v["no secret"] = (COULD_NOT, "scan.py found no dump for this run")
    else:
        held = {u: s["others_secrets_in"] for u, s in seen.items() if not s["holds_none_of_anothers"]}
        searched = sorted(set(scan.get("secrets_searched", [])) | set(scan.get("lend_kept", [])))
        v["no secret"] = (FAILED, f"held: {json.dumps(held)[:300]}") if held else (
            PASSED, f"none of {searched}, nor the other's token, in {sum(s['scanned']['bytes'] for s in seen.values())} bytes")
    return v


ok = bool(runs)
for run in runs:
    f = out / f"pair-{run}.json"
    try:
        d = json.loads(f.read_text())
    except (OSError, ValueError) as e:
        print(f"never-ran        {run}: {e}")
        ok = False
        continue
    for bar, (verdict, why) in bars(run, d).items():
        print(f"{verdict:<16} {run} · {bar}: {why}")
        ok = ok and verdict == PASSED
print("all passed" if ok else "not all passed")
sys.exit(0 if ok else 1)
