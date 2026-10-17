#!/usr/bin/env python3
"""Demo C's dry run with n0 out of the path (PREDICTIONS.md here), copied
from demo/runs/c/dry-2026-10-12-rerun/run.py and changed only for the relay:
every computer and phone on `svrngs relay`'s own relay::run on 127.0.0.1
(crates/svrngs/tests/n0_off.rs `the_verbs_relay`), open, with Sam's group
made `--relay` there; every process's remote addresses sampled with lsof
against n0's; then the same relay over Sam's data admitting only the group,
for its members, Bo, and a newcomer at the line (Fay) and on a phone (Gus).
The rerun's own words follow: demo/c.md's steps on this Mac against the
page built from this commit and served on this host, every person a
program. Sam and Ana from the tarball served there; Sam's Jellyfin through
its middleware; Ana's Navidrome lent by `svrngs share 4533` and set as it
prints; Alex, Bo and Dee each by the line the page gives, one at a time; Cy
on a phone through the page; Sam removes Bo; Ana's computer stopped as a
lid shut; then Eve.

    media.sh <MEDIA>
    W=<scratch> MEDIA=<MEDIA> python3 demo/runs/c/no-n0-2026-10-13/run.py

MEDIA holds photos/ (the trip), music/ (Ana's albums) and after.jpg; FILMS
Demo A's films (else /tmp/onb-e-films). Data in data/ beside this file, the
group's token never in it. scripts/check.sh runs none of this."""
import gzip, hashlib, json, os, re, shutil, signal, socket, subprocess, sys, tarfile, threading, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
OUT = HERE / "data"; W = Path(os.environ["W"]); MEDIA = Path(os.environ["MEDIA"])
FILMS = Path(os.environ.get("FILMS", "/tmp/onb-e-films"))
GROUP, LABEL = "lake-trip", "sh.svrnme.svrngs"
JF_APP = Path("/Applications/Jellyfin.app/Contents"); JF = "http://127.0.0.1:8096"; ND = "http://127.0.0.1:4533"
LOG = "info,door=debug,removal=debug,origin=debug,folder=info,transport=warn,cli=debug,relay=debug"
N0_HOSTS = ["use1-1.relay.n0.iroh.link", "usw1-1.relay.n0.iroh.link", "euc1-1.relay.n0.iroh.link",
            "aps1-1.relay.n0.iroh.link", "use1-1.staging-relay.n0.iroh.link", "euc1-1.staging-relay.n0.iroh.link",
            "dns.iroh.link", "staging-dns.iroh.link"]
OURS = ["relay.svrnme.sh"]
N0_WORDS = r"iroh\.link|pkarr|relay\.n0|relay\.svrnme\.sh"
UID = os.getuid(); TARBALL = "svrngs-darwin-aarch64.tar.gz"
FILM_LEND, MUSIC_LEND = "Sam’s jellyfin", "ana’s navidrome"
PEOPLE = ("sam", "ana", "alex", "bo", "dee", "eve", "fay")
TAKE = ["open"]
procs, secret, summary, hidden = {}, {}, {}, []
pids, lock = {}, threading.Lock()


