#!/usr/bin/env python3
"""The dinner's link, and Sam's computer asleep (PREDICTIONS.md here; row
events-doc-link): events-run's driver (../2026-10-14/run.py) cut to its
setup, the post, the link's steps and the asleep steps. This commit's
svrngs, every computer a HOME and data dir of its own under W; the page
built and served on this host; the reference app run on Sam's computer and
lent by it; computers in Chromium keeping no group, phones in WebKit as
iPhones (people.mjs), every page saved in data/shots/.

    W=<scratch> python3 demo/runs/events/2026-10-14-links/run.py < /dev/null

Data in data/ beside this file, the join links' tokens never in it; the
dinner's links carry none. scripts/check.sh runs none of this."""
import json, os, re, shutil, signal, subprocess, sys, threading, time, urllib.error, urllib.parse, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
OUT = HERE / "data"; SHOTS = OUT / "shots"; W = Path(os.environ["W"]); BIN = REPO / "target/debug/svrngs"
HOUSE, DWEB = "The House", "dWeb"
LOG = "info,door=debug,origin=debug,transport=debug,cli=debug"
DINNER = {"what": "Dinner", "when": "Friday 7", "where": "the house"}
TOLD = ["Open Sam’s events", "did not answer"]

procs, outs, summary, hidden, views = {}, {}, {}, [], []
shots = [0]


def head(): return subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()


