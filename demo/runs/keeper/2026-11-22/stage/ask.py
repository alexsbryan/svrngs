#!/usr/bin/python3
"""Alex's request for Sam's coder, from Alex's account, as a program does:
the lend's address from Alex's svrngs (`svrngs ask /v1/lends`), then
GET /health there, every 0.5 s, until llama-server answers 200 through the
lend, then one chat completion. Each change of what Alex sees is a line,
with the VM's uptime. `--boot` is the boot unit's run, its file named by the
boot's number; `--out <file>` is a run the driver starts (kill -9, unshare).
`--bound <s>` is how long it asks before it says nothing answered.
`--play` is Alex's Play on coder, once: a program here reaches a friend's
lend at its address only while Play is on for it (svrngs-node play.rs
`shown`), and Play stays on across restarts (play.rs PLAY_FILE)."""
import http.client, json, subprocess, sys, time

args = sys.argv[1:]
bound = float(args[args.index("--bound") + 1]) if "--bound" in args else 300
if "--boot" in args:
    with open("/home/alex/asks/boots", "a+") as f:
        f.seek(0); n = len(f.read().split()) + 1; f.write(f"{n}\n")
    out = f"/home/alex/asks/boot-{n}.jsonl"
else:
    out = args[args.index("--out") + 1]


def up(): return float(open("/proc/uptime").read().split()[0])


def say(what, **kw):
    with open(out, "a") as f: f.write(json.dumps({"uptime": up(), "t": time.time(), "what": what, **kw}) + "\n")


def svrngs(route, body=None):
    r = subprocess.run(["/usr/local/bin/svrngs", "ask", route, *([json.dumps(body)] if body else [])],
                       capture_output=True, text=True, timeout=10)
    return json.loads(r.stdout) if r.returncode == 0 else ("not-running" if r.returncode == 3 else f"exit {r.returncode}: {r.stderr.strip()[:160]}")


def get(origin, method, path, body=None):
    host = origin.split("//")[1].rstrip("/")
    c = http.client.HTTPConnection("127.0.0.1", int(host.rsplit(":", 1)[1]), timeout=60)
    t0 = time.perf_counter()
    c.request(method, path, body=json.dumps(body) if body else None, headers={"Host": host, "Content-Type": "application/json"})
    r = c.getresponse(); text = r.read().decode(errors="replace")
    return r.status, round((time.perf_counter() - t0) * 1000, 1), text


def coder():
    groups = svrngs("/v1/groups")
    if not isinstance(groups, dict): return None, {"state": "alex's svrngs", "said": groups}
    lends = svrngs(f"/v1/lends?namespace={groups['groups'][0]['id']}")
    lent = lends.get("lent", []) if isinstance(lends, dict) else []
    found = next((l for l in lent if l.get("name") == "coder"), None)
    return found, None if found else {"state": "no coder listed", "lent": [l.get("name") for l in lent] if lent else lends}


if "--play" in args:
    for _ in range(int(bound * 2)):
        found, seen = coder()
        if found: break
        time.sleep(0.5)
    said = svrngs("/v1/play", {"on": {"key": found["key"], "name": "coder"}}) if found else seen
    say("play", said=said); sys.exit(0 if found else 1)

sessions = subprocess.run(["loginctl", "list-sessions", "--no-legend"], capture_output=True, text=True).stdout
boot_id = open("/proc/sys/kernel/random/boot_id").read().strip()
say("start", boot_id=boot_id, sessions=sessions.strip().splitlines(), bound=bound)
last, t0 = None, time.time()
while time.time() - t0 < bound:
    lend, seen = coder()
    if lend:
        try:
            status, ms, text = get(lend["origin"], "GET", "/health")
            seen = {"state": f"health {status}", "said": text[:200]}
            if status == 200:
                say("answered", ms=ms, said=text[:200])
                s, ms, text = get(lend["origin"], "POST", "/v1/chat/completions",
                                  {"messages": [{"role": "user", "content": "Say hi."}], "max_tokens": 8, "temperature": 0})
                words = json.loads(text)["choices"][0]["message"]["content"] if s == 200 else text[:200]
                say("chat", status=s, ms=ms, said=words)
                break
        except Exception as e:
            seen = {"state": "no answer", "said": f"{type(e).__name__}: {e}"[:200]}
    if seen != last: say("sees", **seen); last = seen
    time.sleep(0.5)
else:
    say("nothing answered", last=last)
say("end")
