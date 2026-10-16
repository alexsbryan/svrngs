#!/usr/bin/env python3
"""One verdict a bar of PREDICTIONS.md §2 (AGENTS.md 6: passed, failed,
could-not-judge, never-ran), read from a take's summary.json; written to
verdicts.json beside it, each with what it read.

    python3 demo/runs/groups/2026-10-16/judge.py <its summary.json>"""
import json, re, sys
from pathlib import Path

LEAVE = ("Leaving Trip stops it reaching you, and stops what you lend reaching its people. What you have of it stays on "
         "this computer, its folder too. Rejoining needs a new invitation, and this computer can’t rejoin yet.")
LEFT = "You left Trip. Rejoining needs a new invitation, and this computer can’t rejoin yet."
HAND = "You keep the order of removals in Trip. Choose who keeps it after you, then leave:"
DECIDES = "You keep the order of removals in Trip: choose who keeps it after you, then leave."
HANDED = "Bo keeps the order of removals in Trip now, so you can leave it."
CANNOT = "You decide who is removed from Trip, so you cannot leave it yet: hand that to someone else first, with `svrngs handover <name> --group Trip`, then leave."
NO_REJOIN = "You’re no longer in Trip on this computer, and you can’t join it again from here yet. Join from your phone or another computer instead."
NO_REJOIN_CMD = "you're no longer in Trip on this computer, and you can't join it again from here yet: join from your phone or another computer instead"
WORDS = re.compile(r"\b(keys?|members?|keepers?|records?|acts?|relays?)\b", re.I)
LENT = {"notes": "Sam's notes", "photos": "Bo's photos", "list": "Ana's list"}


def flat(v):
    if isinstance(v, str): return [v]
    if isinstance(v, list): return [w for x in v for w in flat(x)]
    return []


def text(page, at): return "\n".join(flat((page or {}).get(at)))


def verdict(ok, **read):
    """passed or failed; could-not-judge when what it reads is absent."""
    if ok is None: return {"verdict": "could-not-judge", **read}
    return {"verdict": "passed" if ok else "failed", **read}


