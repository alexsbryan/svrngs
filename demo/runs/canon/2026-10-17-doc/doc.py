#!/usr/bin/env python3
"""Every command in demo/house-rules.md, run as written (row canon-docs (c)),
the way demo/runs/grapevine/2026-10-16/doc.py runs demo/grapevine.md: each
code block read from the document, in order, and typed into the window its
paragraph names, Ana's, Bo's or Cy's, each a shell of its own in this copy
of svrngs, with only the link Ana's window printed pasted where the document
shows `svrngs://join/…`. Then steps 5-10 pressed on the home pages the
windows printed last, each person a Chromium of their own (canon-run's
people.mjs), and "When you're done" typed: `kill %1` in each, then `rm -rf`.

A home page's **Open House rules** is a link opening a tab; people.mjs keeps
one page a person, so the step waits for the home page to show that link
and opens the address it holds, as the computer lists it (`/v1/lends`, the
`origin` of the House-rules lend whose key is that computer's).

    python3 demo/runs/canon/2026-10-17-doc/doc.py

Its transcript, every line typed and what the window printed, links
redacted, goes to data/transcript.txt. It refuses to run if /tmp/house is
there already: the document's home is its scratch, so what is there when it
stops is its own, and it removes it, as the document's last line does."""
import json, os, re, shutil, signal, subprocess, sys, time, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
DOC = REPO / "demo/house-rules.md"; OUT = HERE / "data"; HOMES = Path("/tmp/house")
PEOPLE = HERE.parent / "2026-10-17-house/people.mjs"
WINDOWS = ("ana", "bo", "cy")
LINK = "'svrngs://join/…'"
HOME = r"http://svrngs\.localhost:\d+"
QUIET, WHY = "Quiet hours are 10pm to 7am", "I work late on Fridays"
DAY = r"\d{1,2} \w{3}"
DRAWN = r"How your house decides\n|How does your house decide its rules\?"
transcript, hidden, MADE = [], [], False


def redact(t):
    for h in hidden: t = t.replace(h, "<the token>")
    return t


def note(line):
    transcript.append(redact(line)); print(redact(line)[:300], flush=True)


def blocks():
    """(whose window, its lines) for each code block, by the words just before it."""
    text = DOC.read_text()
    out = []
    for m in re.finditer(r"```\n(.*?)```", text, re.S):
        before = text[max(0, m.start() - 400):m.start()]
        named = ("In a terminal, in your copy", "In Ana's window", "In Bo's window", "In Cy's window", "In each window", "Then:")
        who = max((w for w in named if w in before), key=before.rfind, default=None)
        out.append((who, [l for l in m.group(1).splitlines() if l.strip()]))
    return out


class Window:
    opened = []

    def __init__(self, who):
        self.who, self.log = who, OUT / f"{who}.out"
        self.p = subprocess.Popen(["bash", "-s"], cwd=REPO, stdin=subprocess.PIPE, stdout=open(self.log, "w"),
                                  stderr=subprocess.STDOUT, text=True, env={**os.environ, "CARGO_INCREMENTAL": "0"}, start_new_session=True)
        Window.opened.append(self.p)

    def said(self): return self.log.read_text(errors="replace")

    def type(self, line, link=None):
        sent = line.replace(LINK, f"'{link}'") if link else line
        mark = len(self.said())
        note(f"[{self.who}] $ {sent}")
        self.p.stdin.write(sent + "\n"); self.p.stdin.flush()
        return mark

    def wait(self, pattern, since=0, bound=90):
        t = time.time()
        while time.time() - t < bound:
            m = re.search(pattern, self.said()[since:])
            if m: return m
            time.sleep(0.2)
        sys.exit(f"{self.who}'s window never said {pattern!r}:\n{redact(self.said()[since:])[-800:]}")


def api(who, path):
    port = (HOMES / who / ".svrngs/api").read_text().strip().rstrip("/").rsplit(":", 1)[1]
    with urllib.request.urlopen(f"http://127.0.0.1:{port}{path}", timeout=30) as r: return json.loads(r.read())


def rules_at(who):
    """The address who's own Open House rules link holds: their computer's copy."""
    g = next(g for g in api(who, "/v1/groups")["groups"] if g.get("name") == "Elm")
    keys = {p["person"]: p.get("key") for p in api(who, f"/v1/rail/people?namespace={g['id']}")["people"] if p.get("device") == "computer"}
    return next(l["origin"] for l in api(who, f"/v1/lends?namespace={g['id']}")["lent"]
                if l.get("kind") == "House-rules" and l.get("key") == keys[who.capitalize()])


