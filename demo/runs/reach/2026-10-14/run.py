#!/usr/bin/env python3
"""How long a phone's page takes to reach the computer that made its link
(PREDICTIONS.md here; row v1ui-reach (a)): this commit's svrngs, every
computer a HOME and data dir of its own under W; the page built and served
on this host; Sam's computer making the group; Bo's phone, WebKit as an
iPhone (phone.mjs), joined by the page's address of its link, then opening
the group's page again and again.

    W=<scratch> python3 demo/runs/reach/2026-10-14/run.py   # data/

Two paths, each a group made anew: `loopback`, iroh's relay on 127.0.0.1
(crates/svrngs/tests/n0_off.rs `the_relay`), and `n0`, n0's relays. Each:
WARM opens with Sam's svrngs running, COLD opens each the moment Sam's
svrngs, stopped and started again, answers, and ASLEEP opens with it
stopped. The links' tokens are never in the data. scripts/check.sh runs
none of this."""
import http.client, json, os, re, shutil, signal, statistics, subprocess, sys, threading, time, urllib.parse
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
W = Path(os.environ["W"]); OUT = HERE / "data"; BIN = REPO / "target/debug/svrngs"
WARM, COLD, ASLEEP = int(os.environ.get("WARM", 20)), int(os.environ.get("COLD", 5)), int(os.environ.get("ASLEEP", 2))
LOG = "info,cli=debug,transport=debug"
procs, outs, hidden, summary = {}, {}, [], {}


def head(): return subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()


