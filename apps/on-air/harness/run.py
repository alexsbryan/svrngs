#!/usr/bin/env python3
"""ON_AIR §7's proof of concept, on one host. Ana makes the group `onair`;
Bo, Cy and Dee join by her link, as demo/runs/devices/2026-10-12/run.py
starts them; Ana removes Dee, a computer outside the group that once stood
in it. Ana is the station (station.mjs), Bo and Cy listen (listen.mjs), and
a stranger's headless Chromium reads the public directory (stranger.mjs,
public.py). Then the plants, the carried and scoped asks, and the lapse.
Each of the seven bars is judged from what was kept: passed, failed,
could-not-judge, or never-ran (AGENTS.md 6).

    W=<scratch> python3 apps/on-air/harness/run.py <run dir>

Data in <run dir>/data/. The bars are judged in data/verdicts.json, with
what each read. The link's token is never written there.
scripts/check.sh runs none of this."""
import json, os, re, shutil, signal, subprocess, sys, time, traceback, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; APP = HERE.parent; REPO = APP.parents[1]
OUT = Path(sys.argv[1]).resolve() / "data"; W = Path(os.environ["W"]).resolve()
BIN = REPO / "target/debug/svrngs"
GROUP, SHOW, KEEP, PIECE, SECONDS = "onair", "evening", 60, 4, 40
BOUND_MS = 4000  # 4.1's bound, set before the run: one piece's time on air
LOG = os.environ.get("ONAIR_LOG", "info,rails=debug,folder=debug,blob=debug,sign=debug,removal=debug")
procs, hidden, ev = {}, [], {}


