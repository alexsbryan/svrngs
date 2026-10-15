#!/usr/bin/env python3
"""Each prediction of PREDICTIONS.md §2 judged against one take's
summary.json, written to verdicts.json beside it: AGENTS.md 6's four
verdicts, each with what it read. A step that never happened is failed
when the take reached it, never-ran when it did not.

    python3 demo/runs/v1ui/2026-10-15/judge.py <take's summary.json>

Prediction 15 reads the planted take's verdicts.json
(data/plants/accounts/), never-ran without it."""
import json, re, sys
from pathlib import Path

S = json.load(open(sys.argv[1])); HERE = Path(__file__).resolve().parent
P = S["people"]; ORDER = ["Ann", "Bo", "Cy", "Di", "Eve"]; COMPUTERS = ORDER[:4]
WORDS = re.compile(r"\b(keys?|members?|records?|acts?|hash(es)?|origins?|ports?|relays?)\b", re.I)
verdicts = {}


def judge(n, ok, read):
    verdicts[n] = {"verdict": "passed" if ok else "failed", "read": read}


def ms(step, *keys):
    """The sum of a step's timings, or None if any did not happen."""
    total = 0
    for k in keys:
        v = (step or {}).get(k) or {}
        if v.get("ms") is None: return None
        total += v["ms"]
    return total


def within(v, bound): return v is not None and v <= bound


def seen(p):
    """Every page `p` was shown, as the steps kept them."""
    out = []
    for k in ("link", "home", "group", "group_again", "join"):
        s = p.get(k) or {}
        out += [x for x in (s.get("seen"), s.get("shown")) if isinstance(x, dict)]
    for k in ("money", "library"):
        s = p.get(k) or {}
        out += [x for x in ([s.get("seen")] + (s.get("shown") if isinstance(s.get("shown"), list) else [s.get("shown")])) if isinstance(x, dict)]
    return out


def svrngs_pages(p):
    """The svrngs pages `p` read: all but Jellyfin's, which is the library's own."""
    return [x for x in seen(p) if "/web/" not in (x.get("url") or "")]


def door_of(step): return {d["name"]: (d["drawn"], d["says"]) for d in (step or {}).get("door") or []}


# 1. the link, on a computer
r = {w: {"ms": P[w].get("link", {}).get("ms"), "line": P[w].get("link", {}).get("line", "")[:40]} for w in COMPUTERS}
judge(1, all(within(P[w]["link"].get("ms"), 5000) and (P[w]["link"].get("line") or "").startswith("curl -fsSL ")
             and "On this computer, paste this line in a terminal" in (P[w]["link"].get("seen") or {}).get("words", "") for w in COMPUTERS), r)

# 2. the line
def line_ok(w):
    l = P[w].get("line") or {}
    said = l.get("said") or []
    return l.get("exit") == 0 and within(l.get("ms"), 30000) and f"You're in Elm Street, as {w}." in said \
        and said[-1] == f"Your home page, the same address each time svrngs starts: {l.get('open_asked')}"
judge(2, all(line_ok(w) for w in COMPUTERS), {w: {k: (P[w].get("line") or {}).get(k) for k in ("exit", "ms", "open_asked")} for w in COMPUTERS})

# 3. the home page, the door
def door_ok(w, issuer_off):
    h = P[w].get("home") or {}
    if h.get("ms") is None: return False
    joined = ORDER[:ORDER.index(w) + 1] + (["Eve"] if w == "Di" else [])
    d = door_of(h)
    lit = all(d.get(n) == ("on", "on now") for n in ["Priya", *[x for x in joined if x != "Eve"]] + ([] if issuer_off else ["Sam"]))
    sam = d.get("Sam") == (("off", "did not answer") if issuer_off else ("on", "on now"))
    eve = "Eve" not in joined or d.get("Eve") == ("phone", "in from a phone")
    rows = h.get("index") or []
    return within(h["at"] + h["ms"] - P[w]["line"]["at"], 30000) and lit and sam and eve \
        and any(r[1:2] == ["Money"] for r in rows) and any(r[:3] == ["Open Priya’s jellyfin", "jellyfin", "Priya"] for r in rows)
