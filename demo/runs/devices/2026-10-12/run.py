#!/usr/bin/env python3
"""One person, many devices (PREDICTIONS.md here; row bash-devices (e)): Sam
makes a group and lends Jellyfin through its middleware, Navidrome and a
shelf that echoes who it was told; Ana joins by Sam's link and makes a link
for her devices; her laptop joins by it with no name, her phone (phone.mjs)
opens it; Bo joins by Sam's link. Each of Ana's devices asks each lend; Bo
asks the shelf, lent to one person at once; Sam removes Ana. This commit's
build and the page built and served here; every computer a data dir and
HOME of its own under W.

    W=<scratch> python3 demo/runs/devices/2026-10-12/run.py

Data in data/ beside this file, the links' tokens never in it.
scripts/check.sh runs none of this."""
import http.server as httpserver, json, os, re, shutil, signal, subprocess, sys, threading, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
OUT = HERE / "data"; W = Path(os.environ["W"]); BIN = REPO / "target/debug/svrngs"
JF_APP = Path("/Applications/Jellyfin.app/Contents"); JF = "http://127.0.0.1:8096"; ND = "http://127.0.0.1:4533"
GROUP = "devices"; LOG = "info,door=debug,removal=debug,redeem=debug,cli=debug"
procs, secret, summary, hidden = {}, {}, {}, []


