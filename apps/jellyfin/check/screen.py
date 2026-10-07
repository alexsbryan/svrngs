"""u6-2's judge: each part of demo/runs/u6/2026-10-06-jellyfin-screen/README.md
read by the bar committed there before any run, from what screen.sh wrote:
passed, failed or could-not-judge; a part with no file never ran. Then
the logs beside it: the room door's requests and refusals, the lend's
requests by the person Sam's door named, and the screen's session as
Jellyfin saw it.

usage: python3 screen.py <out dir>    prints the verdicts as JSON; exit 0 only if every part ran
"""
import json
import re
import sys
from pathlib import Path

out = Path(sys.argv[1])
WITHIN = 10_000  # (c)'s bar, ms from the phone's tap to the screen


def load(name):
    f = out / f"{name}.json"
    if not f.exists():
        return None
    try:
        return json.loads(f.read_text())
    except ValueError as e:
        return {"stopped": f"unreadable: {e}"}


def ms(step):
    return (step or {}).get("ms")


def by(ok, why):
    return {"verdict": "passed" if ok else "failed", "why": why}


def elsewhere(d):
    """The screen's requests off the room address: to Sam's Jellyfin by
    another address — what it reports for itself, or its own port — or to
    anyone else."""
    sam = {"127.0.0.1:8096", "localhost:8096"}
    local = (d.get("kept") or {}).get("LocalAddress")
    if local:
        sam.add(re.sub(r"^\w+://", "", local).rstrip("/"))
    off = (d.get("asked") or {}).get("elsewhere", [])
    host = lambda u: re.sub(r"^\w+://", "", u).split("/")[0]
    return ([e["url"] for e in off if host(e["url"]) in sam], [e["url"] for e in off if host(e["url"]) not in sam])


def judge(name, d):
    if d is None:
        return {"verdict": "never-ran"}
    if d.get("stopped"):
        return {"verdict": "could-not-judge", "why": d["stopped"]}
    to_sam, other = elsewhere(d)
    v = {"at_room": (d.get("asked") or {}).get("at_room"), "to_sam_another_way": to_sam, "elsewhere_not_sam": other}
    if name == "a":
        s = d.get("screen", {})
        seeks = [x.get("ms") for x in d.get("seeks", [])]
        v["typed_once"] = by(s.get("signed_in_ms") is not None and d.get("play_ms") is not None and d.get("within_5s") == 3 and len(seeks) == 3,
                             f"signed in {s.get('signed_in_ms')} ms, playing {d.get('play_ms')} ms, seeks {seeks} ms, bar 5,000 each")
        v["every_request_at_room"] = by(not to_sam and not other, f"{len(to_sam)} to Sam's Jellyfin another way, {len(other)} elsewhere")
        v["play_method"] = (d.get("session") or {}).get("play_method")
    elif name == "b":
        v["quick_connect"] = by(d.get("code_shown") and d.get("approved_ms") is not None and d.get("screen_signed_in_ms") is not None
                                and d.get("typed_on_screen") == 0,
                                f"offered {d.get('screen', {}).get('quick_connect_offered')} (enabled as it came: {d.get('quick_connect_enabled')}), "
                                f"approved on the phone in {d.get('approved_ms')} ms, the screen signed in {d.get('screen_signed_in_ms')} ms after, "
                                f"as {(d.get('kept') or {}).get('signed_in_as')}, nothing typed on it")
    elif name == "control":
        v["set"] = by(d.get("before") is False and d.get("after") is True, f"{d.get('label')!r}: {d.get('before')} then {d.get('after')}")
    elif name.startswith("c"):
        v["phone_as"], v["direct"] = d.get("phone_as"), d.get("direct")
        v["listed"] = by(bool(d.get("screen_listed")), f"Play on lists {d.get('play_on_lists')}")
        if d.get("screen_listed"):
            v["started"] = by((ms(d.get("started")) or WITHIN + 1) <= WITHIN, f"{ms(d.get('started'))} ms from the tap, bar {WITHIN}")
            rc = d.get("remote_control_offered")
            why = (f"the button then reads {d.get('header_after_play')!r}, its menu {d.get('menu_after_play')}"
                   if not rc else None)
            for step in ("paused", "sought", "played_on"):
                v[step] = by(rc and (ms(d.get(step)) or WITHIN + 1) <= WITHIN, why or f"{ms(d.get(step))} ms, bar {WITHIN}")
    elif name.startswith("d"):
        k = d.get("kept") or {}
        v["kept_to_room"] = by(not to_sam and k.get("serverAddress", "").rstrip("/") == d["room"].rstrip("/"),
                               f"kept {k.get('ManualAddress')} as typed and {k.get('LocalAddress')} as Jellyfin's own; "
                               f"asks the room at {k.get('serverAddress')}; {len(to_sam)} to Sam's Jellyfin another way")
        if "bounced_at" in d:
            v["after_the_door_stopped"] = by(not to_sam and k.get("signed_in_as") == "dee",
                                             f"the door back {d.get('door_back_at', 0) - d['bounced_at']} ms after it stopped; "
                                             f"{d.get('sockets_after_bounce')} socket opened again, signed in as {k.get('signed_in_as')}")
    return v


def logs():
    room = (out / "dee-room.log").read_text() if (out / "dee-room.log").exists() else ""
    sam = (out / "sam-run.log").read_text() if (out / "sam-run.log").exists() else ""
    people = {}
    for line in sam.splitlines():
        m = re.search(r'person=(\S+).*lend=jellyfin', line)
        if m and ("let in" in line or "guessed in this computer's home" in line):
            people[m.group(1)] = people.get(m.group(1), 0) + 1
    sessions = json.loads((out / "jellyfin-sessions.json").read_text()) if (out / "jellyfin-sessions.json").exists() else []
    return {
        "room_door": {"taken": room.count("room: a caller on this network"), "refused": room.count("room: refused"),
                      "callers": sorted(set(re.findall(r"caller=Some\(([0-9.]+)", room)))},
        "sams_door_jellyfin_by_person": people,
        "sessions_remote_end": sorted({f"{s.get('UserName')}:{s.get('DeviceName')}:{s.get('RemoteEndPoint')}" for s in sessions}),
    }


names = ["a", "b"] + sorted(f.stem for f in out.glob("c*.json")) + ["control"] + sorted(f.stem for f in out.glob("d*.json"))
report = {n: judge(n, load(n)) for n in dict.fromkeys(names)}
report["logs"] = logs()
print(json.dumps(report, indent=1))
sys.exit(0 if all(v.get("verdict") != "never-ran" for k, v in report.items() if k != "logs") else 1)
