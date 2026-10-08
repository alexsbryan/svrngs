"""Movie Night's bars judged (demo/runs/mn/2026-10-08/README.md §3), each
by what it guards; where its letter and its spirit part, both are said.
Four verdicts: passed, failed, could-not-judge (a run that did not
finish, or data that is not there), recorded (measured, not judged). A
plant's run is judged by the bar it plants against, and must fail it.

Reads <out>: phone-<engine>.json, asleep-<engine>.json, computer.json,
limit.json, plant-*.json and asleep-chromium-plant-asleep-empty.json
(mn.mjs), scan.json (apps/jellyfin/check/scan.py), doors.json (doors.py);
the repository for Movie Night's source, the line count by mn-0's rule
and the spike's diff. Writes <out>/verdicts.json; prints a line a verdict,
and whether each plant's run failed the bar it plants against.

usage: python3 judge.py <out dir>
       python3 judge.py --line <run.json>     one line, for steps.log
"""
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
MN = REPO / "apps/movie-night"
# The commit before any of the spike's app or data: mn-0's README
# (c963f93). The queue's first commit (37b4a4c), which the README names,
# precedes si-1, which changed crates/ (ae898f0, effb427, 6028168).
BASE = "c963f93"


def load(out, name):
    p = out / name
    return json.loads(p.read_text()) if p.exists() and p.stat().st_size else None


def people(run):
    return list((run or {}).get("people", {}).values())


def blade(r):
    return [c for c in (r or {}).get("cards", []) if c["title"].startswith("Blade Runner")]


def libs_of(c):
    return sorted({x["library"] for x in c["copies"]})


def line(path):
    run = json.loads(Path(path).read_text())
    said = []
    for p in people(run):
        s = p.get("settled") or p.get("handed") or p.get("first")
        bits = [p["name"]]
        if p.get("stopped"):
            bits.append(f"stopped: {p['stopped'][:160]}")
        if s:
            bits.append(f"'{s['query']}' {len(s['cards'])} card(s) {[len(c['copies']) for c in s['cards']]} copies, "
                        f"first {s['first_card_ms']} ms, all {s['all_said_ms']} ms, {[l['state'] for l in s['said']]}")
        for k in ("never_opened", "opened"):
            if p.get(k):
                bits.append(f"{k}: {p[k].get('landed')}{' signed in' if p[k].get('signed_in') else ''}")
        if p.get("played"):
            bits.append(f"played {p['played'].get('play_ms')} ms")
        if p.get("opened_back"):
            b = p["opened_back"]
            bits.append(f"back {b['presses']} press(es), same {b.get('same_query')}/{b.get('same_cards')}")
        said.append("; ".join(bits))
    return " | ".join(said) + (f" | stopped: {run['stopped']}" if run.get("stopped") else "")


V = []


def verdict(bar, run, verdict, why, **more):
    V.append({"bar": bar, "run": run, "verdict": verdict, "why": why, **more})
    print(f"{verdict:<16} {bar} · {run}: {why}")


def bar1(name, run, labels, plant=False):
    for p in people(run):
        who = f"{name} {p['name']}"
        s = p.get("settled") or p.get("handed")
        if not s:
            verdict("bar 1", who, "could-not-judge", f"no search where every library answered: {p.get('stopped')}")
            continue
        cards = blade(s)
        others = [c["title"] for c in s["cards"] if c not in cards]
        ok = len(cards) == 1 and libs_of(cards[0]) == sorted(labels.values()) and not others
        cold = p.get("cold") or p.get("first") or s
        verdict("bar 1", who, "passed" if ok else "failed",
                f"'{s['query']}': {len(cards)} Blade Runner card(s), copies {[len(c['copies']) for c in cards]}, other cards {others}",
                first_card_ms=s["first_card_ms"], all_said_ms=s["all_said_ms"], searches_to_settle=s.get("searches"),
                cold={"first_card_ms": cold["first_card_ms"], "all_said_ms": cold["all_said_ms"], "said": [l["text"] for l in cold["said"]]},
                warm=p.get("warm") and {"first_card_ms": p["warm"]["first_card_ms"], "all_said_ms": p["warm"]["all_said_ms"]},
                plant=plant)
        k = p.get("kept_film")
        if k is not None:
            brazil = [c for c in k["cards"] if c["title"].startswith("Brazil")]
            want = 0 if p.get("kept") else 1
            ok = len(brazil) == want and all(libs_of(c) == [labels["Sam"]] for c in brazil)
            verdict("bar 1, each their own", who, "passed" if ok else "failed",
                    f"'brazl' after Sam's Brazil was {'kept from them' if want == 0 else 'left to them'} "
                    f"({(p.get('kept') or {}).get('account', '')}): {len(brazil)} Brazil card(s)")


