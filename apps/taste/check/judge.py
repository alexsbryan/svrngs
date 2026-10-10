"""Judge scoped use's run (check.sh) by su-0's bars
(demo/runs/su/2026-10-09/README.md §3), rerun on what svrngs carries,
the computer first (demo/runs/sup/2026-10-09/README.md): four verdicts,
passed, failed, could-not-judge and never-ran (AGENTS.md 6) — a run
missing, or a step it did not reach, is never a pass. Each plant is judged
as watched failing: `failed` is what it should give.

usage: python3 judge.py --line <run.json>    one line about one run
       python3 judge.py <out>               every bar; writes <out>/verdicts.json
"""
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
ENGINES = ("chromium", "webkit")
QUESTION = "Let Sam's tonights-picks use Ana's taste?"
ASK = "the app receives ranked film suggestions; your viewing history stays where it is stored"


def load(p):
    try:
        return json.loads(Path(p).read_text())
    except (OSError, ValueError):
        return None


def text(p):
    p = Path(p)
    return p.read_text() if p.exists() else ""


def get(d, *path):
    for k in path:
        if not isinstance(d, dict) or k not in d:
            return None
        d = d[k]
    return d


def line(run):
    if run is None:
        return "no run"
    bits = [run.get("tag", "?")]
    for user, p in run.get("people", {}).items():
        said = get(p, "yes", "after") or get(p, "picks", "after") or {}
        bits.append(f"{user}: {get(said, 'taste', 'state')} {said.get('films')}" + (f" stopped: {p['stopped']}" if isinstance(p.get("stopped"), str) else ""))
    if run.get("stopped"):
        bits.append(f"stopped: {run['stopped']}")
    return "; ".join(bits)


def verdict(ok, why, **data):
    """`ok` None is could-not-judge: something the bar reads is missing."""
    return {"verdict": "could-not-judge" if ok is None else ("passed" if ok else "failed"), "why": why, **data}


def has(log, *words, since=None, until=None):
    """Lines of `log` with every word, inside [since, until) by their ISO time."""
    return [l for l in log.splitlines() if all(w in l for w in words)
            and (since is None or l[:23] >= since[:23]) and (until is None or l[:23] < until[:23])]


def window(person, step):
    """When `step` began and the next one did, from the first person's marks."""
    marks = person.get("steps", [])
    for i, m in enumerate(marks):
        if m.get("step") == step:
            return m["at"], (marks[i + 1]["at"] if i + 1 < len(marks) else None)
    return None, None


def count():
    """su-0 §4's count, its awk as the README gives it."""
    script = r"""{ find apps/taste -path apps/taste/check -prune -o -type f \
        \( -name '*.js' -o -name '*.mjs' -o -name '*.html' -o -name '*.css' \) -print
      find apps/movie-night/picks -type f \
        \( -name '*.js' -o -name '*.mjs' -o -name '*.html' -o -name '*.css' \) -print
    } | sort | xargs awk '
      FNR == 1 { piece = "unmarked" }
      /^[[:space:]]*(\/\/|\/\*|<!--)[[:space:]]*su: (own|enforce|plumbing|plant) [a-z-]+/ {
        sub(/^[[:space:]]*(\/\/|\/\*|<!--)[[:space:]]*/, ""); piece = $2 " " $3; next }
      /^[[:space:]]*($|\/\/|\/\*|\*|<!--)/ { next }
      { n[piece]++ }
      END { for (p in n) print n[p], p }' | sort -k2"""
    out = subprocess.run(["bash", "-c", script], cwd=ROOT, capture_output=True, text=True, check=True).stdout
    pieces = {}
    for row in out.splitlines():
        n, piece = row.split(" ", 1)
        pieces[piece] = int(n)
    sides = {}
    for piece, n in pieces.items():
        side = piece.split(" ")[0]
        sides[side] = sides.get(side, 0) + n
    return {"pieces": pieces, "sides": sides, "unmarked": pieces.get("unmarked", 0), "awk": out}


def ranked(r, expected):
    return bool(r) and r.get("films") == expected and r.get("order") == "ranked"


