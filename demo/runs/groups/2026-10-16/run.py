#!/usr/bin/env python3
"""Leaving Trip on one host (PREDICTIONS.md here; row groups-run): this
commit's svrngs, every computer a HOME and data dir of its own under W; the
page built and served on this host; each computer's home page read in
Chromium by web/tests/home.mjs, each phone in WebKit as an iPhone
(people.mjs); what each lend answers asked on loopback at its address.

    W=<scratch> python3 demo/runs/groups/2026-10-16/run.py
    PLANT=<untold|deleted|removed|keeper|rejoin> W=<scratch> python3 demo/runs/groups/2026-10-16/run.py

A plant is applied to the tree, svrngs and the page built with it, and the
tree put back before the story starts; the story runs on the build copied
to W. Data in data/ (data/plants/<plant>/), the links' tokens never in it.
judge.py reads the summary. scripts/check.sh runs none of this."""
import json, os, re, shutil, signal, socket, subprocess, sys, threading, time, urllib.error, urllib.parse, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
PLANT = os.environ.get("PLANT")
OUT = HERE / "data" / ("plants/" + PLANT if PLANT else ""); W = Path(os.environ["W"]); BIN = W / "svrngs"
TRIP = "Trip"
LOG = "info,removal=debug,door=debug,origin=debug,join=debug,redeem=debug,ring_routes=debug"

# Each plant: (file, the text there, what replaces it). The run stops if a
# plant's text is not in its file once, so a plant never silently misses.
PLANTS = {
    "untold": [("crates/svrngs-node/src/removal.rs",
                "m.device == Device::Computer && *k != at.host.actor && (left || cut.contains(k))",
                "false && m.device == Device::Computer && *k != at.host.actor && (left || cut.contains(k))")],
    "deleted": [("crates/svrngs-node/src/removal.rs",
                 "    let me = Leave(at.host.actor.clone());",
                 "    for d in crate::folder::dirs_of(&at.door.data_dir, &asked.group).unwrap_or_default() { let _ = std::fs::remove_dir_all(d); }\n"
                 "    let me = Leave(at.host.actor.clone());")],
    "removed": [("crates/svrngs-node/src/home/question.js",
                 "${r.said === 'left' ? `${r.of} left`", "${r.said === 'left' ? `${r.by} removed ${r.of}`")],
    "keeper": [("crates/svrngs-core/src/membership.rs",
                "&& (keeper.as_deref().map(of) != Some(of(key)) || alone);", "&& (true || alone);")],
    "rejoin": [("crates/svrngs-node/src/join.rs",
                "            Ok(Standing::Left(_)) => {", "            Ok(Standing::Left(_)) if false => {")],
}

procs, outs, summary, hidden, pages = {}, {}, {}, [], []


def git(*a): return subprocess.run(["git", "-C", str(REPO), *a], capture_output=True, text=True).stdout.strip()


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


def http(method, url, headers=None, timeout=60):
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


def port(who): return int((W / who / ".svrngs/api").read_text().strip().rstrip("/").rsplit(":", 1)[1])


def api(who, path):
    try: s, b, _ = http("GET", f"http://127.0.0.1:{port(who)}{path}", timeout=30)
    except Exception: return None
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


def command(who, *args, stdin="\n", bound=90):
    t = time.time()
    try:
        run = subprocess.run([BIN, *args], env={**env(who), "RUST_LOG": "error"}, input=stdin, capture_output=True,
                             text=True, timeout=bound)
        code, words = run.returncode, run.stdout + run.stderr
    except subprocess.TimeoutExpired as e:
        code, words = None, f"(still running after {bound} s, stopped) " + ((e.stdout or b"").decode() if isinstance(e.stdout, bytes) else (e.stdout or ""))
    out = {"who": who, "args": [a if not a.startswith(("svrngs://", "http")) else "<link>" for a in args], "exit": code,
           "said": redact(words.strip()), "ms": ms(t)}
    say("command", **out)
    return out


def joined(who, link, name):
    t = start(who, "join", link, "--as", name)
    got = wait(lambda: re.search(r"You're in .*|svrngs: .*", said(who)), 90)
    if not got or not got[0].startswith("You're in"): sys.exit(f"{who} did not join: {said(who)[-400:]}")
    say("joined", who=who, said=got[0], ms=ms(t))


def trip(who):
    """who's computer's Trip, as /v1/groups says it."""
    return next((g for g in (api(who, "/v1/groups") or {}).get("groups", []) if g.get("name") == TRIP), None)


def people(who, gid): return (api(who, f"/v1/rail/people?namespace={gid}") or {}).get("people")


