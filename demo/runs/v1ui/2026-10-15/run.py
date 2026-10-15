#!/usr/bin/env python3
"""The N+1 bar's steps on one host (PREDICTIONS.md here; row v1ui-run):
svrngs in release served as a newcomer gets it, the page served on this
host, a scratch Jellyfin and its middleware; Sam makes Elm Street, Priya
lends the library and brings Money with Gas bill on it; Ann, Bo and Cy
join by the chat's link on computers and each settles Gas bill and plays a
film; Di (a computer) and Eve (a phone) join; Sam's computer is stopped;
Di's steps, then Eve's through another person's computer; then every
computer's door, Money and record read. Browsers are people.mjs's.

    W=<scratch> python3 demo/runs/v1ui/2026-10-15/run.py                   # data/
    PLANT=accounts W=<scratch> python3 demo/runs/v1ui/2026-10-15/run.py    # data/plants/accounts/

FILMS=<dir> keeps the films between takes. The link's token, Jellyfin's
keys and passwords are never in the data. scripts/check.sh runs none of
this."""
import base64, json, os, plistlib, re, shutil, signal, subprocess, sys, threading, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
W = Path(os.environ["W"]); PLANT = os.environ.get("PLANT")
OUT = HERE / "data" / ("plants/" + PLANT if PLANT else ""); SHOTS = OUT / "shots"
FILMS = Path(os.environ.get("FILMS") or W / "films")
RELEASE = REPO / "target/release/svrngs"; MW = REPO / "apps/jellyfin/mw/target/debug/jellyfin-mw"
JF_APP = Path("/Applications/Jellyfin.app/Contents"); JF = "http://127.0.0.1:8096"
GROUP, TARBALL = "Elm Street", "svrngs-darwin-aarch64.tar.gz"
COMPUTERS, PHONE = ["Ann", "Bo", "Cy", "Di"], "Eve"
TITLES = {"Ann": "One", "Bo": "Two", "Cy": "Three", "Di": "One", "Eve": "Two"}
LOG = "info,door=debug,home=debug,cli=debug"
procs, hidden, summary = {}, [], {"people": {}, "plant": PLANT}


def head(): return subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()


