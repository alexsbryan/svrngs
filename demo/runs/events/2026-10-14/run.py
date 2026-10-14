#!/usr/bin/env python3
"""Sam's dinner, drawn from The House and dWeb (PREDICTIONS.md here; row
events-run): this commit's svrngs, every computer a HOME and data dir of
its own under W; the page built and served on this host; the reference app
(apps/events) run on Sam's computer and lent by it; computers in Chromium
and phones in WebKit as iPhones (people.mjs), every press written down; each
computer's copy of each group's record folded by the app's fold (fold.mjs).

    W=<scratch> python3 demo/runs/events/2026-10-14/run.py
    PLANT=<cross|guest|no> W=<scratch> python3 demo/runs/events/2026-10-14/run.py

A plant runs the same story against a copy of the app with one change
(PLANTS), data in data/plants/<plant>/. judge.py reads the summary. Data in
data/ beside this file, the links' tokens never in it. scripts/check.sh runs
none of this."""
import html, json, os, re, shutil, signal, subprocess, sys, threading, time, urllib.error, urllib.parse, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
PLANT = os.environ.get("PLANT")
OUT = HERE / "data" / ("plants/" + PLANT if PLANT else ""); W = Path(os.environ["W"]); BIN = REPO / "target/debug/svrngs"
HOUSE, DWEB = "The House", "dWeb"
LOG = "info,door=debug,origin=debug,cli=debug,redeem=debug"
DINNER = {"what": "Dinner", "when": "Friday 7", "where": "the house"}

# Each plant: (file in the app, the text there, what replaces it). The run
# stops if a plant's text is not found, so a plant never silently misses.
PLANTS = {
    # Every page the host's view of every group this computer holds.
    "cross": [
        ("server.mjs", "if (e.host === me) {", "if (true || e.host === me) {"),
        ("server.mjs", "for (const g of groups.filter((x) => x.me || x.unread)) {", "for (const g of groups) {"),
        ("server.mjs", "answers: g.id === group ? here.answers : (await folded(g.id)).answers, me: g.me });",
         "answers: g.id === group ? here.answers : (await folded(g.id)).answers, me: e.host });"),
    ],
    # Every guest's "came" offered to everyone, written as the member who named them.
    "guest": [
        ("server.mjs", "for (const g of (guests ?? []).filter((x) => !(e.guests_came[me] ?? []).includes(x))) {",
         "for (const g of Object.values(e.coming).flat().filter((x) => !Object.values(e.guests_came).flat().includes(x))) {"),
        ("server.mjs", "else await write(group, { op: 'record', payload: { kind: CAME, event, ...(guest && { guest }) } }, me);",
         "else await write(group, { op: 'record', payload: { kind: CAME, event, ...(guest && { guest }) } }, "
         "(guest && Object.keys(here.events[event].coming).find((p) => here.events[event].coming[p].includes(guest))) || me);"),
    ],
    # "Take my name off" records a no, past the validator.
    "no": [
        ("server.mjs", "    if (newest) await write(group, { op: 'correct', corrects: newest, replacement: payload }, me);",
         "    if (newest && !payload) await store.append(group, { op: 'record', payload: { kind: 'not-coming', event } }, me);\n"
         "    else if (newest) await write(group, { op: 'correct', corrects: newest, replacement: payload }, me);"),
    ],
}

procs, outs, summary, hidden, presses, views = {}, {}, {}, [], [], []


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


def http(method, url, headers=None, timeout=15):
    """(status, body as JSON or text, headers) of one request, or (None, why, {})."""
    req = urllib.request.Request(url, method=method, headers=headers or {})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r: status, blob, hs = r.status, r.read(), dict(r.headers)
    except urllib.error.HTTPError as e: status, blob, hs = e.code, e.read(), dict(e.headers)
    except Exception as e: return None, str(e), {}
    try: return status, json.loads(blob), hs
    except Exception: return status, blob.decode(errors="replace"), hs


# --- each computer: a data dir and HOME of its own --------------------------
def env(who):
    return {**{k: v for k, v in os.environ.items() if k != "PLANT"}, "HOME": str(W / who), "USER": who,
            "SVRNGS_DIR": str(W / who / ".svrngs"), "NO_COLOR": "1", "RUST_LOG": LOG}


