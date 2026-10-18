#!/usr/bin/env python3
"""A house's rules on one host (PREDICTIONS.md here; row canon-run): this
commit's svrngs, House rules embedded in it from apps/canon/, every computer
a HOME and data dir of its own under W; the page built and served on this
host; each person a browser kept by people.mjs, Chromium for a computer and
WebKit as an iPhone for the phone, pressing what their page shows.

    W=<scratch> python3 demo/runs/canon/2026-10-17-house/run.py
    PLANT=<via|now|count|pin> W=<scratch> python3 demo/runs/canon/2026-10-17-house/run.py

A plant is applied to the tree, svrngs and the page built with it, and the
tree put back before the story starts; the story runs on the build copied
to W. Data in data/ (data/plants/<plant>/), the links' tokens never in it.
judge.py reads the summary. scripts/check.sh runs none of this."""
import json, os, re, shutil, signal, subprocess, sys, threading, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
PLANT = os.environ.get("PLANT")
OUT = HERE / "data" / ("plants/" + PLANT if PLANT else ""); W = Path(os.environ["W"]); BIN = W / "svrngs"
ELM, APP = "Elm", "House-rules"
LOG = "info,folder=debug,removal=debug,join=debug,redeem=debug,rails=debug"
QUIET, ROOM, WHY = "Quiet hours are 10pm to 7am", "Ana keeps the big room", "I work late on Fridays"
# An app's page is drawn once it says neither what it says while it reads nor a door's refusal.
DRAWN = r"How your house decides\n|How does your house decide its rules\?"

# Each plant: (file, the text there, what replaces it). The run stops if a
# plant's text is not in its file once, so a plant never silently misses.
PLANTS = {
    "via": [("apps/canon/house.js",
             "const via = o.on_behalf_of != null && o.on_behalf_of !== o.person ? o.person : null;",
             "const via = null;")],
    "now": [("apps/canon/house.js",
             "const { kept, dropped } = recorded(read, ours);",
             "const { kept, dropped } = recorded({ ...read, ops: (read.ops ?? []).filter((o) => (read.people ?? []).includes(o.on_behalf_of ?? o.person)) }, ours);")],
    "count": [("apps/canon/house.js",
               "everyone: { words: 'Everyone agrees', rule: { rule: 'unanimous' } },",
               "everyone: { words: 'Everyone agrees', rule: { rule: 'threshold', approve: 3, block: 1 } },")],
    "pin": [("crates/svrngs-node/src/carried.rs",
             "let files = read.runs.of(kind.as_str(), asker.person.as_str());",
             "let (_, now_files) = (asker, now(&read.placed, kind.as_str()));\n        let files = Some(&now_files);")],
}
# Step 11's replacement of canon's answer: every proposal read as adopted.
RIG = ("    return answer.ok;\n",
       "    if (request.call === 'derive') for (const c of answer.ok.commitments ?? []) if (c.status?.status === 'proposed') c.status = { status: 'active' };\n"
       "    return answer.ok;\n")

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
    return {**{k: v for k, v in os.environ.items() if k != "PLANT"}, "HOME": str(W / who), "USER": who,
            "SVRNGS_DIR": str(W / who / ".svrngs"), "NO_COLOR": "1", "RUST_LOG": LOG}