judge(3, all(door_ok(w, w == "Di") for w in COMPUTERS),
      {w: {"after_line_ms": (P[w].get("home") or {}).get("at", 0) + ((P[w].get("home") or {}).get("ms") or 0) - (P[w].get("line") or {}).get("at", 0),
           "door": door_of(P[w].get("home"))} for w in COMPUTERS})

# Money's acts, as Priya's computer holds them, by whose they are.
log = S["after"]["Priya"]["log"]["ops"]
money = [o for o in log if str((o.get("payload") or {}).get("kind", "")).startswith("Money/")]
by = lambda o: o.get("on_behalf_of") or o.get("person")
def acts_ok(w):
    m = P[w].get("money") or {}
    return len(m.get("presses") or []) == sum(1 for o in money if by(o) == w and not o.get("voided"))

# 4. Money, on a computer
def money_ok(w, bound_open=10000, bound_square=30000):
    m = P[w].get("money") or {}
    first = (m.get("shown") or [{}])[0]
    return within(ms(m, "open"), bound_open) and "Gas bill" in first.get("words", "") and within(ms(m, "square"), bound_square) and acts_ok(w)
judge(4, all(money_ok(w) for w in COMPUTERS), {w: {"open_ms": ms(P[w].get("money"), "open"), "square_ms": ms(P[w].get("money"), "square"),
                                               "presses": [p["pressed"] for p in (P[w].get("money") or {}).get("presses") or []],
                                               "acts": [o["payload"]["kind"] for o in money if by(o) == w]} for w in ORDER})

# 5. the library, on a computer
def library_ok(w, bound_open=15000, bound_play=30000):
    l = P[w].get("library") or {}
    asked = (l.get("shown") or {}).get("sign_in") or (l.get("shown") or {}).get("password")
    return within(ms(l, "open"), bound_open) and not asked and within(ms(l, "card", "playing"), bound_play)
judge(5, all(library_ok(w) for w in COMPUTERS), {w: {"open_ms": ms(P[w].get("library"), "open"), "card_to_playing_ms": ms(P[w].get("library"), "card", "playing"),
                                                 "clicked": (P[w].get("library") or {}).get("clicked"), "stopped": (P[w].get("library") or {}).get("stopped"),
                                                 "video": (P[w].get("library") or {}).get("video")} for w in ORDER})

# 6. Eve joins
j = P["Eve"].get("join") or {}
judge(6, within(ms(j, "asked"), 10000) and within(ms(j, "joined"), 30000) and (j.get("status") or "").startswith("You’re in Elm Street"),
      {"asked_ms": ms(j, "asked"), "joined_ms": ms(j, "joined"), "status": j.get("status")})

# 7. Eve with the issuer stopped
g = P["Eve"].get("group") or {}
through = re.match(r"You’re in Elm Street \(.*\), as Eve, through (\w+)’s computer$", g.get("status") or "")
judge(7, within(g.get("ms"), 15000) and bool(through) and through[1] != "Sam" and door_of(g).get("Sam") == ("off", "did not answer"),
      {"ms": g.get("ms"), "status": g.get("status"), "sam": door_of(g).get("Sam"), "again": (P["Eve"].get("group_again") or {}).get("status")})

# 8. Eve's Money and library, through another's computer
judge(8, money_ok("Eve", 60000, 60000) and library_ok("Eve", 60000, 60000) and within(ms(P["Eve"].get("library"), "open", "card", "playing"), 60000)
      and within(ms(P["Eve"].get("money"), "open", "square"), 60000),
      {"money_ms": ms(P["Eve"].get("money"), "open", "square"), "library_ms": ms(P["Eve"].get("library"), "open", "card", "playing")})

