#!/usr/bin/env python3
"""GRAPEVINE_DEMO §10's scene under svrngs (PREDICTIONS.md here; row gv-run
(b)): this commit's svrngs, Sam's, Jo's and Kim's computers each a HOME and
data dir of its own under W; Sam's 1400 Page and Jo's Waller block made and
joined by svrngs's own `new` and `join`; each person's grapevine
(apps/grapevine, copied to W/app) started by its one line in record mode and
opened at the address it prints; each person a Chromium pressing what their
page shows (people.mjs). Every copy of every group read through its svrngs
while the scene runs and after; every file under W searched after.

    W=<scratch> python3 demo/runs/grapevine/2026-10-16/run.py
    PLANT=<unsealed|spread|uncorrected|upstream> W=<scratch> python3 demo/runs/grapevine/2026-10-16/run.py

A plant runs the same scene against a copy of the app with one change
(PLANTS), data in data/plants/<plant>/. judge.py reads the summary. The
links' tokens are never in the data. scripts/check.sh runs none of this."""
import json, os, re, shutil, signal, subprocess, sys, threading, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
PLANT = os.environ.get("PLANT")
OUT = HERE / "data" / ("plants/" + PLANT if PLANT else ""); W = Path(os.environ["W"]); BIN = REPO / "target/debug/svrngs"
PAGE, WALLER = "1400 Page", "Waller block"
LOG = "info,cli=debug,redeem=debug,surface=debug"
OFFER = {"When": "Saturday morning", "Where": "Waller and Pierce"}
NAMES = {"sam": "Sam Reyes", "kim": "Kim Okafor"}
SEALED = [*OFFER.values(), *NAMES.values()]
OPEN = "tile saw"
BOUND = 60

# Each plant: (file in the app, the text there, what replaces it), the
# laws' own (apps/grapevine/laws.test.mjs, canary.test.mjs). The run stops
# if a plant's text is not found once, so a plant never silently misses.
PLANTS = {
    "unsealed": [("seal.mjs", "return { nonce: b64(nonce), box: b64(nacl.box(text, nonce, them, mine)) };",
                  "return { nonce: b64(nonce), box: JSON.stringify(body) };")],
    "spread": [("grapevine.js", "payload: { app: APP, type: CARRY, kind: ask.kind,", "payload: { ...ask, app: APP, type: CARRY, kind: ask.kind,")],
    "uncorrected": [("grapevine.js", "if (upstream?.voided && carried) relay.push(", "if (false) relay.push(")],
    # Added after the first takes (README.md): `spread` is refused by the app's
    # own validator before it is written, so this one puts a bar-1 break in
    # the records — the carry keeps the ask's own handle, which reads.
    "upstream": [("grapevine.js", "seal_to: ask.seal_to, of: ask.id, handle },", "seal_to: ask.seal_to, of: ask.id, handle: can.up.handle },")],
}

procs, outs, summary, hidden, steps = {}, {}, {}, [], []
seen = {}  # (computer, group id, op id) -> when that computer's copy first held it
watching = [True]


def head(): return subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()