def start(who, *args):
    (W / who).mkdir(parents=True, exist_ok=True)
    outs[who] = W / f"{who}.{len(list(W.glob(f'{who}.*.out'))) + 1}.out"
    procs[who] = subprocess.Popen([BIN, *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(outs[who], "w"), stderr=open(W / f"{who}.log", "a"))
    say("svrngs", who=who, args=[a if not a.startswith(("svrngs://", "http")) else "<link>" for a in args])
    return time.time()


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
    if page: hidden.append(page[2])
    return got[0], page and page[1]


def joined(who, link, name):
    t = start(who, "join", link, "--as", name)
    got = wait(lambda: re.search(r"You're in .*|svrngs: .*", said(who)), 90)
    if not got or not got[0].startswith("You're in"): sys.exit(f"{who} did not join: {said(who)[-400:]}")
    say("joined", who=who, said=got[0], ms=ms(t))


def elm(who): return next((g for g in (api(who, "/v1/groups") or {}).get("groups", []) if g.get("name") == ELM), None)


def people(who, gid): return (api(who, f"/v1/rail/people?namespace={gid}") or {}).get("people")


def rules_acts(who, gid):
    """The House rules acts who's computer holds, in its record's order."""
    ops = (api(who, f"/v1/rail/log?namespace={gid}") or {}).get("ops") or []
    return [o for o in ops if str((o.get("payload") or {}).get("kind", "")).startswith(f"{APP}/")]


def reached(who, gid, n, bound=60):
    """How long until who's computer holds n House rules acts, from now; None past bound."""
    t = time.time()
    return ms(t) if wait(lambda: len(rules_acts(who, gid)) >= n, bound) else None


# --- the browsers --------------------------------------------------------------
browsers = None


def person(do, who, **c):
    t = time.time()
    browsers.stdin.write(json.dumps({"do": do, "who": who, **c}) + "\n"); browsers.stdin.flush()
    line = browsers.stdout.readline()
    got = json.loads(line) if line else {"error": "people.mjs stopped"}
    got["started"] = round(t, 3)
    say("person", do=do, who=who, **{k: (redact(v) if isinstance(v, str) else v) for k, v in got.items() if k != "done"},
        steps=[{"step": redact(json.dumps(d["step"])), "ms": d["ms"], "error": d.get("error"), "text": redact(d["text"][-1500:])}
               for d in got.get("done", [])])
    return got


def steps(key, who, *ss):
    got = person("steps", who, steps=list(ss))
    summary.setdefault("steps", {})[key] = {"who": who, **{k: v for k, v in got.items() if k != "done"},
                                           "done": [{**d, "step": redact(json.dumps(d["step"])), "text": redact(d["text"])} for d in got.get("done", [])]}
    return got


def last(got): return (got.get("done") or [{}])[-1].get("text", "")


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
    if git("status", "--short", "--", "crates", "apps"): sys.exit("the tree is not as it was after the build")


def main():
    global browsers
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
        # The phone's Elm page, opened again until it lists House rules, as a person reopens it.
        LISTED = {"poll": PAGE, "re": "House rules\tHouse rules", "within": 240_000, "every": 10_000}
        browsers = subprocess.Popen(["node", HERE / "people.mjs"], cwd=HERE, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                    stderr=open(W / "people.err", "w"), text=True)
        if not json.loads(browsers.stdout.readline()).get("ready"): sys.exit("people.mjs did not start")
        for who in ("ana", "bo", "cy"): person("open", who, engine="chromium")
        person("open", "dee", engine="webkit")

        # Elm: Ana makes it; Bo and Cy join by its link, Dee's phone by its page address.
        start("ana", "new", ELM, "--as", "Ana", "--origin", PAGE)
        link, page_link = links("ana")
        up("ana")
        joined("bo", link, "Bo"); joined("cy", link, "Cy")
        summary["dee_join"] = person("join", "dee", link=page_link, name="Dee")
        G = elm("ana")["id"]
        summary["group"] = G
        keys = {p["person"]: p.get("key") for p in people("ana", G) if p.get("device") == "computer"}
        summary["computers"] = keys
        home = {who: f"http://svrngs.localhost:{port(who)}/" for who in ("ana", "bo", "cy")}

        # 1. Ana brings House rules alone from her home page.
        got = steps("1", "ana", {"goto": home["ana"]}, {"wait": "Bring the ticked ones to Elm"},
                    {"uncheck": "#home label:has-text('Money') input"}, {"uncheck": "#home label:has-text('Events') input"},
                    {"click": "Bring the ticked ones to Elm", "until": "You brought|Nothing was brought|already"})
        t = got["started"]

        def address(who):
            for l in (api(who, f"/v1/lends?namespace={G}") or {}).get("lent", []):
                if l.get("kind") == APP and l.get("key") == keys[who.capitalize()]: return l["origin"]
        listed = {}
        for who in ("ana", "bo", "cy"):
            listed[who] = {"listed": bool(wait(lambda: address(who), max(1, 90 - (time.time() - t)), 0.5)), "ms": ms(t)}
        summary["listed"] = listed
        vendored = (REPO / "apps/canon/vendor/canon.js").read_bytes()
        def holds(who):
            f = Path(elm(who)["folders"][0]) / "apps" / APP / "vendor/canon.js"
            return f.exists() and f.read_bytes() == vendored
        summary["build"] = {"bytes": len(vendored), **{who: {"held": bool(wait(lambda: holds(who), max(1, 90 - (time.time() - t)), 0.5)), "ms": ms(t)}
                                                    for who in ("bo", "cy")}}
        summary["kinds"] = sorted({l.get("kind") for l in (api("ana", f"/v1/lends?namespace={G}") or {}).get("lent", [])})
        rules = {who: address(who) for who in ("ana", "bo", "cy")}
        summary["addresses"] = rules
        if not all(rules.values()): sys.exit(f"House rules is not listed on every computer: {listed}")

        # 2. The question, answered by Ana.
        steps("2", "ana", {"goto": rules["ana"]}, {"wait": DRAWN, "within": 60_000},
              {"count": "input[name=way]"}, {"count": "input[name=way]:checked"},
              {"check": "input[name=way][value=most]"},
              {"click": "Make this how your house decides", "until": r"How your house decides\n"})

        # 3. Ana proposes the quiet hours.
        steps("3", "ana", {"fill": "#rule", "value": QUIET},
              {"click": "Propose this rule to everyone in your house", "until": "You proposed it on"})
        summary["reached_3"] = {who: reached(who, G, 2) for who in ("bo", "cy")}

        # 4. Dee's phone agrees, through whichever computer its list opens House rules on.
        steps("4", "dee", LISTED,
              {"lent": "House rules", "until": DRAWN}, {"click": "Agree", "in": QUIET, "until": "Agreed: "})
        wait(lambda: len(rules_acts("ana", G)) >= 3, 60)
        dee = [o for o in rules_acts("ana", G) if o.get("on_behalf_of") == "Dee"]
        summary["dee_act"] = dee and {k: dee[-1].get(k) for k in ("id", "person", "on_behalf_of", "ts_unix")}
        summary["reached_4"] = {who: reached(who, G, 3) for who in ("ana", "bo", "cy")}

        # 5. Cy objects.
        steps("5", "cy", {"goto": rules["cy"]}, {"wait": DRAWN, "within": 60_000}, {"read": True},
              {"click": "Object", "in": QUIET, "until": "Why do you object"}, {"fill": "input.why", "value": WHY},
              {"click": "Send your objection", "until": "You object"})
        summary["reached_5"] = {who: reached(who, G, 4) for who in ("ana", "bo")}

        # 6. Bo reads it, and agrees: it holds.
        got = steps("6", "bo", {"goto": rules["bo"]}, {"wait": DRAWN, "within": 60_000}, {"read": True},
                    {"click": "Agree", "in": QUIET, "until": "Your house’s rules\n+" + QUIET})
        t = got["started"] + got["done"][-1]["ms"] / 1000
        summary["reached_6"] = {who: reached(who, G, 5) for who in ("ana", "cy")}
        for who in ("ana", "cy"): steps(f"6-{who}", who, {"goto": rules[who]}, {"wait": DRAWN, "within": 60_000})
        steps("6-dee", "dee", LISTED, {"lent": "House rules", "until": DRAWN})
        summary["held_6_ms"] = ms(t)

        # 7. Bo, who agreed, leaves from his home page.
        got = steps("7", "bo", {"goto": home["bo"]}, {"wait": "Leave Elm"}, {"click": "Leave Elm", "until": "You left Elm"})
        t = got["started"]
        bo_key = keys["Bo"]

        def out(who, key, name):
            g = elm(who) or {}
            left = any(r.get("of") == name and r.get("said") == "left" for r in g.get("removals") or [])
            return left and not any(p.get("key") == key for p in people(who, G) or [])
        summary["held_out_7"] = {who: {"out": bool(wait(lambda: out(who, bo_key, "Bo"), max(1, 60 - (time.time() - t)))), "ms": ms(t)}
                                 for who in ("ana", "cy")}
        time.sleep(2)
        for who in ("ana", "cy"): steps(f"7-{who}", who, {"goto": rules[who]}, {"wait": DRAWN, "within": 60_000})
        steps("7-dee", "dee", LISTED, {"lent": "House rules", "until": DRAWN})

        # 8. Cy proposes "everyone agrees"; Ana agrees.
        steps("8", "cy", {"goto": rules["cy"]}, {"wait": DRAWN, "within": 60_000}, {"summary": "Change how your house decides"},
              {"check": "input[name=change][value=everyone]"},
              {"click": "Propose this change to everyone in your house", "until": "Change how your house decides: A rule holds once everyone"})
        summary["reached_8"] = reached("ana", G, 6)
        got = steps("8-ana", "ana", {"goto": rules["ana"]}, {"wait": DRAWN, "within": 60_000}, {"read": True},
                    {"click": "Agree", "in": "Change how your house decides"},
                    {"poll": rules["ana"], "re": r"How your house decides\n+A rule holds once everyone", "within": 30_000, "every": 2_000})
        summary["reached_8_ana"] = reached("cy", G, 7)
        steps("8-cy", "cy", {"goto": rules["cy"]}, {"wait": DRAWN, "within": 60_000})

        # 9. Under "everyone agrees", Cy leaves.
        got = steps("9", "cy", {"goto": home["cy"]}, {"wait": "Leave Elm"}, {"click": "Leave Elm", "until": "You left Elm"})
        t = got["started"]
        summary["held_out_9"] = {"ana": {"out": bool(wait(lambda: out("ana", keys["Cy"], "Cy"), 60)), "ms": ms(t)}}
        time.sleep(2)
        steps("9-ana", "ana", {"goto": rules["ana"]}, {"wait": DRAWN, "within": 60_000})

        # 10. Dee's phone proposes "most of you agree" again; Ana agrees.
        steps("10", "dee", LISTED, {"lent": "House rules", "until": DRAWN}, {"summary": "Change how your house decides"},
              {"check": "input[name=change][value=most]"},
              {"click": "Propose this change to everyone in your house", "until": "Change how your house decides: A rule holds once more than half"})
        summary["reached_10"] = reached("ana", G, 8)
        got = steps("10-ana", "ana", {"goto": rules["ana"]}, {"wait": DRAWN, "within": 60_000}, {"read": True},
                    {"click": "Agree", "in": "Change how your house decides"},
                    {"poll": rules["ana"], "re": r"How your house decides\n+A rule holds once more than half", "within": 30_000, "every": 2_000})
        steps("10-dee", "dee", LISTED, {"lent": "House rules", "until": DRAWN})

        # 11. Ana proposes the big room, then replaces vendor/call.mjs in Elm's folder.
        steps("11-propose", "ana", {"goto": rules["ana"]}, {"wait": DRAWN, "within": 60_000}, {"fill": "#rule", "value": ROOM},
              {"click": "Propose this rule to everyone in your house", "until": ROOM})
        call = Path(elm("ana")["folders"][0]) / "apps" / APP / "vendor/call.mjs"
        text = call.read_text()
        if text.count(RIG[0]) != 1: sys.exit("the replacement's text is not in call.mjs once")
        call.write_text(text.replace(*RIG))
        t = time.time()
        say("replaced", path=str(call))
        changed = wait(lambda: (api("ana", f"/v1/lends?namespace={G}") or {}).get("changed"), 60, 0.5)
        summary["changed"] = {"ana_lists": changed, "ms": ms(t)}
        steps("11-dee", "dee", {"poll": PAGE, "re": "Open the new version\\?", "within": 60_000}, {"read": True},
              {"lent": "House rules", "until": DRAWN})
        summary["asked_11_ms"] = ms(t)
        steps("11-ana", "ana", {"goto": rules["ana"]}, {"wait": DRAWN, "within": 60_000})
        steps("11-yes", "dee", LISTED, {"wait": "Yes, use the new version of House rules", "within": 60_000},
              {"click": "Yes, use the new version of House rules", "until": "opens at its new version for you now", "within": 60_000},
              LISTED, {"lent": "House rules", "until": DRAWN})
        steps("11-ana-after", "ana", {"goto": rules["ana"]}, {"wait": DRAWN, "within": 60_000})
    finally:
        if browsers:
            try: browsers.stdin.close(); browsers.wait(60)
            except Exception: browsers.kill()
        for name, p in list(procs.items()):
            p.send_signal(signal.SIGINT if name in ("ana", "bo", "cy") else signal.SIGTERM)
        for p in procs.values():
            try: p.wait(20)
            except Exception: p.kill()
        summary["end_load"] = round(os.getloadavg()[0], 1)
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, ensure_ascii=False)) + "\n")
        if (W / "people.err").exists(): (OUT / "people.err").write_text(redact((W / "people.err").read_text()))
        for who in ("ana", "bo", "cy"):
            if (W / f"{who}.log").exists():
                keep = [re.sub(r"\x1b\[[0-9;]*m", "", l) for l in (W / f"{who}.log").read_text(errors="replace").splitlines()
                        if re.search(r"carried: |shelf: |removal: |synced: what runs|ERROR|WARN", l)]
                (OUT / f"{who}.log").write_text(redact("\n".join(keep[-3000:])) + "\n")
        say("end")


if __name__ == "__main__":
    main()
