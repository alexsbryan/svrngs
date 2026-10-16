#!/usr/bin/env python3
"""Every command in demo/grapevine.md, run as written (row gv-run (c)): each
code block read from the document, in order, and typed into the window its
paragraph names — Sam's, Jo's or Kim's, each a shell of its own in this
copy of svrngs, or all three for "In each window" — with only the link a
window printed pasted where the document shows `svrngs://join/…`. Then the
scene's buttons pressed on the pages the start lines printed (people.mjs),
and "When you're done" typed: Control-C, `kill %1`, `rm -rf`.

    python3 demo/runs/grapevine/2026-10-16/doc.py

Its transcript, every line typed and what the window printed, links
redacted, goes to data/doc/transcript.txt. It refuses to run if
/tmp/grapevine is there already: the document's home is its scratch."""
import json, os, re, signal, subprocess, sys, time, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
DOC = REPO / "demo/grapevine.md"; OUT = HERE / "data/doc"; HOMES = Path("/tmp/grapevine")
WINDOWS = ("sam", "jo", "kim")
LINK = "'svrngs://join/…'"
transcript, hidden = [], []


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
        named = ("In a terminal, in your copy", "In Sam's window", "In Jo's window", "In Kim's window", "In each window", "Then:")
        who = max((w for w in named if w in before), key=before.rfind, default=None)
        out.append((who, [l for l in m.group(1).splitlines() if l.strip()]))
    return out


class Window:
    def __init__(self, who):
        self.who, self.log = who, OUT / f"{who}.out"
        self.p = subprocess.Popen(["bash", "-s"], cwd=REPO, stdin=subprocess.PIPE, stdout=open(self.log, "w"),
                                  stderr=subprocess.STDOUT, text=True, env={**os.environ, "CARGO_INCREMENTAL": "0"}, start_new_session=True)

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


def main():
    if HOMES.exists(): sys.exit(f"{HOMES} is there already; the document's scratch must start empty")
    OUT.mkdir(parents=True, exist_ok=True)
    plan = blocks()
    note(f"demo/grapevine.md at {subprocess.run(['git', '-C', REPO, 'rev-parse', '--short', 'HEAD'], capture_output=True, text=True).stdout.strip()}: "
         f"{len(plan)} blocks, {sum(len(l) for _, l in plan)} lines")
    wins = {}
    links = {}
    at = {}
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
            for l in lines: wins["sam"].type(l)
            continue
        targets = WINDOWS if whose == "In each window" else [whose.split()[1].rstrip("'s").lower()]
        for w in targets:
            win = wins[w]
            for l in lines:
                link = links.get("sam") if w == "jo" else links.get("jo") if w == "kim" else None
                if LINK in l and not link: sys.exit(f"{w}'s line wants a link no window printed yet")
                mark = win.type(l, link)
                if " new " in l:
                    m = win.wait(r"svrngs://join/(\S+)", mark)
                    hidden.append(m.group(1)); links[w] = m.group(0)
                    note(f"[{w}] printed a link")
                elif " join " in l:
                    joined = win.wait(r"You're in [^\n]*|svrngs: [^\n]*", mark).group(0)
                    note(f"[{w}] said: {joined}")
                elif "server.mjs" in l:
                    m = win.wait(r'\{"port":\d+,"at":"([^"]+)"[^\n]*', mark)
                    at[w] = m.group(1)
                    note(f"[{w}] printed: {m.group(0)}")
                elif "> $HOME/grapevine.json" in l:
                    # A person types the next line once this one is back; the api file must be there.
                    time.sleep(0.5)
                    note(f"[{w}] grapevine.json: {redact((HOMES / w / 'grapevine.json').read_text().strip()) if (HOMES / w / 'grapevine.json').exists() else 'not written'}")
                else:
                    time.sleep(0.5)
        if whose == "In each window":
            scene(at)
            for w in WINDOWS:
                # "In each window, press Control-C to stop grapevine": the node the window runs in front.
                kids = subprocess.run(["pgrep", "-P", str(wins[w].p.pid), "node"], capture_output=True, text=True).stdout.split()
                for k in kids: os.kill(int(k), signal.SIGINT)
                note(f"[{w}] Control-C ({len(kids)} program stopped)")
                time.sleep(0.5)
                wins[w].type("kill %1")
    time.sleep(2)
    for w, win in wins.items():
        win.p.stdin.close()
        try: win.p.wait(20)
        except subprocess.TimeoutExpired: win.p.kill()
        left = subprocess.run(["pgrep", "-f", f"{HOMES}/{w}"], capture_output=True, text=True).stdout.split()
        note(f"[{w}] window closed, exit {win.p.returncode}; processes left: {len(left)}")
    left = subprocess.run(["pgrep", "-f", "target/debug/svrngs"], capture_output=True, text=True).stdout.split()
    note(f"/tmp/grapevine there after: {HOMES.exists()}")
    for w in WINDOWS: (OUT / f"{w}.out").write_text(redact((OUT / f"{w}.out").read_text(errors="replace")))
    (OUT / "transcript.txt").write_text("\n".join(transcript) + "\n")


