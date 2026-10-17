#!/usr/bin/env python3
"""One verdict a prediction of PREDICTIONS.md §2, read from a take's
summary.json, written beside it as verdicts.json (AGENTS.md 6: passed,
failed, could-not-judge, never-ran). A step the scene never reached leaves
its bars could-not-judge, never passed.

    python3 demo/runs/grapevine/2026-10-16/judge.py <summary.json>"""
import json, re, sys
from pathlib import Path

PAGE, WALLER = "1400 Page", "Waller block"
SEALED = ["Saturday morning", "Waller and Pierce", "Sam Reyes", "Kim Okafor"]
NOT_SAID = re.compile(r"\b(keys?|members?|records?|acts?|hash(es)?|handles?|hops?|seal)\b", re.I)
SHORTCUT = "Everyone in a group a message goes through can see that a sealed message went by"
LOOP = 6.0  # RELAY_EVERY, 5 s, and 1 s for the pass and the 0.2 s read (PREDICTIONS.md 7)

path = Path(sys.argv[1]); s = json.loads(path.read_text())
steps = s.get("steps", [])
gid = s.get("groups", {})
verdicts = {}


def verdict(n, v, why): verdicts[str(n)] = {"verdict": v, "why": why}


def at(who, what):
    return next((x for x in steps if x["who"] == who and x["what"].startswith(what)), None)


def reached(*rows): return all(r and r.get("met") for r in rows)


def gap(a, b): return round(a["s"] - b["s"], 3) if a and b and a.get("s") is not None and b.get("s") is not None else None


ops = {(c["computer"], c["group"]): (c.get("log") or {}).get("ops", []) for c in s.get("copies", [])}
by = lambda o: o.get("on_behalf_of") or o.get("person")
ask = next((o for o in ops.get(("sam", gid.get(PAGE)), []) if (o.get("payload") or {}).get("type") == "ask"), None)
carries = [o for o in ops.get(("kim", gid.get(WALLER)), []) if (o.get("payload") or {}).get("type") == "carry"]

# 1. The town, and each page its own groups.
want = {"sam": {PAGE: "Sam"}, "jo": {PAGE: "Jo", WALLER: "Jo"}, "kim": {WALLER: "Kim"}}
opened = {x["who"]: sorted(x.get("groups") or []) for x in steps if x["step"] == "open"}
if not s.get("town"): verdict(1, "could-not-judge", "the town never stood")
else:
    ok = s["town"] == want and all(opened.get(w) == sorted(g) for w, g in want.items())
    verdict(1, "passed" if ok else "failed", f"svrngs answered {s['town']}; the pages showed {opened}")

# 2. Each press lands within 2 s, refused nowhere.
presses = [x for x in steps if x["what"] in ("Kim lists a tile saw in Waller block", "Jo carries it", "Kim offers", "Sam says yes", "Kim says yes", "Sam takes it back")
           or x["what"].startswith("Sam asks")]
if not presses or any(not x.get("met") for x in presses):
    bad = [x["what"] for x in presses if not x.get("met")]
    verdict(2, "failed" if bad else "could-not-judge", f"not landed: {bad}; {[(x['what'], x.get('error') or x.get('said')) for x in presses if not x.get('met')]}")
else:
    slow = [x for x in presses if x["pressed_ms"] > 2000 or x.get("said")]
    verdict(2, "failed" if slow else "passed", f"{len(presses)} presses, {min(x['pressed_ms'] for x in presses)}–{max(x['pressed_ms'] for x in presses)} ms from the press to the page read again; {len(slow)} slow or refused")

# 3. Jo sees the ask within 10 s; Sam's page offers Sam no carry.
asked, jo_sees = at("sam", "Sam asks"), at("jo", "Jo's page shows Carry")
if not reached(asked, jo_sees): verdict(3, "could-not-judge" if not (asked and asked.get("met")) else "failed", "Jo's page never showed the carry" if asked and asked.get("met") else "Sam's ask never landed")
else:
    sam_carry = [b for x in steps if x["who"] == "sam" for b in (x.get("buttons") or []) if b.startswith("Carry")]
    verdict(3, "passed" if gap(jo_sees, asked) <= 10 and not sam_carry else "failed", f"Jo's page showed Carry to Waller block {gap(jo_sees, asked)} s after Sam's press ({jo_sees['reloads']} reloads); Sam's pages' carry buttons: {sam_carry}")

