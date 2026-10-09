"""Judge scoped use's run (check.sh) by su-0's bars and predictions
(demo/runs/su/2026-10-09/README.md §3, §6): four verdicts, passed, failed,
could-not-judge and never-ran (AGENTS.md 6) — a run missing, or a step it
did not reach, is never a pass. Each plant is judged as watched failing:
`failed` is what it should give.

usage: python3 judge.py --line <run.json>    one line about one run
       python3 judge.py <out>               every bar; writes <out>/verdicts.json
"""
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def load(p):
    try:
        return json.loads(Path(p).read_text())
    except (OSError, ValueError):
        return None


def line(run):
    if run is None:
        return "no run"
    bits = [run.get("tag", "?")]
    for user, p in run.get("people", {}).items():
        said = p.get("yes", {}).get("after", {}) if p.get("yes") else {}
        bits.append(f"{user}: yes→{said.get('taste', {}).get('state')} {said.get('films')}"
                    if said else f"{user}: {p.get('stopped') or 'ok'}")
    if run.get("stopped"):
        bits.append(f"stopped: {run['stopped']}")
    return "; ".join(bits)


def verdict(ok, why, **data):
    """`ok` None is could-not-judge: something the bar reads is missing."""
    return {"verdict": "could-not-judge" if ok is None else ("passed" if ok else "failed"), "why": why, **data}


def get(d, *path):
    for k in path:
        if not isinstance(d, dict) or k not in d:
            return None
        d = d[k]
    return d


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


def bar1(runs, expected):
    rows = {}
    for engine, run in runs.items():
        if run is None:
            rows[engine] = verdict(None, "never-ran")
            continue
        other = "webkit" if engine == "chromium" else "chromium"
        first, second = get(run, "people", f"ana-{engine}"), get(run, "people", f"ana-{other}")
        yes, again, there = get(first, "yes"), get(first, "again"), get(second, "again")
        if not yes or not again or not there or "asks" not in again or "asks" not in there:
            rows[engine] = verdict(None, "a step not reached", stopped=run.get("stopped"))
            continue
        after = yes.get("after", {})
        ok = (yes.get("asked") and after.get("films") == expected and after.get("order") == "ranked"
              and not again["asks"] and again.get("films") == expected and again.get("order") == "ranked"
              and not there["asks"] and there.get("films") == expected and there.get("order") == "ranked")
        rows[engine] = verdict(ok, "asked once, on the first phone; the list in the plant's order after the yes, "
                               "and opened again on each phone with no question",
                               question=yes.get("question"), yes_to_list_ms=yes.get("yes_to_list_ms"),
                               first_open_tries=len(get(yes, "opened", "tries") or []),
                               again_ms=again.get("opened_ms"), other_phone_ms=there.get("opened_ms"),
                               other_phone_tries=len(there.get("tries") or []),
                               first_tries=[(t.get("taste", {}).get("state"), [s.get("state") for s in t.get("said", [])])
                                            for t in get(yes, "opened", "tries") or []],
                               orders=[after.get("films"), again.get("films"), there.get("films")])
    return rows


def has(log, *words):
    return [l for l in log.splitlines() if all(w in l for w in words)]