def redact(text):
    for h in hidden: text = text.replace(h, "<the token>")
    return text


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}))
    print(line, flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def wait(cond, bound, every=0.2):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def http(method, url, body=None, headers=None, host=None, timeout=10):
    """(status, body as JSON or text, headers) of one request, or (None, why, {})."""
    h = {"content-type": "application/json", **(headers or {}), **({"Host": host} if host else {})}
    req = urllib.request.Request(url, method=method, headers=h, data=None if body is None else json.dumps(body).encode())
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r: status, blob, hs = r.status, r.read(), dict(r.headers)
    except urllib.error.HTTPError as e: status, blob, hs = e.code, e.read(), dict(e.headers)
    except Exception as e: return None, str(e), {}
    try: return status, json.loads(blob), hs
    except Exception: return status, blob.decode(errors="replace"), hs


# --- each computer: a data dir and HOME of its own --------------------------
def env(who, user=None):
    return {**os.environ, "HOME": str(W / who), "USER": user or who, "SVRNGS_DIR": str(W / who / ".svrngs"),
            "NO_COLOR": "1", "RUST_LOG": LOG, "CARGO_INCREMENTAL": "0"}


def api(who, path):
    try: base = (W / who / ".svrngs/api").read_text().strip().rstrip("/")
    except Exception: return None
    s, b, _ = http("GET", base + path)
    return b if s == 200 else None


def command(who, *args, stdin="\n", user=None):
    t = time.time()
    run = subprocess.run([BIN, *args], env={**env(who, user), "RUST_LOG": "error"}, input=stdin, capture_output=True, text=True)
    out = {"who": who, "args": list(args), "exit": run.returncode, "said": (run.stdout + run.stderr).strip(),
           "ms": round((time.time() - t) * 1000)}
    say("command", **out)
    return out


def start(who, *args, user=None):
    """A svrngs that runs on, as `new`, `link` and `join` without --no-run do."""
    (W / who).mkdir(parents=True, exist_ok=True)
    procs[who] = subprocess.Popen([BIN, *args], env=env(who, user), stdin=subprocess.DEVNULL,
                                  stdout=open(W / f"{who}.out", "w"), stderr=open(W / f"{who}.log", "w"))
    return time.time()


def said(who): return (W / f"{who}.out").read_text() if (W / f"{who}.out").exists() else ""


def links(who, bound=90):
    got = wait(lambda: re.search(r"(svrngs://join/(\S+))", said(who)) and re.search(r"(http://localhost:\d+/#(\S+))", said(who)), bound)
    if not got: sys.exit(f"{who} printed no link: {said(who)[-400:]}")
    client = re.search(r"(svrngs://join/(\S+))", said(who)); https = re.search(r"(http://localhost:\d+/#(\S+))", said(who))
    hidden.extend([client[2], https[2]])
    return client[1], https[1]


def lends(who):
    return {l["name"]: l["origin"] for l in (api(who, f"/v1/lends?namespace={secret['group']}") or {}).get("lent", [])
            if l.get("person") == "Sam"}


def ask(who, lend, path, origin=None):
    origin = origin or lends(who).get(lend)
    if not origin: return {"status": None, "said": f"{who} lists no {lend}"}
    host = origin.split("//")[1].rstrip("/")
    s, b, hs = http("GET", f"http://127.0.0.1:{host.rsplit(':', 1)[1]}{path}", host=host, timeout=20)
    return {"status": s, "refused": hs.get("x-svrngs-refused"), "body": b if isinstance(b, dict) else str(b)[:300]}


def people():
    rows = (api("sam", f"/v1/rail/people?namespace={secret['group']}") or {}).get("people", [])
    return sorted((r["person"], r["key"], r.get("person_key")) for r in rows)


# --- Sam's lends: Jellyfin and its middleware, Navidrome, the shelf ---------
JF_AUTH = 'MediaBrowser Client="devices", Device="Sam computer", DeviceId="devices-setup", Version="1"'
def jf(method, path, body=None):
    tok = secret.get("jf")
    return http(method, JF + path, body, {"Authorization": JF_AUTH + (f', Token="{tok}"' if tok else "")})


def jf_users(): return sorted(u["Name"] for u in jf("GET", "/Users")[1])


def jellyfin():
    d = W / "jf"
    for sub in ("data", "config", "cache", "log"): (d / sub).mkdir(parents=True, exist_ok=True)
    (d / "config/network.xml").write_text(
        '<?xml version="1.0" encoding="utf-8"?>\n<NetworkConfiguration xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" '
        'xmlns:xsd="http://www.w3.org/2001/XMLSchema">\n  <EnableIPv6>false</EnableIPv6>\n  <LocalNetworkAddresses>'
        '<string>127.0.0.1</string></LocalNetworkAddresses>\n  <EnableRemoteAccess>false</EnableRemoteAccess>\n</NetworkConfiguration>\n')
    procs["jellyfin"] = subprocess.Popen(
        [JF_APP / "MacOS/jellyfin", "-d", d / "data", "-c", d / "config", "-C", d / "cache", "-l", d / "log",
         "-w", JF_APP / "Resources/jellyfin-web", "--ffmpeg", JF_APP / "MacOS/ffmpeg", "--nonetchange"],
        stdout=open(d / "stdout.log", "w"), stderr=subprocess.STDOUT)
    seen = [0]
    def ready():
        s, b, _ = http("GET", JF + "/System/Info/Public", timeout=2)
        seen[0] = seen[0] + 1 if s == 200 and "StartupWizardCompleted" in json.dumps(b) else 0
        return seen[0] >= 2 and b
    info = wait(ready, 120, 1)
    if not info: sys.exit("Jellyfin did not come up")
    secret["sam_pw"] = os.urandom(12).hex()
    jf("POST", "/Startup/Configuration", {"UICulture": "en-US", "MetadataCountryCode": "US", "PreferredMetadataLanguage": "en"})
    jf("GET", "/Startup/User")
    jf("POST", "/Startup/User", {"Name": "sam", "Password": secret["sam_pw"]})
    jf("POST", "/Startup/RemoteAccess", {"EnableRemoteAccess": False, "EnableAutomaticPortMapping": False})
    jf("POST", "/Startup/Complete"); time.sleep(3)
    secret["jf"] = jf("POST", "/Users/AuthenticateByName", {"Username": "sam", "Pw": secret["sam_pw"]})[1]["AccessToken"]
    jf("POST", "/Auth/Keys?app=svrngs-mw")
    key = next(k["AccessToken"] for k in jf("GET", "/Auth/Keys")[1]["Items"] if k["AppName"] == "svrngs-mw")
    subprocess.run(["cargo", "build", "--quiet", "--manifest-path", REPO / "apps/jellyfin/mw/Cargo.toml"], check=True,
                   env={**os.environ, "CARGO_INCREMENTAL": "0"})
    procs["mw"] = subprocess.Popen([REPO / "apps/jellyfin/mw/target/debug/jellyfin-mw", "--listen", "127.0.0.1:8097"],
                                   env={**os.environ, "NO_COLOR": "1", "RUST_LOG": "info"}, stdin=subprocess.PIPE,
                                   stdout=open(W / "mw.out", "w"), stderr=open(W / "mw.log", "w"), text=True)
    procs["mw"].stdin.write(key + "\n"); procs["mw"].stdin.close()
    if not wait(lambda: "listening on port" in (W / "mw.out").read_text(), 30): sys.exit("the middleware did not listen")
    return {"version": info.get("Version"), "users_before": jf_users()}


def nd_users():
    tok = http("POST", ND + "/auth/login", {"username": "sam", "password": secret["nd"]})[1]["token"]
    s, b, _ = http("GET", ND + "/api/user", headers={"x-nd-authorization": f"Bearer {tok}"})
    return sorted(u["userName"] for u in b) if s == 200 else {"status": s, "said": b}


def navidrome():
    d = W / "nd"; (d / "music").mkdir(parents=True, exist_ok=True)
    procs["navidrome"] = subprocess.Popen(["navidrome"], env={
        **os.environ, "ND_MUSICFOLDER": str(d / "music"), "ND_DATAFOLDER": str(d), "ND_ADDRESS": "127.0.0.1", "ND_PORT": "4533",
        "ND_EXTAUTH_USERHEADER": "X-Svrngs-User", "ND_EXTAUTH_TRUSTEDSOURCES": "127.0.0.1/32", "ND_LOGLEVEL": "info"},
        stdout=open(d / "nd.log", "w"), stderr=subprocess.STDOUT)
    if not wait(lambda: http("GET", ND + "/ping", timeout=2)[0] == 200, 60, 1): sys.exit("Navidrome did not come up")
    secret["nd"] = os.urandom(12).hex()
    http("POST", ND + "/auth/createAdmin", {"username": "sam", "password": secret["nd"]})
    version = subprocess.run(["navidrome", "--version"], capture_output=True, text=True).stdout.strip()
    return {"version": version, "users_before": nd_users()}


class Shelf(httpserver.BaseHTTPRequestHandler):
    """Answers every request with who the door told it was asking."""
    def do_GET(self):
        body = json.dumps({h: self.headers.get(h) for h in ("x-svrngs-key", "x-svrngs-user", "x-svrngs-person")}).encode()
        self.send_response(200); self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body))); self.end_headers(); self.wfile.write(body)
    def log_message(self, *a): pass