def ops(who, gid): return [o["id"] for o in (api(who, f"/v1/rail/log?namespace={gid}") or {}).get("ops", [])]


def lends(who, gid):
    return {(l["person"], l["name"]): l["origin"] for l in (api(who, f"/v1/lends?namespace={gid}") or {}).get("lent", [])}


def lend(who, origin):
    """What `origin` answers on who's computer, asked at its address as a program asks."""
    host = urllib.parse.urlsplit(origin).netloc
    s, b, h = http("GET", f"http://127.0.0.1:{port(who)}/", headers={"Host": host})
    return {"status": s, "refused": h.get("x-svrngs-refused"), "said": b if not isinstance(b, str) else b[:300]}


# --- the pages ---------------------------------------------------------------
def home(who, step, **args):
    """who's home page in Chromium, as web/tests/home.mjs reads it."""
    t = time.time()
    ask = f"http://svrngs.localhost:{port(who)}/"
    run = subprocess.run(["node", REPO / "web/tests/home.mjs", json.dumps({"engine": "chromium", "ask": ask, **args})],
                         capture_output=True, text=True, timeout=180)
    try: got = json.loads(run.stdout.strip().splitlines()[-1])
    except Exception: got = {"error": (run.stderr or run.stdout)[-800:]}
    got = {k: got.get(k) for k in ("all", "after", "error") if got.get(k) is not None}
    page = {"step": step, "who": who, "computer": True, "started": round(t, 3), "ms": ms(t), **got}
    pages.append(page)
    say("page", **page)
    return page


phones = None


def phone(**c):
    phones.stdin.write(json.dumps(c) + "\n"); phones.stdin.flush()
    line = phones.stdout.readline()
    got = json.loads(line) if line else {"error": "people.mjs stopped"}
    page = {"step": c.get("step"), "who": c["who"], "computer": False, **got}
    if c["do"] != "open": pages.append(page)
    say("phone", **{k: (redact(v) if isinstance(v, str) else v) for k, v in c.items() if k != "link"}, got=got)
    return got


