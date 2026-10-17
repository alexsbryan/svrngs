#!/usr/bin/env python3
"""Two of one person's devices at once (PREDICTIONS.md here; row
bash-two-at-once (a)): Sam makes a group and lends Jellyfin through its
middleware, Demo A's test films its library; Ana joins by Sam's link and
makes a link for her devices; her laptop joins by it with no name; then
two.mjs plays a film on the laptop and another on her phone, which opens
the same link, at once. This commit's build and the page built and served
here; every computer a data dir and HOME of its own under W.

    W=<scratch> python3 demo/runs/two-at-once/2026-10-12/run.py

The films are FILMS (else /tmp/onb-e-films), as Demo C's rerun read them.
Data in data/ beside this file, the links' tokens never in it.
scripts/check.sh runs none of this."""
import http.server as httpserver, json, os, re, shutil, signal, subprocess, sys, threading, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
OUT = HERE / "data"; W = Path(os.environ["W"]); BIN = REPO / "target/debug/svrngs"
FILMS = Path(os.environ.get("FILMS", "/tmp/onb-e-films"))
JF_APP = Path("/Applications/Jellyfin.app/Contents"); JF = "http://127.0.0.1:8096"
GROUP = "two"; LOG = "info,door=debug,cli=debug"
procs, secret, summary, hidden = {}, {}, {}, []