def main(out):
    out = Path(out)
    planted = load(out / "diary-planted.json")
    expected = planted["expected"] if planted else None
    nights = {e: load(out / f"night-{e}.json") for e in ("chromium", "webkit")}
    asleep = load(out / "asleep.json")
    scan = load(out / "scan.json")
    v = {"bar 1": bar1(nights, expected)}

    b2, b3, b4, gap, b5, b6 = {}, {}, {}, {}, {}, {}
    for engine, run in nights.items():
        if run is None:
            for d in (b2, b3, b4, gap, b5):
                d[engine] = verdict(None, "never-ran")
            continue
        other = "webkit" if engine == "chromium" else "chromium"
        first, last, bo = (get(run, "people", u) for u in (f"ana-{engine}", f"ana-{other}", "bo"))
        taste_log = (out / f"night-{engine}-taste.log").read_text() if (out / f"night-{engine}-taste.log").exists() else ""
        door_log = (out / f"night-{engine}-ana-door.log").read_text() if (out / f"night-{engine}-ana-door.log").exists() else ""
        picks = get(bo, "picks")
        b2[engine] = verdict(None if not picks or "taste" not in picks else
                             picks["taste"].get("state") == "refused-not-yours" and picks.get("order") == "unranked",
                             "Bo's ask refused by the taste's owner check, his films unranked; the door let him in",
                             bo_reads=get(picks, "taste", "text"), bo_films=get(picks, "films"),
                             taste_refused=has(taste_log, "refused not-yours")[:2],
                             door_let_bo_in=has(door_log, "a pass opens this lend", "person=Bo")[:2],
                             tries=len(get(picks, "tries") or []))
        diary = get(first, "diary")
        b3[engine] = verdict(None if not diary or "status" not in diary else
                             diary["status"] == 403 and "lent without /diary" in diary.get("body", "")
                             and not has(taste_log, "GET /diary"),
                             "the door refuses /diary 403 and the taste logs no ask for it",
                             answer=diary, taste_logged_diary=has(taste_log, "GET /diary"),
                             door=has(door_log, "/diary")[:2])
        o = get(first, "other")
        b4[engine] = verdict(None if not o or not get(o, "ask", "state") else
                             o["ask"]["state"] == "rank 403" and "not-asked" in o["ask"]["text"],
                             "the other app's ask refused: Ana's yes names Tonight's Picks",
                             ask=get(o, "ask"), taste=has(taste_log, "refused not-asked")[:2])
        gap[engine] = {"measured": "the other app's yes, sent unasked", "yes_status": get(o, "unasked", "yes"),
                       "then": get(o, "unasked"), "answered": get(o, "unasked", "state") == "unasked 200"}
        s = get(last, "stopped")
        after_first = get(first, "after_stop")
        b5[engine] = verdict(None if not s or not s.get("pressed") or not after_first else
                             get(s, "said", "taste", "state") == "stopped"
                             and get(s, "again", "taste", "state") == "refused-stopped"
                             and get(after_first, "taste", "state") == "refused-stopped",
                             "after the stop, the next ask on either phone refused by the taste; what Ana reads at the stop",
                             at_stop=get(s, "said", "taste", "text"), stop_to_refused_ms=get(s, "stop_to_refused_ms"),
                             other_phone=get(after_first, "taste"), taste=has(taste_log, "refused stopped")[:2])
        b6[engine] = verdict(None if not picks or not s else
                             get(picks, "order") == "unranked" and len(get(picks, "films") or []) == len(expected or [])
                             and get(s, "again", "order") == "unranked" and len(get(s, "again", "films") or []) > 0,
                             "each refusal said as itself, the films shown unranked",
                             bo=get(picks, "taste"), stopped=get(s, "again", "taste"))
    a = get(asleep, "people", "ana-chromium", "asleep", "picks")
    b6["asleep"] = verdict(None if asleep is None or not a else
                           get(a, "taste", "state") == "away" and get(a, "order") == "unranked" and len(a.get("films") or []) > 0,
                           "Ana's computer asleep: said it did not answer, the films unranked",
                           taste=get(a, "taste"), settled_ms=get(a, "settled_ms"), films=get(a, "films"),
                           yes=get(asleep, "people", "ana-chromium", "yes", "after", "taste"))
    v.update({"bar 2": b2, "bar 3": b3, "bar 4": b4, "the unasked yes": gap, "bar 5": b5, "bar 6": b6})

    # The history's markers in no byte any browser received, plants apart.
    if scan is None:
        v["bar 3"]["scan"] = verdict(None, "never-ran")
    else:
        hits = {f"{tag}/{u}": sorted(k for k in r["others_secrets"] if k.startswith("diary_row_"))
                for tag, people in scan["pairs"].items() for u, r in people.items()}
        markers = [k for k in scan["secrets_searched"] if k.startswith("diary_row_")]
        kept = {k: h for k, h in hits.items() if not k.startswith("plant-")}
        v["bar 3"]["scan"] = verdict(None if not markers else not any(kept.values()),
                                     "no planted row's marker in any byte any browser received, the plants' runs apart",
                                     markers=len(markers), browsers=sorted(kept), found={k: h for k, h in hits.items() if h})

    plants = {}
    p = load(out / "plant-bo.json")
    pk = get(p, "people", "bo", "picks", "taste", "state")
    plants["bo"] = verdict(None if pk is None else pk != "ranked", "Bo's ask answered with the owner check out", bo=pk)
    p = load(out / "plant-history.json")
    d = get(p, "people", "ana-chromium", "diary")
    plants["history"] = verdict(None if not d or "status" not in d else not (d["status"] == 200 and "su-diary-" in d["body"]),
                                "the history answered, lent without --refuse", answer=d)
    p = load(out / "plant-other-app.json")
    o = get(p, "people", "ana-chromium", "other", "ask", "state")
    plants["other-app"] = verdict(None if o is None else o != "rank 200", "the other app answered with the app check out", ask=o)
    p = load(out / "plant-after-stop.json")
    s = get(p, "people", "ana-chromium", "stopped", "again", "taste", "state")
    plants["after-stop"] = verdict(None if s is None else s != "ranked", "an ask after the stop answered with the stop check out", again=s)
    p = load(out / "plant-empty.json")
    e = get(p, "people", "bo", "picks")
    plants["empty"] = verdict(None if not e else not (e.get("films") == [] and "No picks" in get(e, "taste", "text")),
                              "a refusal shown as an empty ranking", bo=get(e, "taste"), films=get(e, "films"))
    v["plants (each should be failed)"] = plants
    v["the count"] = count()
    (out / "verdicts.json").write_text(json.dumps(v, indent=1) + "\n")
    for bar, rows in v.items():
        if bar == "the count":
            print(f"{bar}: {rows['sides']} {rows['pieces']}")
            continue
        print(f"{bar}: " + "; ".join(f"{k} {r.get('verdict', r.get('answered'))}" for k, r in rows.items()))


if __name__ == "__main__":
    if sys.argv[1] == "--line":
        print(line(load(sys.argv[2])))
    else:
        main(sys.argv[1])
