"""What a newcomer meets (README.md, row start-measure): a friend's
llama-server on Sam's computer, lent, asked from Bo's by curl, Python's
`openai` and Node's `fetch` before and after Bo's svrngs starts again; what
of llama-server a member reads through a lend keeping back only
`/lora-adapters`, while Cy's chat runs; and Bo's home page walked in
Chromium and WebKit, state by state. Four svrngs processes on this host,
each with its own HOME and SVRNGS_DIR.

    PY=<venv with openai>/bin/python W=<scratch> python3 demo/runs/start/2026-10-13/run.py

Data in data/ beside this file: events.jsonl, summary.json, each node's log
and what each command said, llama-server's log, and shots/. scripts/check.sh
runs none of this."""
import http.client, json, os, re, shutil, signal, subprocess, sys, threading, time, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
OUT = HERE / "data"; SHOTS = OUT / "shots"; W = Path(os.environ.get("W", "/tmp/sm-w"))
PY = os.environ.get("PY", "/tmp/sm-py/bin/python")
PORT, MODEL = 18089, "Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M"
NIGHT, TASTE = 18091, 18092
LOG = "info,door=debug,origin=debug,home=debug,surface=debug,transport=warn"
procs, summary = {}, {}


def say(event, **kw):
    line = json.dumps({"t": round(time.time(), 3), "event": event, **kw})
    print(line[:600], flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def host(): return {"load": os.getloadavg(), "cores": os.cpu_count()}


def wait(cond, bound, every=0.25):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def data(who): return W / who / ".svrngs"
def env(who): return {**os.environ, "HOME": str(W / who), "USER": who, "SVRNGS_DIR": str(data(who)),
                      "NO_COLOR": "1", "RUST_LOG": LOG}
def base(who): return (data(who) / "api").read_text().strip().rstrip("/")


def api(who, path):
    try:
        with urllib.request.urlopen(base(who) + path, timeout=5) as r: return json.loads(r.read())
    except Exception: return None


def up(who, *args):
    """svrngs on `who`'s computer, running; its words to data/<who>.out."""
    data(who).mkdir(parents=True, exist_ok=True)
    (data(who) / "api").unlink(missing_ok=True)
    procs[who] = subprocess.Popen([W / "svrngs", *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(OUT / f"{who}.out", "a"), stderr=open(OUT / f"{who}.log", "a"))
    if not wait(lambda: api(who, "/v1/groups") is not None, 60): sys.exit(f"{who}'s svrngs did not come up")
    say("up", who=who, args=[a if "join/" not in a else "<the link>" for a in args], api=base(who))


def stop(who):
    p = procs.pop(who)
    p.send_signal(signal.SIGTERM)
    try: p.wait(15)
    except Exception: p.kill(); p.wait()
    say("stopped", who=who, exit=p.returncode)


def command(who, *args):
    run = subprocess.run([W / "svrngs", *args], env={**env(who), "RUST_LOG": "error"}, capture_output=True,
                         text=True, stdin=subprocess.DEVNULL, timeout=120)
    said = (run.stdout + run.stderr).strip()
    say("command", who=who, args=[a if "join/" not in a else "<the link>" for a in args], exit=run.returncode, said=said)
    return run.returncode, said


def at(origin):
    """("127.0.0.1", port, Host) for an origin on this host, as the driver asks it."""
    h = origin.split("//")[1].rstrip("/")
    return ("127.0.0.1", int(h.rsplit(":", 1)[1]), h)


def ask(to, method, path, body=None, headers=None, timeout=60):
    ip, port, h = to
    c = http.client.HTTPConnection(ip, port, timeout=timeout)
    hs = {"Content-Type": "application/json", **({"Host": h} if h else {}), **(headers or {})}
    try:
        c.request(method, path, body=json.dumps(body) if body is not None else None, headers=hs)
        r = c.getresponse(); blob = r.read()
    except Exception as e:
        return {"status": None, "error": f"{type(e).__name__}: {e}"}
    try: parsed = json.loads(blob)
    except Exception: parsed = blob.decode(errors="replace")[:300]
    return {"status": r.status, "refused": r.getheader("x-svrngs-refused"), "body": parsed}


def lends(who):
    groups = (api(who, "/v1/groups") or {}).get("groups") or [{}]
    return api(who, f"/v1/lends?namespace={groups[0].get('id')}") or {}


def origin_of(who, name):
    return next((l["origin"] for l in lends(who).get("lent", []) if l["name"] == name), None)


def label(origin): return origin.split("//")[1].split(".")[0]


def home(who): return f"http://svrngs.localhost:{base(who).rsplit(':', 1)[1]}/"


# --- the clients, as Bo's programs -----------------------------------------
CHAT = {"model": "any", "messages": [{"role": "user", "content": "Name three rivers in Spain."}],
        "max_tokens": 32, "temperature": 0, "seed": 1}


def curl(origin):
    url = origin.rstrip("/") + "/v1/chat/completions"
    out = {"origin": origin}
    for way, body in (("whole", CHAT), ("streamed", {**CHAT, "stream": True})):
        r = subprocess.run(["curl", "-sS", "-N", "--max-time", "60", "-w", "\n%{http_code} %{remote_ip}", "-H",
                            "content-type: application/json", "-d", json.dumps(body), url],
                           capture_output=True, text=True)
        tail = r.stdout.rsplit("\n", 1)[-1].split() if r.stdout else []
        out[way] = {"reached": r.returncode == 0, "exit": r.returncode, "error": r.stderr.strip()[:300] or None,
                    "status": tail[0] if tail else None, "remote_ip": tail[1] if len(tail) > 1 else None}
        if way == "streamed": out[way]["events"] = sum(1 for l in r.stdout.splitlines() if l.startswith("data:"))
    out["resolved"] = [out["whole"]["remote_ip"]] if out["whole"]["remote_ip"] else None
    return out


def clients(origin, step):
    got = {"curl": curl(origin)}
    for name, argv in (("python", [PY, str(HERE / "client.py"), origin]), ("node", ["node", str(HERE / "fetch.mjs"), origin])):
        r = subprocess.run(argv, capture_output=True, text=True, timeout=180)
        try: got[name] = json.loads(r.stdout.strip().splitlines()[-1])
        except Exception: got[name] = {"error": (r.stderr or r.stdout)[-400:]}
    for name, g in got.items(): say("client", step=step, client=name, **g)
    return got


# --- the browsers ----------------------------------------------------------
def walk(state, url, click=None):
    got = {}
    for which in ("chromium", "webkit"):
        shot = SHOTS / f"{state}-{which}.png"
        argv = ["node", str(HERE / "walk.mjs"), which, url, str(shot), *([click] if click else [])]
        r = subprocess.run(argv, capture_output=True, text=True, timeout=120)
        try: got[which] = json.loads(r.stdout.strip().splitlines()[-1])
        except Exception: got[which] = {"error": (r.stderr or r.stdout)[-400:]}
        say("walk", state=state, **{k: v for k, v in got[which].items() if k != "text"}, text=got[which].get("text", "")[:400])
    summary.setdefault("walk", {})[state] = got
    return got


def main():
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir(); SHOTS.mkdir()
    shutil.copy(REPO / "target/debug/svrngs", W / "svrngs")
    head = subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    version = subprocess.run(["llama-server", "--version"], capture_output=True, text=True)
    say("start", host=host(), head=head, llama=(version.stdout + version.stderr).strip().splitlines()[:2])
    try:
        procs["llama"] = subprocess.Popen(["llama-server", "-hf", MODEL, "--host", "127.0.0.1", "--port", str(PORT),
                                           "-c", "4096", "--parallel", "2"], stdin=subprocess.DEVNULL,
                                          stdout=open(OUT / "llama.log", "a"), stderr=subprocess.STDOUT)
        straight = ("127.0.0.1", PORT, None)
        if not wait(lambda: ask(straight, "GET", "/health", timeout=2)["status"] == 200, 900, 1):
            sys.exit("llama-server did not come up")
        for port, name in ((NIGHT, "night"), (TASTE, "taste")):
            page = W / name; page.mkdir()
            (page / "index.html").write_text(f"<!doctype html><title>{name}</title><h1>{name}</h1>")
            procs[name] = subprocess.Popen([sys.executable, "-m", "http.server", str(port), "--bind", "127.0.0.1",
                                            "--directory", str(page)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        say("services", llama=PORT, night=NIGHT, taste=TASTE)

        # The group: Sam makes it; Bo is put on as install.sh puts a
        # computer on; Cy joins.
        up("sam", "new", "Lab", "--as", "Sam")
        link = wait(lambda: re.search(r"svrngs://join/\S+", (OUT / "sam.out").read_text()), 60)[0]
        summary["bo_join"] = command("bo", "join", link, "--as", "Bo", "--no-run")
        up("bo")
        up("cy", "join", link, "--as", "Cy")
        if not wait(lambda: "You're in" in (OUT / "cy.out").read_text(), 60): sys.exit("cy did not join")
        summary["shares"] = {
            "llama": command("sam", "share", str(PORT), "--name", "llama", "--kind", "llama", "--limit", "4",
                             "--refuse", "/lora-adapters"),
            "night": command("sam", "share", str(NIGHT), "--name", "night", "--kind", "night"),
            "taste": command("cy", "share", str(TASTE), "--name", "taste", "--kind", "taste",
                             "--ask", "the films you have marked"),
        }
        names = {"llama", "night", "taste"}
        if not wait(lambda: names <= {l["name"] for l in lends("bo").get("lent", [])}, 180, 1):
            sys.exit(f"Bo's computer never listed all three: {lends('bo')}")

        # (a) The clients, at Bo's address for llama.
        llama = origin_of("bo", "llama")
        before = {"api": base("bo"), "home": home("bo"), "llama": llama, "said": (OUT / "bo.out").read_text()}
        summary["before"] = before; say("before", **before)
        summary["clients_before"] = clients(llama, "before")

        # (c) One group with lends: Bo's page, and Sam's, which lends.
        walk("lends-bo", home("bo"), click="Sam’s llama")
        walk("lends-sam", home("sam"))
        # An app asking: Sam's night asks to use Cy's taste, on Bo's page;
        # in Chromium Bo says yes, then the page is read again.
        taste, night = origin_of("bo", "taste"), origin_of("bo", "night")
        walk("asking-bo", f"{home('bo')}ask?lend={label(taste)}&app={label(night)}&back=/", click="Yes")
        walk("after-yes-bo", home("bo"))

        # (b) llama-server's routes, straight and through Bo's lend.
        lent = at(llama)
        ROUTES = [("GET", "/health", None), ("GET", "/props", None), ("POST", "/props", {}), ("GET", "/metrics", None),
                  ("GET", "/slots", None), ("POST", "/slots/0?action=save", {"filename": "x.bin"}),
                  ("GET", "/lora-adapters", None), ("POST", "/lora-adapters", []), ("GET", "/tools", None),
                  ("GET", "/cors-proxy", None), ("GET", "/v1/models", None),
                  ("POST", "/v1/streams/lookup", {"conversation_ids": ["nobody"]}),
                  ("POST", "/v1/chat/completions/control", {"id": "chatcmpl-nobody", "action": "reasoning_end"}),
                  ("POST", "/models/load", {"model": "x"})]
        summary["routes"] = []
        for method, path, body in ROUTES:
            row = {"method": method, "path": path, "straight": ask(straight, method, path, body),
                   "lent": ask(lent, method, path, body)}
            summary["routes"].append(row); say("route", **row)

        # Cy's chat through Cy's address for the lend, while Bo reads /slots,
        # then Cy's stream by its id, then stops another of Cy's by its id.
        cy_lent = at(origin_of("cy", "llama"))
        LONG = {"model": "any", "messages": [{"role": "user", "content": "Tell a long story about a river and a mill."}],
                "max_tokens": 300, "temperature": 0, "seed": 7, "stream": True}

        def cy_chat(conv, into):
            c = http.client.HTTPConnection(*cy_lent[:2], timeout=120); t0 = time.perf_counter()
            c.request("POST", "/v1/chat/completions", body=json.dumps(LONG),
                      headers={"Content-Type": "application/json", "Host": cy_lent[2], "X-Conversation-Id": conv})
            r = c.getresponse(); into["status"] = r.status; into["events"] = 0; words = []
            while True:
                line = r.readline()
                if not line: break
                if line.startswith(b"data:") and b"[DONE]" not in line:
                    into["events"] += 1
                    try: words.append(json.loads(line[5:])["choices"][0]["delta"].get("content") or "")
                    except Exception: pass
                    into.setdefault("first_s", round(time.perf_counter() - t0, 3))
            into["s"] = round(time.perf_counter() - t0, 3); into["words"] = "".join(words)

        cy1 = {}; th = threading.Thread(target=cy_chat, args=("cy-conv-1", cy1)); th.start()
        slots, t0, lookup, stream = [], time.time(), None, None
        while th.is_alive() and time.time() - t0 < 120:
            r = ask(lent, "GET", "/slots")
            slots.append({"t": round(time.time() - t0, 3), "status": r["status"], "slots": r["body"]})
            if len(slots) == 5:
                lookup = ask(lent, "POST", "/v1/streams/lookup", {"conversation_ids": ["cy-conv-1"]})
                c = http.client.HTTPConnection(*lent[:2], timeout=120)
                c.request("GET", "/v1/stream?conv_id=cy-conv-1&from=0", headers={"Host": lent[2]})
                rr = c.getresponse(); seen = rr.read().decode(errors="replace")
                stream = {"status": rr.status, "refused": rr.getheader("x-svrngs-refused"),
                          "events": seen.count("data:"), "words": "".join(
                              (json.loads(l[5:])["choices"][0]["delta"].get("content") or "")
                              for l in seen.splitlines() if l.startswith("data:") and "[DONE]" not in l and '"choices"' in l)}
            time.sleep(0.1)
        th.join()
        summary["cy_chat"] = {"cy": cy1, "bo_slots": slots, "bo_lookup": lookup, "bo_stream": stream}
        say("cy_chat", cy={k: v for k, v in cy1.items() if k != "words"}, slots_reads=len(slots),
            lookup=lookup, stream={k: v for k, v in (stream or {}).items() if k != "words"},
            same_words=bool(stream) and stream["words"] == cy1["words"])
        cy2 = {}; th = threading.Thread(target=cy_chat, args=("cy-conv-2", cy2)); th.start()
        wait(lambda: cy2.get("events", 0) >= 5, 60, 0.02)
        stop_it = ask(lent, "DELETE", "/v1/stream?conv_id=cy-conv-2")
        th.join(120)
        summary["cy_stopped"] = {"cy": cy2, "bo_delete": stop_it, "baseline_events": cy1["events"]}
        say("cy_stopped", cy={k: v for k, v in cy2.items() if k != "words"}, bo_delete=stop_it, baseline=cy1["events"])

        # (c) A computer asleep: Cy's stopped.
        stop("cy")
        wait(lambda: "Cy" in lends("bo").get("unanswered", []), 60, 1)
        summary["asleep_lends"] = lends("bo"); say("asleep", unanswered=summary["asleep_lends"].get("unanswered"))
        walk("asleep-bo", home("bo"))

        # (a) The restart, as launchd restarts it: SIGTERM, then the bare
        # command over the same data dir.
        old_home = home("bo")
        stop("bo")
        up("bo")
        wait(lambda: origin_of("bo", "llama"), 60, 1)
        after = {"api": base("bo"), "home": home("bo"), "llama": origin_of("bo", "llama"),
                 "said": (OUT / "bo.out").read_text()[len(before["said"]):]}
        summary["after"] = after; say("after", **after)
        summary["clients_old_address"] = clients(llama, "after-old")
        if after["llama"] and after["llama"] != llama: summary["clients_new_address"] = clients(after["llama"], "after-new")
        walk("restart-old-home-bo", old_home)
        walk("restart-new-home-bo", home("bo"))

        # (c) Removed: Sam removes Bo.
        summary["remove"] = command("sam", "remove", "Bo", "--group", "Lab")
        wait(lambda: (api("bo", "/v1/groups") or {"groups": [{}]})["groups"][0].get("removed") is True, 120, 1)
        summary["removed_groups"] = api("bo", "/v1/groups")
        walk("removed-bo", home("bo"))

        # (c) No group yet: Dee, the bare command over an empty data dir.
        up("dee")
        walk("no-group-dee", home("dee"))
    finally:
        for p in procs.values():
            p.terminate()
            try: p.wait(10)
            except Exception: p.kill()
        summary["end"] = host()
        (OUT / "summary.json").write_text(json.dumps(summary, indent=1))
        say("end", host=host())


if __name__ == "__main__":
    main()