def start(who, *args):
    """A svrngs that runs on: bare, or as `new`, `link` and `join` run on."""
    (W / who).mkdir(parents=True, exist_ok=True)
    outs[who] = W / f"{who}.{len([p for p in W.glob(f'{who}.*.out')]) + 1}.out"
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
    s, b, _ = http("GET", base + path)
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


def command(who, *args, stdin="\n"):
    t = time.time()
    run = subprocess.run([BIN, *args], env={**env(who), "RUST_LOG": "error"}, input=stdin, capture_output=True, text=True)
    out = {"who": who, "args": list(args), "exit": run.returncode, "said": redact((run.stdout + run.stderr).strip()), "ms": ms(t)}
    say("command", **out)
    return out


def joined(who, link, name):
    t = start(who, "join", link, "--as", name)
    got = wait(lambda: re.search(r"You're in .*|svrngs: .*", said(who)), 90)
    out = {"said": got and got[0], "ms": ms(t)}
    say("joined", who=who, **out)
    if not got or not got[0].startswith("You're in"): sys.exit(f"{who} did not join: {said(who)[-400:]}")
    return out


def lends(who, gid):
    return {(l["person"], l["name"]): l["origin"] for l in (api(who, f"/v1/lends?namespace={gid}") or {}).get("lent", [])}


def people(who, gid):
    return (api(who, f"/v1/rail/people?namespace={gid}") or {}).get("people")


def log(who, gid): return api(who, f"/v1/rail/log?namespace={gid}")


# --- each person's browser ---------------------------------------------------
browser = None


def tell(**c):
    browser.stdin.write(json.dumps(c) + "\n"); browser.stdin.flush()
    line = browser.stdout.readline()
    got = json.loads(line) if line else {"error": "people.mjs stopped"}
    say("person", command={k: (redact(v) if isinstance(v, str) else v) for k, v in c.items()},
        **{k: v for k, v in got.items() if k not in ("logs",)})
    return got


def look(who, url, step):
    got = tell(**{"do": "goto", "who": who, "url": url})
    views.append({"step": step, "who": who, **got})
    return got


def press(who, gid, button, step, fill=None, check=None):
    got = tell(**{"do": "press", "who": who, "button": button, **({"fill": fill} if fill else {}), **({"check": check} if check else {})})
    presses.append({"step": step, "who": who, "group": gid, "button": button, "fill": fill, "check": check,
                    "absent": bool(got.get("absent")), "status": got.get("status"), "ms": got.get("ms"), "error": got.get("error")})
    if not got.get("absent"): views.append({"step": step, "who": who, **got})
    return got


def app():
    """The reference app, copied, a plant applied when one is named."""
    shutil.rmtree(W / "app", ignore_errors=True)
    shutil.copytree(REPO / "apps/events", W / "app")
    for name, old, new in PLANTS.get(PLANT, []):
        f = W / "app" / name
        text = f.read_text()
        if text.count(old) != 1: sys.exit(f"the {PLANT} plant's text is not in {name} once: {old}")
        f.write_text(text.replace(old, new))
    (W / "app.json").write_text(json.dumps({"record": (W / "sam/.svrngs/api").read_text().strip().rstrip("/"),
                                            "who": "x-svrngs-key"}))
    procs["app"] = subprocess.Popen(["node", W / "app/server.mjs", W / "app.json"], env={**os.environ, "EVENTS_DEBUG": "1"},
                                    stdout=subprocess.PIPE, stderr=open(W / "app.trace", "w"), text=True)
    return json.loads(procs["app"].stdout.readline())["port"]


def copies(groups):
    """Every computer's copy of each group's record, once each holds what
    Sam's does (or after 60 s), folded by fold.mjs."""
    held = {}
    for gid, computers in groups.items():
        want = lambda: {o["id"] for o in (log("sam", gid) or {}).get("ops", [])}
        t = time.time()
        same = wait(lambda: all(want() <= {o["id"] for o in (log(c, gid) or {}).get("ops", [])} for c in computers), 60, 0.5)
        held[gid] = {"same_ms": ms(t) if same else None}
    listed = [{"computer": c, "group": gid, "log": log(c, gid)} for gid, cs in groups.items() for c in cs]
    (W / "copies.json").write_text(json.dumps(listed))
    run = subprocess.run(["node", HERE / "fold.mjs", W / "copies.json"], capture_output=True, text=True)
    folded = json.loads(run.stdout) if run.returncode == 0 else [{"unread": run.stderr[-400:]}]
    return {"held": held, "folded": folded}