def bar2_3(name, run):
    for p in people(run):
        who = f"{name} {p['name']}"
        n = p.get("never_opened")
        if n:
            ok = n.get("landed") == "item" and n.get("signed_in")
            verdict("bar 2, never opened", who, "passed" if ok else "failed",
                    f"chose {n.get('chose')}: landed {n.get('landed')} (status {n.get('status')}, from its worker {n.get('from_worker')}) at {n.get('at')}",
                    shown=n.get("shown"))
        o, pl = p.get("opened"), p.get("played")
        if not o:
            verdict("bar 2" + (", opened from the list" if n else ""), who, "could-not-judge", f"never chose: {p.get('stopped')}")
            continue
        ok = o.get("landed") == "item" and o.get("signed_in") and (o.get("title") or "").startswith("Blade Runner") \
            and pl and pl.get("play_ms") is not None and pl["play_ms"] <= 30_000
        verdict("bar 2" + (", opened from the list" if n else ""), who, "passed" if ok else "failed",
                f"landed {o.get('landed')} in {o.get('landed_ms')} ms, {o.get('title')!r}, signed in {o.get('signed_in')} "
                f"(asked from the list: {p.get('opened_jellyfin_asked')}); Play to moving {pl and pl.get('play_ms')} ms"
                f"{', a second tap' if pl and pl.get('second_tap') else ''}")
        b = p.get("opened_back")
        if not b:
            verdict("bar 3", who, "could-not-judge", "no Back pressed")
            continue
        ok = b["back"] and b.get("same_query") and b.get("same_cards")
        verdict("bar 3", who, "passed" if ok else "failed",
                f"{b['presses']} press(es) of Back to Movie Night; query kept {b.get('same_query')}, cards the same {b.get('same_cards')}",
                presses=b["presses"])


def bar4_before(name, run, labels):
    for p in people(run):
        who = f"{name} {p['name']}"
        said = (p.get("listed") or {}).get("said", "")
        searches = [s["search"] for s in p.get("steps", []) if "search" in s]
        if not searches:
            verdict("bar 4, asleep before the list", who, "could-not-judge", "no search")
            continue
        strangers = {l["library"] for s in searches for l in s["said"]} - set(labels.values())
        empty = [l for s in searches for l in s["said"] if l["library"] in strangers and l["state"] == "answered"]
        told = "Cy’s computer did not answer" in said
        verdict("bar 4, asleep before the list", who, "failed" if empty else "passed",
                f"spirit: no library of Cy's shown empty ({len(empty)}); letter: Movie Night names {len(strangers)} library beside "
                f"Sam's and Dee's, so says nothing of Cy's (failed); the group's page said: {told}",
                letter="failed", spirit="failed" if empty else "passed")