def redact(text):
    for h in hidden: text = text.replace(h, "<the token>")
    return text


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}, ensure_ascii=False))
    print(line[:600], flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def ms(t0): return round((time.time() - t0) * 1000)


def wait(cond, bound, every=0.25):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def http(method, url, timeout=15):
    try:
        with urllib.request.urlopen(urllib.request.Request(url, method=method), timeout=timeout) as r: status, blob = r.status, r.read()
    except urllib.error.HTTPError as e: status, blob = e.code, e.read()
    except Exception as e: return None, str(e)
    try: return status, json.loads(blob)
    except Exception: return status, blob.decode(errors="replace")


# --- each computer: a data dir and HOME of its own --------------------------
def env(who):
    return {**os.environ, "HOME": str(W / who), "USER": who, "SVRNGS_DIR": str(W / who / ".svrngs"), "NO_COLOR": "1", "RUST_LOG": LOG}


def start(who, *args):
    (W / who).mkdir(parents=True, exist_ok=True)
    outs[who] = W / f"{who}.{len(list(W.glob(f'{who}.*.out'))) + 1}.out"
    procs[who] = subprocess.Popen([BIN, *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(outs[who], "w"), stderr=open(W / f"{who}.log", "a"))
    say("svrngs", who=who, args=[a if not a.startswith(("svrngs://", "http")) else "<link>" for a in args])
    return time.time()


def stop(who):
    """who's computer asleep: its svrngs stopped as Ctrl-C stops it."""
    t = time.time()
    p = procs.pop(who)
    p.send_signal(signal.SIGINT)
    try: p.wait(20)
    except subprocess.TimeoutExpired: p.kill(); p.wait()
    say("asleep", who=who, ms=ms(t), exit=p.returncode)


def said(who): return outs[who].read_text() if who in outs and outs[who].exists() else ""


def api(who, path):
    try: base = (W / who / ".svrngs/api").read_text().strip().rstrip("/")
    except Exception: return None
    s, b = http("GET", base + path)
    return b if s == 200 else None


def up(who, bound=60):
    if not wait(lambda: api(who, "/v1/groups"), bound): sys.exit(f"{who}'s svrngs did not answer: {said(who)[-400:]}")


def links(who, bound=90):
    got = wait(lambda: re.search(r"svrngs://join/(\S+)", said(who)), bound)
    if not got: sys.exit(f"{who} printed no link: {said(who)[-600:]}")
    hidden.append(got[1])
    page = re.search(r"(http://localhost:\d+/#(\S+))", said(who))
    if page: hidden.append(page[2])
    return got[0], page and page[1]


def joined(who, link, name):
    t = start(who, "join", link, "--as", name)
    got = wait(lambda: re.search(r"You're in .*|svrngs: .*", said(who)), 90)
    say("joined", who=who, said=got and got[0], ms=ms(t))
    if not got or not got[0].startswith("You're in"): sys.exit(f"{who} did not join: {said(who)[-400:]}")


def lends(who, gid):
    return {(l["person"], l["name"]): l["origin"] for l in (api(who, f"/v1/lends?namespace={gid}") or {}).get("lent", [])}


def home(who): return f"http://svrngs.localhost:{urllib.parse.urlsplit((W / who / '.svrngs/api').read_text().strip()).port}/"


# --- each person's browser ---------------------------------------------------
browser = None


def tell(step, **c):
    shots[0] += 1
    c["shot"] = f"{shots[0]:02d}-{c['who']}-{step}"
    browser.stdin.write(json.dumps(c) + "\n"); browser.stdin.flush()
    line = browser.stdout.readline()
    got = json.loads(line) if line else {"error": "people.mjs stopped"}
    say("person", step=step, command={k: (redact(v) if isinstance(v, str) else v) for k, v in c.items()}, **got)
    views.append({"step": step, "do": c["do"], "who": c["who"], "shot": c["shot"], **got})
    return got


def app():
    """The reference app as it is at this commit, on Sam's computer."""
    (W / "app.json").write_text(json.dumps({"record": (W / "sam/.svrngs/api").read_text().strip().rstrip("/"), "who": "x-svrngs-key"}))
    procs["app"] = subprocess.Popen(["node", REPO / "apps/events/server.mjs", W / "app.json"], env={**os.environ, "EVENTS_DEBUG": "1"},
                                    stdout=subprocess.PIPE, stderr=open(W / "app.trace", "w"), text=True)
    return json.loads(procs["app"].stdout.readline())["port"]


def asleep(how, H, at, cy_at):
    """Bo's pasted link, his home page, Cy's dinner opened again and her
    link anew, while Sam's computer is asleep `how`; then awake again."""
    got = {}
    if how == "frozen":
        got["held"] = tell("L6", do="goto", who="cy", url=cy_at + "?n=held")
        procs["sam"].send_signal(signal.SIGSTOP); say("frozen", who="sam")
        got["cy_again"] = tell("L6", do="goto", who="cy", url=cy_at + "?n=frozen")
    else:
        stop("sam")
    step = "L5" if how == "stopped" else "L6"
    got["bo_paste"] = tell(step, do="paste", who="bo", home=home("bo"), link=at[H])
    got["bo_home"] = tell(step, do="home", who="bo", url=home("bo"), words=TOLD)
    if how == "stopped": got["cy_again"] = tell(step, do="goto", who="cy", url=cy_at + "?n=stopped")
    got["cy_link"] = tell(step, do="link", who="cy", url=at[H])
    t = time.time()
    if how == "frozen": procs["sam"].send_signal(signal.SIGCONT); say("woke", who="sam")
    else: start("sam")
    up("sam")
    back = lambda: (lambda v: v if v.get("status") == 200 and "Dinner" in (v.get("text") or "") else None)(
        tell(step + "w", do="paste", who="bo", home=home("bo"), link=at[H]))
    got["woke"] = {"bo": wait(back, 90, 2), "bo_ms": ms(t)}
    got["woke"]["cy"] = tell(step + "w", do="link", who="cy", url=at[H])
    got["woke"]["cy_ms"] = ms(t)
    say(how, **{k: {kk: vv for kk, vv in (v or {}).items() if kk in ("ms", "status", "refused", "url", "landed", "said")} for k, v in got.items() if k != "woke"})
    return got


def main():
    global browser
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    shutil.rmtree(OUT, ignore_errors=True); SHOTS.mkdir(parents=True)
    say("start", head=head(), load=round(os.getloadavg()[0], 1), cores=os.cpu_count(),
        dirty=subprocess.run(["git", "-C", REPO, "status", "--short"], capture_output=True, text=True).stdout.splitlines())
    try:
        t = time.time()
        subprocess.run(["cargo", "build", "--quiet", "-p", "svrngs"], cwd=REPO, check=True, env={**os.environ, "CARGO_INCREMENTAL": "0"})
        subprocess.run([REPO / "scripts/build-web.sh", "--out", W / "site"], check=True, capture_output=True)
        procs["page"] = subprocess.Popen(["node", REPO / "web/tests/serve.mjs", W / "site"], stdout=subprocess.PIPE, text=True)
        port = re.search(r"port (\d+)", procs["page"].stdout.readline())[1]
        threading.Thread(target=lambda: [None for _ in procs["page"].stdout], daemon=True).start()
        PAGE = f"http://localhost:{port}/"
        say("built", s=round(time.time() - t), page=PAGE)
        browser = subprocess.Popen(["node", HERE / "people.mjs"], cwd=HERE, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=open(W / "people.err", "w"), text=True, env={**os.environ, "SHOTS": str(SHOTS)})
        if not json.loads(browser.stdout.readline()).get("ready"): sys.exit("people.mjs did not start")
        for who, engine in (("sam", "chromium"), ("bo", "chromium"), ("di", "chromium"), ("cy", "webkit"), ("eve", "webkit")):
            browser.stdin.write(json.dumps({"do": "open", "who": who, "engine": engine}) + "\n"); browser.stdin.flush()
            browser.stdout.readline()

        # The House: Sam makes it; Bo joins by its link, Cy's phone by its page. dWeb: Di, Eve's phone.
        start("sam", "new", HOUSE, "--as", "Sam", "--origin", PAGE)
        house_link, house_page = links("sam")
        joined("bo", house_link, "Bo")
        tell("0", do="join", who="cy", link=house_page, name="Cy")
        stop("sam")
        start("sam", "new", DWEB, "--as", "Sam", "--origin", PAGE)
        dweb_link, dweb_page = links("sam")
        joined("di", dweb_link, "Di")
        tell("0", do="join", who="eve", link=dweb_page, name="Eve")
        groups = {g["name"]: g["id"] for g in api("sam", "/v1/groups")["groups"]}
        H, D = groups[HOUSE], groups[DWEB]
        summary["groups"] = groups

        # The app on Sam's computer, lent.
        app_port = app()
        share = subprocess.run([BIN, "share", str(app_port), "--kind", "events", "--name", "events"], env={**env("sam"), "RUST_LOG": "error"},
                               stdin=subprocess.DEVNULL, capture_output=True, text=True)
        summary["share"] = {"exit": share.returncode, "said": (share.stdout + share.stderr).strip()}
        for who, gid in (("sam", H), ("sam", D), ("bo", H), ("di", D)):
            if not wait(lambda: ("Sam", "events") in lends(who, gid), 120, 0.5): sys.exit(f"{who} lists no events in {gid}")
        origin = lends("sam", H)[("Sam", "events")].rstrip("/")
        label = urllib.parse.urlsplit(origin).hostname.split(".")[0]
        summary["lend"] = {"label": label, "page": PAGE, "sam": origin}

        # Sam posts the dinner to both groups.
        tell("1", do="goto", who="sam", url=origin + "/")
        posted = tell("1", do="press", who="sam", button="Post", fill={f"input[name={k}]": v for k, v in DINNER.items()},
                      check=[f"input[value='{H}']", f"input[value='{D}']"])
        m = re.search(r"/g/([^/]+)/e/([0-9a-f]+)$", posted.get("url") or "")
        if not m: sys.exit(f"the post landed on no event: {posted}")
        E = m[2]
        summary["event"] = {"id": E, "landed": posted.get("url"), "ms": posted.get("ms")}

        # L1. Sam copies each group's link.
        at = {}
        for gid in (H, D):
            copied = tell("L1", do="copy", who="sam", group=gid)
            at[gid] = copied.get("box")
        summary["links"] = at
        if not all(at.values()): sys.exit(f"no link on Sam's page: {at}")

        # L2. The phones open their group's link, and answer.
        cy = tell("L2", do="link", who="cy", url=at[H])
        tell("L2", do="press", who="cy", button="I'm coming")
        tell("L2", do="link", who="eve", url=at[D])
        tell("L2", do="press", who="eve", button="I'm coming")
        cy_at = (cy.get("url") or "").split("?")[0]

        # L3. The computers open theirs: the page's words, then pasted on the home page.
        for who, gid in (("bo", H), ("di", D)):
            opened = tell("L3", do="link", who=who, url=at[gid])
            line = (opened.get("said") or {}).get("line") or at[gid]
            tell("L3", do="paste", who=who, home=home(who), link=line)
            tell("L3", do="press", who=who, button="I'm coming")

        # L4. The other group's link.
        tell("L4", do="link", who="cy", url=at[D])
        tell("L4", do="paste", who="di", home=home("di"), link=at[H])

        # Sam's page with the answers; Cy back on the dinner.
        tell("L4s", do="goto", who="sam", url=f"{origin}/g/{H}/e/{E}")
        summary["stopped"] = asleep("stopped", H, at, cy_at)
        summary["frozen"] = asleep("frozen", H, at, cy_at)
    finally:
        if "sam" in procs: procs["sam"].send_signal(signal.SIGCONT)
        if browser:
            try: browser.stdin.close(); browser.wait(30)
            except Exception: browser.kill()
        for name, p in list(procs.items()):
            p.send_signal(signal.SIGINT if name not in ("page", "app") else signal.SIGTERM)
        for p in procs.values():
            try: p.wait(20)
            except Exception: p.kill()
        summary.update({"views": views, "end_load": round(os.getloadavg()[0], 1)})
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, ensure_ascii=False)) + "\n")
        for f in ("app.trace", "people.err"):
            if (W / f).exists(): (OUT / f).write_text(redact((W / f).read_text()))
        for who in ("sam", "bo", "di"):
            if (W / f"{who}.log").exists():
                keep = [re.sub(r"\x1b\[[0-9;]*m", "", l) for l in (W / f"{who}.log").read_text(errors="replace").splitlines()
                        if re.search(r"door: |origin: |dial|asleep|refused|ERROR|WARN", l)]
                (OUT / f"{who}.log").write_text(redact("\n".join(keep[-3000:])) + "\n")
        say("end")


if __name__ == "__main__":
    main()