def main():
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir()
    phone = None
    try:
        say("start", load=round(os.getloadavg()[0], 1), commit=subprocess.run(["git", "-C", REPO, "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True).stdout.strip())
        subprocess.run(["cargo", "build", "--quiet", "-p", "svrngs"], cwd=REPO, check=True, env={**os.environ, "CARGO_INCREMENTAL": "0"})
        subprocess.run([REPO / "scripts/build-web.sh", "--out", W / "site"], check=True, capture_output=True)
        procs["page"] = subprocess.Popen(["node", REPO / "web/tests/serve.mjs", W / "site"], stdout=subprocess.PIPE, text=True)
        port = re.search(r"port (\d+)", procs["page"].stdout.readline())[1]
        threading.Thread(target=lambda: [None for _ in procs["page"].stdout], daemon=True).start()
        page = f"http://localhost:{port}/"
        summary["apps"] = {"jellyfin": jellyfin(), "navidrome": navidrome()}
        shelf = httpserver.ThreadingHTTPServer(("127.0.0.1", 0), Shelf)
        threading.Thread(target=shelf.serve_forever, daemon=True).start()

        start("sam", "new", GROUP, "--as", "Sam", "--origin", page)
        sams, _ = links("sam")
        secret["group"] = wait(lambda: ((api("sam", "/v1/groups") or {}).get("groups") or [{}])[0].get("id"), 60)
        summary["sam_shares"] = [command("sam", "share", "8096", "--sign-in", "8097"), command("sam", "share", "4533"),
                                 command("sam", "share", str(shelf.server_port), "--name", "shelf", "--kind", "shelf", "--limit", "1")]

        # 1. Ana, then a link for her devices; her laptop and her phone by it.
        sam_https = re.search(r"http://localhost:\d+/#\S+", said("sam"))[0]
        summary["ana_join"] = command("ana", "join", sam_https, "--as", "Ana", "--no-run")
        start("ana", "link", GROUP, "--device")
        ana_client, ana_https = links("ana")
        summary["ana_device_link_said"] = redact(said("ana").strip())
        t0 = start("laptop", "join", ana_client, user="analaptop")
        line = wait(lambda: re.search(r"You're in .*|svrngs: .*", said("laptop") + (W / "laptop.log").read_text()), 60)
        summary["laptop_join"] = {"said": line and line[0], "ms": round((time.time() - t0) * 1000), "user": "analaptop", "as_given": None}
        start("bo", "join", sams, "--as", "Bo")
        wait(lambda: "You're in" in said("bo"), 60)
        phone = subprocess.Popen(["node", HERE / "phone.mjs"], cwd=HERE, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                 stderr=open(W / "phone.err", "w"), text=True)
        if not json.loads(phone.stdout.readline()).get("ready"): sys.exit("the phone did not start")
        def tell(**c):
            phone.stdin.write(json.dumps(c) + "\n"); phone.stdin.flush()
            got = json.loads(phone.stdout.readline())
            say("phone", command=c, **{k: v for k, v in got.items() if k != "logs"})
            return got
        summary["phone_join"] = tell(**{"do": "join", "link": ana_https})
        summary["people_after_joins"] = wait(lambda: (lambda p: len(p) >= 5 and p)(people()), 60)

        # 4. The shelf, one person at once: Ana's three, then Bo.
        for who in ("ana", "laptop", "bo"):
            if not wait(lambda: {"jellyfin", "navidrome", "shelf"} <= set(lends(who)), 120, 1):
                say("unlisted", who=who, lends=list(lends(who)))
        shelf_t0 = time.time()
        summary["shelf"] = {"ana": ask("ana", "shelf", "/"), "laptop": ask("laptop", "shelf", "/"),
                            "phone": tell(**{"do": "open", "lend": "Sam’s shelf", "want": "x-svrngs-key"}),
                            "bo": ask("bo", "shelf", "/")}
        summary["shelf"]["s_from_first_to_bo"] = round(time.time() - shelf_t0, 1)

        # 2, 3. Jellyfin and Navidrome from each of Ana's devices.
        summary["jellyfin"] = {"ana": ask("ana", "jellyfin", "/System/Info/Public"),
                               "laptop": ask("laptop", "jellyfin", "/System/Info/Public"),
                               "phone": tell(**{"do": "open", "lend": "Sam’s jellyfin", "dwell": 5000})}
        summary["navidrome"] = {"ana": ask("ana", "navidrome", "/app/"), "laptop": ask("laptop", "navidrome", "/app/"),
                                "phone": tell(**{"do": "open", "lend": "Sam’s navidrome", "dwell": 3000})}
        for k in ("ana", "laptop"): summary["navidrome"][k]["body"] = str(summary["navidrome"][k]["body"])[:120]
        asks = re.findall(r"the door asks who a person is to Jellyfin.*?key=(\w+)", (W / "mw.log").read_text())
        summary["accounts"] = {"jellyfin": jf_users(), "navidrome": nd_users(), "sign_in_asks": asks}

        # Sam's door's word on each request for the shelf: counted, or guessed home.
        summary["shelf"]["door"] = [re.sub(r"\x1b\[[0-9;]*m", "", l)[:400] for l in (W / "sam.log").read_text().splitlines()
                                    if "shelf" in l and ("let in, and told" in l or "guessed in this computer" in l
                                                         or "as many people as the lend" in l)]

        # 5. Sam removes Ana; each of her computers asks the shelf's origin it
        # had, as a page there would.
        kept = {who: lends(who).get("shelf") for who in ("ana", "laptop")}
        r0 = time.time()
        summary["remove"] = command("sam", "remove", "Ana", "--group", GROUP)
        gone = {}
        for who in ("ana", "laptop"):
            got = wait(lambda: (lambda a: a if a["status"] == 403 else None)(ask(who, "shelf", "/", kept[who])), 30, 0.5)
            gone[who] = {"answer": got or ask(who, "shelf", "/", kept[who]), "s": round(time.time() - r0, 2)}
        gone["phone"] = tell(**{"do": "page", "url": page})
        gone["phone"]["s"] = round(time.time() - r0, 2)
        summary["removed"] = gone
        summary["people_after_remove"] = people()
    finally:
        if phone:
            try: phone.stdin.close(); phone.wait(20)
            except Exception: phone.kill()
        for p in procs.values(): p.send_signal(signal.SIGTERM)
        for p in procs.values():
            try: p.wait(15)
            except Exception: p.kill()
        summary["end_load"] = round(os.getloadavg()[0], 1)
        text = redact(json.dumps(summary, indent=1, ensure_ascii=False))
        (OUT / "summary.json").write_text(text + "\n")
        for who in ("sam", "ana", "laptop", "bo"):
            if (W / f"{who}.out").exists(): (OUT / f"{who}.out").write_text(redact(said(who)))
        if (W / "mw.log").exists(): (OUT / "mw.log").write_text((W / "mw.log").read_text())
        say("end")


if __name__ == "__main__":
    main()