def redact(text):
    for h in hidden: text = text.replace(h, "<the token>")
    return text


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}, ensure_ascii=False))
    print(line[:300], flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def wait(cond, bound, every=0.1):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def env(who):
    return {**os.environ, "HOME": str(W / who), "USER": who, "SVRNGS_DIR": str(W / who / ".svrngs"),
            "NO_COLOR": "1", "RUST_LOG": LOG}


def start(who, *args):
    (W / who).mkdir(parents=True, exist_ok=True)
    outs[who] = W / f"{who}.{len(list(W.glob(f'{who}.*.out'))) + 1}.out"
    procs[who] = subprocess.Popen([BIN, *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(outs[who], "w"), stderr=open(W / f"{who}.log", "a"))
    say("svrngs", who=who, args=[a if not a.startswith(("svrngs://", "http")) else "<link>" for a in args])


def stop(who):
    p = procs.pop(who)
    p.send_signal(signal.SIGINT)
    try: p.wait(20)
    except subprocess.TimeoutExpired: p.kill(); p.wait()
    say("stopped", who=who, exit=p.returncode)


def said(who): return outs[who].read_text() if who in outs and outs[who].exists() else ""


def answers(who):
    """Whether who's svrngs answers on its loopback API now."""
    try:
        port = urllib.parse.urlsplit((W / who / ".svrngs/api").read_text().strip()).port
        c = http.client.HTTPConnection("127.0.0.1", port, timeout=2)
        c.request("GET", "/v1/groups")
        return c.getresponse().status == 200
    except Exception:
        return False


def link(who, bound=90):
    got = wait(lambda: re.search(r"svrngs://join/(\S+)", said(who)), bound)
    if not got: sys.exit(f"{who} printed no link: {said(who)[-600:]}")
    hidden.append(got[1])
    page = re.search(r"(http://localhost:\d+/#(\S+))", said(who))
    hidden.append(page[2])
    return page[1]


phone = None


def tell(**c):
    phone.stdin.write(json.dumps(c) + "\n"); phone.stdin.flush()
    line = phone.stdout.readline()
    return json.loads(line) if line else {"error": "phone.mjs stopped"}


def spans(lines):
    """From the page's stamped lines: the members' dial, from "dialing" to
    established or refused; the bind, from the wasm's entry to the relay's
    handshake done. Each ms, or None."""
    at = lambda pat: next((t for t, l in lines if re.search(pat, l)), None)
    dialing = at(r"LAYER tls/alpn\+relay-hop: dialing .* on the Member protocol")
    done = at(r"LAYER tls/alpn\+relay-hop: (ok|REFUSED)")
    entered, online = at(r"LAYER page: wasm module loaded"), at(r"LAYER relay-connect: ok")
    return {"dial_ms": done - dialing if dialing is not None and done is not None else None,
            "dialled": bool(at(r"LAYER tls/alpn\+relay-hop: ok")),
            "bind_ms": online - entered if entered is not None and online is not None else None}


def opened(path, kind, i, url):
    got = tell(do="open", url=url)
    r = {"path": path, "kind": kind, "try": i + 1, "ms": got.get("ms"), "status": got.get("status"),
         "good": got.get("good"), "error": got.get("error"), **spans(got.get("lines") or [])}
    if kind == "asleep" or not got.get("good"): r["said"] = got.get("said"); r["lines"] = [l for _, l in got.get("lines") or []][-12:]
    say("open", **r)
    return r


def one_path(path, page, relay):
    """Sam makes a group met at `relay` (n0's when None); Bo's phone joins it,
    then opens it WARM, COLD and ASLEEP times."""
    shutil.rmtree(W / "sam", ignore_errors=True); procs.pop("sam", None)
    tell(do="fresh")
    start("sam", "new", "house", "--as", "Sam", "--origin", page, *(["--relay", relay] if relay else []))
    joined = tell(do="join", link=link("sam"), name="Bo")
    say("joined", path=path, ms=joined.get("ms"), status=joined.get("status"), good=joined.get("good"), error=joined.get("error"))
    if not joined.get("good"): sys.exit(f"Bo's phone did not join on {path}: {joined}")
    # The relay the link names, for the README: the n0 one Sam's computer reached.
    via = [l for _, l in joined.get("lines") or [] if "LAYER parse: ok — relay" in l][:1]
    rows = [opened(path, "warm", i, page) for i in range(WARM)]
    for i in range(COLD):
        stop("sam"); start("sam")
        if not wait(lambda: answers("sam"), 60, 0.02): sys.exit(f"Sam's svrngs did not answer on {path}")
        rows.append(opened(path, "cold", i, page))
    stop("sam")
    rows += [opened(path, "asleep", i, page) for i in range(ASLEEP)]
    start("sam")
    if not wait(lambda: answers("sam"), 60): sys.exit(f"Sam's svrngs did not answer on {path}")
    stop("sam")
    return {"relay": via, "joined_ms": joined.get("ms"), "opens": rows}


def spread(rows, kind):
    xs = sorted(r["dial_ms"] for r in rows if r["kind"] == kind and r["good"] and r["dial_ms"] is not None)
    whole = sorted(r["ms"] for r in rows if r["kind"] == kind and r["ms"] is not None)
    return {"in": sum(1 for r in rows if r["kind"] == kind and r["good"]), "of": sum(1 for r in rows if r["kind"] == kind),
            "dial_ms": {"min": xs[0], "median": statistics.median(xs), "max": xs[-1]} if xs else None,
            "open_ms": {"min": whole[0], "median": statistics.median(whole), "max": whole[-1]} if whole else None}


def main():
    global phone
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir(parents=True)
    say("start", head=head(), load=round(os.getloadavg()[0], 1), cores=os.cpu_count(),
        dirty=subprocess.run(["git", "-C", REPO, "status", "--short"], capture_output=True, text=True).stdout.splitlines())
    try:
        e = {**os.environ, "CARGO_INCREMENTAL": "0"}
        subprocess.run(["cargo", "build", "--quiet", "-p", "svrngs"], cwd=REPO, check=True, env=e)
        built = subprocess.run(["cargo", "test", "-p", "svrngs", "--test", "n0_off", "--no-run", "--message-format=json"],
                               cwd=REPO, check=True, env=e, capture_output=True, text=True)
        relay_bin = next(json.loads(l)["executable"] for l in built.stdout.splitlines()
                         if '"executable"' in l and "n0_off" in (json.loads(l).get("executable") or ""))
        relays = [f"https://{r}.relay.n0.iroh.link/" for r in ("usw1-1", "use1-1", "euc1-1", "aps1-1")]
        n0 = {r: subprocess.run([REPO / "scripts/relay-answers.sh", r], capture_output=True).returncode == 0 for r in relays}
        summary["n0"] = n0
        say("n0", answered=n0)
        subprocess.run([REPO / "scripts/build-web.sh", "--out", W / "site"], check=True, capture_output=True)
        procs["page"] = subprocess.Popen(["node", REPO / "web/tests/serve.mjs", W / "site"], stdout=subprocess.PIPE, text=True)
        page = f"http://localhost:{re.search(r'port (\d+)', procs['page'].stdout.readline())[1]}/"
        threading.Thread(target=lambda: [None for _ in procs["page"].stdout], daemon=True).start()
        at = W / "relay-at"
        procs["relay"] = subprocess.Popen([relay_bin, "--ignored", "--exact", "the_relay", "--nocapture"],
                                          env={**os.environ, "N0OFF_RELAY_AT": str(at), "RUST_LOG": "warn"},
                                          stdout=open(W / "relay.log", "w"), stderr=subprocess.STDOUT)
        if not wait(lambda: at.exists() and at.read_text().strip(), 30): sys.exit("the relay on 127.0.0.1 did not start")
        loop = at.read_text().strip()
        say("served", page=page, relay=loop)
        phone = subprocess.Popen(["node", HERE / "phone.mjs"], cwd=HERE, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                 stderr=open(W / "phone.err", "w"), text=True)
        if not json.loads(phone.stdout.readline()).get("ready"): sys.exit("phone.mjs did not start")
        paths = {"loopback": one_path("loopback", page, loop)}
        if any(n0.values()): paths["n0"] = one_path("n0", page, None)
        else: summary["n0_verdict"] = "could-not-judge: no n0 relay answered"
        summary["paths"] = {p: {"relay": v["relay"], "joined_ms": v["joined_ms"],
                                **{k: spread(v["opens"], k) for k in ("warm", "cold", "asleep")}} for p, v in paths.items()}
        working = [r["dial_ms"] for v in paths.values() for r in v["opens"] if r["kind"] != "asleep" and r["good"] and r["dial_ms"] is not None]
        summary["slowest_working_dial_ms"] = max(working) if working else None
    finally:
        if phone:
            try: phone.stdin.close(); phone.wait(30)
            except Exception: phone.kill()
        for p in procs.values(): p.send_signal(signal.SIGINT)
        for p in procs.values():
            try: p.wait(20)
            except Exception: p.kill()
        summary["end_load"] = round(os.getloadavg()[0], 1)
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, ensure_ascii=False)) + "\n")
        if (W / "phone.err").exists(): (OUT / "phone.err").write_text(redact((W / "phone.err").read_text()))
        say("end")


if __name__ == "__main__":
    main()