# 4. Kim sees it carried by Jo, never Sam; only Kim told she might have one.
carried, kim_sees = at("jo", "Jo carries it"), at("kim", "Kim's page: carried by Jo")
if not reached(carried, kim_sees): verdict(4, "failed" if carried and carried.get("met") else "could-not-judge", "Kim's page never showed it carried by Jo" if carried and carried.get("met") else "Jo's carry never landed")
else:
    # Kim's pages before her own yes: after it both have said yes, and her page is meant to name Sam.
    yes = at("kim", "Kim says yes")
    before = [x for x in steps if x["who"] == "kim" and (yes is None or steps.index(x) < steps.index(yes))]
    sam_on_kim = [x["what"] for x in before if "Sam" in (x.get("all") or "")]
    told = [x["who"] for x in steps if x["who"] != "kim" and "You might have this" in (x.get("all") or "")]
    ok = gap(kim_sees, carried) <= 10 and not sam_on_kim and not told
    verdict(4, "passed" if ok else "failed", f"Kim's page showed it {gap(kim_sees, carried)} s after Jo's press; Sam named on Kim's page before the names: {sam_on_kim}; told they might have one, but Kim: {sorted(set(told))}")

# 5. The offer reaches Sam within 15 s; Jo's page says a sealed message went by, and none of it.
offered, sam_reads, jo_passed = at("kim", "Kim offers"), at("sam", "Sam's page shows the offer"), at("jo", "Jo's page: a sealed message")
if not reached(offered): verdict(5, "could-not-judge", "Kim's offer never landed")
elif not reached(sam_reads, jo_passed): verdict(5, "failed", f"Sam's page showed the offer: {bool(sam_reads and sam_reads.get('met'))}; Jo's page said one went by: {bool(jo_passed and jo_passed.get('met'))}")
else:
    leaked = [(x["what"], w) for x in steps if x["who"] == "jo" for w in SEALED if w in (x.get("all") or "")]
    went = re.search(r"A sealed message went by toward the person who asked: (\d+) bytes, ([^.]+)\.", jo_passed.get("all") or "")
    ok = gap(sam_reads, offered) <= 15 and went and not leaked
    verdict(5, "passed" if ok else "failed", f"Sam's page showed the offer {gap(sam_reads, offered)} s after Kim's press ({sam_reads['reloads']} reloads); Jo's page: {went and went[0]!r}; sealed words on Jo's pages: {leaked}")

# 6. Names after two yeses.
kim_told, kim_yes, kim_names, sam_names = at("kim", "Kim's page: the person who asked said yes"), at("kim", "Kim says yes"), at("kim", "Kim's page names Sam"), at("sam", "Sam's page names Kim")
if not reached(at("sam", "Sam says yes")): verdict(6, "could-not-judge", "Sam's yes never landed")
elif not reached(kim_told, kim_yes, kim_names, sam_names): verdict(6, "failed", f"reached: {[(x['what'], x.get('met')) for x in (kim_told, kim_yes, kim_names, sam_names) if x]}")
else:
    early = "Sam Reyes" in (kim_told.get("all") or "")
    ok = not early and gap(kim_names, kim_yes) <= 15 and gap(sam_names, kim_yes) <= 15
    verdict(6, "passed" if ok else "failed", f"Kim's page holding Sam's yes named Sam: {early}; after Kim's yes, Kim's page named Sam {gap(kim_names, kim_yes)} s and Sam's named Kim {gap(sam_names, kim_yes)} s after it")

# 7. The withdrawal: gone from Kim's page within 20 s; Jo's correction within one loop of Jo's computer holding it.
back, gone = at("sam", "Sam takes it back"), at("kim", "the ask gone")
seen = {}
for e in s.get("seen", []): seen.setdefault((e["computer"], e["group"], e["op"]), e["t"])
if not reached(back): verdict(7, "could-not-judge", "Sam's take-back never landed")
elif not ask or not carries: verdict(7, "could-not-judge", "no ask or carry in the records")
else:
    withdrawal = next((o for o in ops.get(("jo", gid[PAGE]), []) if o.get("corrects") == ask["id"]), None)
    correction = next((o for c in carries for o in ops.get(("jo", gid[WALLER]), []) if o.get("corrects") == c["id"] and by(o) == "Jo"), None)
    held = withdrawal and seen.get(("jo", gid[PAGE], withdrawal["id"]))
    fixed = correction and seen.get(("jo", gid[WALLER], correction["id"]))
    loop = round(fixed - held, 3) if held and fixed else None
    ok = gone and gone.get("met") and gap(gone, back) <= 20 and loop is not None and loop <= LOOP
    verdict(7, "passed" if ok else "failed", f"Kim's page dropped it {gap(gone, back) if gone and gone.get('met') else 'never'} s after Sam's press; Jo's computer held the withdrawal and then held Jo's correction {loop} s later (bound {LOOP} s)" + ("" if correction else "; no correction of the carry by Jo in any copy"))