def bar4_after(name, run, labels, plant=False):
    for p in people(run):
        who = f"{name} {p['name']}"
        f = p.get("first")
        if not f or not p.get("cy_listed"):
            verdict("bar 4, asleep after the list", who, "could-not-judge",
                    f"Cy listed: {p.get('cy_listed')}; searched: {bool(f)}; {p.get('stopped')}")
            continue
        cy = {l["library"] for l in f["said"]} - set(labels.values())
        seen = [x for x in f["seen"] if x["library"] in cy]
        empty = [x for x in seen if x["state"] == "answered"]
        away = [x for x in seen if x["state"] == "away"]
        others = min([x["t"] for x in f["seen"] if x["library"] not in cy and x["state"] == "answered"] or [None]) if f["seen"] else None
        ok = bool(away) and not empty and others is not None and others < away[0]["t"]
        verdict("bar 4, asleep after the list", who, "passed" if ok else "failed",
                f"Cy's library: {[(x['state'], x['t']) for x in seen]}; shown empty {len(empty)}; said not to have answered at "
                f"{away[0]['t'] if away else None} ms; the others' first answer at {others} ms",
                said=[x["text"] for x in seen], away_ms=away[0]["t"] if away else None, plant=plant)


def bar5(out, scan, runs):
    for run, persons in (scan or {}).get("pairs", {}).items():
        if run.startswith("smoke"):
            continue
        for u, r in persons.items():
            ok = r["holds_none_of_anothers"] and r["own_token"] == 0
            verdict("bar 5, no secret in a byte", f"{run} {u}", "passed" if ok else "failed",
                    f"{r['scanned']['bytes']} bytes in {r['scanned']['responses']} responses: others' secrets {r['others_secrets']} "
                    f"in {r['others_secrets_in']}; their own token {r['own_token']}", plant=run.startswith("plant-lender-token"))
    if scan:
        verdict("bar 5, no secret in a byte", "searched for", "recorded", ", ".join(scan["secrets_searched"]))
    labels = {l for r in runs for l in (r or {}).get("labels", {}).values()} if runs else set()
    words = ["x-svrngs-", "svrngs-lend", ".localhost", *sorted(labels)]
    creds = re.compile(r"authorization|cookie|token|\bpass\b|\bholder\b|localStorage", re.I)
    named, sets = [], []
    for f in sorted(MN.glob("*.*")):
        if f.suffix not in (".js", ".mjs", ".html", ".css"):
            continue
        piece = "unmarked"
        for n, text in enumerate(f.read_text().splitlines(), 1):
            m = re.match(r"\s*(//|/\*|<!--)\s*mn: (own|plumbing|plant) ([a-z-]+)", text)
            if m:
                piece = f"{m.group(2)} {m.group(3)}"
                continue
            code = re.sub(r"//.*$", "", text)
            for w in words:
                if w in text:
                    named.append(f"{f.name}:{n} ({piece}) `{w}`")
            if creds.search(code) and not piece.startswith("plant"):
                sets.append(f"{f.name}:{n} ({piece}): {code.strip()[:100]}")
    verdict("bar 5, svrngs's names in Movie Night's source", "apps/movie-night", "failed" if named else "passed",
            f"{len(named)} line(s): {named}", lines=named)
    verdict("bar 5, a credential in Movie Night's code", "apps/movie-night", "recorded",
            f"{len(sets)} line(s) naming one outside the plants: {sets}", lines=sets)


def count():
    awk = r"""find apps/movie-night -path apps/movie-night/check -prune -o -type f \
      \( -name '*.js' -o -name '*.mjs' -o -name '*.html' -o -name '*.css' \) -print \
      | sort | xargs awk '
      FNR == 1 { piece = "unmarked" }
      /^[[:space:]]*(\/\/|\/\*|<!--)[[:space:]]*mn: (own|plumbing|plant) [a-z-]+/ {
        sub(/^[[:space:]]*(\/\/|\/\*|<!--)[[:space:]]*/, ""); piece = $2 " " $3; next }
      /^[[:space:]]*($|\/\/|\/\*|\*|<!--)/ { next }
      { n[piece]++ }
      END { for (p in n) print n[p], p }' | sort -k2"""
    got = subprocess.run(["bash", "-c", awk], cwd=REPO, capture_output=True, text=True).stdout.split("\n")
    pieces = {" ".join(l.split()[1:]): int(l.split()[0]) for l in got if l.strip()}
    sides = {s: sum(n for p, n in pieces.items() if p.startswith(s)) for s in ("own", "plumbing", "plant")}
    verdict("the count", "apps/movie-night", "could-not-judge" if "unmarked" in pieces else "recorded",
            f"{sides}; {pieces}", pieces=pieces, sides=sides)


