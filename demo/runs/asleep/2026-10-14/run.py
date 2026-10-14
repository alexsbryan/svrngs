#!/usr/bin/env python3
"""How long a dial to a lend's computer takes (PREDICTIONS.md here; row
events-asleep (a)): this commit's svrngs, every computer a HOME and data
dir of its own under W; the page built and served on this host; Sam's
computer lending a probe; Bo's computer asking it at his origin for it; Cy's
phone, WebKit as an iPhone (phone.mjs), opening it at its own.

    PART=working W=<scratch> python3 demo/runs/asleep/2026-10-14/run.py   # data/working/
    PART=asleep  W=<scratch> python3 demo/runs/asleep/2026-10-14/run.py   # data/asleep/

working: ROUNDS rounds, each Sam's svrngs restarted, then Bo asks once
(cold) and WARM times more, and Cy's phone opens the probe, its dial timed
from the worker's trace. asleep: Sam stopped by Ctrl-C, then frozen by
SIGSTOP, ASLEEP rounds each, Bo and Cy asking at once. The links' tokens are
never in the data. scripts/check.sh runs none of this."""
import http.client, json, os, re, shutil, signal, subprocess, sys, threading, time, urllib.parse
from datetime import datetime
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
PART = os.environ["PART"]; W = Path(os.environ["W"]); OUT = HERE / "data" / PART; BIN = REPO / "target/debug/svrngs"
ROUNDS, WARM, ASLEEP = int(os.environ.get("ROUNDS", 12)), 4, int(os.environ.get("ASLEEP", 3))
LOG = "info,door=debug,origin=debug,transport=debug,cli=debug"
WORDS = "probe here"
procs, outs, hidden, summary = {}, {}, [], {}


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


def port_of(who): return int(urllib.parse.urlsplit((W / who / ".svrngs/api").read_text().strip()).port)


def get(who, path, host=None, timeout=120):
    """(status, body, refused kind, ms) of one request to who's svrngs, as a program asks."""
    t = time.time()
    try:
        c = http.client.HTTPConnection("127.0.0.1", port_of(who), timeout=timeout)
        c.request("GET", path, headers={"Host": host} if host else {})
        r = c.getresponse()
        body = r.read().decode(errors="replace")
        return r.status, body, r.getheader("x-svrngs-refused"), ms(t)
    except Exception as e:
        return None, str(e), None, ms(t)


def api(who, path):
    try: s, b, _, _ = get(who, path, timeout=15)
    except Exception: return None
    try: return json.loads(b) if s == 200 else None
    except Exception: return None


def up(who, bound=60):
    if not wait(lambda: api(who, "/v1/groups"), bound): sys.exit(f"{who}'s svrngs did not answer: {said(who)[-400:]}")


def link(who, bound=90):
    got = wait(lambda: re.search(r"svrngs://join/(\S+)", said(who)), bound)
    if not got: sys.exit(f"{who} printed no link: {said(who)[-600:]}")
    hidden.append(got[1])
    page = re.search(r"(http://localhost:\d+/#(\S+))", said(who))
    hidden.append(page[2])
    return got[0], page[1]


phone = None


def tell(**c):
    phone.stdin.write(json.dumps(c) + "\n"); phone.stdin.flush()
    line = phone.stdout.readline()
    got = json.loads(line) if line else {"error": "phone.mjs stopped"}
    say("phone", command={k: redact(v) for k, v in c.items()}, **{k: v for k, v in got.items() if k != "text" or c["do"] != "trace"})
    return got


def dials(trace, since):
    """Each dial the worker began after `since` (unix s): ms from "dialling" to
    the connection established, or None with the line that ended it."""
    found, began = [], None
    for line in trace.splitlines():
        m = re.match(r"(\d{4}-\d\d-\d\dT[\d:.]+Z) (.*)", line)
        if not m: continue
        at = datetime.fromisoformat(m[1].replace("Z", "+00:00")).timestamp()
        if at < since: continue
        if "RUNTIME: dialling" in m[2]: began = at
        elif began and "LAYER tls/alpn+relay-hop:" in m[2] and ("ok" in m[2] or "REFUSED" in m[2]):
            found.append({"ms": round((at - began) * 1000), "ok": "established" in m[2], "line": m[2][:200]})
            began = None
    if began: found.append({"ms": None, "ok": False, "line": "dialling, never ended"})
    return found