def main():
    global browser
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir(parents=True)
    say("start", head=head(), plant=PLANT, load=round(os.getloadavg()[0], 1), cores=os.cpu_count(),
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
                                   stderr=open(W / "people.err", "w"), text=True)
        if not json.loads(browser.stdout.readline()).get("ready"): sys.exit("people.mjs did not start")
        for who, engine in (("sam", "chromium"), ("bo", "chromium"), ("di", "chromium"), ("fay", "chromium"),
                            ("cy", "webkit"), ("eve", "webkit")):
            tell(**{"do": "open", "who": who, "engine": engine})

        # The House: Sam makes it; Bo joins by its link, Cy's phone by its page address.
        start("sam", "new", HOUSE, "--as", "Sam", "--origin", PAGE)
        house_link, house_page = links("sam")
        summary["bo_join"] = joined("bo", house_link, "Bo")
        summary["cy_join"] = tell(**{"do": "join", "who": "cy", "link": house_page, "name": "Cy"})
        # dWeb: Sam's svrngs stops and makes it; Di joins, Eve's phone.
        stop("sam")
        start("sam", "new", DWEB, "--as", "Sam", "--origin", PAGE)
        dweb_link, dweb_page = links("sam")
        summary["di_join"] = joined("di", dweb_link, "Di")
        summary["eve_join"] = tell(**{"do": "join", "who": "eve", "link": dweb_page, "name": "Eve"})
        groups = {g["name"]: g["id"] for g in api("sam", "/v1/groups")["groups"]}
        H, D = groups[HOUSE], groups[DWEB]
        summary["groups"] = groups

        # The app on Sam's computer, lent.
        app_port = app()
        summary["app"] = {"port": app_port, "plant": PLANT,
                          "share": command("sam", "share", str(app_port), "--kind", "events", "--name", "events")}
        for who, gid in (("sam", H), ("sam", D), ("bo", H), ("di", D)):
            if not wait(lambda: ("Sam", "events") in lends(who, gid), 120, 0.5): sys.exit(f"{who} lists no events in {gid}")
        origin = {who: lends(who, gid)[("Sam", "events")].rstrip("/") for who, gid in (("sam", H), ("bo", H), ("di", D))}
        label = urllib.parse.urlsplit(origin["bo"]).hostname.split(".")[0]
        summary["origins"] = {**origin, "label": label}
        phone = {}

        # 1. Sam posts the dinner to both groups.
        look("sam", origin["sam"] + "/", "1")
        posted = press("sam", f"{H},{D}", "Post", "1", fill={f"input[name={k}]": v for k, v in DINNER.items()},
                       check=[f"input[value='{H}']", f"input[value='{D}']"])
        m = re.search(r"/g/([^/]+)/e/([0-9a-f]+)$", posted.get("url") or "")
        if not m: sys.exit(f"the post landed on no event: {posted}")
        E = m[2]
        path = {H: f"/g/{H}/e/{E}", D: f"/g/{D}/e/{E}"}
        phone_at = lambda gid: f"{PAGE}#open={label}&at={urllib.parse.quote(path[gid], safe='')}"
        summary["event"] = {"id": E, "landed": posted.get("url"), "ms": posted.get("ms")}

        # 2-3. Bo, changing his mind to bring Maya; Cy's phone; Di; Eve's phone.
        look("bo", origin["bo"] + path[H], "2")
        press("bo", H, "I'm coming", "2")
        press("bo", H, "Save", "2", fill={"input[name=guests]": "Maya"})
        cy_at = look("cy", phone_at(H), "3").get("url") or ""
        phone["cy"] = ("{0.scheme}://{0.netloc}".format(urllib.parse.urlsplit(cy_at)))
        press("cy", H, "I'm coming", "3")
        look("di", origin["di"] + path[D], "3")
        press("di", D, "I'm coming", "3")
        phone["eve"] = "{0.scheme}://{0.netloc}".format(urllib.parse.urlsplit(look("eve", phone_at(D), "3").get("url") or ""))
        press("eve", D, "I'm coming", "3")

        # 4-7. Di's computer asleep; Eve takes her name off; Cy looks for
        # Maya's button; Bo came, and Maya; Cy came.
        stop("di"); di_slept = time.time()
        press("eve", D, "Take my name off", "5")
        summary["cy_maya"] = press("cy", H, "Maya came", "6")
        press("bo", H, "I came", "7")
        press("bo", H, "Maya came", "7")
        press("cy", H, "I came", "7")

        # 8. Di's computer wakes, holds what it missed; Di came.
        summary["di_asleep_s"] = round(time.time() - di_slept, 1)
        t = start("di"); up("di")
        want = {o["id"] for o in log("sam", D)["ops"]}
        caught = wait(lambda: want <= {o["id"] for o in (log("di", D) or {}).get("ops", [])}, 90, 0.5)
        summary["di_woke"] = {"caught_up_ms": ms(t) if caught else None, "ops_missed": len(want)}
        say("woke", who="di", **summary["di_woke"])
        look("di", origin["di"] + path[D], "8")
        press("di", D, "I came", "8")

        # 9. Bo's link; Fay's computer joins after the post, and answers.
        stop("bo")
        start("bo", "link", HOUSE)
        fay_link, _ = links("bo")
        up("bo")
        summary["fay_join"] = joined("fay", fay_link, "Fay")
        if not wait(lambda: ("Sam", "events") in lends("fay", H), 120, 0.5): sys.exit("Fay lists no events")
        origin["fay"] = lends("fay", H)[("Sam", "events")].rstrip("/")
        look("fay", origin["fay"] + "/", "9")
        look("fay", origin["fay"] + path[H], "9")
        press("fay", H, "I'm coming", "9")

        # 10. Each person's view; each opening the other group's address.
        for who, url in (("sam", origin["sam"] + path[H]), ("bo", origin["bo"] + path[H]), ("cy", phone_at(H)),
                         ("di", origin["di"] + path[D]), ("eve", phone_at(D)), ("fay", origin["fay"] + path[H])):
            look(who, url, "10")
        summary["cross"] = {"cy": look("cy", phone["cy"] + path[D], "10x"), "di": look("di", origin["di"] + path[H], "10x")}

        # 11. Every computer's copy, folded; who stands in each group.
        summary["copies"] = copies({H: ["sam", "bo", "fay"], D: ["sam", "di"]})
        summary["people"] = {H: people("sam", H), D: people("sam", D)}

        # 12. Sam's computer asleep: Bo's browser, Cy's phone, the app asked as the door would.
        sam_key = next(p["person_key"] for p in summary["people"][H] if p["person"] == "Sam")
        stop("sam"); t = time.time()
        asleep = {"bo": look("bo", origin["bo"] + path[H], "12"), "cy": look("cy", phone["cy"] + path[H], "12")}
        s, b, _ = http("GET", f"http://127.0.0.1:{app_port}{path[H]}", headers={"x-svrngs-key": sam_key})
        words = re.sub(r"<style.*?</style>|<[^>]+>", " ", b if isinstance(b, str) else json.dumps(b), flags=re.S)
        asleep["app"] = {"status": s, "said": " ".join(html.unescape(words).split())}
        asleep["s"] = round(time.time() - t, 1)
        t = start("sam"); up("sam")
        back = wait(lambda: (lambda v: v if v.get("status") == 200 else None)(tell(**{"do": "goto", "who": "bo", "url": origin["bo"] + path[H]})), 90, 2)
        if back: views.append({"step": "12w", "who": "bo", **back})
        asleep["woke"] = {"ms": ms(t), "bo": back}
        summary["asleep"] = asleep
    finally:
        if browser:
            try: browser.stdin.close(); browser.wait(30)
            except Exception: browser.kill()
        for name, p in list(procs.items()):
            p.send_signal(signal.SIGINT if name not in ("page", "app") else signal.SIGTERM)
        for p in procs.values():
            try: p.wait(20)
            except Exception: p.kill()
        summary.update({"presses": presses, "views": views, "end_load": round(os.getloadavg()[0], 1)})
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, ensure_ascii=False)) + "\n")
        for f in ("app.trace", "people.err"):
            if (W / f).exists(): (OUT / f).write_text(redact((W / f).read_text()))
        for who in ("sam", "bo", "di", "fay"):
            if (W / f"{who}.log").exists():
                keep = [re.sub(r"\x1b\[[0-9;]*m", "", l) for l in (W / f"{who}.log").read_text(errors="replace").splitlines()
                        if re.search(r"door: |origin: |refused|ERROR|WARN", l)]
                (OUT / f"{who}.log").write_text(redact("\n".join(keep[-3000:])) + "\n")
        say("end")


if __name__ == "__main__":
    main()