def product():
    diff = subprocess.run(["git", "diff", "--stat", f"{BASE}..HEAD", "--", "crates/*/src", "web"], cwd=REPO,
                          capture_output=True, text=True).stdout.strip()
    dirty = subprocess.run(["git", "diff", "--stat", "--", "crates/*/src", "web"], cwd=REPO, capture_output=True, text=True).stdout.strip()
    verdict("the product untouched", f"git diff {BASE}..HEAD -- crates/*/src web", "passed" if not diff and not dirty else "failed",
            diff or dirty or "empty")
    n = (REPO / "scripts/check.sh").read_text().lower().count("jellyfin")
    verdict("scripts/check.sh free of Jellyfin", "scripts/check.sh", "passed" if n == 0 else "failed", f"{n} mention(s)")


def limit(doors):
    if not doors:
        return verdict("the limit", "doors.json", "could-not-judge", "no doors.json")
    verdict("the limit", "most counted at once", "recorded", json.dumps(doors["most_counted"]))
    verdict("the limit", "guessed home", "recorded", json.dumps(doors["guessed_home"]) or "nobody")
    verdict("the limit", "relayed or not, by person", "recorded", json.dumps(doors["relayed"]))
    l = doors.get("limit")
    if l:
        verdict("the limit", f"{l['second']}'s searches after {l['first']}'s last", "recorded", json.dumps(l["second_after"]))
    verdict("the limit", "refusals", "recorded", json.dumps(doors["refused"]))


def main(out):
    runs = {f.stem: json.loads(f.read_text()) for f in sorted(out.glob("*.json"))
            if f.stem.startswith(("phone-", "asleep-", "computer", "limit", "plant-"))}
    labels = next((r["labels"] for r in runs.values() if r.get("labels")), None)
    if not labels:
        print("could-not-judge  every bar: no run names its labels")
        return 1
    for name, run in runs.items():
        planted = "plant" in name
        if run.get("stopped"):
            verdict("the run", name, "could-not-judge", f"stopped: {run['stopped']}")
        if name.startswith("phone-"):
            bar1(name, run, labels)
            bar2_3(name, run)
            bar4_before(name, run, labels)
        elif name.startswith("asleep-"):
            bar4_after(name, run, labels, plant=planted)
        elif name == "computer":
            p = people(run)[0] if people(run) else {}
            nh = p.get("nothing_handed")
            verdict("reads on a computer, discover", "computer Alex", "failed" if nh and not nh["said"][0].get("library") else "could-not-judge",
                    f"with nothing handed Movie Night said: {[l['text'] for l in (nh or {}).get('said', [])]}; this origin's stores: {p.get('store')}")
            h = p.get("handed") or {}
            verdict("reads on a computer, request", "computer Alex", "recorded",
                    f"the labels handed by the run: {[(l['state'], l['text']) for l in h.get('said', [])]}")
            bar1(name, run, labels)
            bar2_3(name, run)
        elif name.startswith("plant-per-copy"):
            bar1(name, run, labels, plant=True)
        elif name == "limit" or name.startswith("plant-lender-token"):
            pass
    bar5(out, load(out, "scan.json"), list(runs.values()))
    limit(load(out, "doors.json"))
    count()
    product()
    (out / "verdicts.json").write_text(json.dumps(V, indent=1) + "\n")
    plants_failed = all(v["verdict"] == "failed" for v in V if v.get("plant"))
    print(f"plants: each failed its bar: {plants_failed} ({sum(1 for v in V if v.get('plant'))} verdicts)")
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "--line":
        print(line(sys.argv[2]))
    else:
        sys.exit(main(Path(sys.argv[1])))
