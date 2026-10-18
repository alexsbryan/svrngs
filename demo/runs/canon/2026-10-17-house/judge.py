#!/usr/bin/env python3
"""Each prediction of PREDICTIONS.md §2 judged from a take's summary.json:
one verdict a bar (AGENTS.md 6: passed, failed, could-not-judge, never-ran),
with what it read and how long each step took, written beside it as
verdicts.json.

    python3 demo/runs/canon/2026-10-17-house/judge.py <a take's summary.json>

A bar whose data the take never wrote is could-not-judge, never a pass."""
import json, re, sys
from pathlib import Path

QUIET, ROOM, WHY = "Quiet hours are 10pm to 7am", "Ana keeps the big room", "I work late on Fridays"
DAY = r"\d{1,2} [A-Z][a-z]{2}"
MOST = "A rule holds once more than half of the people in your house when it was proposed agree, whoever proposed it among them."
EVERYONE = "A rule holds once everyone in your house when it was proposed agrees, and one objection stops it."
NOT_SAID = re.compile(r"\b(scope|ratif\w*|commitments?|acts?|actors?|canon|key|record)\b", re.I)
HEADS = ("How your house decides", "Your house’s rules", "What’s proposed", "Not adopted", "Propose a rule")


class Unread(Exception):
    """The take holds no data for the bar."""