# 8. The whole scene under three minutes (predicted under 90 s).
if not (gone and gone.get("met")): verdict(8, "could-not-judge" if not reached(back) else "failed", "the scene never reached its end")
else: verdict(8, "passed" if gone["s"] <= 180 else "failed", f"Sam's Ask to the ask gone from Kim's page: {gone['s']} s; the start, from no svrngs to three pages open: {s.get('start_s')} s")

# 9. Bar 1 on the records: every copy of Waller block's carry is Jo's, names neither Sam nor 1400 Page nor the ask's handle.
if not ask or not carries: verdict(9, "could-not-judge", "no ask or carry in the records")
else:
    bad = []
    for (who, g), log in ops.items():
        if g != gid[WALLER]: continue
        for o in log:
            if by(o) == "Sam" or o.get("person") == "Sam": bad.append(f"{who}: an act in Waller block is Sam's")
            p = o.get("payload") or {}
            if p.get("type") != "carry": continue
            if by(o) != "Jo": bad.append(f"{who}: the carry is {by(o)}'s")
            text = json.dumps(p)
            for name, w in (("Sam", '"Sam"'), ("1400 Page's id", gid[PAGE]), ("1400 Page", PAGE), ("the ask's handle", ask["payload"]["handle"])):
                if w in text: bad.append(f"{who}: the carry holds {name}")
            extra = sorted(set(p) - {"app", "type", "of", "kind", "fields", "hops", "name", "expires", "seal_to", "handle"})
            if extra: bad.append(f"{who}: the carry holds {extra}")
    verdict(9, "failed" if bad else "passed", "; ".join(sorted(set(bad))) or f"{len(carries)} carry in Kim's copy, each Jo's, holding §3's fields alone, none of Sam, 1400 Page or the ask's handle; no act in Waller block is Sam's")

# 10. Bar 3: no file but a state file, and no record kept, holds a sealed word or a secret key; the search sees `tile saw`.
sc = s.get("scan") or {}
records = json.dumps(s.get("copies", []))
in_records = [w for w in SEALED if w in records]
if not sc: verdict(10, "could-not-judge", "nothing was searched")
elif sorted(sc.get("open", {})) != ["jo", "kim", "sam"]: verdict(10, "could-not-judge", f"`tile saw` was found only in {sorted(sc.get('open', {}))}: the search could not have seen a plaintext elsewhere")
else:
    bad = sc.get("sealed") or sc.get("secrets") or in_records
    verdict(10, "failed" if bad else "passed", f"{sc['files']} files, {sc['bytes']:,} bytes searched; sealed words in {sc.get('sealed')}; secret keys outside a state file in {sc.get('secrets')}; sealed words in the records kept: {in_records}; `tile saw` found in {', '.join(f'{w}: {len(f)}' for w, f in sorted(sc['open'].items()))} files")

# 11. Bar 2 as Jo's page says it, once carried.
if not reached(carried): verdict(11, "could-not-judge", "Jo's carry never landed")
else:
    nots = carried.get("not") or []
    verdict(11, "passed" if f"{WALLER}: {WALLER} already has it" in nots else "failed", f"Jo's page after the carry, under Where it can’t go: {nots}")

# 12. No page says a word to look up; every conversation says the shortcut.
pages = [x for x in steps if x.get("all")]
said = sorted({(x["who"], m.group(0)) for x in pages for m in NOT_SAID.finditer(x["all"])})
threads = [(x["who"], t["role"]) for x in pages for t in (x.get("threads") or []) if SHORTCUT not in t["text"]]
n_threads = sum(len(x.get("threads") or []) for x in pages)
if not pages: verdict(12, "could-not-judge", "no page was read")
else: verdict(12, "failed" if said or threads else "could-not-judge" if not n_threads else "passed", f"{len(pages)} pages read: words to look up {said}; {n_threads} conversations shown, without the shortcut {threads}")

verdict(13, "never-ran", "judged across takes: README.md, against each plant's verdicts.json")
verdict(14, "never-ran", "bar 6 (two carriers), GRAPEVINE §7 bars 4 and 7, its product bars and kill bar (the operator's, with real neighbours), a second host and a phone")

(path.parent / "verdicts.json").write_text(json.dumps(verdicts, indent=1, ensure_ascii=False) + "\n")
for n, v in verdicts.items(): print(f"{n:>3} {v['verdict']:<16} {v['why'][:300]}")
