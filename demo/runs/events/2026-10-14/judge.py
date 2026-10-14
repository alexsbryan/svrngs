#!/usr/bin/env python3
"""Each prediction of PREDICTIONS.md §2 judged from one run's summary.json
(run.py here), by AGENTS.md 6's verdicts: passed, failed, could-not-judge
(the run stopped short of what the bar reads), never-ran. Writes
verdicts.json beside the summary and prints it; exits 1 when any bar
failed or could not be judged, so a plant's run is red on the bar it
plants.

    python3 demo/runs/events/2026-10-14/judge.py data/summary.json"""
import json, re, sys
from collections import Counter
from pathlib import Path

FIELDS = {"event": {"kind", "event", "what", "when", "where"}, "coming": {"kind", "event", "guests"}, "came": {"kind", "event", "guest"}}
NOT_SAID = re.compile(r"\b(key|member|record|act|namespace|origin)\b", re.I)
REFUSAL = re.compile(r"\b(not coming|declined?|no-show|won't come|didn't come|isn't coming|absent)\b", re.I)
# What each plant is caught by (run.py PLANTS).
PLANTED = {"cross": ["6"], "guest": ["3", "9"], "no": ["8"]}

path = Path(sys.argv[1]); s = json.loads(path.read_text())
views, presses = s.get("views", []), s.get("presses", [])
verdicts = {}


def bar(n, what):
    """Judge bar `n`: `what()` gives [(check, held, evidence)]; a missing
    datum is could-not-judge, never a pass."""
    try:
        checks = what()
        verdict = "passed" if all(ok for _, ok, _ in checks) else "failed"
        verdicts[n] = {"verdict": verdict, "checks": [{"check": c, "held": ok, "evidence": e} for c, ok, e in checks]}
    except (KeyError, IndexError, TypeError, StopIteration) as e:
        verdicts[n] = {"verdict": "could-not-judge", "why": f"the run holds no {e!r}"}


groups = s.get("groups", {}); H, D = groups.get("The House"), groups.get("dWeb")
E = s.get("event", {}).get("id")
copies = s.get("copies", {}).get("folded", [])
sams = {c["group"]: c for c in copies if c.get("computer") == "sam"}
def view(who, step): return next(v for v in views if v["who"] == who and v["step"] == step)
def ev(group): return sams[group]["events"][E]
def acts(group): return sams[group]["acts"]
def voided(group, act):
    """Whether `act` was taken back by its own person's correction."""
    return any(a["corrects"] == act["id"] and a["by"] == act["by"] for a in acts(group))
def live(group, by, kind):
    return [a for a in acts(group) if a["op"] and a["by"] == by and a["op"]["kind"] == kind and not voided(group, a)]
def phone(who): return who in ("cy", "eve")
def bare(sections): return [{k: v for k, v in x.items() if k not in ("group", "unread")} for x in sections]
# A page's sections in the order this computer lists its groups (/v1/groups,
# by id): each read by its group, never by its place.
def sec(who, step, group): return next(x for x in view(who, step)["sections"] if x["group"] == group)
def by_group(sections): return {x["group"]: {k: v for k, v in x.items() if k not in ("group", "unread")} for x in sections}


bar("1", lambda: [
    ("one event, under one id, in each group's record, by Sam",
     all([a["by"] for a in acts(g) if a["op"] and a["op"]["kind"] == "event"] == ["Sam"] for g in (H, D)) and ev(H)["host"] == ev(D)["host"] == "Sam",
     {g[:8]: [a["by"] for a in acts(g) if a["op"] and a["op"]["kind"] == "event"] for g in (H, D)}),
    ("Sam's page lands on the event, in a group it was posted to, within 5 s",
     any(f"/g/{g}/e/{E}" in s["event"]["landed"] for g in (H, D)) and s["event"]["ms"] <= 5000,
     {"ms": s["event"]["ms"], "landed_in": "The House" if f"/g/{H}/" in s["event"]["landed"] else "dWeb"}),
])

def answered():
    out = []
    for p in presses:
        if p["absent"] or p["who"] == "sam": continue
        bound = 15000 if phone(p["who"]) else 5000
        out.append((f"{p['who']}'s \"{p['button']}\" answered within {bound // 1000} s", p["status"] == 200 and p["ms"] <= bound, [p["status"], p["ms"]]))
    for who, group in (("cy", H), ("eve", D)):
        o = next(v for v in views if v["who"] == who and v["step"] == "3")
        out.append((f"{who}'s phone opens the event at #open= within 30 s", bool(o["sections"]) and o["ms"] <= 30000, o["ms"]))
    by = {g: sorted({a["by"] for a in acts(g)}) for g in (H, D)}
    out.append(("Bo's, Cy's and Fay's acts in The House, Di's and Eve's in dWeb", by[H] == ["Bo", "Cy", "Fay", "Sam"] and by[D] == ["Di", "Eve", "Sam"], by))
    return out
