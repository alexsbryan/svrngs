#!/usr/bin/env python3
"""A visit on one host (PREDICTIONS.md here; row guests-run): this commit's
svrngs, Ana's, Kim's and Bo's computers each a HOME and data dir of its own
under W; Kim's and Bo's Jellyfin a stand-in on this host (jellyfin.mjs);
the page built and served on this host; each person a browser kept by
people.mjs, pressing what their page shows. Carried over from
demo/runs/canon/2026-10-17-house/run.py, with a guest's story.

    W=<scratch> python3 demo/runs/guests/2026-10-20-run/run.py
    PLANT=<host|bo|count|lapsed> TRIAL=1 W=<scratch> python3 demo/runs/guests/2026-10-20-run/run.py
    TRIAL=1 W=<scratch> python3 demo/runs/guests/2026-10-20-run/run.py

TRIAL skips step 8's wait for the visit until tonight to end. A plant is
applied to the tree, svrngs built with it, and the tree put back before the
story starts; the story runs on the build copied to W. Data in data/
(data/plants/<plant>/; a trial's in data/trial/), the links' tokens never
in it. judge.py reads the summary. scripts/check.sh runs none of this."""
import json, os, re, shutil, signal, subprocess, sys, threading, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
PLANT, TRIAL = os.environ.get("PLANT"), bool(os.environ.get("TRIAL"))
OUT = HERE / "data" / ("plants/" + PLANT if PLANT else "trial" if TRIAL else ""); W = Path(os.environ["W"]); BIN = W / "svrngs"
OAK = "Oak"
LOG = "info,door=debug,redeem=debug,link=debug,removal=debug"
COMPUTERS = ("ana", "kim", "bo")
# The guests, each a browser of their own: Sam as an iPhone, the rest Chromium.
GUESTS = {"sam": ("Sam", "webkit"), "jo": ("Jo", "chromium"), "max": ("Max", "chromium"), "lee": ("Lee", "chromium")}
LENGTH = "label:has-text('For how long') select"

# Each plant: (file, the text there, what replaces it). The run stops if a
# plant's text is not in its file once, so a plant never silently misses.
PLANTS = {
    "host": [("crates/svrngs-node/src/door.rs",
              "        person: guest.visit.guest,\n        host: Some(host),\n",
              "        person: host,\n        host: None,\n")],
    "bo": [("crates/svrngs-node/src/door.rs",
            "    let own = guest.host == me && matches!(lend.source, Source::Port(_));\n",
            "    let own = true || guest.host == me && matches!(lend.source, Source::Port(_));\n"),
           ("crates/svrngs-node/src/door.rs",
            "    let named = guest.visit.opens.get(me).is_some_and(|l| l.contains(name));\n",
            "    let named = true || guest.visit.opens.get(me).is_some_and(|l| l.contains(name));\n")],
    "count": [("crates/svrngs-core/src/membership.rs",
               "let is = !used && timely && on.len() + 1 < members && theirs < GUESTS_PER_HOST;",
               "let is = !used && timely && (true || on.len() + 1 < members && theirs < GUESTS_PER_HOST);")],
    "lapsed": [("crates/svrngs-node/src/door.rs",
                "    let live = guest.live(&register.cut, now);\n",
                "    let live = true || guest.live(&register.cut, now);\n"),
               ("crates/svrngs-node/src/door.rs",
                "    let gone = register.cut.get(&guest.host).is_some_and(|c| c.at <= now);\n",
                "    let gone = false && register.cut.get(&guest.host).is_some_and(|c| c.at <= now);\n"),
               ("crates/svrngs-node/src/door.rs",
                "    let stopped = guest.stopped.is_some_and(|s| s <= now);\n",
                "    let stopped = false && guest.stopped.is_some_and(|s| s <= now);\n")],
}

procs, outs, summary, hidden = {}, {}, {}, []


def git(*a): return subprocess.run(["git", "-C", str(REPO), *a], capture_output=True, text=True).stdout.strip()


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