def main():
    global MADE
    if HOMES.exists(): sys.exit(f"{HOMES} is there already; the document's scratch must start empty")
    MADE = True
    OUT.mkdir(parents=True, exist_ok=True)
    plan = blocks()
    note(f"demo/house-rules.md at {subprocess.run(['git', '-C', REPO, 'rev-parse', '--short', 'HEAD'], capture_output=True, text=True).stdout.strip()}: "
         f"{len(plan)} blocks, {sum(len(l) for _, l in plan)} lines")
    wins, home, link = {}, {}, None
    t0 = time.time()
    for whose, lines in plan:
        if whose is None: sys.exit(f"a block with no window named before it: {lines}")
        if whose == "In a terminal, in your copy":
            for l in lines:
                note(f"[copy] $ {l}")
                r = subprocess.run(["bash", "-c", l], cwd=REPO, capture_output=True, text=True, env={**os.environ, "CARGO_INCREMENTAL": "0"})
                note(f"[copy] exit {r.returncode} {r.stdout[-300:]}{r.stderr[-300:]}")
                if r.returncode: sys.exit("the build failed")
            for w in WINDOWS: wins[w] = Window(w)
            continue
        if whose == "Then:":
            scene(home)
            # "In each window, type `kill %1`".
            for w in WINDOWS: wins[w].type("kill %1")
            time.sleep(2)
            for l in lines: wins["ana"].type(l)
            continue
        if whose == "In each window":
            for w in WINDOWS:
                for l in lines:
                    home[w] = wins[w].wait(HOME, wins[w].type(l)).group(0) + "/"
                    note(f"[{w}] home page: {home[w]}")
            continue
        w = whose.split()[1].rstrip("'s").lower()
        win = wins[w]
        for l in lines:
            if LINK in l and not link: sys.exit(f"{w}'s line wants a link no window printed yet")
            mark = win.type(l, link)
            if " new " in l:
                m = win.wait(r"svrngs://join/(\S+)", mark)
                hidden.append(m.group(1)); link = m.group(0)
                note(f"[{w}] printed a link")
            elif " join " in l:
                joined = win.wait(r"You're in [^\n]*|svrngs: [^\n]*", mark).group(0)
                note(f"[{w}] said: {joined}")
                if not joined.startswith(f"You're in Elm, as {w.capitalize()}"): sys.exit(f"{w} did not join as the document says")
            else:
                time.sleep(0.5)
    time.sleep(2)
    for w, win in wins.items():
        win.p.stdin.close()
        try: win.p.wait(20)
        except subprocess.TimeoutExpired: win.p.kill()
        left = subprocess.run(["pgrep", "-f", f"{HOMES}/{w}"], capture_output=True, text=True).stdout.split()
        note(f"[{w}] window closed, exit {win.p.returncode}; processes left: {len(left)}")
    note(f"/tmp/house there after: {HOMES.exists()}; {round(time.time() - t0, 1)} s from the first window")
    for w in WINDOWS: (OUT / f"{w}.out").write_text(redact((OUT / f"{w}.out").read_text(errors="replace")))
    (OUT / "transcript.txt").write_text("\n".join(transcript) + "\n")