def redact(text):
    for h in hidden: text = text.replace(h, "<the token>")
    return text


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}, ensure_ascii=False))
    print(line[:400], flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def ms(t0): return round((time.time() - t0) * 1000)


def wait(cond, bound, every=0.25):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def http(url, timeout=10):
    try:
        with urllib.request.urlopen(url, timeout=timeout) as r: return r.status, json.loads(r.read())
    except urllib.error.HTTPError as e: return e.code, None
    except Exception as e: return None, str(e)


# --- each computer: a data dir and HOME of its own --------------------------
def env(who):
    return {**{k: v for k, v in os.environ.items() if k != "PLANT"}, "HOME": str(W / who), "USER": who.capitalize(),
            "SVRNGS_DIR": str(W / who / ".svrngs"), "NO_COLOR": "1", "RUST_LOG": LOG}


def start(who, *args):
    (W / who).mkdir(parents=True, exist_ok=True)
    outs[who] = W / f"{who}.{len(list(W.glob(f'{who}.*.out'))) + 1}.out"
    procs[who] = subprocess.Popen([BIN, *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(outs[who], "w"), stderr=open(W / f"{who}.log", "a"))
    say("svrngs", who=who, args=[a if not a.startswith("svrngs://") else "<link>" for a in args])


def stop(who):
    p = procs.pop(who)
    p.send_signal(signal.SIGINT)
    try: p.wait(20)
    except subprocess.TimeoutExpired: p.kill(); p.wait()


def said(who): return outs[who].read_text() if who in outs and outs[who].exists() else ""


def base(who): return (W / who / ".svrngs/api").read_text().strip().rstrip("/")


def api(who, path):
    try: s, b = http(base(who) + path)
    except Exception: return None
    return b if s == 200 else None


def link(who):
    got = wait(lambda: re.search(r"svrngs://join/(\S+)", said(who)), 90)
    if not got: sys.exit(f"{who} printed no link: {said(who)[-600:]}")
    hidden.append(got[1])
    return got[0]


def joined(who, link, name):
    t = time.time()
    start(who, "join", link, "--as", name)
    got = wait(lambda: re.search(r"You're in .*|svrngs: .*", said(who)), 90)
    say("joined", who=who, said=got and got[0], ms=ms(t))
    if not got or not got[0].startswith("You're in"): sys.exit(f"{who} did not join: {said(who)[-400:]}")
    return {"said": got[0], "ms": ms(t)}


def log(who, gid): return api(who, f"/v1/rail/log?namespace={gid}")


def watch(copies):
    """Every copy of every group, read every 0.2 s: when each act first reached each computer."""
    while watching[0]:
        for who, gid in copies:
            for o in (log(who, gid) or {}).get("ops", []):
                seen.setdefault((who, gid, o["id"]), time.time())
        time.sleep(0.2)


# --- each person's browser ---------------------------------------------------
browser = None


def tell(**c):
    browser.stdin.write(json.dumps(c) + "\n"); browser.stdin.flush()
    line = browser.stdout.readline()
    got = json.loads(line) if line else {"error": "people.mjs stopped"}
    say("person", command=c, **{k: v for k, v in got.items() if k not in ("all", "logs")})
    return got


def step(n, who, what, got, t0):
    """One step of the scene, its time from Sam's Ask, what the page showed."""
    row = {"step": n, "who": who, "what": what, "s": round(time.time() - t0, 3) if t0 else None,
           "ms": got.get("ms"), "pressed_ms": got.get("pressed_ms"),
           "met": got.get("met", not got.get("absent") and not got.get("error") and got.get("said") is None),
           "said": got.get("said"), "error": got.get("error"), "reloads": got.get("reloads"),
           "text": got.get("text"), "all": got.get("all"), "not": got.get("not"), "threads": got.get("threads"),
           "groups": got.get("groups"), "buttons": got.get("buttons")}
    steps.append(row)
    say("step", **{k: v for k, v in row.items() if k not in ("text", "all", "threads")})
    return row["met"]


def app():
    """The app, copied, a plant applied when one is named."""
    shutil.rmtree(W / "app", ignore_errors=True)
    shutil.copytree(REPO / "apps/grapevine", W / "app", ignore=shutil.ignore_patterns("*.test.mjs"))
    for name, old, new in PLANTS.get(PLANT, []):
        f = W / "app" / name
        text = f.read_text()
        if text.count(old) != 1: sys.exit(f"the {PLANT} plant's text is not in {name} once: {old}")
        f.write_text(text.replace(old, new))


def grapevine(who):
    """The person's one start line: a config naming their svrngs, and the server, opened at what it prints."""
    cfg = W / who / "grapevine.json"
    (W / who / "grapevine").mkdir(parents=True, exist_ok=True)
    cfg.write_text(json.dumps({"record": base(who), "data": str(W / who / "grapevine")}))
    t = time.time()
    procs[f"app-{who}"] = subprocess.Popen(["node", W / "app/server.mjs", cfg], env={**os.environ, "GRAPEVINE_DEBUG": "1"},
                                           stdout=subprocess.PIPE, stderr=open(W / f"{who}.trace", "w"), text=True)
    line = json.loads(procs[f"app-{who}"].stdout.readline())
    say("started", who=who, ms=ms(t), **line)
    return line["at"]


def scan():
    """Every file under W but the copied app's source, searched for each sealed word and each secret key."""
    keys = {}
    for who in ("sam", "jo", "kim"):
        f = W / who / "grapevine/state.json"
        state = json.loads(f.read_text()) if f.exists() else {}
        keys[who] = list((state.get("keys") or {}).values())
    secrets = [k for ks in keys.values() for k in ks]
    out = {"files": 0, "bytes": 0, "sealed": {}, "secrets": [], "open": {}, "states": {w: len(k) for w, k in keys.items()}}
    for f in sorted(W.rglob("*")):
        if not f.is_file() or (W / "app") in f.parents: continue
        blob = f.read_bytes(); rel = str(f.relative_to(W)); out["files"] += 1; out["bytes"] += len(blob)
        state = f.name == "state.json" and f.parent.name == "grapevine"
        if OPEN.encode() in blob: out["open"].setdefault(rel.split("/")[0], []).append(rel)
        for w in SEALED:
            if w.encode() in blob: out["sealed"].setdefault(w, []).append(rel)
        if not state:
            for k in secrets:
                if k.encode() in blob: out["secrets"].append(rel)
    return out


def main():
    global browser
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    for f in OUT.glob("*") if OUT.exists() else []:
        if f.name != "plants": shutil.rmtree(f) if f.is_dir() else f.unlink()
    OUT.mkdir(parents=True, exist_ok=True)
    say("start", head=head(), plant=PLANT, load=round(os.getloadavg()[0], 1), cores=os.cpu_count(),
        dirty=subprocess.run(["git", "-C", REPO, "status", "--short"], capture_output=True, text=True).stdout.splitlines())
    t_run = time.time()
    try:
        subprocess.run(["cargo", "build", "--quiet", "-p", "svrngs"], cwd=REPO, check=True, env={**os.environ, "CARGO_INCREMENTAL": "0"})
        app()
        browser = subprocess.Popen(["node", HERE / "people.mjs"], cwd=HERE, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                   stderr=open(W / "people.err", "w"), text=True)
        if not json.loads(browser.stdout.readline()).get("ready"): sys.exit("people.mjs did not start")
        for who in ("sam", "jo", "kim"): tell(do="open", who=who)

        # The town: Sam makes 1400 Page, Jo joins; Jo's svrngs stops and makes Waller block, Kim joins.
        start("sam", "new", PAGE, "--as", "Sam")
        summary["jo_join"] = joined("jo", link("sam"), "Jo")
        stop("jo")
        start("jo", "new", WALLER, "--as", "Jo")
        summary["kim_join"] = joined("kim", link("jo"), "Kim")
        want = {"sam": {PAGE}, "jo": {PAGE, WALLER}, "kim": {WALLER}}
        held = lambda who: {g["name"]: g for g in (api(who, "/v1/groups") or {}).get("groups", [])}
        if not all(wait(lambda: set(held(w)) == g and all(x.get("person") for x in held(w).values()), 60) for w, g in want.items()):
            sys.exit(f"the town did not stand: {[(w, held(w)) for w in want]}")
        summary["town"] = {w: {n: g.get("person") for n, g in held(w).items()} for w in want}
        gid = {n: g["id"] for n, g in {**held("sam"), **held("jo")}.items()}
        summary["groups"] = gid
        copies = [("sam", gid[PAGE]), ("jo", gid[PAGE]), ("jo", gid[WALLER]), ("kim", gid[WALLER])]
        threading.Thread(target=watch, args=(copies,), daemon=True).start()

        # The start: each person's line, and their page opened at the address it printed.
        at = {who: grapevine(who) for who in ("sam", "jo", "kim")}
        summary["at"] = at
        for who in ("sam", "jo", "kim"):
            got = tell(do="goto", who=who, url=at[who])
            step("open", who, "the page its start line printed", got, None)
        summary["start_s"] = round(time.time() - t_run, 1)

        # 0. Kim lists a tile saw in Waller block.
        got = tell(do="press", who="kim", within=[["section", "What you have"], ["div.kind", "tool"]], fill={"What": OPEN},
                   select={"In": WALLER}, button="List it")
        step(0, "kim", "Kim lists a tile saw in Waller block", got, None)

        # 1. Sam asks: a tile saw, Saturday, one ring away. The clock starts at the press.
        t0 = time.time(); summary["t0"] = t0
        got = tell(do="press", who="sam", within=[["section", "Ask your groups"], ["div.kind", "tool"]],
                   fill={"What": OPEN, "When": "Saturday"}, select={"In": PAGE, "How far": "One ring away"}, button="Ask")
        ok = step(1, "sam", "Sam asks in 1400 Page for a tile saw, Saturday, one ring away", got, t0)

        # 2. Jo's page shows the carry; Jo taps it.
        ok = ok and step(2, "jo", "Jo's page shows Carry to Waller block", tell(do="await", who="jo", words=["Carry to Waller block"], bound=BOUND), t0)
        ok = ok and step(2, "jo", "Jo carries it", tell(do="press", who="jo", within=[["li.ask", OPEN]], button="Carry to Waller block"), t0)

        # 3. Kim's page: carried by Jo, you might have this. Kim offers.
        ok = ok and step(3, "kim", "Kim's page: carried by Jo, you might have this",
                         tell(do="await", who="kim", words=["carried by Jo", "You might have this"], bound=BOUND), t0)
        ok = ok and step(3, "kim", "Kim offers", tell(do="press", who="kim", within=[["li.ask", OPEN]], fill=OFFER, button="Offer"), t0)

        # 4. Sam's page opens the offer; Jo's says a sealed message went by.
        ok = ok and step(4, "sam", "Sam's page shows the offer",
                         tell(do="await", who="sam", words=["offered: when Saturday morning; where Waller and Pierce"], bound=BOUND), t0)
        if ok: step(4, "jo", "Jo's page: a sealed message went by", tell(do="await", who="jo", words=["A sealed message went by"], bound=BOUND), t0)

        # 5. Sam says yes; Kim, once her page shows it, says yes; each sees the other's name.
        name = "Your name, which they’ll see once you’ve both said yes"
        ok = ok and step(5, "sam", "Sam says yes", tell(do="press", who="sam", within=[["li.thread", "About your ask"]], fill={name: NAMES["sam"]}, button="Yes"), t0)
        ok = ok and step(5, "kim", "Kim's page: the person who asked said yes",
                         tell(do="await", who="kim", words=["The person who asked said yes"], bound=BOUND), t0)
        ok = ok and step(5, "kim", "Kim says yes", tell(do="press", who="kim", within=[["li.thread", "which you offered to help with"]], fill={name: NAMES["kim"]}, button="Yes"), t0)
        ok = ok and step(5, "kim", "Kim's page names Sam", tell(do="await", who="kim", words=[f"they’re {NAMES['sam']}"], bound=BOUND), t0)
        ok = ok and step(5, "sam", "Sam's page names Kim", tell(do="await", who="sam", words=[f"they’re {NAMES['kim']}"], bound=BOUND), t0)

        # 6. Sam takes it back; Kim's page no longer shows it.
        ok = ok and step(6, "sam", "Sam takes it back", tell(do="press", who="sam", within=[["li.ask", OPEN]], button="Take it back"), t0)
        if ok:
            t6 = time.time(); summary["t6"] = t6
            step(6, "kim", "the ask gone from Kim's page", tell(do="await", who="kim", gone=["needs: tile saw, Saturday"], bound=BOUND), t0)
            # One loop after Jo's computer holds the withdrawal, and the look at Jo's page after.
            time.sleep(7)
            step(6, "jo", "Jo's page after", tell(do="goto", who="jo", url=at["jo"]), t0)
        summary["scene_s"] = round(time.time() - t0, 1)
        summary["ok"] = bool(ok)

        # After: every copy of every group, as its svrngs answers it.
        time.sleep(1)
        summary["copies"] = [{"computer": w, "group": g, "log": log(w, g)} for w, g in copies]
        summary["people"] = {n: api("jo", f"/v1/rail/people?namespace={g}") for n, g in gid.items()}
    finally:
        watching[0] = False
        if browser:
            try: browser.stdin.close(); browser.wait(30)
            except Exception: browser.kill()
        for name, p in list(procs.items()):
            p.send_signal(signal.SIGINT if not name.startswith("app-") else signal.SIGTERM)
        for p in procs.values():
            try: p.wait(20)
            except Exception: p.kill()
        summary.update({"steps": steps, "seen": [{"computer": c, "group": g, "op": o, "t": t} for (c, g, o), t in sorted(seen.items(), key=lambda x: x[1])],
                        "end_load": round(os.getloadavg()[0], 1), "run_s": round(time.time() - t_run, 1)})
        summary["scan"] = scan()
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, ensure_ascii=False)) + "\n")
        for who in ("sam", "jo", "kim"):
            if (W / f"{who}.trace").exists(): (OUT / f"{who}.trace").write_text(redact((W / f"{who}.trace").read_text()))
            if (W / f"{who}.log").exists():
                keep = [re.sub(r"\x1b\[[0-9;]*m", "", l) for l in (W / f"{who}.log").read_text(errors="replace").splitlines()
                        if re.search(r"surface|refused|ERROR|WARN", l)]
                (OUT / f"{who}.log").write_text(redact("\n".join(keep[-2000:])) + "\n")
        if (W / "people.err").exists(): (OUT / "people.err").write_text((W / "people.err").read_text())
        say("end")


if __name__ == "__main__":
    main()