def http(url, timeout=30):
    try:
        with urllib.request.urlopen(url, timeout=timeout) as r: return r.status, json.loads(r.read())
    except urllib.error.HTTPError as e: return e.code, None
    except Exception: return None, None


# --- each computer: a data dir and HOME of its own --------------------------
def env(who):
    return {**{k: v for k, v in os.environ.items() if k not in ("PLANT", "TRIAL")}, "HOME": str(W / who), "USER": who,
            "SVRNGS_DIR": str(W / who / ".svrngs"), "NO_COLOR": "1", "RUST_LOG": LOG}


def start(who, *args):
    (W / who).mkdir(parents=True, exist_ok=True)
    outs[who] = W / f"{who}.{len(list(W.glob(f'{who}.*.out'))) + 1}.out"
    procs[who] = subprocess.Popen([BIN, *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(outs[who], "w"), stderr=open(W / f"{who}.log", "a"))
    say("svrngs", who=who, args=[a if not a.startswith(("svrngs://", "http")) else "<link>" for a in args])
    return time.time()


def command(who, *args):
    r = subprocess.run([BIN, *args], env=env(who), stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60)
    said = (r.stdout + r.stderr).strip()
    say("command", who=who, args=list(args), exit=r.returncode, said=said[:600])
    return {"exit": r.returncode, "said": said}


def said(who): return outs[who].read_text() if who in outs and outs[who].exists() else ""


def port(who): return int((W / who / ".svrngs/api").read_text().strip().rstrip("/").rsplit(":", 1)[1])


def api(who, path):
    try: s, b = http(f"http://127.0.0.1:{port(who)}{path}")
    except Exception: return None
    return b if s == 200 else None


def up(who, bound=60):
    if not wait(lambda: api(who, "/v1/groups"), bound): sys.exit(f"{who}'s svrngs did not answer: {said(who)[-400:]}")


def links(who, bound=90):
    got = wait(lambda: re.search(r"svrngs://join/(\S+)", said(who)), bound)
    if not got: sys.exit(f"{who} printed no link: {said(who)[-600:]}")
    hidden.append(got[1])
    page = re.search(r"(http://localhost:\d+/#(\S+))", said(who))
    if not page: sys.exit(f"{who} printed no page link: {said(who)[-600:]}")
    hidden.append(page[2])
    return page[1]


def joined(who, link, name):
    t = start(who, "join", link, "--as", name)
    got = wait(lambda: re.search(r"You're in .*|svrngs: .*", said(who)), 90)
    if not got or not got[0].startswith("You're in"): sys.exit(f"{who} did not join: {said(who)[-400:]}")
    say("joined", who=who, said=got[0], ms=ms(t))


def oak(who): return next((g for g in (api(who, "/v1/groups") or {}).get("groups", []) if g.get("name") == OAK), None)


def people(who, gid): return (api(who, f"/v1/rail/people?namespace={gid}") or {}).get("people")


def visiting(who):
    """Who `who`'s computer says is visiting Oak now; None when it cannot say."""
    g = oak(who)
    return None if g is None or g.get("visiting") is None else g["visiting"]


def guests_on(who): return sorted(v.get("guest") for v in visiting(who) or [])


def record(who, gid):
    """`who`'s computer's record of Oak: each act it holds by id, its time, and its gaps."""
    log = api(who, f"/v1/rail/log?namespace={gid}") or {}
    return {o.get("id"): o.get("ts_unix") for o in log.get("ops") or []}, log.get("gaps") or []


def holds(who, gid, op):
    """Whether `who`'s computer holds the act `op`, counted or not."""
    ops, gaps = record(who, gid)
    return op in ops or any(op in (g.get("id"), g.get("missing")) for g in gaps)


def written(who):
    """Each visit `who`'s computer wrote at a guest's link, as its log says it (redeem.rs `serve_visit`)."""
    text = re.sub(r"\x1b\[[0-9;]*m", "", (W / f"{who}.log").read_text(errors="replace")) if (W / f"{who}.log").exists() else ""
    return [dict(re.findall(r"\b(id|key|until|counted)=(\S+)", l)) for l in text.splitlines() if "its Visit written" in l]


def the_visit(host, guest, gid):
    """`guest`'s visit as `host`'s computer lists it, with its act's time from its record."""
    v = next((x for x in visiting(host) or [] if x.get("guest") == guest), None)
    return v and {**v, "at": record(host, gid)[0].get(v["id"])}


def until(cond, bound, every=0.25):
    """How long until cond holds, from now, in ms; None past bound."""
    t = time.time()
    return ms(t) if wait(cond, bound, every) else None


# --- the browsers --------------------------------------------------------------
browsers = None


def person(do, who, step=None, **c):
    t = time.time()
    browsers.stdin.write(json.dumps({"do": do, "who": who, **c}) + "\n"); browsers.stdin.flush()
    line = browsers.stdout.readline()
    got = json.loads(line) if line else {"error": "people.mjs stopped"}
    got["started"] = round(t, 3)
    got = json.loads(redact(json.dumps(got, ensure_ascii=False)))
    say("person", do=do, who=who, step=step, **{k: v for k, v in got.items() if k not in ("done", "text", "logs")},
        text=(got.get("text") or "")[-1500:],
        steps=[{"step": json.dumps(d["step"], ensure_ascii=False), "ms": d["ms"], "error": d.get("error"), "text": d["text"][-1500:]}
               for d in got.get("done", [])])
    if step: summary.setdefault("steps", {})[step] = {"who": who, "do": do, **got}
    return got


def steps(step, who, *ss): return person("steps", who, step=step, steps=list(ss))


def last(got): return (got.get("done") or [{}])[-1].get("text", "") if "done" in got else got.get("text", "")


def homes(key, home, want=None, bound=30_000):
    """Each computer's home page read, after it says `want` when given."""
    for who in COMPUTERS:
        if who in home:
            steps(f"{key}-{who}", who, {"poll": home[who], "re": want or "Have a guest|Leave", "within": bound, "every": 3_000})


# --- the build, a plant applied while it builds --------------------------------
def build():
    planted = []
    try:
        for name, old, new in PLANTS.get(PLANT, []):
            f = REPO / name
            text = f.read_text()
            if text.count(old) != 1: sys.exit(f"the {PLANT} plant's text is not in {name} once: {old}")
            planted.append((f, text)) if all(p[0] != f for p in planted) else None
            f.write_text(text.replace(old, new))
        t = time.time()
        subprocess.run(["cargo", "build", "--quiet", "-p", "svrngs"], cwd=REPO, check=True, env={**os.environ, "CARGO_INCREMENTAL": "0"})
        shutil.copy2(REPO / "target/debug/svrngs", BIN)
        subprocess.run([REPO / "scripts/build-web.sh", "--out", W / "site"], check=True, capture_output=True)
        say("built", s=round(time.time() - t), plant=PLANT, planted=sorted({str(f.relative_to(REPO)) for f, _ in planted}))
    finally:
        for f, text in planted: f.write_text(text)
    if git("status", "--short", "--", "crates", "web"): sys.exit("the tree is not as it was after the build")


def film(path):
    subprocess.run(["ffmpeg", "-v", "error", "-y", "-f", "lavfi", "-i", "testsrc2=duration=60:size=640x360:rate=24",
                    "-vf", "noise=alls=12:allf=t", "-c:v", "libx264", "-preset", "ultrafast", "-crf", "27", "-pix_fmt", "yuv420p",
                    "-g", "24", "-movflags", "+faststart", str(path)], check=True)


def lend(who):
    """`who`'s Jellyfin stand-in, lent with `svrngs share <port> --kind jellyfin`."""
    heads = W / f"{who}.heads.jsonl"
    heads.touch()
    p = subprocess.Popen(["node", HERE / "jellyfin.mjs", W / "title.mp4", heads], stdout=subprocess.PIPE, text=True)
    procs[f"{who}-jellyfin"] = p
    at = re.search(r"port (\d+)", p.stdout.readline())[1]
    summary.setdefault("share", {})[who] = command(who, "share", at, "--kind", "jellyfin")
    return heads


def heads(f): return [json.loads(l) for l in f.read_text().splitlines() if l.strip()] if f.exists() else []


def make_link(key, who, home, length):
    """`who` presses Have a guest on their home page, picks `length`, and makes the link: (link, the answer)."""
    got = steps(key, who, {"goto": home[who]}, {"wait": "Have a guest"},
                {"click": "Have a guest", "until": "For how long"}, {"options": LENGTH},
                *([{"select": LENGTH, "label": length}] if length else []),
                {"click": "Make a link for your guest", "until": "Send this link to your guest"}, {"text": "p.link code"})
    link = (got.get("done") or [{}])[-1].get("value") if not got.get("failed") else None
    if link and "#visit=" in link: hidden.append(link.split("#visit=", 1)[1])
    return link, got


def main():
    global browsers
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    for f in OUT.glob("*") if OUT.exists() else []:
        if f.name not in ("plants", "trial"): shutil.rmtree(f) if f.is_dir() else f.unlink()
    OUT.mkdir(parents=True, exist_ok=True)
    say("start", head=git("rev-parse", "--short", "HEAD"), plant=PLANT, trial=TRIAL, load=round(os.getloadavg()[0], 1),
        cores=os.cpu_count(), dirty=git("status", "--short").splitlines())
    summary.update(plant=PLANT, trial=TRIAL)
    try:
        build()
        film(W / "title.mp4")
        procs["page"] = subprocess.Popen(["node", REPO / "web/tests/serve.mjs", W / "site"], stdout=subprocess.PIPE, text=True)
        page_port = re.search(r"port (\d+)", procs["page"].stdout.readline())[1]
        threading.Thread(target=lambda: [None for _ in procs["page"].stdout], daemon=True).start()
        PAGE = f"http://localhost:{page_port}/"
        browsers = subprocess.Popen(["node", HERE / "people.mjs"], cwd=HERE, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                    stderr=open(W / "people.err", "w"), text=True)
        if not json.loads(browsers.stdout.readline()).get("ready"): sys.exit("people.mjs did not start")
        for who in COMPUTERS: person("open", who, engine="chromium")
        for who, (_, engine) in GUESTS.items(): person("open", who, engine=engine)

        # Oak: Ana makes it; Kim and Bo join by its page link, so each keeps its page.
        start("ana", "new", OAK, "--as", "Ana", "--origin", PAGE)
        page_link = links("ana")
        up("ana")
        joined("kim", page_link, "Kim"); joined("bo", page_link, "Bo")
        G = oak("ana")["id"]
        summary["group"] = G
        keys = {p["person"]: p.get("key") for p in people("ana", G) if p.get("device") == "computer"}
        summary["computers"] = keys
        home = {who: f"http://svrngs.localhost:{port(who)}/" for who in COMPUTERS}
        kept = {who: lend(who) for who in ("kim", "bo")}
        summary["listed"] = {who: bool(wait(lambda: any(l.get("kind") == "jellyfin" for l in (api(who, f"/v1/lends?namespace={G}") or {}).get("lent", [])), 30))
                             for who in ("kim", "bo")}

        # 1. Kim makes a link at the 48 hours it starts at.
        link, got = make_link("1", "kim", home, None)
        summary["link_1"] = {"on_page": bool(link and link.startswith(f"{PAGE}#visit=")), "length": next((d.get("value") for d in got.get("done", []) if "options" in d["step"]), None)}
        if not link: sys.exit("Kim's home page made no link")

        # 2. Sam opens it and watches Kim's film.
        v = person("visit", "sam", step="2-visit", link=link, name="Sam")
        w = person("watch", "sam", step="2-watch", lend="Kim’s jellyfin")
        sam_film = w.get("url")
        played = (w.get("played") or {}).get("ms")
        summary["sam"] = {"film_ms": round((w["started"] - v["started"]) * 1000 + w.get("opened", 0) + played) if played is not None else None,
                          "in_ms": v.get("ms"), "opened_ms": w.get("opened"), "played": w.get("played"), "film": sam_film}
        summary["sam"]["visit"] = the_visit("kim", "Sam", G)
        summary["sam"]["visiting"] = {who: [x for x in visiting(who) or [] if x.get("guest") == "Sam"] for who in COMPUTERS}
        t_in = v["started"] + v.get("ms", 0) / 1000

        # 3. Bo's Jellyfin refuses Sam: a pass the page never offers, signed with the visit's key.
        # Bo's door judges a visit's key by the visit Bo's computer holds, so the
        # pass waits for it there: before, it is refused as no one's (the trial).
        summary["bo_holds_sam_ms"] = until(lambda: "Sam" in guests_on("bo"), 120)
        bo_before = len(heads(kept["bo"]))
        steps("3-page", "sam", {"goto": PAGE}, {"wait": "\\S", "within": 60_000})
        person("forge", "sam", step="3-forge", token=link.split("#visit=", 1)[1], key=keys["Bo"], name="jellyfin", person="Bo")
        summary["bo_from_sam"] = [h for h in heads(kept["bo"])[bo_before:] if (h.get("x-svrngs-person") or "").startswith("Sam")]

        # 4. Every home page shows Sam visiting.
        for who in COMPUTERS:
            steps(f"4-{who}", who, {"poll": home[who], "re": "Sam, Kim’s guest, until", "within": 120_000, "every": 3_000})
        summary["home_4_ms"] = ms(t_in)
        summary["sam"]["until_on"] = {who: next((x.get("until") for x in visiting(who) or [] if x.get("guest") == "Sam"), None) for who in COMPUTERS}

        # 5. A visit until tonight: Kim picks 6 hours; Jo watches Kim's film.
        link5, _ = make_link("5", "kim", home, "6 hours")
        if not link5: sys.exit("Kim's home page made no second link")
        v = person("visit", "jo", step="5-visit", link=link5, name="Jo")
        w = person("watch", "jo", step="5-watch", lend="Kim’s jellyfin")
        jo_film = w.get("url")
        summary["jo"] = {"film": jo_film, "played": w.get("played"), "visit": the_visit("kim", "Jo", G), "in": v["started"] + v.get("ms", 0) / 1000}
        for who in COMPUTERS:
            steps(f"5-{who}", who, {"poll": home[who], "re": "Jo, Kim’s guest, until", "within": 120_000, "every": 3_000})

        # 6. Ana's guest Max: the third guest with three members. Ana's computer
        # judges Max's visit by the visits it holds, so Ana's link waits for Jo there.
        summary["ana_holds_jo_ms"] = until(lambda: "Jo" in guests_on("ana"), 120)
        link6, _ = make_link("6", "ana", home, None)
        if not link6: sys.exit("Ana's home page made no link")
        person("visit", "max", step="6-visit", link=link6, name="Max")
        t6 = time.time()
        max_act = (wait(lambda: written("ana"), 10) or [{}])[-1]
        summary["max"] = {"written": max_act,
                          "held": {who: until(lambda: holds(who, G, max_act.get("id")), max(1, 60 - (time.time() - t6))) for who in COMPUTERS},
                          "visiting": {who: guests_on(who) for who in COMPUTERS}}
        homes("6", home)

        # 7. Kim's Stop ends Sam's visit.
        got = steps("7", "kim", {"goto": home["kim"]}, {"wait": "Stop Sam’s visit"}, {"click": "Stop Sam’s visit"})
        t7 = got["started"]
        summary["stop_7"] = {who: {"ms": until(lambda: visiting(who) is not None and "Sam" not in guests_on(who), max(1, 120 - (time.time() - t7))), "on": guests_on(who)}
                             for who in COMPUTERS}
        homes("7", home)
        steps("7-sam", "sam", {"goto": sam_film}, {"wait": "\\S", "within": 60_000}, {"read": True})

        # 8. Tonight's visit ends then, on every copy.
        jo_until = (summary["jo"].get("visit") or {}).get("until")
        summary["tonight"] = {"until": jo_until, "ran": not TRIAL and jo_until is not None}
        if summary["tonight"]["ran"]:
            say("waiting", until=jo_until, s=round(jo_until - time.time()))
            while time.time() < jo_until: time.sleep(min(60, max(0.05, jo_until - time.time())))
            summary["tonight"]["ended"] = {who: {"ms": until(lambda: visiting(who) is not None and "Jo" not in guests_on(who), 60), "on": guests_on(who)}
                                           for who in COMPUTERS}
            homes("8", home)
            steps("8-jo", "jo", {"goto": jo_film}, {"wait": "\\S", "within": 60_000}, {"read": True})

        # 9. Kim's leave ends a third: Lee watches Kim's film, then Kim leaves.
        link9, _ = make_link("9", "kim", home, None)
        if not link9: sys.exit("Kim's home page made no third link")
        person("visit", "lee", step="9-visit", link=link9, name="Lee")
        w = person("watch", "lee", step="9-watch", lend="Kim’s jellyfin")
        lee_film = w.get("url")
        summary["lee"] = {"film": lee_film, "played": w.get("played"), "visit": the_visit("kim", "Lee", G), "visiting": {who: guests_on(who) for who in COMPUTERS}}
        got = steps("9-leave", "kim", {"goto": home["kim"]}, {"wait": "Leave Oak"}, {"click": "Leave Oak", "until": "You left Oak"})
        t9 = got["started"]
        kim_key = keys["Kim"]

        def out(who):
            g = oak(who) or {}
            left = any(r.get("of") == "Kim" and r.get("said") == "left" for r in g.get("removals") or [])
            return left and not any(p.get("key") == kim_key for p in people(who, G) or []) and visiting(who) is not None and "Lee" not in guests_on(who)
        summary["leave_9"] = {who: {"ms": until(lambda: out(who), max(1, 120 - (time.time() - t9))), "on": guests_on(who)} for who in ("ana", "bo")}
        homes("9", {w: home[w] for w in ("ana", "bo")})
        steps("9-lee", "lee", {"goto": lee_film}, {"wait": "\\S", "within": 60_000}, {"read": True})
        summary["max"]["visiting_end"] = {who: guests_on(who) for who in ("ana", "bo")}
        summary["visits_written"] = {who: written(who) for who in COMPUTERS}
        summary["end_record"] = {who: dict(zip(("ops", "gaps"), record(who, G))) for who in COMPUTERS}
    finally:
        if browsers:
            try: browsers.stdin.close(); browsers.wait(60)
            except Exception: browsers.kill()
        for name, p in list(procs.items()):
            p.send_signal(signal.SIGINT if name in COMPUTERS else signal.SIGTERM)
        for p in procs.values():
            try: p.wait(20)
            except Exception: p.kill()
        summary["end_load"] = round(os.getloadavg()[0], 1)
        for who in ("kim", "bo"):
            summary.setdefault("heads", {})[who] = heads(W / f"{who}.heads.jsonl")
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, ensure_ascii=False)) + "\n")
        if (W / "people.err").exists(): (OUT / "people.err").write_text(redact((W / "people.err").read_text()))
        for who in COMPUTERS:
            if (W / f"{who}.log").exists():
                keep = [re.sub(r"\x1b\[[0-9;]*m", "", l) for l in (W / f"{who}.log").read_text(errors="replace").splitlines()
                        if re.search(r"door: (refused|a visit's key)|redeem: |removal: |ERROR|WARN", l)]
                (OUT / f"{who}.log").write_text(redact("\n".join(keep[-3000:])) + "\n")
        say("end")


if __name__ == "__main__":
    main()