def redact(text):
    for h in hidden: text = text.replace(h, "<the token>")
    return text


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}, ensure_ascii=False))
    print(line[:400], flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def wait(cond, bound, every=0.2):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def http(method, url, body=None, timeout=20):
    """{status, said, bytes, ms} of one request as it came, or status None and why."""
    data = body if isinstance(body, (bytes, type(None))) else body.encode()
    req = urllib.request.Request(url, method=method, data=data, headers={"content-type": "application/octet-stream"})
    t0 = time.time()
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r: status, blob = r.status, r.read()
    except urllib.error.HTTPError as e: status, blob = e.code, e.read()
    except Exception as e: return {"status": None, "said": f"could not ask: {e}", "bytes": b"", "ms": round((time.time() - t0) * 1000)}
    return {"status": status, "said": blob[:300].decode(errors="replace"), "bytes": blob, "ms": round((time.time() - t0) * 1000)}


def shown(r): return {k: v for k, v in r.items() if k != "bytes"} | {"length": len(r.get("bytes", b""))}


# --- each computer: a data dir and HOME of its own --------------------------
def env(who):
    return {**os.environ, "HOME": str(W / who), "USER": who, "SVRNGS_DIR": str(W / who / ".svrngs"),
            "NO_COLOR": "1", "RUST_LOG": LOG, "CARGO_INCREMENTAL": "0"}


def base(who):
    try: return (W / who / ".svrngs/api").read_text().strip().rstrip("/")
    except Exception: return None


def api(who, path):
    b = base(who)
    if not b: return None
    r = http("GET", b + path)
    return json.loads(r["bytes"]) if r["status"] == 200 else None


def command(who, *args):
    run = subprocess.run([BIN, *args], env={**env(who), "RUST_LOG": "error"}, input="\n", capture_output=True, text=True)
    out = {"who": who, "args": list(args), "exit": run.returncode, "said": (run.stdout + run.stderr).strip()}
    say("command", **out)
    return out


def start(who, *args):
    (W / who).mkdir(parents=True, exist_ok=True)
    procs[who] = subprocess.Popen([BIN, *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(W / f"{who}.out", "w"), stderr=open(W / f"{who}.log", "w"))


def said(who): return (W / f"{who}.out").read_text() if (W / f"{who}.out").exists() else ""


def stop(name):
    p = procs.pop(name, None)
    if not p: return None
    p.send_signal(signal.SIGTERM)
    try: p.wait(15)
    except Exception: p.kill(); p.wait()
    return p.returncode


def people():
    rows = (api("ana", f"/v1/rail/people?namespace={ev['group']}") or {}).get("people", [])
    return {r["person"]: {"key": r["key"], "person_key": r.get("person_key")} for r in rows}


def node(script, *args, **kw):
    return subprocess.Popen(["node", APP / script, *map(str, args)], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, **kw)


def field(o, key):
    """The first `key` in an op as /v1/rail/log gives it, at any depth."""
    if isinstance(o, dict):
        if key in o: return o[key]
        for v in o.values():
            got = field(v, key)
            if got is not None: return got
    return None


def blake3(data):
    """The bytes' BLAKE3, by the page's own vendored copy (page/air.mjs)."""
    js = (f"await import({json.dumps(str(APP / 'vendored.mjs'))}); const a = await import({json.dumps(str(APP / 'page/air.mjs'))});"
          "const c = []; for await (const x of process.stdin) c.push(x); console.log(a.blake3hex(new Uint8Array(Buffer.concat(c))))")
    return subprocess.run(["node", "--input-type=module", "-e", js], input=data, capture_output=True, check=True).stdout.decode().strip()


def jsonl(path):
    return [json.loads(l) for l in Path(path).read_text().splitlines() if l.strip()] if Path(path).exists() else []


def stations(*args):
    """A plant's show, sent as fast as it goes; its record."""
    run = subprocess.run(["node", APP / "station.mjs", *map(str, args)], capture_output=True, text=True, timeout=120)
    return [json.loads(l) for l in run.stdout.splitlines() if l.startswith("{")], run.returncode, run.stderr[-500:]


def strangers(links, bound):
    p = node("harness/stranger.mjs", "--for", bound, *links)
    return p


def read_strangers(p, bound):
    try: out, err = p.communicate(timeout=bound + 60)
    except subprocess.TimeoutExpired: p.kill(); out, err = p.communicate()
    return {"exit": p.returncode, "tabs": [json.loads(l) for l in out.splitlines() if l.startswith("{")], "stderr": err[-500:]}


# --- the run -------------------------------------------------------------
def main():
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    OUT.mkdir(parents=True, exist_ok=True)
    for f in OUT.iterdir():
        if f.is_file(): f.unlink()
    say("start", commit=subprocess.run(["git", "-C", REPO, "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip(),
        load=round(os.getloadavg()[0], 1))
    subprocess.run(["cargo", "build", "--quiet", "-p", "svrngs"], cwd=REPO, check=True, env={**os.environ, "CARGO_INCREMENTAL": "0"})
    mp3 = W / "tone.mp3"
    subprocess.run(["ffmpeg", "-y", "-hide_banner", "-loglevel", "error", "-f", "lavfi", "-i",
                    f"aevalsrc=0.2*sin(2*PI*(220*pow(1.5\\,floor(t/2)/4))*t):s=44100:d={SECONDS}",
                    "-ac", "1", "-c:a", "libmp3lame", "-b:a", "128k", "-write_xing", "0", "-id3v2_version", "0",
                    "-map_metadata", "-1", mp3], check=True)
    ev["sound"] = {"made_by": f"ffmpeg aevalsrc, {SECONDS} s, mono, 128 kbit/s: generated here, nobody's rights", "bytes": mp3.stat().st_size}
    public = W / "public"
    shutil.rmtree(public, ignore_errors=True); public.mkdir()
    shutil.copytree(APP / "page", public / "page")
    procs["public"] = subprocess.Popen([sys.executable, HERE / "public.py", public], stdout=subprocess.PIPE, text=True)
    port = re.search(r"port (\d+)", procs["public"].stdout.readline())[1]
    page = f"http://127.0.0.1:{port}/page/"

    # Ana's group, and Bo, Cy and Dee in it by her link.
    start("ana", "new", GROUP, "--as", "Ana")
    got = wait(lambda: re.search(r"svrngs://join/(\S+)", said("ana")), 90)
    if not got: raise SystemExit(f"Ana printed no link: {said('ana')[-400:]}")
    hidden.append(got[1]); link = got[0]
    ev["group"] = wait(lambda: ((api("ana", "/v1/groups") or {}).get("groups") or [{}])[0].get("id"), 60)
    for who, name in (("bo", "Bo"), ("cy", "Cy"), ("dee", "Dee")):
        start(who, "join", link, "--as", name)
        if not wait(lambda: "You're in" in said(who), 90): raise SystemExit(f"{name} is not in: {said(who)[-400:]}")
    ev["people_joined"] = wait(lambda: (lambda p: len(p) >= 4 and p)(people()), 60)
    ev["remove_dee"] = command("ana", "remove", "Dee", "--group", GROUP)
    ev["people"] = wait(lambda: (lambda p: "Dee" not in p and len(p) == 3 and p)(people()), 60) or people()
    # Each member's own record must hold the removal too, or its live push
    # still offers Dee the payload (the debug run: Bo's note `delivered: 3`).
    ev["removal_seen"] = {w: wait(lambda: "Dee" not in {r["person"] for r in (api(w, f"/v1/rail/people?namespace={ev['group']}") or {}).get("people", [{"person": "Dee"}])}, 60) is not None
                          for w in ("bo", "cy")}
    keys = ev["people"]
    if not all(k in keys for k in ("Ana", "Bo", "Cy")): raise SystemExit(f"the group is not Ana, Bo and Cy: {keys}")
    ana_key = keys["Ana"]["key"]
    wait(lambda: all(base(w) for w in ("ana", "bo", "cy", "dee")), 30)
    say("group", group=ev["group"], people=keys)

    # Bo and Cy listen; the stranger opens the show; Bo says a word on the lane.
    listeners = {w: node("listen.mjs", "--api", base(w), "--group", ev["group"], "--station", "Ana", "--out", W / w,
                         "--for", SECONDS + 40) for w in ("bo", "cy")}
    time.sleep(1)
    tab = strangers([f"{page}#key={ana_key}&show={SHOW}"], SECONDS + 50)
    ev["bo_note"] = shown(http("POST", f"{base('bo')}/v1/rail/live?namespace={ev['group']}", json.dumps({"note": "Bo, on the lane"})))
    say("bo note", **ev["bo_note"])

    # The show.
    t0 = time.time()
    run = subprocess.run(["node", APP / "station.mjs", "--api", base("ana"), "--group", ev["group"], "--mp3", mp3, "--show", SHOW,
                          "--public", public, "--keep", str(KEEP), "--piece", str(PIECE), "--out", W / "station.jsonl"],
                         capture_output=True, text=True, timeout=SECONDS + 60)
    ev["station"] = {"exit": run.returncode, "stderr": run.stderr[-800:], "s": round(time.time() - t0, 1)}
    say("station", **ev["station"])
    station = jsonl(W / "station.jsonl")
    last_put = time.time()
    for w, p in listeners.items():
        try: p.wait(SECONDS + 60)
        except subprocess.TimeoutExpired: p.kill()
        ev[f"{w}_listen_exit"] = p.returncode
    ev["page"] = read_strangers(tab, SECONDS + 50)

    # 4.1, carried, before Ana stops: two blobs on Ana, Y got by Bo.
    Y, Z = b"onair carried Y " + os.urandom(4096), b"onair carried Z " + os.urandom(4096)
    hy, hz = blake3(Y), blake3(Z)
    ev["carried"] = {"hash_Y": hy,
                     "put_Y": shown(http("PUT", f"{base('ana')}/v1/blob?group={ev['group']}&keep={KEEP}", Y)),
                     "put_Z": shown(http("PUT", f"{base('ana')}/v1/blob?group={ev['group']}&keep={KEEP}", Z))}
    last_put = time.time()
    ev["carried"]["bo_get_Y"] = shown(http("GET", f"{base('bo')}/v1/blob/{hy}?group={ev['group']}"))
    ev["carried"]["hash_Z"] = hz

    # 4.2's plants: no end, Bo's signature, a byte flipped, an act's signature.
    plants = {}
    recs, code, err = stations("--api", base("ana"), "--group", ev["group"], "--mp3", mp3, "--show", "no-ended", "--public", public,
                               "--limit", 2, "--pace", 0, "--no-ended")
    plants["no-ended"] = {"exit": code, "stderr": err, "lines": [r.get("sign") for r in recs if r["event"] == "piece"]}
    recs, code, err = stations("--api", base("bo"), "--group", ev["group"], "--mp3", mp3, "--show", "bo-signed", "--public", public,
                               "--limit", 2, "--pace", 0)
    plants["bo-signed"] = {"exit": code, "stderr": err, "lines": [r.get("sign") for r in recs if r["event"] in ("piece", "ended")]}
    air = public / "air"
    shutil.copytree(air / SHOW, air / "flipped")
    one = json.loads((air / "flipped/lines/1.json").read_text())
    h1 = json.loads(one["line"])["hash"]
    piece = bytearray((air / "flipped/pieces" / h1).read_bytes()); piece[1000] ^= 0x01
    (air / "flipped/pieces" / h1).write_bytes(bytes(piece))
    plants["flipped"] = {"piece": 1, "hash": h1, "byte": 1000}
    # An act's signature offered as an app's. /v1/rail/log gives admitted acts
    # without their signatures (crates/svrngs-core/src/admit.rs:309,
    # AdmittedOp), so the act and its signature are read from Ana's journal,
    # as written: one of her acts, signed by the key that signs for the app.
    journal = W / "ana/.svrngs/rings" / ev["group"] / "ring_oplog.jsonl"
    acts = [a for a in jsonl(journal) if a.get("actor") == ana_key and a.get("sig")]
    logged = (api("ana", f"/v1/rail/log?namespace={ev['group']}") or {}).get("ops", [])
    shutil.copytree(air / SHOW, air / "act-sig")
    if acts:
        act = acts[0]
        line0 = json.loads((air / SHOW / "lines/0.json").read_text())["line"]
        (air / "act-sig/lines/0.json").write_text(json.dumps({"line": act["id"], "key": ana_key, "sig": act["sig"]}))
        (air / "act-sig/lines/1.json").write_text(json.dumps({"line": line0, "key": ana_key, "sig": act["sig"]}))
        plants["act-sig"] = {"act": act["id"], "seq": act.get("seq"), "from": str(journal.relative_to(W)), "lines": [0, 1],
                             "log_op_fields": sorted(logged[0]) if logged else None}
    else:
        plants["act-sig"] = {"act": None, "why": f"Ana's journal {journal.relative_to(W)} holds no signed act of hers"}
    ev["plants"] = plants
    tabs = strangers([f"{page}#key={ana_key}&show={s}" for s in ("no-ended", "bo-signed", "flipped", "act-sig")], 60)
    ev["plant_pages"] = read_strangers(tabs, 60)

    # 4.1, carried: Ana stopped, Cy asks Y (Bo holds it) and Z (no one running does).
    ev["carried"]["ana_stopped"] = stop("ana")
    t = time.time()
    ev["carried"]["cy_get_Y"] = shown(http("GET", f"{base('cy')}/v1/blob/{hy}?group={ev['group']}", timeout=60))
    ev["carried"]["cy_got_Y_bytes"] = http("GET", f"{base('cy')}/v1/blob/{hy}?group={ev['group']}", timeout=60)["bytes"] == Y
    ev["carried"]["cy_get_Z"] = shown(http("GET", f"{base('cy')}/v1/blob/{hz}?group={ev['group']}", timeout=60))
    ev["carried"]["s"] = round(time.time() - t, 1)

    # 4.1, scoped: Dee, outside the group, asks Y.
    ev["scoped"] = {"dee_get_Y": shown(http("GET", f"{base('dee')}/v1/blob/{hy}?group={ev['group']}", timeout=60))}

    # 4.1, lapses: past keep, Bo and Cy ask piece 0, Y and a hash never held.
    piece0 = next((r["hash"] for r in station if r["event"] == "piece" and r["n"] == 0), None)
    left = last_put + KEEP + 5 - time.time()
    say("lapse wait", s=round(max(left, 0), 1))
    if left > 0: time.sleep(left)
    never = os.urandom(32).hex()
    ev["lapses"] = {"s_after_last_put": round(time.time() - last_put, 1), "never_held": never, "piece0": piece0,
                    **{f"{w}_{name}": shown(http("GET", f"{base(w)}/v1/blob/{h}?group={ev['group']}", timeout=60))
                       for w in ("bo", "cy") for name, h in (("piece0", piece0), ("Y", hy), ("never", never)) if h}}


# --- the bars, judged from what was kept ------------------------------------
def judge():
    v = {}
    station = jsonl(W / "station.jsonl")
    pieces = [r for r in station if r["event"] == "piece"]
    ended = next((r for r in station if r["event"] == "ended"), None)
    heard = {w: jsonl(W / w / "listen.jsonl") for w in ("bo", "cy")}
    keys = ev.get("people") or {}

    def bar(name, verdict, why, **read): v[name] = {"verdict": verdict, "why": why, **read}

    if not pieces: bar("4.1", "never-ran", "the station sent no piece")
    else:
        held = {r["n"]: r["ms"] for r in heard["bo"] if r["event"] == "held"}
        slow = {n: ms for n, ms in held.items() if ms > BOUND_MS}
        puts = sorted({str(r["put"]["status"]) for r in pieces})
        refusals = sorted({r.get("why") or f"{r['event']} {r.get('status')}" for r in heard["bo"] if r["event"] in ("refused", "not held", "refused piece")})
        ok = len(held) == len(pieces) and not slow
        bar("4.1", "passed" if ok else "failed",
            f"Bo held {len(held)} of {len(pieces)} pieces, {len(slow)} past {BOUND_MS} ms", bound_ms=BOUND_MS,
            held_ms=held, station_puts=puts, bo_refusals=refusals)

    c = ev.get("carried")
    if not c or "cy_get_Y" not in c: bar("4.1, carried", "never-ran", "the run stopped before it")
    else:
        y, z = c.get("cy_get_Y", {}), c.get("cy_get_Z", {})
        trace = [l for l in log_lines("cy") if c["hash_Y"][:12] in l and keys.get("Bo", {}).get("key", "-")[:12] in l]
        bar_ok = c.get("cy_got_Y_bytes") and trace
        plant_ok = z.get("status") not in (200, None) and bool(z.get("said"))
        bar("4.1, carried", "passed" if bar_ok and plant_ok else "failed",
            f"Cy's get of Y with Ana stopped: {y.get('status')} ({y.get('length')} bytes), Bo named in Cy's trace: {bool(trace)}; "
            f"the plant Z: {z.get('status')} {z.get('said')!r}", ana_put_Y=c["put_Y"]["status"], bo_get_Y=c["bo_get_Y"]["status"],
            cy_trace=trace[:5])

    l = ev.get("lapses")
    if not l: bar("4.1, lapses", "never-ran", "the run stopped before it")
    else:
        answers = {k: (a["status"], a["said"]) for k, a in l.items() if isinstance(a, dict)}
        lapsed = [k for k, (s, w) in answers.items() if "never" not in k and s != 200 and "lapse" in w.lower()]
        told = all(answers[k] != answers[f"{k.split('_')[0]}_never"] for k in answers if "never" not in k)
        held_still = [k for k, (s, _) in answers.items() if s == 200]
        ok = not held_still and told and len(lapsed) == len([k for k in answers if "never" not in k])
        bar("4.1, lapses", "passed" if ok else "failed",
            f"{l['s_after_last_put']} s after the last put: still held {held_still}; said lapsed {lapsed}; "
            f"lapsed told apart from never held: {told}", answers=answers)

    s = ev.get("scoped")
    if not s: bar("4.1, scoped", "never-ran", "the run stopped before it")
    else:
        d = s["dee_get_Y"]
        dee_key = (ev.get("people_joined") or {}).get("Dee", {}).get("key", "-")
        refused = [l for w in ("bo",) for l in log_lines(w) if "refused" in l and dee_key[:12] in l]
        ok = d["status"] not in (200, None) and refused
        bar("4.1, scoped", "passed" if ok else "failed",
            f"Dee's get of Y: {d['status']} {d['said']!r}; a provider's refusal of Dee in Bo's trace: {bool(refused)}",
            provider_refusals=refused[:5])

    pages = (ev.get("page") or {}).get("tabs") or []
    if not ev.get("plant_pages"): pages = []  # 4.2 is judged with its plants, or not at all
    plant_tabs = {re.search(r"show=([\w-]+)", t["link"])[1]: t for t in (ev.get("plant_pages") or {}).get("tabs") or []}
    if not pages: bar("4.2", "could-not-judge", "the stranger read no tab", page=ev.get("page"))
    else:
        a = pages[0].get("air") or {}
        played = [p["n"] for p in a.get("played", [])]
        pr = {k: [p["n"] for p in (t.get("air") or {}).get("played", [])] for k, t in plant_tabs.items()}
        whys = {k: sorted({r["why"] for r in (t.get("air") or {}).get("refused", [])}) for k, t in plant_tabs.items()}
        ok = (played == list(range(len(pieces))) and pr.get("bo-signed") == [] and pr.get("act-sig") == [n for n in range(len(pieces)) if n > 1]
              and 1 not in pr.get("flipped", [1]) and len(pr.get("flipped", [])) == len(pieces) - 1)
        bar("4.2", "passed" if ok else "failed",
            f"the page on {SHOW} played {len(played)} of {len(pieces)}; plants played {pr}",
            signs=sorted({str(r["sign"]["status"]) for r in pieces}), page_refused=sorted({r["why"] for r in a.get("refused", [])}),
            page_said=[a.get("status"), a.get("why")], plant_refusals=whys)

    if not any(heard.values()): bar("4.3", "never-ran", "no listener kept a record")
    else:
        drained = {w: [r for r in h if "sender" in r] for w, h in heard.items()}
        named = {w: [r["sender"] for r in d] for w, d in drained.items()}
        ana = keys.get("Ana", {})
        lines_ok = all(isinstance(s, dict) and s.get("key") == ana.get("key") and s.get("person_key") == ana.get("person_key")
                       for w in ("bo", "cy") for r, s in zip(drained[w], named[w]) if "note" not in r["payload"])
        note = [r for r in drained["cy"] if "note" in r["payload"]]
        note_ok = bool(note) and isinstance(note[0]["sender"], dict) and note[0]["sender"].get("key") == keys.get("Bo", {}).get("key")
        bar("4.3", "passed" if lines_ok and note_ok and all(drained.values()) else "failed",
            f"Bo drained {len(drained['bo'])}, Cy {len(drained['cy'])}; senders named: "
            f"{sorted({json.dumps(s) for w in named for s in named[w]})}; Bo's note in Cy's drain: {note[0]['sender'] if note else 'not drained'}",
            live_delivered=sorted({str(r["live"].get("delivered")) for r in station if "live" in r}), bo_note=ev.get("bo_note"))

    if not ended: bar("off air", "never-ran", "the station sent no ended")
    else:
        said_ended = {w: next((r for r in h if r["event"] == "ended"), None) for w, h in heard.items()}
        stopped = {w: next((r.get("why") for r in h if r["event"] == "stopped"), None) for w, h in heard.items()}
        page_end = pages[0] if pages else {}
        within = {w: bool(r) and r["t"] - ended["t"] <= PIECE * 1000 for w, r in said_ended.items()}
        page_ok = page_end.get("stopped") == "ended" and page_end.get("said_at", 1e18) - ended["t"] <= PIECE * 1000 + 1000
        plant = plant_tabs.get("no-ended", {})
        plant_ok = plant.get("stopped") == "cannot reach"
        bar("off air", "passed" if all(within.values()) and page_ok and plant_ok else "failed",
            f"ended said within {PIECE} s: Bo {within['bo']}, Cy {within['cy']}, page {page_ok} (it said "
            f"{page_end.get('stopped')!r}); the plant no-ended: {plant.get('stopped')!r}",
            listeners_stopped=stopped, page_why=(page_end.get("air") or {}).get("why"),
            plant_why=(plant.get("air") or {}).get("why"))
    return v


def log_lines(who):
    p = W / f"{who}.log"
    return [re.sub(r"\x1b\[[0-9;]*m", "", l) for l in p.read_text(errors="replace").splitlines()] if p.exists() else []


if __name__ == "__main__":
    failed = None
    try: main()
    except BaseException as e:
        failed = f"{type(e).__name__}: {e}"
        say("harness stopped", why=failed, trace=traceback.format_exc()[-1500:])
    finally:
        for p in [n for n in procs if n != "public"] + ["public"]: stop(p)
        verdicts = judge()
        if failed:
            verdicts = {k: x if x["verdict"] != "passed" else {**x, "verdict": "could-not-judge", "why": f"{x['why']}; the run stopped: {failed}"}
                        for k, x in verdicts.items()}
            verdicts["the run"] = {"verdict": "failed", "why": failed}
        (OUT / "verdicts.json").write_text(redact(json.dumps(verdicts, indent=1, ensure_ascii=False)) + "\n")
        (OUT / "evidence.json").write_text(redact(json.dumps(ev, indent=1, ensure_ascii=False, default=str)) + "\n")
        for f in ["station.jsonl", "bo/listen.jsonl", "cy/listen.jsonl"]:
            if (W / f).exists(): shutil.copy(W / f, OUT / f.replace("/", "-"))
        for w in ("ana", "bo", "cy", "dee"):
            keep = [l for l in log_lines(w) if re.search(r"rail live|blob|/v1/sign|folder|removal|refused", l)]
            (OUT / f"{w}.trace").write_text(redact("\n".join(keep[-3000:])) + "\n")
            if (W / f"{w}.out").exists(): (OUT / f"{w}.out").write_text(redact(said(w)))
        for k, x in verdicts.items(): print(f"{x['verdict']:16} {k}: {x['why']}")
        say("end", load=round(os.getloadavg()[0], 1))