bar("2", answered)

bar("3", lambda: [
    ("Bo's live coming names Maya", [a["op"].get("guests") for a in live(H, "Bo", "coming")] == [["Maya"]], [a["op"] for a in live(H, "Bo", "coming")]),
    ("Maya's came is Bo's, and only Bo's", [a["by"] for a in acts(H) if a["op"] and a["op"].get("guest") == "Maya"] == ["Bo"],
     [a["by"] for a in acts(H) if a["op"] and a["op"].get("guest") == "Maya"]),
    ("no \"Maya came\" offered on Cy's page", s["cy_maya"].get("absent") is True, s["cy_maya"].get("buttons")),
    ("Sam's view says Bo + Maya, and Maya, Bo's guest", "Bo + Maya" in sec("sam", "10", H)["coming"]
     and "Maya, Bo's guest" in sec("sam", "10", H)["came"], sec("sam", "10", H)),
])

def mind():
    bo = [a for a in acts(H) if a["op"] and a["by"] == "Bo" and a["op"]["kind"] == "coming" and a["id"] != (a.get("corrects") or "")]
    eve = [a for a in acts(D) if a["op"] and a["by"] == "Eve" and a["op"]["kind"] == "coming"]
    # Every page read once she had taken it back (step 5 on).
    eve_seen = [(v["who"], v["step"], k) for v in views if v["step"] not in ("1", "2", "3") for x in v.get("sections", [])
                for k in ("coming", "came") if any("Eve" in n for n in x[k])]
    return [
        ("Bo's first coming taken back by his own correction", len(bo) >= 1 and voided(H, bo[0]), [[a["op"], voided(H, a)] for a in bo]),
        ("Eve's coming taken back by her own correction", len(eve) == 1 and voided(D, eve[0]), [[a["op"], voided(D, a)] for a in eve]),
        ("Eve on no page but as invited", not eve_seen, eve_seen),
    ]
bar("4", mind)

bar("5", lambda: [("Sam's view: both groups, invited, coming and came", by_group(view("sam", "10")["sections"]) == {
    H: {"name": "The House", "invited": ["Bo", "Cy", "Fay"], "coming": ["Bo + Maya", "Cy", "Fay"], "came": ["Bo", "Cy", "Maya, Bo's guest"]},
    D: {"name": "dWeb", "invited": ["Di", "Eve"], "coming": ["Di"], "came": ["Di"]}}, bare(view("sam", "10")["sections"]))])

def own():
    out = []
    for who, group in (("bo", H), ("cy", H), ("fay", H), ("di", D), ("eve", D)):
        secs = view(who, "10")["sections"]
        out.append((f"{who}'s view: their own group's alone", [x["group"] for x in secs] == [group], [x["name"] for x in secs]))
    for who, other in (("cy", "dWeb"), ("di", "The House")):
        v = view(who, "10x")
        out.append((f"{who} refused {other}'s address of it", v["status"] == 403 and f"You're not in {other}" in v["text"] and not v["sections"], [v["status"], v["text"][:80]]))
    return out
bar("6", own)

def same():
    out = []
    for g, cs in ((H, ["sam", "bo", "fay"]), (D, ["sam", "di"])):
        folds = {c["computer"]: [{k: v for k, v in c["events"][E].items()}, [x["code"] for x in c["gaps"]]] for c in copies if c.get("group") == g}
        out.append((f"{'The House' if g == H else 'dWeb'}: every copy folds the same", sorted(folds) == sorted(cs) and all(f == folds["sam"] for f in folds.values()),
                    {k: [v[0]["coming"], v[0]["came"], v[1]] for k, v in folds.items()}))
        out.append(("within 30 s of the last act", s["copies"]["held"][g]["same_ms"] is not None and s["copies"]["held"][g]["same_ms"] <= 30000, s["copies"]["held"][g]))
    out.append(("Di's copy holds what it slept through within 60 s of waking", (s["di_woke"]["caught_up_ms"] or 10**9) <= 60000, s["di_woke"]))
    return out
bar("7", same)

