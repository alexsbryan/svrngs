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

A run through Jellyfin's middleware (drive.mjs `mw`, mw-2) is read
against mw-0's bars instead (demo/runs/mw/2026-10-08/README.md §3):

1. no sign-in — each person, from the list to a playing film, is never
   asked "Please sign in" and types nothing; and seeks, as 2 above.
2. no lender's key — no byte either browser received holds the key the
   lender gave the middleware, the lender's password or token, a password,
   or the other person's token (scan.py); whether a person's bytes hold
   their own token is recorded, not judged.
3. apart — while one seeks the other is left alone (drive.mjs `judge`);
   afterwards each is still signed in as an account of their own, the one
   the middleware named for them (<their name>-<16 hex>); and Jellyfin
   holds the device jellyfin-web names (its `_deviceId2`) in a session of
   that account's, the two people's devices apart. Through the sign-in
   (mwp-3) that device is the person's own, which its script names.

Each bar is passed, failed, or could-not-judge (what it reads never
happened, as seeks when nobody played); a run with no file never ran.

usage: python3 judge.py <out dir> <run>...    exit 0 only if every bar of every run passed
"""
import json
import re
import sys
from pathlib import Path

PASSED, FAILED, COULD_NOT = "passed", "failed", "could-not-judge"
out, runs = Path(sys.argv[1]), sys.argv[2:]
scan = json.loads((out / "scan.json").read_text()) if (out / "scan.json").exists() else {}


def seeks(d):
    phases = [d[k] for k in ("first_seeks", "then_seeks") if d.get(k)]
    if len(phases) < 2:
        return phases, (COULD_NOT, "the two did not both play, so nobody seeked")
    all_ms = {ph["seeker"]: [s["ms"] for s in ph["seeks"]] for ph in phases}
    slow = {u: ms for u, ms in all_ms.items() if any(m is None or m > 5000 for m in ms)}
    return phases, (FAILED if slow else PASSED, f"{all_ms} ms")


def scanned(run):
    """scan.py's word on `run`, read the one way both bars read it: each
    person's, whose bytes held another's secret (and where), and every
    secret searched for. None when scan.py found no dump for it."""
    seen = scan.get("pairs", {}).get(run)
    if not seen:
        return None
    held = {u: s["others_secrets_in"] for u, s in seen.items() if not s["holds_none_of_anothers"]}
    searched = sorted(set(scan.get("secrets_searched", [])) | set(scan.get("lend_kept", [])))
    return seen, held, searched


def mw_bars(run, d):
    people = d.get("people", {})
    v = {}
    # 1. no sign-in
    if not people:
        v["no sign-in"] = (FAILED, d.get("stopped", "nobody entered"))
    else:
        def why(p):
            if p.get("asked"):
                return "asked to sign in"
            if p.get("play_ms") is None:
                return p.get("stopped", "no play")
            return None
        missed = [f"{u}: {why(p)}" for u, p in people.items() if why(p)]
        v["no sign-in"] = (FAILED, "; ".join(missed)[:300]) if missed else (
            PASSED, ", ".join(f"{u} ({p['name']}) open {p['open_ms']} + play {p['play_ms']} ms, never asked, "
                              f"stored token: {p.get('stored', {}).get('token')}" for u, p in people.items()))
    phases, v["seeks"] = seeks(d)
    # 2. no lender's key
    read = scanned(run)
    if not read:
        v["no lender's key"] = (COULD_NOT, "scan.py found no dump for this run")
    else:
        seen, held, searched = read
        own = {u: f"{s['own_token']} in {s['own_token_in']}" for u, s in seen.items()}
        v["no lender's key"] = (FAILED, f"held: {json.dumps(held)[:300]}") if held else (
            PASSED, f"none of {searched}, nor the other's token, in "
                    f"{sum(s['scanned']['bytes'] for s in seen.values())} bytes; recorded, their own token: {own}")
    # 3. apart
    after = {u: p.get("signed_in_after") for u, p in people.items()}
    own_account = {u: bool(re.fullmatch(re.escape(p.get("name", "")) + r"-[0-9a-f]{16}", str(after[u])))
                   for u, p in people.items()}
    sessions = {u: p.get("sessions") for u, p in people.items()}
    in_own = {u: bool(sessions[u]) and all(s["user"] == after[u] for s in sessions[u]) for u in people}
    watched = {ph["watched"]: ph["watch"]["left_alone"] for ph in phases if ph.get("watch")}
    told = (f"signed in after: {after}; sessions on their own device: {sessions}; "
            f"left alone while the other seeked: {watched}; devices apart: {d.get('devices_apart')}")
    outside = [u for u, p in people.items() if p.get("signed_in_ms") is None or p.get("play_ms") is None]
    if not all(watched.values()):
        v["apart"] = (FAILED, told)
    elif outside or len(people) < 2:
        # Nobody to keep apart: a person who never got in, or never played.
        v["apart"] = (COULD_NOT, f"{outside} never got to a playing film; {told}")
    elif (not all(own_account.values()) or len(set(after.values())) < len(after) or not all(in_own.values())
            or d.get("devices_apart") is False):
        v["apart"] = (FAILED, told)
    elif len(watched) < 2:
        v["apart"] = (COULD_NOT, told)
    else:
        v["apart"] = (PASSED, told)
    return v


def bars(run, d):
    if d.get("mw"):
        return mw_bars(run, d)
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
    phases, v["seeks"] = seeks(d)
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
    read = scanned(run)
    if not read:
        v["no secret"] = (COULD_NOT, "scan.py found no dump for this run")
    else:
        seen, held, searched = read
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