def redact(text):
    for h in hidden: text = text.replace(h, "<the token>")
    return text


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}))
    print(line[:600], flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def wait(cond, bound, every=0.2):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def http(method, url, body=None, headers=None, timeout=10):
    """(status, body as JSON or text) of one request, or (None, why)."""
    h = {"content-type": "application/json", **(headers or {})}
    req = urllib.request.Request(url, method=method, headers=h, data=None if body is None else json.dumps(body).encode())
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r: status, blob = r.status, r.read()
    except urllib.error.HTTPError as e: status, blob = e.code, e.read()
    except Exception as e: return None, str(e)
    try: return status, json.loads(blob)
    except Exception: return status, blob.decode(errors="replace")


def env(who, user=None):
    return {**os.environ, "HOME": str(W / who), "USER": user or who, "SVRNGS_DIR": str(W / who / ".svrngs"),
            "NO_COLOR": "1", "RUST_LOG": LOG, "CARGO_INCREMENTAL": "0"}


def api(who, path):
    try: base = (W / who / ".svrngs/api").read_text().strip().rstrip("/")
    except Exception: return None
    s, b = http("GET", base + path)
    return b if s == 200 else None


def command(who, *args, user=None):
    t = time.time()
    run = subprocess.run([BIN, *args], env={**env(who, user), "RUST_LOG": "error"}, input="\n", capture_output=True, text=True)
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


def people():
    rows = (api("sam", f"/v1/rail/people?namespace={secret['group']}") or {}).get("people", [])
    return sorted((r["person"], r["key"], r.get("person_key")) for r in rows)


# --- Sam's Jellyfin, its films, and its middleware --------------------------
JF_AUTH = 'MediaBrowser Client="two", Device="Sam computer", DeviceId="two-setup", Version="1"'
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
        s, b = http("GET", JF + "/System/Info/Public", timeout=2)
        seen[0] = seen[0] + 1 if s == 200 and "StartupWizardCompleted" in json.dumps(b) else 0
        return seen[0] >= 2 and b
    info = wait(ready, 120, 1)
    if not info: sys.exit("Jellyfin did not come up")
    secret["sam_pw"] = os.urandom(12).hex()
    jf("POST", "/Startup/Configuration", {"UICulture": "en-US", "MetadataCountryCode": "US", "PreferredMetadataLanguage": "en"})
    jf("GET", "/Startup/User")
    jf("POST", "/Startup/User", {"Name": "sam", "Password": secret["sam_pw"]})
    jf("POST", "/Library/VirtualFolders?name=Movies&collectionType=movies&refreshLibrary=true",
       {"LibraryOptions": {"PathInfos": [{"Path": str(FILMS)}], "EnableRealtimeMonitor": False}})
    jf("POST", "/Startup/RemoteAccess", {"EnableRemoteAccess": False, "EnableAutomaticPortMapping": False})
    jf("POST", "/Startup/Complete"); time.sleep(3)
    secret["jf"] = jf("POST", "/Users/AuthenticateByName", {"Username": "sam", "Pw": secret["sam_pw"]})[1]["AccessToken"]
    hidden.append(secret["jf"])
    jf("POST", "/Library/Refresh")
    def film(term):
        s, b = jf("GET", f"/Items?Recursive=true&IncludeItemTypes=Movie&SearchTerm={term}")
        items = b.get("Items", []) if s == 200 and isinstance(b, dict) else []
        return items[0]["Id"] if items else None
    films = {"laptop": wait(lambda: film("Two"), 120, 1), "phone": wait(lambda: film("One"), 120, 1)}
    if not all(films.values()): sys.exit(f"Jellyfin did not list both films: {films}")
    jf("POST", "/Auth/Keys?app=svrngs-mw")
    key = next(k["AccessToken"] for k in jf("GET", "/Auth/Keys")[1]["Items"] if k["AppName"] == "svrngs-mw")
    hidden.append(key)
    subprocess.run(["cargo", "build", "--quiet", "--manifest-path", REPO / "apps/jellyfin/mw/Cargo.toml"], check=True,
                   env={**os.environ, "CARGO_INCREMENTAL": "0"})
    procs["mw"] = subprocess.Popen([REPO / "apps/jellyfin/mw/target/debug/jellyfin-mw", "--listen", "127.0.0.1:8097"],
                                   env={**os.environ, "NO_COLOR": "1", "RUST_LOG": "debug,hyper=info,reqwest=info,h2=info"}, stdin=subprocess.PIPE,
                                   stdout=open(W / "mw.out", "w"), stderr=open(W / "mw.log", "w"), text=True)
    procs["mw"].stdin.write(key + "\n"); procs["mw"].stdin.close()
    if not wait(lambda: "listening on port" in (W / "mw.out").read_text(), 30): sys.exit("the middleware did not listen")
    return {"server": info["Id"], "version": info.get("Version"), "films": films, "users_before": jf_users()}


def main():
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir()
    try:
        say("start", load=round(os.getloadavg()[0], 1), commit=subprocess.run(["git", "-C", REPO, "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True).stdout.strip())
        subprocess.run(["cargo", "build", "--quiet", "-p", "svrngs"], cwd=REPO, check=True, env={**os.environ, "CARGO_INCREMENTAL": "0"})
        subprocess.run([REPO / "scripts/build-web.sh", "--out", W / "site"], check=True, capture_output=True)
        procs["page"] = subprocess.Popen(["node", REPO / "web/tests/serve.mjs", W / "site"], stdout=subprocess.PIPE, text=True)
        port = re.search(r"port (\d+)", procs["page"].stdout.readline())[1]
        threading.Thread(target=lambda: [None for _ in procs["page"].stdout], daemon=True).start()
        page = f"http://localhost:{port}/"
        summary["jellyfin"] = app = jellyfin()

        start("sam", "new", GROUP, "--as", "Sam", "--origin", page)
        links("sam")
        secret["group"] = wait(lambda: ((api("sam", "/v1/groups") or {}).get("groups") or [{}])[0].get("id"), 60)
        summary["sam_share"] = command("sam", "share", "8096", "--sign-in", "8097")

        # Ana, then a link for her devices; her laptop by it, no name.
        sam_https = re.search(r"http://localhost:\d+/#\S+", said("sam"))[0]
        summary["ana_join"] = command("ana", "join", sam_https, "--as", "Ana", "--no-run")
        start("ana", "link", GROUP, "--device")
        ana_client, ana_https = links("ana")
        summary["ana_device_link_said"] = redact(said("ana").strip())
        t0 = start("laptop", "join", ana_client, user="analaptop")
        line = wait(lambda: re.search(r"You're in .*|svrngs: .*", said("laptop") + (W / "laptop.log").read_text()), 60)
        summary["laptop_join"] = {"said": line and line[0], "ms": round((time.time() - t0) * 1000), "user": "analaptop", "as_given": None}
        laptop = wait(lambda: lends("laptop").get("jellyfin"), 120, 1)
        if not laptop: sys.exit(f"the laptop lists no jellyfin: {lends('laptop')}")

        # Both devices at once (two.mjs).
        args = {"laptop": laptop, "link": ana_https, "server": app["server"], "films": app["films"], "window_s": 180, "seeks": 4}
        say("two", load=round(os.getloadavg()[0], 1))
        run = subprocess.run(["node", HERE / "two.mjs", json.dumps(args)], cwd=HERE, capture_output=True, text=True, timeout=900,
                             env={**os.environ, "JF": JF, "ADMIN": secret["jf"]})
        (W / "two.err").write_text(run.stderr)
        try: summary["two"] = json.loads(run.stdout.strip().splitlines()[-1])
        except Exception: summary["two"] = {"never_ran": True, "exit": run.returncode, "stdout": run.stdout[-800:], "stderr": run.stderr[-800:]}
        summary["people"] = people()
        summary["accounts"] = jf_users()
        mw = (W / "mw.log").read_text()
        summary["sign_in_asks"] = re.findall(r"the door asks who a person is to Jellyfin.*?key=(\w+)", mw)
        log = re.sub(r"\x1b\[[0-9;]*m", "", (W / "sam.log").read_text())
        summary["door_sign_in"] = {k: log.count(k) for k in ("the person's kept answer", "the person's ask is out", "door: sign-in — asked for the person", "refused a kept answer")}
    finally:
        for p in procs.values(): p.send_signal(signal.SIGTERM)
        for p in procs.values():
            try: p.wait(15)
            except Exception: p.kill()
        summary["end_load"] = round(os.getloadavg()[0], 1)
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, ensure_ascii=False)) + "\n")
        for who in ("sam", "ana", "laptop"):
            if (W / f"{who}.out").exists(): (OUT / f"{who}.out").write_text(redact(said(who)))
        if (W / "mw.log").exists(): (OUT / "mw.log").write_text(redact((W / "mw.log").read_text()))
        say("end")


if __name__ == "__main__":
    main()