def judge(s):
    v = {}
    pages = s.get("pages", [])
    at = lambda step, who: [p for p in pages if p.get("step") == step and p.get("who") == who]
    try:
        leave = s["ana_leave"]
        before, after = text(leave, "all"), text(leave, "after")
        v["1"] = verdict(LEAVE in before and "Leave Trip" in before and HAND not in before and LEFT in after
                         and "Groups you left" in after and f"its folder at {s['folders']['ana']}" in after
                         and "Leave Trip" not in after and leave["ms"] <= 5000,
                         ms=leave["ms"], after=flat(leave.get("after"))[:3])
    except (KeyError, TypeError): v["1"] = verdict(None)
    try:
        h = s["held_out"]
        v["2"] = verdict(all(h[w]["out"] and h[w]["ms"] <= 30_000 for w in ("sam", "bo")),
                         ms={w: h[w]["ms"] for w in h}, out={w: h[w]["out"] for w in h})
    except (KeyError, TypeError): v["2"] = verdict(None)
    try:
        b, a = s["before"], s["after"]
        reached = all(b[k]["status"] == 200 and LENT[k] in str(b[k]["said"]) for k in LENT)
        refused = all(a[k]["status"] != 200 and LENT[k] not in str(a[k]["said"]) for k in LENT)
        v["3"] = verdict(reached and refused and ["Ana", "list"] not in a["sam_lists"] if reached else None,
                         before={k: b[k]["status"] for k in b}, after={k: [a[k]["status"], a[k]["refused"], a[k]["said"]] for k in LENT})
    except (KeyError, TypeError): v["3"] = verdict(None)
    try:
        c = s["copy"]
        v["4"] = verdict(all(c[k]["status"] != 200 and LENT[k] not in str(c[k]["said"]) for k in ("notes", "photos"))
                         and c["left"] and c["left_ms"] <= 60_000,
                         asked={k: [c[k]["status"], c[k]["refused"], c[k]["said"]] for k in ("notes", "photos")},
                         left=c["left"], left_ms=c["left_ms"])
    except (KeyError, TypeError): v["4"] = verdict(None)
    try:
        k, t = s["ana_copy"], s["after_txt"]
        v["5"] = verdict(k["kept"] and k["after"] == k["before"] + 1 and k["folder"] and k["before_txt"]
                         and t["bo"] and t["bo_ms"] <= 60_000 and not t["ana"] if s["before_txt"]["reached"] else None,
                         acts=[k["before"], k["after"]], folder=k["folder"], before_txt=k["before_txt"], after_txt=t)
    except (KeyError, TypeError): v["5"] = verdict(None)
    try:
        lines = {w: [l for p in at("7", w) for l in flat(p.get("all"))] for w in ("sam", "bo")}
        lines["cy"] = s["cy"]["removals"]
        said = {w: [l for l in ls if l.startswith("Ana ")] for w, ls in lines.items()}
        v["6"] = verdict(all(any(l.startswith("Ana left on ") for l in said[w]) for w in said)
                         and not any("removed" in l for ls in said.values() for l in ls)
                         and s["held_out"]["bo"]["ms"] <= 30_000 and s["cy_ms"] <= 30_000,
                         said=said, bo_ms=s["held_out"]["bo"]["ms"], cy_ms=s["cy_ms"])
    except (KeyError, TypeError): v["6"] = verdict(None)
    try:
        k = s["keeper"]
        v["7"] = verdict(HAND in text(k["page"], "all") and DECIDES in text(k["page"], "after") and k["left_after_press"] is False
                         and CANNOT in k["command"]["said"] and k["left_after_command"] is False
                         and HANDED in text(k["hand"], "after") and "You left Trip." in text(k["leave"], "after")
                         and k["bo"]["sam_left"] and k["bo"]["ms"] <= 30_000 and k["bo"]["keeps"] is True,
                         left_after_press=k["left_after_press"], command=[k["command"]["exit"], k["command"]["said"]],
                         bo=k["bo"])
    except (KeyError, TypeError): v["7"] = verdict(None)
    try:
        p, c = s["rejoin_page"], s["rejoin_command"]
        anas = [x for x in s["sam_people_after_rejoin"] if x[0] == "Ana"]
        v["8"] = verdict(NO_REJOIN in text(p, "after") and "Join Trip" not in text(p, "after")
                         and c["exit"] == 1 and NO_REJOIN_CMD in c["said"] and not anas,
                         page=flat(p.get("after"))[-1:], command=[c["exit"], c["said"]], anas_at_sam=anas)
    except (KeyError, TypeError): v["8"] = verdict(None)
    try:
        ph = s["ana_phone"]
        anas = [x for x in s["sam_people_after_phone"] if x[0] == "Ana"]
        sam10 = [l for p in at("10", "sam") for l in flat(p.get("all"))]
        v["9"] = verdict(ph.get("status", "").startswith("You’re in Trip") and ph["status"].endswith(", as Ana") and ph["ms"] <= 30_000
                         and len(anas) == 1 and anas[0][1] != s["computers"]["Ana"]
                         and any(l.startswith("Ana left on ") for l in sam10),
                         status=ph.get("status"), ms=ph.get("ms"), anas_at_sam=anas)
    except (KeyError, TypeError): v["9"] = verdict(None)
    read = [w for p in pages for w in flat(p.get("all")) + flat(p.get("after")) + flat(p.get("text"))
            + flat(p.get("status")) + flat(p.get("removals"))]
    read += [s[k]["said"] for k in ("rejoin_command",) if k in s] + ([s["keeper"]["command"]["said"]] if "keeper" in s else [])
    hits = sorted({m.group(0) + " ⟵ " + w[:120] for w in read for m in [WORDS.search(w)] if m})
    v["10"] = verdict(not hits if read else None, read=len(read), hits=hits)
    v["11"] = {"verdict": "could-not-judge", "people": "never-ran", "why": "one host, programs for people (AGENTS.md 7)"}
    return v


if __name__ == "__main__":
    path = Path(sys.argv[1])
    v = judge(json.loads(path.read_text()))
    (path.parent / "verdicts.json").write_text(json.dumps(v, indent=1, ensure_ascii=False) + "\n")
    for k, x in v.items(): print(k, x["verdict"])