def asked_once(said, expected):
    """Invited, asked svrngs's question in the lender's words, said yes, ranked."""
    q = said.get("question") or "" if said else ""
    return bool(said) and said.get("asked") and QUESTION in q and ASK in q and ranked(said.get("after"), expected)


def unranked(r, n):
    return bool(r) and r.get("order") == "unranked" and len(r.get("films") or []) == n


def judge_run(kind, engine, run, logs, expected):
    """Every bar of one run: `kind` computer or phone."""
    v = {}
    if run is None:
        return {b: {"verdict": "never-ran"} for b in ("bar 1", "bar 2", "bar 3", "bar 4", "the unasked yes", "bar 5", "bar 6")}
    people = run.get("people", {})
    anas = [p for p in people.values() if p.get("name") == "Ana"]
    first, last = (anas[0], anas[-1]) if anas else (None, None)
    other_person = next((p for p in people.values() if p.get("name") != "Ana"), None)
    taste, door = logs["taste"], logs["door"]
    n = len(expected or [])
    picks_label = (get(first, "yes", "after", "at") or "").split("//")[-1].split(".")[0]

    # Bar 1: asked once on the computer; once on each phone.
    yes, again = get(first, "yes"), get(first, "again")
    if not yes or not again or "asks" not in again:
        v["bar 1"] = verdict(None, "a step not reached", stopped=run.get("stopped"), person=get(first, "stopped"))
    else:
        ok = asked_once(yes, expected) and not again["asks"] and ranked(again, expected)
        row = dict(question=yes.get("question"), question_at=yes.get("question_at"),
                   invite_to_question_ms=yes.get("invite_to_question_ms"), yes_to_list_ms=yes.get("yes_to_list_ms"),
                   first_open_tries=[(get(t, "taste", "state"), [s.get("state") for s in t.get("said", [])])
                                     for t in get(yes, "opened", "tries") or []],
                   again_ms=again.get("opened_ms"), again_asks=again.get("asks"))
        if kind == "phone":
            there = get(last, "again")
            if not there or not get(there, "reopened", "tries"):
                v["bar 1"] = verdict(None, "the other phone's step not reached", **row)
            else:
                ok = ok and asked_once(there, expected) and not get(there, "reopened", "asks") and ranked(there.get("reopened"), expected)
                row.update(other_phone_question=there.get("question"), other_phone_yes_to_list_ms=there.get("yes_to_list_ms"),
                           other_phone_reopened_ms=get(there, "reopened", "opened_ms"))
        if "bar 1" not in v:
            v["bar 1"] = verdict(ok, "asked once, in svrngs's words around the lender's, on the page no app runs at; "
                                 "the list in the plant's order after the yes; opened again, no question"
                                 + (" — once on each phone, each its own key" if kind == "phone" else ""), **row)

    # Bar 2: another person said yes, and the taste refused them.
    o = get(other_person, "picks")
    v["bar 2"] = verdict(None if not o or not get(o, "after") else
                         o.get("asked") and get(o, "after", "taste", "state") == "refused-not-yours" and unranked(o["after"], n),
                         "another person said yes on their own page, and the taste refused them by its owner check, the films unranked",
                         who=get(other_person, "name"), question=get(o, "question"), reads=get(o, "after", "taste", "text"),
                         taste_refused=has(taste, "refused not-yours")[:2])

    # Bar 3: the history kept back, by the door.
    d = get(first, "diary")
    s0, s1 = window(first or {}, "diary")
    refused_path = has(door, "the lend refuses this path", "lend=taste", since=s0, until=s1)
    v["bar 3"] = verdict(None if not d or "body" not in d else
                         d.get("status") != 200 and bool(refused_path) and not has(taste, "GET /diary"),
                         "the door refuses /diary and the taste logs no ask for it; on a computer the page cannot read svrngs's refusal",
                         answer=d, door=refused_path[:2], taste_logged_diary=has(taste, "GET /diary"))

    # Bar 4: the other app refused by the door, and its own yes refused.
    ot = get(first, "other")
    s0, s1 = window(first or {}, "other")
    unasked = [l for l in has(door, "judged=Err(Unasked)", since=s0, until=s1) if picks_label not in l]
    if not ot or not get(ot, "unasked"):
        v["bar 4"] = verdict(None, "the other app's step not reached", other=ot)
        v["the unasked yes"] = verdict(None, "the other app's step not reached")
    else:
        ask_refused = (get(ot, "ask", "refused") == "unasked") or (kind == "computer" and get(ot, "ask", "state") == "rank null")
        v["bar 4"] = verdict(ask_refused and bool(unasked) and not has(taste, "POST /rank", since=s0, until=s1),
                             "the other app's ask refused `unasked` at the door: Ana's yes names Tonight's Picks; the taste saw none of it",
                             ask=get(ot, "ask"), door=unasked[:3])
        y = get(ot, "unasked", "yes") or {}
        home_refused = has(door, "home:", "path=/choices", "answers=false", since=s0, until=s1)
        if kind == "computer":
            said_refused = y.get("how") == "the home page" and bool(home_refused)
        else:
            said_refused = y.get("how") == "the door, as the holder" and y.get("status") == 403 and y.get("refused") == "closed"
        again_refused = get(ot, "unasked", "refused") == "unasked" or (kind == "computer" and get(ot, "unasked", "state") == "unasked null")
        v["the unasked yes"] = verdict(said_refused and again_refused,
                                       "the other app's own yes refused by svrngs — at the home page on a computer, at the door "
                                       "on a phone — and its next ask refused `unasked`",
                                       yes=y, home_page=home_refused[:2], then=get(ot, "unasked"))

    # Bar 5: stopped, at the door, its delay recorded.
    st = get(last, "stopped")
    s_at = get(st, "at")
    stopped_door = has(door, "judged=Err(Stopped)", since=s_at) if s_at else []
    if not st or not st.get("pressed"):
        v["bar 5"] = verdict(None, "the stop not pressed", stopped=st)
    else:
        refused_again = get(st, "again", "taste", "state") in (("refused-stopped",) if kind == "phone" else ("refused-stopped", "refused-unread"))
        ok = get(st, "said", "taste", "state") == "stopped" and refused_again and bool(stopped_door)
        row = dict(at_stop=get(st, "said", "taste", "text"), stop_to_refused_ms=st.get("stop_to_refused_ms"),
                   again=get(st, "again", "taste"), door=stopped_door[:2])
        if kind == "computer":
            hs = get(first, "home_stop") or {}
            h_door = has(door, "judged=Err(Stopped)", since=hs["at"]) if hs.get("at") else []
            h_ok = (asked_once(hs.get("yes"), expected) and hs.get("answer") == 200
                    and get(hs, "after", "taste", "state", ) in ("refused-stopped", "refused-unread") and bool(h_door)
                    and "was stopped using" in (hs.get("uses_after") or ""))
            ok = ok and h_ok and "was stopped using" in (st.get("uses") or "")
            row.update(uses_after_app_stop=st.get("uses"), home_stop_uses_before=hs.get("uses_before"),
                       home_stop_uses_after=hs.get("uses_after"), home_stop_to_refused_ms=hs.get("stop_to_refused_ms"),
                       home_stop_again=get(hs, "after", "taste"), home_stop_door=h_door[:2], yes_again_question=get(hs, "yes", "question"))
        else:
            row.update(other_phone_after_stop=get(first, "after_stop", "taste"))
        v["bar 5"] = verdict(ok, "after a stop, the next ask refused at the door, said honestly"
                             + (" — the app's Stop, and the home page's" if kind == "computer" else
                                " — on the phone that stopped; the other phone's own yes is its own key's"), **row)

    # Bar 6: each refusal said, the films unranked.
    rows = {"another person": get(o, "after"), "stopped": get(st, "again")}
    v["bar 6"] = verdict(None if not all(rows.values()) else all(unranked(r, n) and get(r, "taste", "text") for r in rows.values()),
                         "each refusal said as itself, the films shown unranked",
                         **{k: get(r, "taste") for k, r in rows.items()})
    return v