def scene(home):
    """Steps 5-10 pressed as the document says, on each person's page (people.mjs)."""
    b = subprocess.Popen(["node", PEOPLE], cwd=PEOPLE.parent, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True,
                         stderr=open(OUT / "people.err", "w"))
    json.loads(b.stdout.readline())

    def steps(step, who, *ss):
        b.stdin.write(json.dumps({"do": "steps", "who": who, "steps": list(ss)}) + "\n"); b.stdin.flush()
        got = json.loads(b.stdout.readline())
        done = got.get("done") or [{}]
        bad = got.get("error") or got.get("errors") or (done[-1].get("error") if got.get("failed") else None)
        note(f"[{who}] step {step}: {'ok' if not bad else 'NOT ' + str(bad)} ({got.get('ms')} ms)")
        if bad: sys.exit(f"the scene did not go as the document says, at step {step}:\n{redact(done[-1].get('text', ''))[-1200:]}")
        return done

    def says(step, text, *res):
        for r in res:
            if not re.search(r, text): sys.exit(f"step {step}: the page never said {r!r}:\n{redact(text)[-1200:]}")

    for w in WINDOWS:
        b.stdin.write(json.dumps({"do": "open", "who": w, "engine": "chromium"}) + "\n"); b.stdin.flush()
        json.loads(b.stdout.readline())
    open_it = lambda w: {"poll": home[w], "re": "Open House rules", "within": 60_000, "every": 2_000}

    # 5. Ana brings House rules, Money and Events unticked.
    done = steps(5, "ana", {"goto": home["ana"]}, {"wait": "Bring the ticked ones to Elm"},
                 {"count": "#home label:has-text('Money') input:checked"}, {"count": "#home label:has-text('Events') input:checked"},
                 {"count": "#home label:has-text('House rules') input:checked"},
                 {"uncheck": "#home label:has-text('Money') input"}, {"uncheck": "#home label:has-text('Events') input"},
                 {"click": "Bring the ticked ones to Elm", "until": "You brought|Nothing was brought|already"})
    if [d.get("value") for d in done[2:5]] != [1, 1, 1]: sys.exit(f"step 5: Money, Events and House rules not each ticked: {[d.get('value') for d in done[2:5]]}")
    says(5, done[-1]["text"], r"You brought House rules to Elm: everyone in it can open it in a few seconds\.")

    # 6. Ana opens House rules and says how the house decides.
    steps("6, the link", "ana", open_it("ana"))
    done = steps(6, "ana", {"goto": rules_at("ana")}, {"wait": DRAWN, "within": 60_000},
                 {"count": "input[name=way]"}, {"count": "input[name=way]:checked"},
                 {"check": "input[name=way][value=most]"},
                 {"click": "Make this how your house decides", "until": r"How your house decides\n"})
    if [d.get("value") for d in done[2:4]] != [3, 0]: sys.exit(f"step 6: not three choices, none ticked: {[d.get('value') for d in done[2:4]]}")
    says(6, done[1]["text"], r"How does your house decide its rules\?", r"Most of you agree\.")
    says(6, done[-1]["text"], r"A rule holds once more than half of the people in your house when it was proposed agree, whoever proposed it among them\.")

    # 7. Ana proposes quiet hours.
    done = steps(7, "ana", {"fill": "#rule", "value": QUIET},
                 {"click": "Propose this rule to everyone in your house", "until": "You proposed it on"})
    says(7, done[-1]["text"], rf"What’s proposed\n+{QUIET}\n+You proposed it on {DAY}\.")

    # 8. Cy objects.
    steps("8, the link", "cy", open_it("cy"))
    done = steps(8, "cy", {"goto": rules_at("cy")}, {"wait": QUIET, "within": 60_000},
                 {"click": "Object", "in": QUIET, "until": r"Why do you object\?"}, {"fill": "input.why", "value": WHY},
                 {"click": "Send your objection", "until": "You object"})
    says(8, done[-1]["text"], f"You object: “{WHY}”", r"Your house has no rules yet\.")

    # 9. Bo agrees; it holds, on Ana's page too.
    steps("9, the link", "bo", open_it("bo"))
    done = steps(9, "bo", {"goto": rules_at("bo")}, {"wait": f"Cy objects: “{WHY}”", "within": 60_000},
                 {"click": "Agree", "in": QUIET, "until": "Your house’s rules\n+" + QUIET})
    says(9, done[-1]["text"], rf"{QUIET} \(Ana proposed it on {DAY}\)")
    done = steps("9, Ana's", "ana", {"goto": rules_at("ana")}, {"wait": DRAWN, "within": 60_000})
    says("9, Ana's", done[-1]["text"], rf"{QUIET} \(you proposed it on {DAY}\)")

    # 10. Bo moves out; Ana's page, reloaded. The two are named in the order
    # svrngs's `spans` lists them (house.js `standing`), which differs from
    # one house to the next: the document names them, not their order.
    done = steps(10, "bo", {"goto": home["bo"]}, {"wait": "Leave Elm"}, {"click": "Leave Elm", "until": r"You left Elm\."})
    done = steps("10, Ana's", "ana", {"poll": rules_at("ana"), "re": r"Everyone in your house has a say: (Cy and you|you and Cy)\.", "within": 60_000, "every": 2_000})
    says("10, Ana's", done[-1]["text"], rf"Your house’s rules\n+{QUIET} \(you proposed it on {DAY}\)",
         rf"Bo had a say from {DAY} until {DAY}, when they left\.")
    b.stdin.close(); b.wait(30)


if __name__ == "__main__":
    try: main()
    finally:
        # A check that stops early stops every window it opened, and what runs in each (their own sessions).
        for p in Window.opened:
            try: os.killpg(p.pid, signal.SIGINT)
            except ProcessLookupError: pass
        if MADE: shutil.rmtree(HOMES, ignore_errors=True)