def main(path):
    s = json.loads(Path(path).read_text())
    st = s.get("steps", {})

    def step(k):
        if k not in st: raise Unread(f"step {k} never ran")
        return st[k]

    def entry(k, kind, n=0):
        """The n-th done entry of step k whose step is `kind`, or the last one when kind is None."""
        done = step(k)["done"]
        found = [d for d in done if kind is None or f'"{kind}"' in d["step"]]
        if len(found) <= n: raise Unread(f"step {k} has no {kind} #{n} ({step(k).get('done', [{}])[-1].get('error')})")
        return found[n]

    def took(k, kind, n=0):
        """How long step k's `kind` entry took, from the entry before it."""
        done = step(k)["done"]
        d = entry(k, kind, n)
        i = done.index(d)
        return d["ms"] - (done[i - 1]["ms"] if i else 0)

    def ok(k, kind, n=0):
        d = entry(k, kind, n)
        if d.get("error"): raise Failed(f"step {k}'s {kind}: {d['error'][:200]}")
        return d["text"]

    def under(text, item):
        """The heading the first line starting with `item` is under, or None."""
        at, head = None, None
        for line in text.splitlines():
            if line.strip() in HEADS: head = line.strip()
            if line.startswith(item): return head
        return at

    def say(text, viewer):
        m = re.search(r"Everyone in your house has a say: (.*)\.", text)
        if not m: return None
        names = re.split(r", | and ", m[1])
        return sorted(viewer if n == "you" else n for n in names)

    class Failed(Exception):
        pass

    def bar(fn):
        try:
            got = fn()
            return {"verdict": "passed", **(got or {})}
        except Failed as e:
            return {"verdict": "failed", "why": str(e)}
        except Unread as e:
            return {"verdict": "could-not-judge", "why": str(e)}

    def need(cond, why):
        if not cond: raise Failed(why)

    def b1():
        text = ok("1", "click")
        need("You brought House rules to Elm: everyone in it can open it in a few seconds." in text, f"not said brought: {text[-300:]}")
        need("You brought Money" not in text and "You brought Events" not in text, "something else brought")
        ms = took("1", "click")
        need(ms <= 10_000, f"said in {ms} ms")
        listed, build = s.get("listed") or {}, s.get("build") or {}
        if not listed or not build: raise Unread("no listing or build held")
        for who in ("bo", "cy"):
            need(listed[who]["listed"] and listed[who]["ms"] <= 60_000, f"{who} lists House rules: {listed[who]}")
            need(build[who]["held"] and build[who]["ms"] <= 60_000, f"{who} holds the build: {build[who]}")
        need(s.get("kinds") == ["House-rules"], f"kinds listed: {s.get('kinds')}")
        return {"said_ms": ms, "listed": listed, "build": build}

    def b2():
        first = ok("2", "wait")
        need("How does your house decide its rules?" in first, "no question")
        ways, ticked = entry("2", "count", 0).get("value"), entry("2", "count", 1).get("value")
        need(ways == 3 and ticked == 0, f"{ways} choices, {ticked} ticked")
        text = ok("2", "click")
        need(re.search(r"How your house decides\n+" + re.escape(MOST), text), f"rule not said: {text[:300]}")
        ms = took("2", "click")
        need(ms <= 5_000, f"{ms} ms")
        return {"ms": ms}

    def b3():
        text = ok("3", "click")
        need(re.search(r"What’s proposed\n+" + re.escape(QUIET) + r"\n+You proposed it on " + DAY + r"\.", text), f"{text[:400]}")
        ms = took("3", "click")
        need(ms <= 5_000, f"{ms} ms")
        r = s.get("reached_3") or {}
        for who in ("bo", "cy"): need(r.get(who) is not None and r[who] <= 30_000, f"{who} held it in {r.get(who)} ms")
        return {"ms": ms, "reached_ms": r}

    def via_words(viewer, x):
        return "Dee, through your computer" if viewer == x else f"Dee, through {x}’s computer"

    def b4():
        act = s.get("dee_act")
        if not act: raise Unread("no act of Dee's read")
        x = act["person"]
        need(act["on_behalf_of"] == "Dee" and x != "Dee", f"written as {act}")
        opened = ok("4", "lent")
        draw = took("4", "lent")
        need(draw <= 120_000, f"drawn in {draw} ms")
        need(under(opened, QUIET) == "What’s proposed" and "Agree Object" in opened, f"{opened[:400]}")
        mine = ok("4", "click")
        need(f"Agreed: you, through {x}’s computer." in mine, f"the phone: {mine[:500]}")
        seen = {"dee": mine}
        for k, viewer in (("5", "Cy"), ("6", "Bo")):
            text = ok(k, "read")
            seen[viewer] = text
            need(f"Agreed: {via_words(viewer, x)}." in text, f"{viewer}'s page: {text[:500]}")
        return {"through": x, "drawn_ms": draw, "listed_ms": took("4", "poll")}

    def b5():
        asked = ok("5", "click", 0)
        need("Why do you object?" in asked, "no why asked")
        text = ok("5", "click", 1)
        need(f"You object: “{WHY}”" in text, f"{text[:500]}")
        need(under(text, QUIET) == "What’s proposed", "no longer proposed")
        r = s.get("reached_5") or {}
        return {"ms": took("5", "click", 1), "reached_ms": r}

    def b6():
        before = ok("6", "read")
        need(f"Cy objects: “{WHY}”" in before, f"Bo's page: {before[:500]}")
        texts = {"Bo": ok("6", "click"), "Ana": ok("6-ana", "wait"), "Cy": ok("6-cy", "wait"), "Dee": ok("6-dee", "lent")}
        for who, text in texts.items():
            whose = "you" if who == "Ana" else "Ana"
            need(re.search(r"Your house’s rules\n+" + re.escape(f"{QUIET} ({whose} proposed it on ") + DAY + r"\)", text), f"{who}: {text[:400]}")
        r = s.get("reached_6") or {}
        for who in ("ana", "cy"): need(r.get(who) is not None and r[who] <= 30_000, f"{who} held it in {r.get(who)} ms")
        need(s.get("held_6_ms", 10**9) <= 30_000, f"every page read by {s.get('held_6_ms')} ms")
        return {"reached_ms": r, "every_page_ms": s["held_6_ms"], "phone_listed_ms": took("6-dee", "poll")}

    def left(names, gone, steps_):
        for k, viewer, kind in steps_:
            text = ok(k, kind)
            need(under(text, QUIET) == "Your house’s rules", f"{k}: the quiet hours under {under(text, QUIET)}")
            need(say(text, viewer) == sorted(names), f"{k}: {say(text, viewer)} have a say")
            for g in gone: need(re.search(rf"{g} had a say( from {DAY})? until {DAY}, when they left\.", text), f"{k}: {g}'s leave not said")

    def b7():
        h = s.get("held_out_7")
        if not h: raise Unread("no leave watched")
        need(ok("7", "click"), "")
        for who in ("ana", "cy"): need(h[who]["out"] and h[who]["ms"] <= 30_000, f"{who}: {h[who]}")
        left(["Ana", "Cy", "Dee"], ["Bo"], [("7-ana", "Ana", "wait"), ("7-cy", "Cy", "wait"), ("7-dee", "Dee", "lent")])
        return {"held_out": h, "phone_listed_ms": took("7-dee", "poll")}

    def b8():
        text = ok("8", "click")
        need(under(text, f"Change how your house decides: {EVERYONE}") == "What’s proposed", f"{text[:500]}")
        for k in ("8-ana", "8-cy"):
            t = ok(k, "poll" if k == "8-ana" else "wait")
            need(re.search(r"How your house decides\n+" + re.escape(EVERYONE), t), f"{k}: {t[:300]}")
        return {"proposed_ms": took("8", "click"), "agreed_to_read_ms": took("8-ana", "click") + took("8-ana", "poll"),
                "right_after_agree": ok("8-ana", "click").split("\n\nYour house’s rules")[0]}

    def b9():
        h = s.get("held_out_9")
        if not h: raise Unread("no leave watched")
        need(h["ana"]["out"] and h["ana"]["ms"] <= 30_000, f"{h}")
        text = ok("9-ana", "wait")
        need(say(text, "Ana") == ["Ana", "Dee"], f"{say(text, 'Ana')}")
        need(re.search(r"How your house decides\n+" + re.escape(EVERYONE), text), "not under everyone agrees")
        return {"held_out": h}

    def b10():
        text = ok("10", "click")
        need(under(text, f"Change how your house decides: {MOST}") == "What’s proposed", f"{text[:500]}")
        ok("10-ana", "click")
        t = ok("10-ana", "poll")
        ms = took("10-ana", "poll")
        need(re.search(r"How your house decides\n+" + re.escape(MOST), t), f"Ana: {t[:300]}")
        need(ms <= 30_000, f"read in {ms} ms after the press")
        d = ok("10-dee", "lent")
        need(re.search(r"How your house decides\n+" + re.escape(MOST), d), f"Dee: {d[:300]}")
        return {"phone_listed_ms": took("10", "poll"), "after_press_ms": ms,
                "right_after_agree": ok("10-ana", "click").split("\n\nYour house’s rules")[0]}

    def b11():
        need(s.get("changed", {}).get("ana_lists"), "Ana's computer lists no change")
        asked = ok("11-dee", "poll")
        need(re.search(r"Ana changed House rules’s files on " + DAY + r"\. Open the new version\?", asked), f"{asked[:600]}")
        need("Yes, use the new version of House rules" in asked, "no yes offered")
        need(s.get("asked_11_ms", 10**9) <= 60_000, f"asked by {s.get('asked_11_ms')} ms")
        for k, kind, want in (("11-propose", "click", "What’s proposed"), ("11-dee", "lent", "What’s proposed"),
                              ("11-ana", "wait", "What’s proposed"), ("11-yes", "lent", "Your house’s rules"),
                              ("11-ana-after", "wait", "What’s proposed")):
            t = ok(k, kind)
            need(under(t, ROOM) == want, f"{k}: the big room under {under(t, ROOM)}, not {want}")
        need("House rules opens at its new version for you now." in ok("11-yes", "click"), "the yes not said")
        return {"asked_ms": s["asked_11_ms"], "changed_ms": s["changed"]["ms"]}

    def b12():
        hits = []
        for k, v in st.items():
            for d in v["done"]:
                for m in NOT_SAID.finditer(d["text"]): hits.append([k, v["who"], m[0], d["text"][max(0, m.start() - 60):m.end() + 40]])
        need(not hits, f"{len(hits)} hits: {hits[:5]}")
        return {"read": sum(len(v["done"]) for v in st.values())}

    def b13():
        if not st: raise Unread("no step ran")
        bad = [[k, v["done"][-1].get("error", "")[:200]] for k, v in st.items() if v.get("failed")]
        errors = [[k, e] for k, v in st.items() for e in v.get("errors") or []]
        need(not bad and not errors, f"failed {bad}; errors {errors}")
        return {"steps": len(st)}

    verdicts = {str(i + 1): bar(f) for i, f in enumerate((b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13))}
    verdicts["14"] = {"verdict": "never-ran", "why": "a house with real people, a second host and a phone on cellular are the operator's"}
    out = Path(path).with_name("verdicts.json")
    out.write_text(json.dumps(verdicts, indent=1, ensure_ascii=False) + "\n")
    for k, v in verdicts.items(): print(k, v["verdict"], (v.get("why") or "")[:200])


if __name__ == "__main__":
    main(sys.argv[1])