# 9. Di with the issuer stopped
judge(9, door_ok("Di", True) and money_ok("Di") and library_ok("Di"), {"door": door_of(P["Di"].get("home")),
                                                                        "issuer_stopped_before_di_home": S["issuer_stopped_at"] < P["Di"]["t0"] + P["Di"]["home"]["at"] / 1000})

# 10. within three minutes
done = {w: max((P[w].get("money") or {}).get("done_at") or 10**9, (P[w].get("library") or {}).get("done_at") or 10**9)
        if ms(P[w].get("money"), "square") is not None and ms(P[w].get("library"), "playing") is not None else None for w in ORDER}
judge(10, all(within(v, 180000) for v in done.values()) and within(max((v or 10**9) for v in done.values()), 120000), {"done_ms": done})

# 11. zero accounts and zero key prompts
prompts = {w: sum((x.get("password") or 0) + (x.get("sign_in") or 0) for x in seen(P[w])) for w in ORDER}
users = S.get("jellyfin_users") or []
made = sorted(u for u in users if re.fullmatch(r"(Ann|Bo|Cy|Di|Eve)-[0-9a-f]{16}", u))
judge(11, not any(prompts.values()) and all((P[w].get("line") or {}).get("exit") == 0 for w in COMPUTERS)
      and [u.split("-")[0] for u in made] == sorted(ORDER) and set(users) == set(made) | {"priya"},
      {"prompts_shown": prompts, "jellyfin_users": users})

# 12. the door after everyone
want = {"Sam": ("off", "did not answer"), "Eve": ("phone", "in from a phone"), **{n: ("on", "on now") for n in ["Priya", *COMPUTERS]}}
doors = {w: door_of(a) for w, a in S["after"].items()}
judge(12, len(doors) == 5 and all(d == want for d in doors.values()), {"doors": doors})

# 13. Money agrees everywhere
copies = {w: sorted(o["id"] for o in a["log"]["ops"] if str((o.get("payload") or {}).get("kind", "")).startswith("Money/")) for w, a in S["after"].items()}
pages = {w: all(s in (a.get("money") or {}).get("words", "") for s in ("You’re square with everyone.", "Nobody owes anybody.")) for w, a in S["after"].items()}
pressed = sum(len((P[w].get("money") or {}).get("presses") or []) for w in ORDER) + 1
judge(13, all(pages.values()) and len({tuple(c) for c in copies.values()}) == 1 and len(money) == pressed and all(acts_ok(w) for w in ORDER),
      {"square_on": pages, "money_acts_each_copy": {w: len(c) for w, c in copies.items()}, "presses_and_gas_bill": pressed})

# 14. words
found = {w: sorted({m[0].lower() for x in svrngs_pages(P[w]) for m in WORDS.finditer(x.get("words", ""))}) for w in ORDER}
judge(14, not any(found.values()), {"found": found, "pages": {w: len(svrngs_pages(P[w])) for w in ORDER}})

# 15. the plant
planted = HERE / "data/plants/accounts/verdicts.json"
if S.get("plant"):
    verdicts[15] = {"verdict": "never-ran", "read": "this is the planted take"}
elif planted.exists():
    v = {int(k): x["verdict"] for k, x in json.load(open(planted)).items()}
    judge(15, all(v[n] == "failed" for n in (5, 8, 10, 11)) and all(v[n] == "passed" for n in (4, 13)), {"planted": v})
else:
    verdicts[15] = {"verdict": "never-ran", "read": "no planted take"}

# 16. not run here
verdicts[16] = {"verdict": "could-not-judge", "read": "real people, a phone on cellular, a second host: not run; the people bar never-ran"}

Path(sys.argv[1]).with_name("verdicts.json").write_text(json.dumps(verdicts, indent=1, ensure_ascii=False) + "\n")
for n, v in verdicts.items(): print(n, v["verdict"])