def redact(text):
    for h in hidden: text = text.replace(h, "<hidden>")
    return text


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}, ensure_ascii=False))
    print(line[:400], flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def ms(t0): return round((time.time() - t0) * 1000)


def wait(cond, bound, every=0.1):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def http(method, url, body=None, headers=None, timeout=10):
    req = urllib.request.Request(url, method=method, data=None if body is None else json.dumps(body).encode(),
                                 headers={"content-type": "application/json", **(headers or {})})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r: status, blob = r.status, r.read()
    except urllib.error.HTTPError as e: status, blob = e.code, e.read()
    except Exception as e: return None, str(e)
    try: return status, json.loads(blob) if blob else None
    except Exception: return status, blob.decode(errors="replace")


# --- each computer: a HOME of its own, the stubs first on its PATH ----------
def home(who): return W / who.lower()
def data(who): return home(who) / ".svrngs"
def env(who):
    e = {k: v for k, v in os.environ.items() if k not in ("SVRNGS_DIR", "RUST_LOG")}
    return {**e, "HOME": str(home(who)), "USER": who, "PATH": f"{W / 'stub'}:{e['PATH']}", "NO_COLOR": "1"}
def api(who): return (data(who) / "api").read_text().strip().rstrip("/")
def home_page(who): return f"http://svrngs.localhost:{api(who).rsplit(':', 1)[1]}/"


def run_svrngs(who, binary, *args, out=None):
    """who's svrngs, run on: `binary args`, its words to `out`."""
    home(who).mkdir(parents=True, exist_ok=True)
    out = out or home(who) / "svrngs.out"
    procs[who] = subprocess.Popen([binary, *args], env={**env(who), "SVRNGS_DIR": str(data(who)), "RUST_LOG": LOG},
                                  stdin=subprocess.DEVNULL, stdout=open(out, "a"), stderr=open(W / f"{who.lower()}.log", "a"))
    return out


def stop(who):
    p = procs.pop(who)
    t = time.time()
    p.send_signal(signal.SIGINT)
    try: p.wait(20)
    except subprocess.TimeoutExpired: p.kill(); p.wait()
    say("stopped", who=who, exit=p.returncode, ms=ms(t))


def answers(who):
    try: return http("GET", api(who) + "/v1/groups", timeout=2)[0] == 200
    except Exception: return False


# --- the browsers ------------------------------------------------------------
browsers = None
lock = threading.Lock()


def tell(**c):
    with lock:
        browsers.stdin.write(json.dumps(c) + "\n"); browsers.stdin.flush()
        line = browsers.stdout.readline()
    got = json.loads(line) if line else {"error": "people.mjs stopped"}
    say(c["do"], who=c.get("who"), **{k: v for k, v in got.items()})
    return got


def shot(who, name, **kw): tell(do="shot", who=who, name=name, **kw)


# --- Jellyfin, as apps/jellyfin/check/check.sh `up` makes one ----------------
JF_CLIENT = 'MediaBrowser Client="v1ui-run", Device="Priya computer", DeviceId="v1ui-run-setup", Version="1"'


def jf(method, path, body=None, token=None):
    status, got = http(method, JF + path, body, {"Authorization": JF_CLIENT + (f', Token="{token}"' if token else "")}, timeout=30)
    if status is None or status >= 400: sys.exit(f"Jellyfin {method} {path}: {status} {str(got)[:200]}")
    return got


def films():
    FILMS.mkdir(parents=True, exist_ok=True)
    ff = JF_APP / "MacOS/ffmpeg"
    made = []
    for name, src, hz in (("One", "testsrc2", 440), ("Two", "testsrc", 660), ("Three", "smptebars", 550)):
        to = FILMS / f"Test Pattern {name} (2026).mp4"
        if to.exists() and to.stat().st_size: continue
        part = FILMS / f".part-{name}.mp4"
        made.append((to, part, subprocess.Popen([ff, "-v", "error", "-y", "-f", "lavfi", "-i", f"{src}=size=1280x720:rate=24", "-f", "lavfi",
                                                 "-i", f"sine=frequency={hz}:sample_rate=48000", "-t", "120", "-c:v", "libx264", "-preset", "veryfast",
                                                 "-b:v", "3M", "-g", "48", "-pix_fmt", "yuv420p", "-c:a", "aac", "-b:a", "128k", "-movflags", "+faststart",
                                                 "-f", "mp4", part])))
    for to, part, p in made:
        if p.wait() != 0: sys.exit(f"the film {to.name} was not made")
        part.rename(to)


def jellyfin():
    jw = W / "jf"
    for d in ("data", "config", "cache", "log"): (jw / d).mkdir(parents=True, exist_ok=True)
    (jw / "config/network.xml").write_text('<?xml version="1.0" encoding="utf-8"?>\n<NetworkConfiguration xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">\n  <EnableIPv6>false</EnableIPv6>\n  <LocalNetworkAddresses><string>127.0.0.1</string></LocalNetworkAddresses>\n  <EnableRemoteAccess>false</EnableRemoteAccess>\n</NetworkConfiguration>\n')
    procs["jellyfin"] = subprocess.Popen([JF_APP / "MacOS/jellyfin", "-d", jw / "data", "-c", jw / "config", "-C", jw / "cache", "-l", jw / "log",
                                          "-w", JF_APP / "Resources/jellyfin-web", "--ffmpeg", JF_APP / "MacOS/ffmpeg", "--nonetchange"],
                                         stdout=open(jw / "stdout.log", "w"), stderr=subprocess.STDOUT)
    # Kestrel answers before the app is ready: the document twice in a row.
    ok = 0
    for _ in range(120):
        status, info = http("GET", JF + "/System/Info/Public", timeout=2)
        ok = ok + 1 if status == 200 and "StartupWizardCompleted" in json.dumps(info) else 0
        if ok >= 2: break
        time.sleep(1)
    if ok < 2: sys.exit("Jellyfin did not come up")
    pw = base64.b32encode(os.urandom(15)).decode()
    hidden.append(pw)
    jf("POST", "/Startup/Configuration", {"UICulture": "en-US", "MetadataCountryCode": "US", "PreferredMetadataLanguage": "en"})
    jf("GET", "/Startup/User")
    jf("POST", "/Startup/User", {"Name": "priya", "Password": pw})
    jf("POST", "/Library/VirtualFolders?name=Movies&collectionType=movies&refreshLibrary=true",
       {"LibraryOptions": {"PathInfos": [{"Path": str(FILMS)}], "EnableRealtimeMonitor": False}})
    jf("POST", "/Startup/RemoteAccess", {"EnableRemoteAccess": False, "EnableAutomaticPortMapping": False})
    jf("POST", "/Startup/Complete")
    time.sleep(3)
    token = jf("POST", "/Users/AuthenticateByName", {"Username": "priya", "Pw": pw})["AccessToken"]
    hidden.append(token)
    jf("POST", "/Library/Refresh", token=token)
    n = wait(lambda: jf("GET", "/Items?Recursive=true&IncludeItemTypes=Movie", token=token)["TotalRecordCount"] >= 3 or None, 90, 1)
    if not n: sys.exit("Jellyfin did not list the three films")
    jf("POST", "/Auth/Keys?app=svrngs-mw", token=token)
    key = next(k["AccessToken"] for k in jf("GET", "/Auth/Keys", token=token)["Items"] if k["AppName"] == "svrngs-mw")
    hidden.append(key)
    users = sorted(u["Name"] for u in jf("GET", "/Users", token=token))
    say("jellyfin", version=info.get("Version"), films=sorted(f.name for f in FILMS.glob("*.mp4")), users=users)
    return token, key


def middleware(key):
    procs["mw"] = subprocess.Popen([MW, "--listen", "127.0.0.1:8097"], stdin=subprocess.PIPE, text=True,
                                   stdout=open(W / "mw.out", "w"), stderr=open(W / "mw.log", "w"),
                                   env={**os.environ, "NO_COLOR": "1", "RUST_LOG": "info,jellyfin_mw=debug"})
    procs["mw"].stdin.write(key + "\n"); procs["mw"].stdin.close()
    if not wait(lambda: "listening on port" in (W / "mw.out").read_text(), 30): sys.exit("the middleware did not listen")
    say("middleware", said=(W / "mw.out").read_text().strip().splitlines())


# --- launchd, played (PREDICTIONS.md §1) -------------------------------------
def launchd(who, line):
    """While the line runs: its `launchctl bootstrap <plist>` answered by
    starting what the plist names, as launchd would."""
    asked = home(who) / "launchctl"
    while line.poll() is None:
        if asked.exists() and who not in procs:
            words = asked.read_text().split()
            if "bootstrap" in words:
                plist = plistlib.loads(Path(words[-1]).read_bytes())
                args, dirs = plist["ProgramArguments"], plist["EnvironmentVariables"]["SVRNGS_DIR"]
                procs[who] = subprocess.Popen(args, env={**env(who), "SVRNGS_DIR": dirs, "RUST_LOG": LOG}, stdin=subprocess.DEVNULL,
                                              stdout=open(plist["StandardOutPath"], "a"), stderr=open(W / f"{who.lower()}.log", "a"))
                say("launchd", who=who, started=[redact(a) for a in args], asked=" ".join(words))
        time.sleep(0.02)


def line_run(who, shown):
    """The line as the page showed it, pasted in `sh` with who's HOME,
    nothing typed into it."""
    t = time.time()
    p = subprocess.Popen(["sh", "-c", shown], env=env(who), stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                         stderr=subprocess.STDOUT, text=True)
    launchd(who, p)
    said = p.stdout.read().strip().splitlines()
    opened = (home(who) / "opened").read_text().split() if (home(who) / "opened").exists() else [None, None]
    return {"exit": p.returncode, "ms": ms(t), "said": [redact(l) for l in said], "open_asked": opened[1]}


# --- a newcomer's steps -------------------------------------------------------
def computer_joins(who, link):
    me = summary["people"][who] = {"as": "computer", "film": TITLES[who]}
    tell(do="open", who=who, **{"as": "computer"})
    me["t0"] = t0 = time.time()
    got = tell(do="link", who=who, url=link)
    me["link"] = {"ms": got.get("ms"), "line": redact(got.get("line") or ""), "stopped": got.get("stopped"), "seen": got.get("seen")}
    if not got.get("line"): return me
    me["line"] = line_run(who, got["line"])
    me["line"]["at"] = ms(t0)
    return me


def computer_steps(who, t_issuer=None):
    me = summary["people"][who]; t0 = me["t0"]
    if not me.get("line") or me["line"]["exit"] != 0: return me
    me["home"] = {"at": ms(t0), **tell(do="home", who=who, url=me["line"]["open_asked"])}
    shot(who, f"{who.lower()}-home", home=True)
    me["money"] = {"at": ms(t0), **tell(do="money", who=who)}
    shot(who, f"{who.lower()}-money")
    me["money"]["done_at"] = ms(t0)
    me["library"] = {"at": ms(t0), **tell(do="library", who=who, film=TITLES[who])}
    shot(who, f"{who.lower()}-library")
    me["library"]["done_at"] = ms(t0)
    tell(do="back", who=who)
    return me


def phone_joins(link):
    me = summary["people"][PHONE] = {"as": "phone", "film": TITLES[PHONE]}
    tell(do="open", who=PHONE, **{"as": "phone"})
    me["t0"] = time.time()
    me["join"] = tell(do="join", who=PHONE, url=link, name=PHONE)
    shot(PHONE, "eve-joined")
    return me


def phone_steps(page):
    me = summary["people"][PHONE]; t0 = me["t0"]
    me["group"] = {"at": ms(t0), **tell(do="group", who=PHONE, url=page)}
    shot(PHONE, "eve-group")
    me["money"] = {"at": ms(t0), **tell(do="money", who=PHONE)}
    shot(PHONE, "eve-money")
    me["money"]["done_at"] = ms(t0)
    me["group_again"] = {"at": ms(t0), **tell(do="group", who=PHONE, url=page)}
    me["library"] = {"at": ms(t0), **tell(do="library", who=PHONE, film=TITLES[PHONE])}
    shot(PHONE, "eve-library")
    me["library"]["done_at"] = ms(t0)
    return me


# --- the run ---------------------------------------------------------------------
def build():
    e = {**os.environ, "CARGO_INCREMENTAL": "0"}
    subprocess.run(["cargo", "build", "--quiet", "--release", "--locked", "-p", "svrngs"], cwd=REPO, check=True, env=e)
    subprocess.run(["cargo", "build", "--quiet", "--manifest-path", REPO / "apps/jellyfin/mw/Cargo.toml"], check=True, env=e)
    subprocess.run([REPO / "scripts/build-web.sh", "--out", W / "site"], check=True, capture_output=True)
    get = W / "site/get"; get.mkdir(parents=True, exist_ok=True)
    subprocess.run(["tar", "--no-xattrs", "-czf", get / TARBALL, "-C", RELEASE.parent, "svrngs"], check=True, env={**os.environ, "COPYFILE_DISABLE": "1"})
    sums = subprocess.run(["shasum", "-a", "256", TARBALL], cwd=get, check=True, capture_output=True, text=True).stdout
    (get / "SHA256SUMS").write_text(sums)
    return sums.split()[0]


def main():
    global browsers
    for p in ("launchctl print gui/%d/sh.svrnme.svrngs" % os.getuid(),):
        if subprocess.run(p.split(), capture_output=True).returncode == 0: sys.exit("a svrngs is kept by launchd on this host already")
    if http("GET", JF + "/System/Info/Public", timeout=2)[0] is not None: sys.exit("something answers on 8096 already")
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    # A take clears its own data alone: the clean take keeps the planted's.
    for old in (OUT.iterdir() if OUT.exists() else []):
        if old.name != "plants": shutil.rmtree(old) if old.is_dir() else old.unlink()
    SHOTS.mkdir(parents=True)
    say("start", head=head(), plant=PLANT, load=round(os.getloadavg()[0], 1), cores=os.cpu_count(),
        dirty=subprocess.run(["git", "-C", REPO, "status", "--short"], capture_output=True, text=True).stdout.splitlines())
    try:
        summary["tarball_sha256"] = build()
        (W / "stub").mkdir()
        (W / "stub/launchctl").write_text('#!/bin/sh\nprintf \'%s\\n\' "$*" >>"$HOME/launchctl"\n')
        (W / "stub/open").write_text('#!/bin/sh\nprintf \'%s %s\\n\' "$(python3 -c \'import time;print(int(time.time()*1000))\')" "$1" >"$HOME/opened"\n')
        for s in ("launchctl", "open"): (W / "stub" / s).chmod(0o755)
        procs["page"] = subprocess.Popen(["node", REPO / "web/tests/serve.mjs", W / "site"], stdout=subprocess.PIPE, text=True)
        page = f"http://localhost:{re.search(r'port (\d+)', procs['page'].stdout.readline())[1]}/"
        threading.Thread(target=lambda: [None for _ in procs["page"].stdout], daemon=True).start()
        films()
        token, key = jellyfin()
        middleware(key)
        say("served", page=page)
        browsers = subprocess.Popen(["node", HERE / "people.mjs"], cwd=HERE, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                    stderr=open(W / "people.err", "w"), text=True, env={**os.environ, "SHOTS": str(SHOTS)})
        if not json.loads(browsers.stdout.readline()).get("ready"): sys.exit("people.mjs did not start")

        # Before anyone new (PREDICTIONS.md §1): Sam's group, Priya's lend and Money.
        made = run_svrngs("Sam", RELEASE, "new", GROUP, "--as", "Sam", "--origin", page)
        got = wait(lambda: re.search(rf"({re.escape(page)}#([A-Za-z0-9_-]+))", made.read_text()), 90)
        if not got: sys.exit(f"Sam's svrngs printed no link: {made.read_text()[-600:]}")
        link, client = got[1], re.search(r"svrngs://join/\S+", made.read_text())[0]
        hidden.extend([got[2], client.rsplit("/", 1)[1]])
        joined = run_svrngs("Priya", RELEASE, "join", client, "--as", "Priya")
        if not wait(lambda: "You're in" in joined.read_text(), 90): sys.exit(f"Priya did not join: {joined.read_text()[-400:]}")
        group = next(g for g in http("GET", api("Priya") + "/v1/groups")[1]["groups"] if g["name"] == GROUP)["id"]
        summary["group"] = group
        tell(do="open", who="Priya", **{"as": "computer"})
        tell(do="home", who="Priya", url=home_page("Priya"))
        if PLANT == "accounts":
            summary["lent"] = tell(do="lend", who="Priya", kind="jellyfin")
        else:
            lent = subprocess.run([RELEASE, "share", "8096", "--sign-in", "8097"], env={**env("Priya"), "SVRNGS_DIR": str(data("Priya"))},
                                  stdin=subprocess.DEVNULL, capture_output=True, text=True)
            summary["lent"] = {"exit": lent.returncode, "said": redact(lent.stdout + lent.stderr).strip().splitlines()}
            say("share", who="Priya", **summary["lent"])
        summary["brought"] = tell(do="bring", who="Priya")
        listed = lambda: any(r[1:2] == ["Money"] for r in tell(do="home", who="Priya", url=home_page("Priya")).get("index") or [])
        if not wait(listed, 60, 1): sys.exit("Money is never listed on Priya's home page")
        summary["gas_bill"] = tell(do="money", who="Priya", add={"what": "Gas bill", "amount": "60"})
        tell(do="back", who="Priya")
        shot("Priya", "priya-home", home=True)

        # The five, then Sam's computer stopped, then Di's steps and Eve's.
        for who in COMPUTERS[:3]:
            computer_joins(who, link); computer_steps(who)
        computer_joins("Di", link)
        phone_joins(link)
        summary["issuer_stopped_at"] = time.time()
        stop("Sam")
        computer_steps("Di")
        phone_steps(page)

        # After everyone: each computer still on, its door, Money and record.
        after = summary["after"] = {}
        for who in ["Priya", *COMPUTERS]:
            got = tell(do="home", who=who, url=home_page(who))
            shot(who, f"after-{who.lower()}", home=True)
            money = tell(do="money", who=who, read=True)
            tell(do="back", who=who)
            log = http("GET", f"{api(who)}/v1/rail/log?namespace={group}")[1]
            people = http("GET", f"{api(who)}/v1/rail/people?namespace={group}")[1]
            after[who] = {"door": got.get("door"), "index": got.get("index"), "money": money.get("shown", [None])[-1], "log": log, "people": people}
        summary["jellyfin_users"] = sorted(u["Name"] for u in jf("GET", "/Users", token=token))
    finally:
        if browsers:
            try: browsers.stdin.close(); browsers.wait(60)
            except Exception: browsers.kill()
        for p in procs.values(): p.send_signal(signal.SIGINT)
        for p in procs.values():
            try: p.wait(20)
            except Exception: p.kill()
        summary["end_load"] = round(os.getloadavg()[0], 1)
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, ensure_ascii=False, default=str)) + "\n")
        for f in ("people.err", "mw.log"):
            if (W / f).exists(): (OUT / f).write_text(redact((W / f).read_text())[-200_000:])
        say("end")


if __name__ == "__main__":
    main()
