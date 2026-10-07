"""u6-9's judge: Demo B's dry run on this host, read by what
demo/runs/b/2026-10-06/README.md committed before it ran, from what
screen.sh `night` wrote: each person passed, failed or could-not-judge;
a part with no file never ran. Then the record demo/README.md names, for
a dry run: what a script can count, and null with the reason for what
only the recordings can say.

usage: python3 night.py <out dir> [five's json]    prints the verdicts as JSON;
                                                   exit 0 only if every part ran
"""
import json
import re
import sys
from pathlib import Path

out = Path(sys.argv[1])
five_file = sys.argv[2] if len(sys.argv) > 2 else "night-five.json"
CLOCK = 180_000  # the gate's three minutes, from the link to the film on the TV
WITHIN = 10_000  # u6-2's bar for a phone's pause and seek on the TV


def load(name):
    f = out / name
    if not f.exists():
        return None
    try:
        return json.loads(f.read_text())
    except ValueError as e:
        return {"stopped": f"unreadable: {e}"}


def by(ok, why):
    return {"verdict": "passed" if ok else "failed", "why": why}


def elsewhere(d):
    """The TV's requests off the room address, as screen.py reads them: to
    Sam's Jellyfin by another address — what it reports for itself, or its
    own port — or to anyone else."""
    sam = {"127.0.0.1:8096", "localhost:8096"}
    local = (d.get("kept") or {}).get("LocalAddress")
    if local:
        sam.add(re.sub(r"^\w+://", "", local).rstrip("/"))
    off = (d.get("asked") or {}).get("elsewhere", [])
    host = lambda u: re.sub(r"^\w+://", "", u).split("/")[0]
    return ([e["url"] for e in off if host(e["url"]) in sam], [e["url"] for e in off if host(e["url"]) not in sam])


def person(r):
    if r.get("clock_ms") is None:
        return {"name": r.get("name"), "verdict": "failed", "why": r.get("stopped") or "the TV never played their film"}
    v = {"name": r["name"], "engine": r["engine"], "device": r.get("device"), "film": r["film"]}
    v["clock"] = by(r["clock_ms"] <= CLOCK, f"{r['clock_ms']} ms from the link to {r['film']} on the TV, bar {CLOCK}")
    v["steps_ms"] = {k: r.get(k) for k in ("list_ms", "open_ms", "shown_ms", "signed_in_ms", "film_ms", "play_on_ms", "tv_playing_ms")}
    v["play_method"] = (r.get("session") or {}).get("play_method")
    v["tv_listed_among"] = r.get("play_on_lists")
    if not r.get("remote_control_offered"):
        v["pause_and_seek"] = {"verdict": "failed", "why": f"no Remote Control; the menu then lists {r.get('menu_after_play')}"}
    else:
        steps = {k: (r.get(k) or {}).get("ms") for k in ("paused", "sought", "played_on")}
        v["pause_and_seek"] = by(all(x is not None and x <= WITHIN for x in steps.values()),
                                 f"{steps} ms, bar {WITHIN} each; the length shown on the phone after {r.get('length_shown_ms')} ms "
                                 f"({r.get('length_shown')!r}); the TV at {(r.get('sought') or {}).get('session_at_s')} s by the lender's Jellyfin")
    return v


def users(name):
    d = load(name)
    return d if isinstance(d, list) else None


report = {}
tv = load("night-tv.json")
if tv is None:
    report["tv"] = {"verdict": "never-ran"}
elif tv.get("stopped"):
    report["tv"] = {"verdict": "could-not-judge", "why": tv["stopped"]}
else:
    report["tv"] = by(tv.get("code_shown") and tv.get("tv_signed_in_ms") is not None and tv.get("typed_on_tv") == 0,
                      f"Quick Connect approved on Dee's phone in {tv.get('approved_ms')} ms, the TV signed in {tv.get('tv_signed_in_ms')} ms "
                      f"after, as {(tv.get('kept') or {}).get('signed_in_as')}, nothing typed on it")
five = load(five_file)
if five is None:
    report["five"] = {"verdict": "never-ran"}
elif five.get("stopped"):
    report["five"] = {"verdict": "could-not-judge", "why": five["stopped"]}
else:
    people = [person(r) for r in five.get("people", [])]
    to_sam, other = elsewhere(five)
    report["five"] = {
        "people": people,
        "in_time": sum(1 for p in people if (p.get("clock") or {}).get("verdict") == "passed"),
        "tv_requests_at_room": (five.get("asked") or {}).get("at_room"),
        "tv_to_sam_another_way": by(not to_sam, f"{len(to_sam)} to Sam's Jellyfin another way: {to_sam}"),
        "tv_elsewhere_not_sam": other,
    }
made, after = users("users-made.json"), users("users-after.json")
in_time = (report["five"] or {}).get("in_time") if isinstance(report.get("five"), dict) else None
report["record"] = {
    "demo": "b", "dry": True, "participants": len((five or {}).get("people", [])),
    "succeeded_in_time": in_time,
    "accounts_created": len(set(after) - set(made)) if made is not None and after is not None else None,
    "key_prompts": None, "key_prompts_why": "a script answers only what it is asked; the recordings say what a phone asked",
    "assisted": None, "assisted_why": "a script is not a person; there was no one to help",
}
print(json.dumps(report, indent=1))
sys.exit(0 if all(isinstance(v, dict) and v.get("verdict") != "never-ran" for v in (report["tv"], report["five"])) else 1)