def bo_asks(host, into):
    s, b, refused, t = get("bo", "/", host)
    into.update({"status": s, "refused": refused, "ms": t, "shown": WORDS in b, "said": b[:200] if s != 200 else None})


opens = [0]


def fresh(url):
    """`url` with a query no open asked before, so no browser cache answers for Sam's computer."""
    opens[0] += 1
    return f"{url}?open={opens[0]}"


def lend_url(phone_url): return "{0.scheme}://{0.netloc}/".format(urllib.parse.urlsplit(phone_url))


def main():
    global phone
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir(parents=True)
    say("start", part=PART, head=head(), load=round(os.getloadavg()[0], 1), cores=os.cpu_count(),
        dirty=subprocess.run(["git", "-C", REPO, "status", "--short"], capture_output=True, text=True).stdout.splitlines())
    try:
        subprocess.run(["cargo", "build", "--quiet", "-p", "svrngs"], cwd=REPO, check=True, env={**os.environ, "CARGO_INCREMENTAL": "0"})
        # n0's four relays as relay-park's run listed them (demo/runs/park/2026-10-13/README.md, 1).
        relays = [f"https://{r}.relay.n0.iroh.link/" for r in ("usw1-1", "use1-1", "euc1-1", "aps1-1")]
        n0 = {r: subprocess.run([REPO / "scripts/relay-answers.sh", r], capture_output=True).returncode == 0 for r in relays}
        summary["n0"] = n0
        say("n0", answered=n0)
        if not any(n0.values()):
            summary["verdict"] = "could-not-judge: no n0 relay answered"
            return
        subprocess.run([REPO / "scripts/build-web.sh", "--out", W / "site"], check=True, capture_output=True)
        procs["page"] = subprocess.Popen(["node", REPO / "web/tests/serve.mjs", W / "site"], stdout=subprocess.PIPE, text=True)
        PAGE = f"http://localhost:{re.search(r'port (\d+)', procs['page'].stdout.readline())[1]}/"
        threading.Thread(target=lambda: [None for _ in procs["page"].stdout], daemon=True).start()
        (W / "probe").mkdir()
        (W / "probe/index.html").write_text(f"<!doctype html><title>probe</title><p>{WORDS}</p>\n")
        procs["probe"] = subprocess.Popen([sys.executable, "-u", "-m", "http.server", "0", "--bind", "127.0.0.1", "-d", W / "probe"],
                                          stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
        probe_port = re.search(r"port (\d+)", procs["probe"].stdout.readline())[1]
        phone = subprocess.Popen(["node", HERE / "phone.mjs"], cwd=HERE, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                 stderr=open(W / "phone.err", "w"), text=True)
        if not json.loads(phone.stdout.readline()).get("ready"): sys.exit("phone.mjs did not start")

        # Sam makes the group; Bo joins by its link, Cy's phone by the page's address of it.
        start("sam", "new", "house", "--as", "Sam", "--origin", PAGE)
        house_link, house_page = link("sam")
        start("bo", "join", house_link, "--as", "Bo")
        if not wait(lambda: re.search(r"You're in", said("bo")), 90): sys.exit(f"Bo did not join: {said('bo')[-400:]}")
        summary["cy_join"] = tell(do="join", link=house_page, name="Cy")
        gid = api("sam", "/v1/groups")["groups"][0]["id"]
        share = subprocess.run([BIN, "share", probe_port, "--kind", "probe", "--name", "probe", "--upload", "100"],
                               env={**env("sam"), "RUST_LOG": "error"}, capture_output=True, text=True)
        say("share", exit=share.returncode, said=(share.stdout + share.stderr).strip()[:300])
        lent = lambda: {(l["person"], l["name"]): l["origin"] for l in (api("bo", f"/v1/lends?namespace={gid}") or {}).get("lent", [])}
        if not wait(lambda: ("Sam", "probe") in lent(), 120, 0.5): sys.exit("Bo lists no probe")
        host = urllib.parse.urlsplit(lent()[("Sam", "probe")]).netloc
        label = host.split(".")[0]
        first = tell(do="open", url=f"{PAGE}#open={label}&at=%2F", words=WORDS)
        at = lend_url(first.get("url", ""))
        summary["setup"] = {"label": label, "phone_first": first, "bo_first": get("bo", "/", host)[0]}

        if PART == "working":
            rounds = []
            for i in range(ROUNDS):
                stop("sam"); start("sam"); up("sam")
                since = time.time()
                r = {"round": i + 1, "bo": []}
                for _ in range(1 + WARM):
                    got = {}; bo_asks(host, got); r["bo"].append(got)
                r["cy"] = tell(do="open", url=fresh(at), words=WORDS)
                r["cy"]["dials"] = dials(tell(do="trace", url=at + "svrngs-trace").get("text", ""), since)
                say("round", **r)
                rounds.append(r)
            summary["rounds"] = rounds
        else:
            rounds = []
            for mode in ("stopped", "frozen"):
                for i in range(ASLEEP):
                    awake = {}; bo_asks(host, awake)
                    ready = tell(do="open", url=fresh(at), words=WORDS)
                    if awake.get("status") != 200 or not ready.get("shown"): sys.exit(f"Sam's probe not answering before asleep: {awake} {ready}")
                    if mode == "stopped": stop("sam")
                    else: procs["sam"].send_signal(signal.SIGSTOP); say("frozen", who="sam")
                    t = time.time(); bo = {}
                    asking = threading.Thread(target=bo_asks, args=(host, bo)); asking.start()
                    cy = tell(do="open", url=fresh(at), words=WORDS)
                    asking.join()
                    r = {"mode": mode, "round": i + 1, "bo": bo, "cy": cy, "both_s": round(time.time() - t, 1)}
                    cy["dials"] = dials(tell(do="trace", url=at + "svrngs-trace").get("text", ""), t)
                    if mode == "stopped": start("sam")
                    else: procs["sam"].send_signal(signal.SIGCONT); say("woke", who="sam")
                    up("sam")
                    r["woke_ms"] = ms(t)
                    if not wait(lambda: get("bo", "/", host)[0] == 200, 90, 1): sys.exit("Sam's probe did not answer Bo again")
                    say("round", **r)
                    rounds.append(r)
            summary["rounds"] = rounds
    finally:
        if phone:
            try: phone.stdin.close(); phone.wait(30)
            except Exception: phone.kill()
        for name, p in list(procs.items()):
            if name == "sam": p.send_signal(signal.SIGCONT)
            p.send_signal(signal.SIGINT if name in ("sam", "bo") else signal.SIGTERM)
        for p in procs.values():
            try: p.wait(20)
            except Exception: p.kill()
        summary["end_load"] = round(os.getloadavg()[0], 1)
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, ensure_ascii=False)) + "\n")
        if (W / "phone.err").exists(): (OUT / "phone.err").write_text(redact((W / "phone.err").read_text()))
        for who in ("sam", "bo"):
            if (W / f"{who}.log").exists():
                keep = [re.sub(r"\x1b\[[0-9;]*m", "", l) for l in (W / f"{who}.log").read_text(errors="replace").splitlines()
                        if re.search(r"door: (let in|guessed|refused)|origin: |iroh bridge|dial|ERROR|WARN", l)]
                (OUT / f"{who}.log").write_text(redact("\n".join(keep[-4000:])) + "\n")
        say("end")


if __name__ == "__main__":
    main()