def serving(who, words):
    """A page with `words`, served on this host for who to lend."""
    d = W / f"{who}-lend"; d.mkdir(parents=True, exist_ok=True)
    (d / "index.html").write_text(f"<!doctype html><title>{words}</title><h1>{words}</h1>\n")
    with socket.socket() as s: s.bind(("127.0.0.1", 0)); p = s.getsockname()[1]
    procs[f"{who}-lend"] = subprocess.Popen([sys.executable, "-m", "http.server", str(p), "--bind", "127.0.0.1", "--directory", d],
                                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    if not wait(lambda: http("GET", f"http://127.0.0.1:{p}/", timeout=2)[0] == 200, 20): sys.exit(f"{who}'s page did not answer")
    return p


# --- the build, a plant applied while it builds --------------------------------
def build():
    planted = []
    try:
        for name, old, new in PLANTS.get(PLANT, []):
            f = REPO / name
            text = f.read_text()
            if text.count(old) != 1: sys.exit(f"the {PLANT} plant's text is not in {name} once: {old}")
            planted.append((f, text))
            f.write_text(text.replace(old, new))
        t = time.time()
        subprocess.run(["cargo", "build", "--quiet", "-p", "svrngs"], cwd=REPO, check=True, env={**os.environ, "CARGO_INCREMENTAL": "0"})
        shutil.copy2(REPO / "target/debug/svrngs", BIN)
        subprocess.run([REPO / "scripts/build-web.sh", "--out", W / "site"], check=True, capture_output=True)
        say("built", s=round(time.time() - t), plant=PLANT, planted=[str(f.relative_to(REPO)) for f, _ in planted])
    finally:
        for f, text in planted: f.write_text(text)
    if git("status", "--short", "--", "crates"): sys.exit("the tree is not as it was after the build")


def main():
    global phones
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    for f in OUT.glob("*") if OUT.exists() else []:
        if f.name != "plants": shutil.rmtree(f) if f.is_dir() else f.unlink()
    OUT.mkdir(parents=True, exist_ok=True)
    say("start", head=git("rev-parse", "--short", "HEAD"), plant=PLANT, load=round(os.getloadavg()[0], 1),
        cores=os.cpu_count(), dirty=git("status", "--short").splitlines())
    try:
        build()
        procs["page"] = subprocess.Popen(["node", REPO / "web/tests/serve.mjs", W / "site"], stdout=subprocess.PIPE, text=True)
        page_port = re.search(r"port (\d+)", procs["page"].stdout.readline())[1]
        threading.Thread(target=lambda: [None for _ in procs["page"].stdout], daemon=True).start()
        PAGE = f"http://localhost:{page_port}/"
        phones = subprocess.Popen(["node", HERE / "people.mjs"], cwd=HERE, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                  stderr=open(W / "people.err", "w"), text=True)
        if not json.loads(phones.stdout.readline()).get("ready"): sys.exit("people.mjs did not start")
        phone(do="open", who="cy"); phone(do="open", who="ana-phone")

        # Trip: Sam makes it; Ana and Bo join by its link, Cy's phone by its page address.
        start("sam", "new", TRIP, "--as", "Sam", "--origin", PAGE)
        link, page_link = links("sam")
        up("sam")
        joined("ana", link, "Ana"); joined("bo", link, "Bo")
        phone(do="join", who="cy", link=page_link, name="Cy", step="setup")
        G = trip("sam")["id"]
        summary["group"] = G
        keys = {p["person"]: p for p in people("sam", G) if p.get("device") == "computer"}
        summary["computers"] = {k: v.get("key") for k, v in keys.items()}
        for who, words, name in (("sam", "Sam's notes", "notes"), ("bo", "Bo's photos", "photos"), ("ana", "Ana's list", "list")):
            command(who, "share", str(serving(who, words)), "--kind", name, "--name", name)

        # 1. Before: each lend reaches the others; before.txt reaches the folders.
        want = {("Sam", "notes"), ("Bo", "photos")}
        if not wait(lambda: want <= set(lends("ana", G)), 120, 0.5): sys.exit(f"Ana lists no lends: {lends('ana', G)}")
        if not wait(lambda: ("Ana", "list") in lends("sam", G), 120, 0.5): sys.exit("Sam lists not Ana's list")
        origin = {"notes": lends("ana", G)[("Sam", "notes")], "photos": lends("ana", G)[("Bo", "photos")],
                  "list": lends("sam", G)[("Ana", "list")]}
        summary["origins"] = origin
        summary["before"] = {"notes": lend("ana", origin["notes"]), "photos": lend("ana", origin["photos"]),
                             "list": lend("sam", origin["list"])}
        folder = {who: Path(trip(who)["folders"][0]) for who in ("sam", "ana", "bo")}
        summary["folders"] = {k: str(v) for k, v in folder.items()}
        before = b"before Ana leaves\n"
        (folder["sam"] / "before.txt").write_bytes(before)
        t = time.time()
        got = wait(lambda: all((folder[w] / "before.txt").exists() and (folder[w] / "before.txt").read_bytes() == before
                               for w in ("ana", "bo")), 120, 0.5)
        summary["before_txt"] = {"reached": bool(got), "ms": ms(t)}

        # 2. Ana's computer asleep, its data dir copied whole, awake.
        stop("ana")
        shutil.copytree(W / "ana/.svrngs", W / "ana-copy/.svrngs")
        start("ana"); up("ana")
        held = ops("ana", G)
        summary["ana_ops_before"] = len(held)

        # 3. Ana's home page: Trip's section read, "Leave Trip" pressed.
        all_ = "#home > p, #home section h2, #home section p"
        leave = home("ana", "3", all=all_, **{"do": [["#home section button:has-text('Leave Trip')", None]]})
        pressed = leave["started"]
        summary["ana_leave"] = leave

        # 4. Each lender holds her out, timed from when the page was asked.
        ana_key = summary["computers"].get("Ana")
        def out(who):
            g = trip(who) or {}
            left = any(r.get("of") == "Ana" and r.get("said") == "left" for r in g.get("removals") or [])
            standing = any(p.get("key") == ana_key for p in people(who, G) or [])
            return left and not standing
        summary["held_out"] = {}
        for who in ("sam", "bo"):
            got = wait(lambda: out(who), max(1, 60 - (time.time() - pressed)), 0.25)
            summary["held_out"][who] = {"out": bool(got), "ms": ms(pressed),
                                        "removals": (trip(who) or {}).get("removals"),
                                        "people": [p.get("person") for p in people(who, G) or []]}

        # 5. Nothing lent reaches her, nor hers them.
        summary["after"] = {"notes": lend("ana", origin["notes"]), "photos": lend("ana", origin["photos"]),
                            "list": lend("sam", origin["list"]), "sam_lists": [list(k) for k in lends("sam", G)],
                            "ana_lists": [list(k) for k in lends("ana", G)]}

        # 6. Her copy kept; after.txt reaches Bo's folder, not hers.
        now = ops("ana", G)
        summary["ana_copy"] = {"before": len(held), "after": len(now), "kept": set(held) <= set(now),
                               "new": [o for o in now if o not in held],
                               "folder": folder["ana"].exists(),
                               "before_txt": (folder["ana"] / "before.txt").exists()
                               and (folder["ana"] / "before.txt").read_bytes() == before}
        (folder["sam"] / "after.txt").write_bytes(b"after Ana left\n")
        t = time.time()
        at_bo = wait(lambda: (folder["bo"] / "after.txt").exists(), 90, 0.5)
        summary["after_txt"] = {"bo": bool(at_bo), "bo_ms": ms(t)}
        time.sleep(20)
        summary["after_txt"]["ana"] = (folder["ana"] / "after.txt").exists()

        # 7. "Ana left" on the others' pages.
        for who in ("sam", "bo"):
            home(who, "7", all="#home .removals li, #home section h2")
        cy = phone(do="read", who="cy", url=PAGE, want="^Ana ", within=60_000, step="7")
        summary["cy_ms"] = ms(pressed)
        summary["cy"] = cy

        # 8. Her computer as it was before the leave, in its place.
        stop("ana")
        t = start("ana-copy"); up("ana-copy")
        summary["copy"] = {"up_ms": ms(t), "notes": lend("ana-copy", origin["notes"]), "photos": lend("ana-copy", origin["photos"])}
        got = wait(lambda: (trip("ana-copy") or {}).get("left") is True, 60, 0.5)
        summary["copy"].update({"left": bool(got), "left_ms": ms(t)})
        stop("ana-copy")
        start("ana"); up("ana")

        # 9. A new link; Ana pastes it on her home page, then runs svrngs join.
        stop("sam")
        start("sam", "link", TRIP)
        link2, page_link2 = links("sam")
        up("sam")
        summary["rejoin_page"] = home("ana", "9", all="#join p, #join button",
                                      **{"do": [["#join input[name=link]", link2, True]]})
        stop("ana")
        summary["rejoin_command"] = command("ana", "join", link2, "--as", "Ana")
        start("ana"); up("ana")
        summary["sam_people_after_rejoin"] = [[p.get("person"), p.get("key"), p.get("device")] for p in people("sam", G) or []]

        # 10. Ana's phone, by the same link's page address.
        summary["ana_phone"] = phone(do="join", who="ana-phone", link=page_link2, name="Ana", step="10")
        time.sleep(2)
        summary["sam_people_after_phone"] = [[p.get("person"), p.get("key"), p.get("device")] for p in people("sam", G) or []]
        home("sam", "10", all="#home .removals li")

        # 11. The keeper: Sam leaves before handing on, by the page and the command; hands on; leaves.
        keeper = summary["keeper"] = {}
        keeper["page"] = home("sam", "11a", all="#home section p", **{"do": [["#home section button:has-text('Leave Trip')", None, True]]})
        keeper["left_after_press"] = (trip("sam") or {}).get("left")
        keeper["command"] = command("sam", "leave", TRIP)
        keeper["left_after_command"] = (trip("sam") or {}).get("left")
        keeper["hand"] = home("sam", "11b", all="#home section p", **{"do": [["#home section button:has-text('Hand it on')", None, True]]})
        keeper["leave"] = home("sam", "11c", all=all_ + ", #left p", **{"do": [["#home section button:has-text('Leave Trip')", None]]})
        t = keeper["leave"]["started"]
        got = wait(lambda: any(r.get("of") == "Sam" and r.get("said") == "left" for r in (trip("bo") or {}).get("removals") or []), 60, 0.25)
        keeper["bo"] = {"sam_left": bool(got), "ms": ms(t), "keeps": (trip("bo") or {}).get("keeps"), "sam_left_flag": (trip("sam") or {}).get("left")}
        home("bo", "11d", all="#home .removals li, #home section p")
    finally:
        if phones:
            try: phones.stdin.close(); phones.wait(30)
            except Exception: phones.kill()
        for name, p in list(procs.items()):
            p.send_signal(signal.SIGINT if name in ("sam", "ana", "bo", "ana-copy") else signal.SIGTERM)
        for p in procs.values():
            try: p.wait(20)
            except Exception: p.kill()
        summary.update({"pages": pages, "end_load": round(os.getloadavg()[0], 1)})
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, ensure_ascii=False)) + "\n")
        if (W / "people.err").exists(): (OUT / "people.err").write_text(redact((W / "people.err").read_text()))
        for who in ("sam", "ana", "bo", "ana-copy"):
            if (W / f"{who}.log").exists():
                keep = [re.sub(r"\x1b\[[0-9;]*m", "", l) for l in (W / f"{who}.log").read_text(errors="replace").splitlines()
                        if re.search(r"removal: |door: refused|origin: refused|join: |redeem: |told_cut|ring_routes|ERROR|WARN", l)]
                (OUT / f"{who}.log").write_text(redact("\n".join(keep[-3000:])) + "\n")
        say("end")


if __name__ == "__main__":
    main()