def redact(text):
    for h in hidden: text = text.replace(h, "<the token>")
    return text


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}))
    print(line, flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def load(): return os.getloadavg()[0]


def wait(cond, bound, every=0.2):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def http(method, url, body=None, headers=None, host=None, raw=False, timeout=10):
    """(status, body as JSON or text) of one request, or (None, why)."""
    h = {"content-type": "application/json", **(headers or {}), **({"Host": host} if host else {})}
    req = urllib.request.Request(url, method=method, headers=h,
                                 data=None if body is None else (body if raw else json.dumps(body)).encode())
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r: status, blob = r.status, r.read()
    except urllib.error.HTTPError as e: status, blob = e.code, e.read()
    except Exception as e: return None, str(e)
    try: return status, json.loads(blob)
    except Exception: return status, blob.decode(errors="replace")


# --- each person's computer: a HOME of its own -----------------------------
def home(who): return W / who
def data(who): return home(who) / ".svrngs"
def binary(who): return home(who) / ".local/bin/svrngs"
def folder(who): return home(who) / "svrngs" / GROUP
def env(who):
    e = {k: v for k, v in os.environ.items() if k != "SVRNGS_DIR"}
    return {**e, "HOME": str(home(who)), "USER": who, "PATH": f"{W / 'stub'}:{e['PATH']}", "NO_COLOR": "1"}


def base(who): return (data(who) / "api").read_text().strip().rstrip("/")
def home_page(who): return f"http://svrngs.localhost:{base(who).rsplit(':', 1)[1]}/"


def api(who, path):
    try: status, body = http("GET", base(who) + path)
    except Exception: return None
    return body if status == 200 else None


def command(who, *args, stdin=""):
    t = time.time()
    run = subprocess.run([binary(who), *args], env={**env(who), "SVRNGS_DIR": str(data(who)), "RUST_LOG": "error"},
                         input=stdin, capture_output=True, text=True)
    out = {"who": who, "args": list(args), "exit": run.returncode, "said": (run.stdout + run.stderr).strip(),
           "ms": round((time.time() - t) * 1000)}
    say("command", **out)
    return out


def run_node(who):
    """who's svrngs as install.sh's LaunchAgent runs it: the bare command,
    SVRNGS_DIR, its log; run here, with who's HOME, debug traces on."""
    (data(who) / "api").unlink(missing_ok=True)
    procs[who] = subprocess.Popen([binary(who)], env={**env(who), "SVRNGS_DIR": str(data(who)), "RUST_LOG": LOG},
                                  stdin=subprocess.DEVNULL, stdout=open(data(who) / "svrngs.log", "a"),
                                  stderr=subprocess.STDOUT)
    if not wait(lambda: api(who, "/v1/groups"), 60): sys.exit(f"{who}'s svrngs did not come up")


def take_over(who):
    """The one LaunchAgent label freed for the next person's line: who's
    svrngs booted out of launchd and run here instead (PREDICTIONS.md)."""
    t = time.time()
    subprocess.run(["launchctl", "bootout", f"gui/{UID}/{LABEL}"], capture_output=True)
    wait(lambda: subprocess.run(["launchctl", "print", f"gui/{UID}/{LABEL}"], capture_output=True).returncode != 0, 30)
    run_node(who)
    say("taken-over", who=who, ms=round((time.time() - t) * 1000))


def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()


# --- the relay: svrngs relay's relay::run on 127.0.0.1 ----------------------
def start_relay(take, root, open_):
    """The verb's relay over `root` at RELAY, open or for its groups; what it
    said as it started."""
    log = W / f"relay-{take}.log"
    e = {**os.environ, "SVRNGS_DIR": str(root), "RELAY_AT": RELAY_AT, "RUST_LOG": "relay=debug,iroh_relay=info", "NO_COLOR": "1"}
    if open_: e["RELAY_OPEN"] = "1"
    procs["relay"] = subprocess.Popen([RELAY_BIN, "--ignored", "--exact", "the_verbs_relay", "--nocapture"], env=e,
                                      stdout=open(log, "w"), stderr=subprocess.STDOUT)
    with lock: pids[procs["relay"].pid] = "relay"
    said = wait(lambda: re.search(r"Relaying at .*", log.read_text()), 30)
    if not said: sys.exit(f"the relay ({take}) did not start: {log.read_text()[-400:]}")
    say("relay", take=take, said=said[0], pid=procs["relay"].pid)
    return said[0]


def stop_relay():
    p = procs.pop("relay"); p.send_signal(signal.SIGINT)
    try: p.wait(20)
    except subprocess.TimeoutExpired: p.kill()


def relay_says(take):
    """The relay's decisions, in order: (seconds since the run's start, admitted or refused, key, person)."""
    out = []
    for l in (W / f"relay-{take}.log").read_text(errors="replace").splitlines():
        m = re.search(r'relay: (admitted|refused) — .*?key="?(\w+)"?(?:.*?person="?([^"\s]+))?', l)
        if m: out.append((m[1], m[2], m[3]))
    return out


# --- every remote address each process opened -------------------------------
def watched_now():
    with lock: got = dict(pids)
    for who in PEOPLE:
        for pid in subprocess.run(["pgrep", "-f", str(binary(who))], capture_output=True, text=True).stdout.split():
            got[int(pid)] = who
    for pid in subprocess.run(["pgrep", "-f", "ms-playwright"], capture_output=True, text=True).stdout.split():
        got.setdefault(int(pid), "browser")
    got.pop(os.getpid(), None)
    def alive(pid):
        try: os.kill(pid, 0); return True
        except OSError: return False
    return {pid: who for pid, who in got.items() if alive(pid)}


def sampler(stop):
    seen = set()
    while not stop.is_set():
        watched = watched_now()
        r = subprocess.run(["lsof", "-i", "-n", "-P", "-a", "-p", ",".join(map(str, watched))], capture_output=True, text=True)
        for line in r.stdout.splitlines()[1:]:
            f = line.split()
            if len(f) < 9: continue
            name = " ".join(f[8:])
            remote = name.split("->", 1)[1].split()[0] if "->" in name else None
            key = (watched.get(int(f[1]), "?"), f[0], f[7], remote or name.split()[0])
            if key not in seen:
                seen.add(key)
                with open(OUT / "lsof.jsonl", "a") as out:
                    out.write(json.dumps({"t": round(time.time(), 3), "take": TAKE[0], "who": key[0], "command": key[1], "pid": int(f[1]),
                                          "proto": key[2], "remote": remote, "local": None if remote else key[3]}) + "\n")
        stop.wait(2)


def addresses(hosts):
    found = {}
    for host in hosts:
        ips = []
        for kind in ("A", "AAAA"):
            r = subprocess.run(["dig", "+short", kind, host], capture_output=True, text=True)
            ips += [l for l in r.stdout.split() if re.match(r"^[0-9a-f.:]+$", l) and (":" in l or l.count(".") == 3)]
        found[host] = ips
    return found


def lsof_read():
    seen = [json.loads(l) for l in (OUT / "lsof.jsonl").read_text().splitlines()] if (OUT / "lsof.jsonl").exists() else []
    host = lambda r: r.rsplit(":", 1)[0].strip("[]")
    n0 = {ip for ips in summary["n0"].values() for ip in ips}
    ours = {ip for ips in summary["ours"].values() for ip in ips}
    off = [s for s in seen if s["remote"] and not re.match(r"^(127\.|::1$|localhost)", host(s["remote"]))]
    return {"sampled": len(seen), "processes": sorted({f"{s['who']}:{s['command']}" for s in seen}),
            "n0_hits": [s for s in seen if s["remote"] and host(s["remote"]) in n0],
            "ours_hits": [s for s in seen if s["remote"] and host(s["remote"]) in ours],
            "off_host": sorted({f"{s['who']}:{s['command']} {s['proto']} {s['remote']}" for s in off})}


def browser(mode, args, timeout=600):
    run = subprocess.run(["node", str(HERE / "c.mjs"), mode, json.dumps(args)], cwd=REPO, capture_output=True,
                         text=True, timeout=timeout)
    try: return json.loads(run.stdout.strip().splitlines()[-1])
    except Exception: return {"stopped": f"c.mjs printed nothing: {run.stderr[-400:]}"}


# --- Sam's Jellyfin and its middleware; Ana's Navidrome ---------------------
JF_AUTH = 'MediaBrowser Client="c-dry", Device="Sam computer", DeviceId="c-dry-setup", Version="1"'
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
    jf("POST", "/Library/Refresh")
    def film_one():
        s, b = jf("GET", "/Items?Recursive=true&IncludeItemTypes=Movie&SearchTerm=One")
        items = b.get("Items", []) if s == 200 and isinstance(b, dict) else []
        return items[0]["Id"] if items else None
    one = wait(film_one, 90, 1)
    if not one: sys.exit("Jellyfin did not list Test Pattern One")
    jf("POST", "/Auth/Keys?app=svrngs-mw")
    key = next(k["AccessToken"] for k in jf("GET", "/Auth/Keys")[1]["Items"] if k["AppName"] == "svrngs-mw")
    subprocess.run(["cargo", "build", "--quiet", "--manifest-path", REPO / "apps/jellyfin/mw/Cargo.toml"], check=True,
                   env={**os.environ, "CARGO_INCREMENTAL": "0"})
    procs["mw"] = subprocess.Popen([REPO / "apps/jellyfin/mw/target/debug/jellyfin-mw", "--listen", "127.0.0.1:8097"],
                                   env={**os.environ, "NO_COLOR": "1", "RUST_LOG": "info"}, stdin=subprocess.PIPE,
                                   stdout=open(W / "mw.out", "w"), stderr=open(W / "mw.log", "w"), text=True)
    procs["mw"].stdin.write(key + "\n"); procs["mw"].stdin.close()
    if not wait(lambda: "listening on port" in (W / "mw.out").read_text(), 30): sys.exit("the middleware did not listen")
    return {"server": info["Id"], "jellyfin_version": info.get("Version"), "film": one, "users_before": jf_users()}


def nd_token():
    return http("POST", ND + "/auth/login", {"username": "ana", "password": secret["nd"]})[1]["token"]


def nd_users():
    s, b = http("GET", ND + "/api/user", headers={"x-nd-authorization": f"Bearer {nd_token()}"})
    return sorted(u["userName"] for u in b) if s == 200 else {"status": s, "said": b}


def nd_start(settings):
    """Ana's Navidrome on 4533, with `settings` (what `svrngs share 4533`
    printed, once it has) in its environment and nothing else of ExtAuth."""
    d = W / "nd"; d.mkdir(parents=True, exist_ok=True)
    nd_env = {**{k: v for k, v in os.environ.items() if not k.startswith("ND_")}, "ND_MUSICFOLDER": str(W / "ana-music"),
              "ND_DATAFOLDER": str(d), "ND_ADDRESS": "127.0.0.1", "ND_PORT": "4533", "ND_LOGLEVEL": "info", **settings}
    procs["navidrome"] = subprocess.Popen(["navidrome"], env=nd_env, stdout=open(d / "nd.log", "a"), stderr=subprocess.STDOUT)
    if not wait(lambda: http("GET", ND + "/ping", timeout=2)[0] == 200, 60, 1): sys.exit("Navidrome did not come up")


def navidrome():
    shutil.copytree(MEDIA / "music", W / "ana-music")
    nd_start({})
    secret["nd"] = os.urandom(12).hex()
    http("POST", ND + "/auth/createAdmin", {"username": "ana", "password": secret["nd"]})
    def albums():
        s, b = http("GET", ND + f"/rest/getAlbumList2.view?u=ana&p={secret['nd']}&v=1.16.1&c=c-dry&f=json&type=alphabeticalByName")
        got = b.get("subsonic-response", {}).get("albumList2", {}).get("album", []) if s == 200 else []
        return len(got) >= 3 and got
    got = wait(albums, 60, 1)
    if not got: sys.exit("Navidrome did not list Ana's three albums")
    return {"navidrome_version": subprocess.run(["navidrome", "--version"], capture_output=True, text=True).stdout.strip(),
            "album": got[0]["id"], "album_name": got[0]["name"], "users_before_nd": nd_users()}


# --- a newcomer: the link, the line, their home page, the three things ------
def newcomer(who, apps, music=True):
    t0 = time.time()
    line = browser("line", {"link": LINK, "t0": round(t0 * 1000)})
    if not line.get("line"): sys.exit(f"{who}: the page showed no line: {line}")
    pasted = time.time()
    run = subprocess.run(["sh", "-c", line["line"]], env=env(who), capture_output=True, text=True)
    done = time.time()
    (OUT / f"{who}-install.out").write_text(redact(run.stdout + run.stderr))
    opened = (home(who) / "opened").read_text().split() if (home(who) / "opened").exists() else [None, None]
    person = browser("person", {
        "t0": round(t0 * 1000), "home": opened[1] or home_page(who), "group": GROUP, "dir": str(folder(who)),
        "film_lend": FILM_LEND, "music_lend": MUSIC_LEND if music else None, "server": apps["server"],
        "film": apps["film"], "album": apps["album"], "photo": str(folder(who) / "trip/IMG_0001.jpg"),
        "sha": apps["sha"]})
    out = {"line": {k: line.get(k) for k in ("shown", "title", "stopped")}, "pasted_ms": round((pasted - t0) * 1000),
           "line_exit": run.returncode, "line_done_ms": round((done - t0) * 1000), "said_last": redact(run.stdout.strip().splitlines()[-1] if run.stdout.strip() else run.stderr[-200:]),
           "home_opened_ms": int(opened[0]) - round(t0 * 1000) if opened[0] else None, **person,
           "listed_after_line_ms": (person.get("home") or {}).get("listed") and person["home"]["listed"] - round((done - t0) * 1000),
           "load": [round(load(), 1)]}
    say("newcomer", who=who, **out)
    return out, done


def ask_lend(who, origin, path):
    host = origin.split("//")[1].rstrip("/")
    s, b = http("GET", f"http://127.0.0.1:{host.rsplit(':', 1)[1]}{path}", host=host, timeout=10)
    return s, (b.get("kind") if isinstance(b, dict) else None), (b.get("error") if isinstance(b, dict) else str(b)[:100])


def lends_of(who):
    body = api(who, f"/v1/lends?namespace={GROUP_ID}") or {}
    return {f"{l['person']}/{l['name']}": l["origin"] for l in body.get("lent", [])}


def people_at(who):
    return sorted(p["person"] for p in (api(who, f"/v1/rail/people?namespace={GROUP_ID}") or {}).get("people", []))


def keys_at(who):
    """Each person's name and the first 16 hex of their person key, the
    door's `x-svrngs-user` (crates/svrngs-node/src/door.rs, user_name)."""
    return sorted(f"{p['person']}-{p['person_key'][:16]}"
                  for p in (api(who, f"/v1/rail/people?namespace={GROUP_ID}") or {}).get("people", []))


def main():
    global LINK, GROUP_ID, RELAY, RELAY_AT, RELAY_BIN
    if subprocess.run(["launchctl", "print", f"gui/{UID}/{LABEL}"], capture_output=True).returncode == 0:
        sys.exit(f"a svrngs is kept running on this host already ({LABEL}): this run would replace it")
    shutil.rmtree(W, ignore_errors=True); shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir(parents=True)
    (W / "stub").mkdir(parents=True)
    (W / "stub/open").write_text('#!/bin/sh\nprintf \'%s %s\\n\' "$(python3 -c \'import time;print(int(time.time()*1000))\')" "$1" >"$HOME/opened"\n')
    (W / "stub/open").chmod(0o755)
    head = subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    say("start", head=head, load=os.getloadavg(), cores=os.cpu_count(),
        macos=subprocess.run(["sw_vers", "-productVersion"], capture_output=True, text=True).stdout.strip())
    global PAGE
    stop = threading.Event()
    try:
        # 0. The page host: build-web.sh's site with build-release.sh's get/
        # in it, built from this commit, served on this host
        # (runs/onboarding/2026-10-11/run.sh). Then what it serves: the
        # tarball against its sums; Sam's and Ana's svrngs from it.
        built = {}
        for script in ("build-web.sh", "build-release.sh"):
            t = time.time()
            r = subprocess.run([REPO / "scripts" / script, "--out", W / "site"], capture_output=True, text=True,
                               env={**os.environ, "CARGO_INCREMENTAL": "0"})
            (W / f"{script}.log").write_text(r.stdout + r.stderr)
            built[script] = {"exit": r.returncode, "s": round(time.time() - t), "last": (r.stdout + r.stderr).strip().splitlines()[-1:]}
        r = subprocess.run(["cargo", "test", "--release", "-p", "svrngs", "--test", "n0_off", "--no-run", "--message-format=json"],
                           cwd=REPO, capture_output=True, text=True, env={**os.environ, "CARGO_INCREMENTAL": "0"})
        RELAY_BIN = next(json.loads(l)["executable"] for l in r.stdout.splitlines()
                         if '"executable"' in l and "n0_off" in (json.loads(l).get("executable") or ""))
        built["relay"] = {"exit": r.returncode, "binary": Path(RELAY_BIN).name}
        say("built", **built)
        summary["n0"], summary["ours"] = addresses(N0_HOSTS), addresses(OURS)
        say("addresses", n0=summary["n0"], ours=summary["ours"])
        threading.Thread(target=sampler, args=(stop,), daemon=True).start()
        with socket.socket() as sk: sk.bind(("127.0.0.1", 0)); port = sk.getsockname()[1]
        RELAY_AT, RELAY = f"127.0.0.1:{port}", f"http://127.0.0.1:{port}/"
        (W / "relay-open").mkdir()
        summary["relay"] = {"at": RELAY, "open": start_relay("open", W / "relay-open", True)}
        if not (W / "site/get" / TARBALL).is_file(): sys.exit(f"build-release.sh made no {TARBALL}: {built}")
        procs["page"] = subprocess.Popen(["node", REPO / "web/tests/serve.mjs", W / "site"], stdout=subprocess.PIPE, text=True)
        pids[procs["page"].pid] = "page"
        PAGE = f"http://localhost:{procs['page'].stdout.readline().split()[1]}/"
        threading.Thread(target=lambda: [None for _ in procs["page"].stdout], daemon=True).start()
        sums = http("GET", PAGE + "get/SHA256SUMS")[1]
        want = next(l.split()[0] for l in sums.splitlines() if l.split()[-1].lstrip("*") == TARBALL)
        blob = urllib.request.urlopen(PAGE + "get/" + TARBALL, timeout=60).read()
        (W / TARBALL).write_bytes(blob)
        got = hashlib.sha256(blob).hexdigest()
        summary["published"] = {"page": PAGE, "built": built, "tarball": TARBALL, "bytes": len(blob), "sum_matches": got == want,
                                "sums": sums.strip().splitlines()}
        say("published", **summary["published"])
        for who in ("sam", "ana"):
            (home(who) / ".local/bin").mkdir(parents=True)
            with tarfile.open(W / TARBALL) as t: t.extract("svrngs", home(who) / ".local/bin")
        summary["published"]["binary_sha256"] = sha(binary("sam"))

        # 1. Before the evening: Sam's library, Ana's music, the trip.
        apps = jellyfin(); apps.update(navidrome())
        with lock: pids.update({procs[n].pid: n for n in ("jellyfin", "mw", "navidrome")})
        apps["sha"] = sha(MEDIA / "photos/IMG_0001.jpg")
        summary["apps"] = apps; say("apps", **apps)

        # 2. Sam makes the group and lends the library; the link as a chat's
        # preview fetches it.
        data("sam").mkdir(parents=True)
        procs["sam"] = subprocess.Popen([binary("sam"), "new", GROUP, "--as", "Sam", "--origin", PAGE, "--relay", RELAY],
                                        env={**env("sam"), "SVRNGS_DIR": str(data("sam")), "RUST_LOG": LOG},
                                        stdin=subprocess.DEVNULL, stdout=open(W / "sam-new.out", "w"),
                                        stderr=open(data("sam") / "svrngs.log", "w"))
        m = wait(lambda: re.search(re.escape(PAGE) + r"#([A-Za-z0-9_-]+)", (W / "sam-new.out").read_text()), 90)
        if not m: sys.exit("Sam's new printed no link on the page")
        LINK = m[0]; hidden.append(m[1])
        hidden.extend(re.findall(r"svrngs://join/([A-Za-z0-9_-]+)", (W / "sam-new.out").read_text()))
        wait(lambda: api("sam", "/v1/groups"), 60)
        GROUP_ID = api("sam", "/v1/groups")["groups"][0]["id"]
        (OUT / "sam-new.out").write_text(redact((W / "sam-new.out").read_text()))
        s, html = http("GET", PAGE, headers={"user-agent": "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)"})
        summary["preview"] = {"status": s, **{k: (re.search(rf'<meta property="og:{k}" content="([^"]*)"', html or "") or [None, None])[1]
                                              for k in ("title", "description")}}
        summary["sam"] = {"share": command("sam", "share", "8096", "--sign-in", "8097")}
        said = wait(lambda: re.search(r".*folder is .*", (W / "sam-new.out").read_text()), 30)
        summary["sam"]["folder"] = {"said": said and said[0], "is_dir": folder("sam").is_dir()}
        say("sam", group=GROUP_ID, preview=summary["preview"])

        # 3. Ana brings the trip: the line, then the music and the photos.
        t0 = time.time()
        line = browser("line", {"link": LINK, "t0": round(t0 * 1000)})
        if not line.get("line"): sys.exit(f"ana: the page showed no line: {line}")
        run = subprocess.run(["sh", "-c", line["line"]], env=env("ana"), capture_output=True, text=True)
        (OUT / "ana-install.out").write_text(redact(run.stdout + run.stderr))
        summary["ana"] = {"line_exit": run.returncode, "line_ms": round((time.time() - t0) * 1000),
                          "folder": folder("ana").is_dir()}
        take_over("ana")
        # Lent as svrngs recognises it; Navidrome set as `share` prints, and
        # started again.
        summary["ana"]["share"] = command("ana", "share", "4533")
        settings = dict(re.findall(r"\b(ND_[A-Z_]+)=(\S+?)(?=[\s,;]|$)", summary["ana"]["share"]["said"]))
        procs["navidrome"].terminate(); procs["navidrome"].wait(15)
        nd_start(settings)
        with lock: pids[procs["navidrome"].pid] = "navidrome"
        summary["ana"]["navidrome_set"] = settings
        shutil.copytree(MEDIA / "photos", folder("ana") / "trip")
        summary["ana"]["photos"] = {"files": len(list((folder("ana") / "trip").iterdir())),
                                    "bytes": sum(p.stat().st_size for p in (folder("ana") / "trip").iterdir())}
        say("ana", **summary["ana"])
        trip = {p.name: sha(p) for p in (MEDIA / "photos").iterdir()}

        # 4. The newcomers, one at a time.
        summary["newcomers"] = {}
        for who in ("alex", "bo", "dee"):
            summary["newcomers"][who], _ = newcomer(who, apps)
            take_over(who)
        summary["names"] = {w: people_at(w) for w in ("sam", "alex")}
        cy = {"link": LINK, "name": "Cy", "server": apps["server"], "film": apps["film"], "album": apps["album"],
              "film_lend": FILM_LEND, "music_lend": MUSIC_LEND, "profile": str(W / "cy-phone")}
        summary["cy"] = browser("phone", {"t0": round(time.time() * 1000), **cy}, timeout=900)
        say("cy", **summary["cy"])
        keys = {p["person"]: p["key"] for p in (api("sam", f"/v1/rail/people?namespace={GROUP_ID}") or {}).get("people", [])}

        # 5. Sam removes Bo.
        origins = lends_of("bo"); say("bo-lends", origins=origins)
        paths = {"Sam/jellyfin": "/System/Info/Public", "ana/navidrome": "/ping"}
        before = {n: ask_lend("bo", o, paths.get(n, "/"))[:2] for n, o in origins.items()}
        cut0 = time.time()
        cut = command("sam", "remove", "bo", "--group", GROUP)
        first = {}
        def poll(name, origin):
            last = None
            while time.time() - cut0 < 60:
                s, kind, words = ask_lend("bo", origin, paths.get(name, "/"))
                if (s, kind) != last: last = (s, kind); say("bo-asks", lend=name, status=s, kind=kind, words=words, since_cut_s=round(time.time() - cut0, 3))
                if kind == "removed" and name not in first: first[name] = round(time.time() - cut0, 3)
                time.sleep(0.2)
        threads = [threading.Thread(target=poll, args=i) for i in origins.items()]
        [t.start() for t in threads]
        told = browser("read", {"url": home_page("bo"), "until": f"You are no longer in {GROUP}", "within": 60000, "t0": round(cut0 * 1000)})
        [t.join() for t in threads]
        w0 = time.time(); shutil.copy(MEDIA / "after.jpg", folder("sam") / "after.jpg")
        reached = {}
        def arrives(who):
            t = wait(lambda: (folder(who) / "after.jpg").is_file() and sha(folder(who) / "after.jpg") == sha(MEDIA / "after.jpg"), 60, 0.2)
            reached[who] = round(time.time() - w0, 2) if t else None
        ts = [threading.Thread(target=arrives, args=(w,)) for w in ("ana", "alex", "dee", "bo")]
        [t.start() for t in ts]; [t.join() for t in ts]
        bo_tries = command("bo", "remove", "Sam", "--group", GROUP)
        time.sleep(5)
        summary["removed"] = {"before": before, "command": cut, "refused_removed_s": first, "bo_told": {k: told.get(k) for k in ("at", "said", "stopped")},
                              "after_jpg_s": reached, "bo_tries": bo_tries, "standing": {w: people_at(w) for w in ("sam", "ana", "alex")},
                              "bo_keeps": len(list((folder("bo") / "trip").glob("*.jpg")))}
        say("removed", **summary["removed"])

        # 6. Ana's laptop closed; then Eve.
        for p in ("ana", "navidrome"): procs[p].send_signal(signal.SIGSTOP)
        slept = time.time(); say("ana-asleep")
        eve, eve_in = newcomer("eve", apps, music=False)
        have = lambda: all((folder("eve") / "trip" / n).is_file() and sha(folder("eve") / "trip" / n) == h for n, h in trip.items())
        t_all = wait(have, max(1, 120 - (time.time() - eve_in)), 1)
        all_s = round(time.time() - eve_in, 1) if t_all else None
        take_over("eve")
        unanswered = {w: browser("read", {"url": home_page(w), "until": "ana's computer did not answer", "within": 120000, "t0": round(slept * 1000)})
                      for w in ("eve", "alex")}
        plays = {w: ask_lend(w, lends_of(w).get("Sam/jellyfin", "http://x:1/"), "/System/Info/Public")[0] for w in ("alex", "dee", "eve")}
        summary["asleep"] = {"eve": eve, "all_thirty_s_after_eve_in": all_s,
                             "eve_has": sum((folder("eve") / "trip" / n).is_file() for n in trip),
                             "unanswered": {w: {k: r.get(k) for k in ("at", "said", "stopped")} for w, r in unanswered.items()},
                             "jellyfin_answers": plays}
        say("asleep", **summary["asleep"])
        for p in ("ana", "navidrome"): procs[p].send_signal(signal.SIGCONT)

        # 7. No accounts.
        summary["accounts"] = {"x_svrngs_user": keys_at("sam"), "jellyfin_before": apps["users_before"], "jellyfin_after": jf_users(),
                               "navidrome_before": apps["users_before_nd"],
                               "navidrome_after": nd_users()}
        say("accounts", **summary["accounts"])
        keys.update({p["person"]: p["key"] for p in (api("sam", f"/v1/rail/people?namespace={GROUP_ID}") or {}).get("people", [])})
        summary["keys"] = keys

        # 8. The group's relay: the open one stopped and the relay started
        # again at its address over Sam's data, admitting only the group.
        TAKE[0] = "group"
        stop_relay(); g0 = time.time()
        summary["group"] = {"said": start_relay("group", data("sam"), False)}
        members = {n: keys[n] for n in ("Sam", "ana", "alex", "dee", "eve") if n in keys}
        first = {}
        def heard():
            for verdict, key, _ in relay_says("group"):
                name = next((n for n, k in keys.items() if k == key), key[:16])
                first.setdefault(f"{name} {verdict}", round(time.time() - g0, 1))
            return all(f"{n} admitted" in first for n in members) and "bo refused" in first
        wait(heard, 90, 1)
        summary["group"]["first"] = dict(first)
        say("group-relay", **summary["group"])
        w0 = time.time(); shutil.copy(MEDIA / "after2.jpg", folder("sam") / "after2.jpg")
        reached = {}
        def arrives2(who):
            t = wait(lambda: (folder(who) / "after2.jpg").is_file() and sha(folder(who) / "after2.jpg") == sha(MEDIA / "after2.jpg"), 60, 0.2)
            reached[who] = round(time.time() - w0, 2) if t else None
        ts = [threading.Thread(target=arrives2, args=(w,)) for w in ("ana", "alex", "dee", "eve", "bo")]
        [t.start() for t in ts]; [t.join() for t in ts]
        summary["group"]["after2_jpg_s"] = reached
        alex = browser("read", {"url": home_page("alex"), "until": MUSIC_LEND, "within": 60000, "t0": round(time.time() * 1000)})
        summary["group"]["alex_home"] = {k: alex.get(k) for k in ("at", "said", "stopped")}
        summary["group"]["cy"] = browser("phone", {"t0": round(time.time() * 1000), **cy, "again": True}, timeout=900)
        say("group-members", **{k: summary["group"][k] for k in ("after2_jpg_s", "alex_home", "cy")})

        # 9. Newcomers through it: Fay at the line, Gus on a phone.
        t0 = time.time()
        line = browser("line", {"link": LINK, "t0": round(t0 * 1000)})
        fay = {"line": {k: line.get(k) for k in ("shown", "stopped")}}
        if line.get("line"):
            try:
                run = subprocess.run(["sh", "-c", line["line"]], env=env("fay"), capture_output=True, text=True, timeout=300)
                fay.update(exit=run.returncode, said=redact(run.stdout + run.stderr))
            except subprocess.TimeoutExpired as e:
                fay.update(exit=None, said=redact(((e.stdout or b"").decode() if isinstance(e.stdout, bytes) else (e.stdout or "")) + "(stopped after 300 s)"))
            fay["ms"] = round((time.time() - t0) * 1000)
            fay["people_after"] = people_at("sam")
        (OUT / "fay-install.out").write_text(fay.get("said") or "")
        summary["group"]["fay"] = fay; say("fay", **fay)
        gus0 = time.time()
        summary["group"]["gus"] = browser("phone", {"t0": round(gus0 * 1000), **cy, "name": "Gus", "profile": str(W / "gus-phone")},
                                          timeout=900)
        summary["group"]["gus"]["people_after"] = people_at("sam")
        say("gus", **summary["group"]["gus"])
        summary["group"]["decisions"] = relay_says("group")
    finally:
        stop.set()
        summary["end"] = {"load": os.getloadavg()}
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1)) + "\n")
        subprocess.run(["launchctl", "bootout", f"gui/{UID}/{LABEL}"], capture_output=True)
        for name, p in procs.items():
            try: p.send_signal(signal.SIGCONT); p.terminate(); p.wait(15)
            except Exception: p.kill()
        for who in PEOPLE: subprocess.run(["pkill", "-f", str(binary(who))], capture_output=True)
        verdicts = {"full": "as many people as the lend's limit", "home": "door: guessed in this computer's home",
                    "let_in": "door: let in, and told"}
        summary["door"] = {who: {k: (data(who) / "svrngs.log").read_text(errors="replace").count(v) for k, v in verdicts.items()}
                           for who in ("sam", "ana") if (data(who) / "svrngs.log").exists()}
        traces = {who: data(who) / "svrngs.log" for who in PEOPLE} | {f"relay-{t}": W / f"relay-{t}.log" for t in ("open", "group")}
        summary["n0_in_traces"] = {who: [l[:200] for l in f.read_text(errors="replace").splitlines() if re.search(N0_WORDS, l)][:5]
                                   for who, f in traces.items() if f.exists()}
        summary["traced_lines"] = {who: len(f.read_text(errors="replace").splitlines()) for who, f in traces.items() if f.exists()}
        for t in ("open", "group"):
            if (W / f"relay-{t}.log").exists():
                (OUT / f"relay-{t}.log.gz").write_bytes(gzip.compress(redact((W / f"relay-{t}.log").read_text(errors="replace")).encode()))
        if "n0" in summary: summary["lsof"] = lsof_read()
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1)) + "\n")
        for who in PEOPLE:
            log = data(who) / "svrngs.log"
            if log.exists():
                keep = [l for l in log.read_text(errors="replace").splitlines()
                        if re.search(r"removal|refused|removed|door: |folder: |round membership|ERROR|WARN|cli: |not reached|cannot be reached", l)]
                (OUT / f"{who}.log").write_text(redact("\n".join(keep[-4000:])) + "\n")
        say("end", load=os.getloadavg())


if __name__ == "__main__":
    main()