def no():
    odd = [(g[:8], a["by"], a["op"]) for g in (H, D) for a in acts(g) if a["op"] and (a["op"].get("kind") not in FIELDS or set(a["op"]) - FIELDS[a["op"]["kind"]])]
    said = sorted({m.group(0) for v in views for m in REFUSAL.finditer(v.get("text", ""))})
    return [("every payload one of three kinds, saying only what the fold reads", not odd, odd),
            ("no page says a refusal", not said, said)]
bar("8", no)

def theirs():
    """Each act matched to a press by the person it names, in its group."""
    def key(p):
        b = p["button"]
        if b == "Post": return "event"
        if b == "I'm coming": return "coming"
        if b in ("Save", "Take my name off"): return "correct"
        if b == "I came": return "came"
        if b.endswith(" came"): return "came:" + b[:-5]
    names = {"sam": "Sam", "bo": "Bo", "cy": "Cy", "di": "Di", "eve": "Eve", "fay": "Fay"}
    out = []
    for g in (H, D):
        pressed = Counter((names[p["who"]], key(p)) for p in presses if not p["absent"] and p["status"] == 200 and g in p["group"].split(","))
        written = Counter((a["by"], "correct" if a["corrects"] else a["op"]["kind"] + (":" + a["op"]["guest"] if a["op"].get("guest") else ""))
                          for a in acts(g))
        # A press of "I'm coming" on a taken-back answer writes a correction in its place.
        unpressed = [(k, n) for k, n in written.items() if n > pressed[k] + (pressed[(k[0], "coming")] if k[1] == "correct" else 0)]
        out.append((f"{'The House' if g == H else 'dWeb'}: every act its own person's press", not unpressed, {"unpressed": unpressed}))
    return out
bar("9", theirs)

def asleep():
    a = s["asleep"]
    return [("Bo's browser: svrngs says Sam's computer did not answer", a["bo"]["status"] == 502 and a["bo"]["refused"] == "away" and "did not answer" in a["bo"]["text"], [a["bo"]["status"], a["bo"]["text"]]),
            ("Cy's phone: the same, in the page's words", a["cy"]["status"] == 502 and "did not answer" in a["cy"]["text"], [a["cy"]["status"], a["cy"]["text"]]),
            ("the app: this group can't be read", a["app"]["status"] == 502 and "can't be read" in a["app"]["said"], a["app"]),
            ("never an event with no one coming", not a["bo"]["sections"] and not a["cy"]["sections"], None),
            ("awake again: the same answers", a["woke"]["bo"] and bare(a["woke"]["bo"]["sections"]) == bare(view("bo", "10")["sections"]), a["woke"]["ms"])]
bar("10a", asleep)
bar("10b", lambda: [(f"{w}'s told within 30 s", s["asleep"][w]["ms"] <= 30000, s["asleep"][w]["ms"]) for w in ("bo", "cy")])

def invited():
    fay = next(p for p in s["people"][H] if p["person"] == "Fay")
    posted = ev(H)["at"]
    return [("Fay joined after the post", fay["joined"]["at"] > posted, {"joined": fay["joined"], "posted": posted}),
            ("Fay sees the event and says she's coming", "Fay" in ev(H)["coming"], list(ev(H)["coming"])),
            ("Fay invited on Sam's view", "Fay" in sec("sam", "10", H)["invited"], sec("sam", "10", H)["invited"])]
bar("11", invited)

bar("12", lambda: [("no page says key, member, record, act, namespace or origin",
                    not [m.group(0) for v in views for m in NOT_SAID.finditer(v.get("text", ""))],
                    sorted({m.group(0) for v in views for m in NOT_SAID.finditer(v.get("text", ""))}))])
verdicts["13"] = {"verdict": "could-not-judge", "why": "a phone on cellular, a second host: the operator's (CHARTER.md)"}
verdicts["people"] = {"verdict": "never-ran", "why": "every person is a program (AGENTS.md 7)"}

out = {"plant": s.get("app", {}).get("plant"), "planted_on": PLANTED.get(s.get("app", {}).get("plant"), []),
       "verdicts": {k: v["verdict"] for k, v in verdicts.items()}, "bars": verdicts}
(path.parent / "verdicts.json").write_text(json.dumps(out, indent=1, ensure_ascii=False) + "\n")
print(json.dumps(out["verdicts"]))
for k, v in verdicts.items():
    for c in v.get("checks", []):
        if not c["held"]: print(f"  {k} failed: {c['check']}: {json.dumps(c['evidence'], ensure_ascii=False)[:300]}")
    if "why" in v and v["verdict"] == "could-not-judge" and k != "13": print(f"  {k} could not judge: {v['why']}")
sys.exit(0 if all(v["verdict"] == "passed" for k, v in verdicts.items() if k not in ("13", "people")) else 1)