def main(out):
    out = Path(out)
    planted = load(out / "diary-planted.json")
    expected = planted["expected"] if planted else None
    logs = lambda tag: {"taste": text(out / f"{tag}-taste.log"), "door": text(out / f"{tag}-ana-door.log")}
    v = {}
    for kind in ("computer", "phone"):
        for e in ENGINES:
            tag = f"{kind}-{e}"
            v[tag] = judge_run(kind, e, load(out / f"{tag}.json"), logs(tag), expected)

    asleep = load(out / "asleep.json")
    a = get(asleep, "people", "ana-chromium", "asleep", "picks")
    v["asleep"] = verdict(None if asleep is None or not a else
                          get(a, "taste", "state") == "away" and get(a, "order") == "unranked" and len(a.get("films") or []) > 0,
                          "bar 6: Ana's computer asleep, said not to have answered, the films unranked",
                          taste=get(a, "taste"), settled_ms=get(a, "settled_ms"), films=get(a, "films"),
                          yes=get(asleep, "people", "ana-chromium", "yes", "after", "taste"))

    scan = load(out / "scan.json")
    if scan is None:
        v["bar 3 scan"] = {"verdict": "never-ran"}
    else:
        hits = {f"{tag}/{u}": sorted(k for k in r["others_secrets"] if k.startswith("diary_row_"))
                for tag, people in scan["pairs"].items() for u, r in people.items()}
        markers = [k for k in scan["secrets_searched"] if k.startswith("diary_row_")]
        kept = {k: h for k, h in hits.items() if not k.startswith("plant-")}
        v["bar 3 scan"] = verdict(None if not markers else not any(kept.values()),
                                  "no planted row's marker in any byte any browser received, the plants' runs apart",
                                  markers=len(markers), browsers=sorted(kept), found={k: h for k, h in hits.items() if h})

    plants = {}
    alex = lambda p: get(p, "people", "alex-computer-chromium", "picks", "after")
    ana = lambda p, *k: get(p, "people", "ana-computer-chromium", *k)
    p = load(out / "plant-bo.json")
    plants["bo"] = verdict(None if not alex(p) else get(alex(p), "taste", "state") != "ranked",
                           "Alex ranked by Ana's taste with its owner check out", alex=get(alex(p), "taste"))
    p = load(out / "plant-history.json")
    d = ana(p, "diary")
    plants["history"] = verdict(None if not d or "body" not in d else not (d.get("status") == 200 and "su-diary-" in d["body"]),
                                "the history answered, lent without --refuse", answer=d)
    p = load(out / "plant-other-app.json")
    o = ana(p, "other", "ask", "state")
    plants["other-app"] = verdict(None if o is None else o != "rank 200", "the other app answered, the taste lent without --ask", ask=o)
    p = load(out / "plant-after-stop.json")
    s = ana(p, "stopped")
    plants["after-stop"] = verdict(None if not get(s, "again") else get(s, "again", "taste", "state") != "ranked",
                                   "an ask after the stop answered, the stop's record put back to the yes",
                                   planted=get(s, "planted"), again=get(s, "again", "taste"))
    p = load(out / "plant-empty.json")
    e = alex(p)
    plants["empty"] = verdict(None if not e else not (e.get("films") == [] and "No picks" in (get(e, "taste", "text") or "")),
                              "a refusal shown as an empty ranking", alex=get(e, "taste"), films=get(e, "films"))
    v["plants (each should be failed)"] = plants
    v["the count"] = count()
    (out / "verdicts.json").write_text(json.dumps(v, indent=1) + "\n")
    for bar, rows in v.items():
        if bar == "the count":
            print(f"{bar}: {rows['sides']} {rows['pieces']}")
        elif "verdict" in rows:
            print(f"{bar}: {rows['verdict']}")
        else:
            print(f"{bar}: " + "; ".join(f"{k} {r.get('verdict')}" for k, r in rows.items()))


if __name__ == "__main__":
    if sys.argv[1] == "--line":
        print(line(load(sys.argv[2])))
    else:
        main(sys.argv[1])