def scene(at):
    """Steps 5-11 pressed as the document says, on each person's page (people.mjs)."""
    b = subprocess.Popen(["node", HERE / "people.mjs"], cwd=HERE, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, stderr=open(OUT / "people.err", "w"))
    json.loads(b.stdout.readline())

    def tell(**c):
        b.stdin.write(json.dumps(c) + "\n"); b.stdin.flush()
        got = json.loads(b.stdout.readline())
        ok = got.get("met", not got.get("absent") and not got.get("error") and got.get("said") is None)
        note(f"[{c['who']}] {c['do']} {c.get('button') or c.get('words') or c.get('gone') or c.get('url')}: {'ok' if ok else 'NOT ' + str(got.get('error') or got.get('said') or 'met')} ({got.get('ms')} ms)")
        if not ok: sys.exit("the scene did not go as the document says")
        return got

    for w in WINDOWS:
        tell(do="open", who=w)
        got = tell(do="goto", who=w, url=at[w])
        note(f"[{w}] page shows {got['groups']}")
    name = "Your name, which they’ll see once you’ve both said yes"
    tell(do="press", who="kim", within=[["section", "What you have"], ["div.kind", "tool"]], fill={"What": "tile saw"}, select={"In": "Waller block"}, button="List it")
    tell(do="press", who="sam", within=[["section", "Ask your groups"], ["div.kind", "tool"]], fill={"What": "tile saw", "When": "Saturday"},
         select={"In": "1400 Page", "How far": "One ring away"}, button="Ask")
    tell(do="await", who="jo", words=["Sam asks: tile saw, Saturday", "Carry to Waller block", "Everyone in Waller block will see what was asked, and that you carried it; never who asked."])
    tell(do="press", who="jo", within=[["li.ask", "tile saw"]], button="Carry to Waller block")
    tell(do="await", who="kim", words=["A neighbour of Jo’s needs: tile saw, Saturday — carried by Jo", "You might have this."])
    tell(do="press", who="kim", within=[["li.ask", "tile saw"]], fill={"When": "Saturday morning", "Where": "Waller and Pierce"}, button="Offer")
    tell(do="await", who="sam", words=["Someone who saw it offered: when Saturday morning; where Waller and Pierce", "Sealed: only you two can read this."])
    tell(do="await", who="jo", words=["A sealed message went by toward the person who asked: 96 bytes"])
    tell(do="press", who="sam", within=[["li.thread", "About your ask"]], fill={name: "Sam Reyes"}, button="Yes")
    got = tell(do="await", who="kim", words=["The person who asked said yes"])
    if "Sam Reyes" in got["text"]: sys.exit("Kim's page named Sam before her own yes")
    tell(do="press", who="kim", within=[["li.thread", "which you offered to help with"]], fill={name: "Kim Okafor"}, button="Yes")
    tell(do="await", who="kim", words=["You both said yes: they’re Sam Reyes"])
    tell(do="await", who="sam", words=["they’re Kim Okafor"])
    tell(do="press", who="sam", within=[["li.ask", "tile saw"]], button="Take it back")
    tell(do="await", who="kim", gone=["needs: tile saw, Saturday"])
    b.stdin.close(); b.wait(30)


if __name__ == "__main__":
    main()
