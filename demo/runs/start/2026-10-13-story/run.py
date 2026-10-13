#!/usr/bin/env python3
"""The getting-started story (README.md here, §1): the page and tarball
built from this commit and served on this host, as
demo/runs/c/dry-2026-10-12-rerun/run.py serves them, every person a
program. Sam, with no link, by the founder's line, makes Lab on the home
page and lends llama-server; Bo by Sam's link and the one line, chatting in
Chromium and WebKit, a program at the address the home page copies, before
and after launchd restarts Bo's svrngs; what the lend keeps back; Cy, Dee
and Eve on phones, three at once, with no limit and then 2 set on Sam's
home page. Browsers are story.mjs's, the program program.py's.

    W=<scratch> python3 demo/runs/start/2026-10-13-story/run.py build
    PY=<venv with openai>/bin/python W=<scratch> python3 demo/runs/start/2026-10-13-story/run.py

The build is its own step, so neither outlasts one sitting of a shell; the
story refuses a site built at another commit. Data in data/ beside this
file, the group's token never in it. scripts/check.sh runs none of this."""
import hashlib, json, os, re, shutil, signal, subprocess, sys, tarfile, threading, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
OUT = HERE / "data"; SHOTS = OUT / "shots"; W = Path(os.environ["W"])
PY = os.environ.get("PY", "/tmp/sm-py/bin/python")
GROUP, LABEL, UID, TARBALL = "Lab", "sh.svrnme.svrngs", os.getuid(), "svrngs-darwin-aarch64.tar.gz"
PORT, MODEL = 18089, "Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M"
LEND, LEND_NAME = "Sam’s llama-server", "llama-server"
QUESTION, LONG = "Name three rivers in Spain.", "Tell a long story about a river and a mill."
LOG = "info,door=debug,limit=debug,home=debug,group=debug,transport=warn"
procs, summary, hidden = {}, {}, []


def head(): return subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()


def redact(text):
    for h in hidden: text = text.replace(h, "<the token>")
    return text


def say(event, **kw):
    line = redact(json.dumps({"t": round(time.time(), 3), "event": event, **kw}))
    print(line, flush=True)
    with open(OUT / "events.jsonl", "a") as f: f.write(line + "\n")


def ms(t0): return round((time.time() - t0) * 1000)


def wait(cond, bound, every=0.2):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def http(method, url, host=None, timeout=10):
    """(status, headers, body as JSON or text) of one request, or (None, {}, why)."""
    req = urllib.request.Request(url, method=method, headers={"Host": host} if host else {})
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r: status, hd, blob = r.status, dict(r.headers), r.read()
    except urllib.error.HTTPError as e: status, hd, blob = e.code, dict(e.headers), e.read()
    except Exception as e: return None, {}, str(e)
    try: return status, hd, json.loads(blob)
    except Exception: return status, hd, blob.decode(errors="replace")


# --- each person's computer: a HOME of its own -----------------------------
def home(who): return W / who
def data(who): return home(who) / ".svrngs"
def binary(who): return home(who) / ".local/bin/svrngs"
def env(who):
    e = {k: v for k, v in os.environ.items() if k != "SVRNGS_DIR"}
    return {**e, "HOME": str(home(who)), "USER": who.capitalize(), "PATH": f"{W / 'stub'}:{e['PATH']}", "NO_COLOR": "1"}


def base(who): return (data(who) / "api").read_text().strip().rstrip("/")
def home_page(who): return f"http://svrngs.localhost:{base(who).rsplit(':', 1)[1]}/"


def api(who, path):
    try: status, _, body = http("GET", base(who) + path)
    except Exception: return None
    return body if status == 200 else None


def opened(who):
    """(ms since t0, address) the `open` stub was last asked for, or Nones."""
    f = home(who) / "opened"
    return f.read_text().split() if f.exists() else [None, None]


def command(who, *args):
    """who types `svrngs <args>` in a terminal, by the path install.sh put it at."""
    t = time.time()
    run = subprocess.run([binary(who), *args], env={**env(who), "RUST_LOG": "error"}, capture_output=True, text=True)
    out = {"who": who, "typed": f"~/.local/bin/svrngs {' '.join(args)}", "exit": run.returncode,
           "said": redact((run.stdout + run.stderr).strip()), "ms": ms(t)}
    say("command", **out)
    return out


def take_over(who):
    """The one LaunchAgent label freed for the next person's line: who's
    svrngs booted out of launchd and run here as its LaunchAgent runs it,
    the bare command and its data dir (README §1)."""
    t = time.time()
    subprocess.run(["launchctl", "bootout", f"gui/{UID}/{LABEL}"], capture_output=True)
    wait(lambda: subprocess.run(["launchctl", "print", f"gui/{UID}/{LABEL}"], capture_output=True).returncode != 0, 30)
    before = home_page(who)
    procs[who] = subprocess.Popen([binary(who)], env={**env(who), "SVRNGS_DIR": str(data(who)), "RUST_LOG": LOG},
                                  stdin=subprocess.DEVNULL, stdout=open(data(who) / "svrngs.log", "a"), stderr=subprocess.STDOUT)
    if not wait(lambda: api(who, "/v1/groups"), 60): sys.exit(f"{who}'s svrngs did not come up again")
    say("taken-over", who=who, ms=ms(t), home_before=before, home_after=home_page(who))


def browser(mode, args, timeout=900):
    run = subprocess.run(["node", str(HERE / "story.mjs"), mode, json.dumps({"shots": str(SHOTS), **args})], cwd=REPO,
                         capture_output=True, text=True, timeout=timeout)
    try: got = json.loads(run.stdout.strip().splitlines()[-1])
    except Exception: got = {"stopped": f"story.mjs printed nothing: {run.stderr[-400:]}"}
    say(f"browser-{mode}", **{k: v for k, v in got.items() if k != "mode"})
    return got


def program(address, how):
    run = subprocess.run([PY, str(HERE / "program.py"), address, how], capture_output=True, text=True, timeout=180)
    try: got = json.loads(run.stdout.strip().splitlines()[-1])
    except Exception: got = {"stopped": f"program.py printed nothing: {run.stderr[-400:]}"}
    say("program", how=how, **got)
    return got


def line(who, shown):
    """The line as the page showed it, pasted in `sh` with who's HOME."""
    t = time.time()
    run = subprocess.run(["sh", "-c", shown], env=env(who), capture_output=True, text=True, timeout=300)
    (OUT / f"{who}-line.out").write_text(redact(run.stdout + run.stderr))
    said = (run.stdout + run.stderr).strip().splitlines()
    at = opened(who)
    out = {"exit": run.returncode, "ms": ms(t), "said": [redact(l) for l in said],
           "open_asked": at[1], "open_ms": int(at[0]) - round(t * 1000) if at[0] else None}
    say("line", who=who, **out)
    return out


def kept_back():
    row = next(l for l in (REPO / "crates/svrngs/src/well_known.txt").read_text().splitlines()
               if l.startswith("- llama-server keeps:"))
    return row.split(":", 1)[1].split()


def build():
    shutil.rmtree(W / "site", ignore_errors=True); W.mkdir(parents=True, exist_ok=True)
    built = {"head": head(), "dirty": subprocess.run(["git", "-C", str(REPO), "status", "--short"], capture_output=True, text=True).stdout.splitlines()}
    for script in ("build-web.sh", "build-release.sh"):
        t = time.time()
        r = subprocess.run([REPO / "scripts" / script, "--out", W / "site"], capture_output=True, text=True,
                           env={**os.environ, "CARGO_INCREMENTAL": "0"})
        (W / f"{script}.log").write_text(r.stdout + r.stderr)
        built[script] = {"exit": r.returncode, "s": round(time.time() - t), "last": (r.stdout + r.stderr).strip().splitlines()[-1:]}
    (W / "built.json").write_text(json.dumps(built, indent=1))
    print(json.dumps(built))
    sys.exit(0 if all(built[s]["exit"] == 0 for s in ("build-web.sh", "build-release.sh")) else 1)


def main():
    global PAGE
    if subprocess.run(["launchctl", "print", f"gui/{UID}/{LABEL}"], capture_output=True).returncode == 0:
        sys.exit(f"a svrngs is kept running on this host already ({LABEL}): this run would replace it")
    built = json.loads((W / "built.json").read_text())
    if built["head"] != head(): sys.exit(f"the site was built at {built['head']}, not at {head()}: build again")
    for who in ("sam", "bo", "stub"): shutil.rmtree(W / who, ignore_errors=True)
    shutil.rmtree(OUT, ignore_errors=True); SHOTS.mkdir(parents=True)
    (W / "stub").mkdir(parents=True)
    (W / "stub/open").write_text('#!/bin/sh\nprintf \'%s %s\\n\' "$(python3 -c \'import time;print(int(time.time()*1000))\')" "$1" >"$HOME/opened"\n')
    (W / "stub/open").chmod(0o755)
    llama = subprocess.run(["llama-server", "--version"], capture_output=True, text=True)
    say("start", head=head(), built=built, load=os.getloadavg(), cores=os.cpu_count(),
        macos=subprocess.run(["sw_vers", "-productVersion"], capture_output=True, text=True).stdout.strip(),
        llama=(llama.stdout + llama.stderr).strip().splitlines()[:1])
    try:
        # 0. The page host, and what it serves: the tarball against its sums.
        procs["page"] = subprocess.Popen(["node", REPO / "web/tests/serve.mjs", W / "site"], stdout=subprocess.PIPE, text=True)
        PAGE = f"http://localhost:{procs['page'].stdout.readline().split()[1]}/"
        threading.Thread(target=lambda: [None for _ in procs["page"].stdout], daemon=True).start()
        sums = http("GET", PAGE + "get/SHA256SUMS")[2]
        want = next(l.split()[0] for l in sums.splitlines() if l.split()[-1].lstrip("*") == TARBALL)
        blob = urllib.request.urlopen(PAGE + "get/" + TARBALL, timeout=60).read()
        summary["served"] = {"page": PAGE, "tarball": TARBALL, "bytes": len(blob),
                             "sum_matches": hashlib.sha256(blob).hexdigest() == want}
        say("served", **summary["served"])

        # 1. llama-server on Sam's computer.
        t = time.time()
        procs["llama"] = subprocess.Popen(["llama-server", "-hf", MODEL, "--host", "127.0.0.1", "--port", str(PORT),
                                           "-c", "4096", "--parallel", "2"], stdin=subprocess.DEVNULL,
                                          stdout=open(OUT / "llama.log", "a"), stderr=subprocess.STDOUT)
        if not wait(lambda: http("GET", f"http://127.0.0.1:{PORT}/health", timeout=2)[0] == 200, 900, 1):
            sys.exit("llama-server did not come up")
        say("llama", up_ms=ms(t))

        # 2. Sam, with no link: the page, the founder's line, the home page.
        t0 = time.time(); T = round(t0 * 1000)
        page = browser("page", {"url": PAGE, "name": "sam-page", "t0": T})
        if not (page.get("line") or "").startswith("curl "): sys.exit(f"the page with no link offered no line: {page}")
        sam_line = line("sam", page["line"])
        sam_home = sam_line["open_asked"] or home_page("sam")
        make = browser("make", {"home": sam_home, "group": GROUP, "person": "Sam", "t0": T})
        link = make.get("link") or ""
        m = re.search(r"#([A-Za-z0-9_-]+)$", link)
        if not m: sys.exit(f"Sam's home page made no link: {make}")
        hidden.append(m[1]); LINK = link
        groups = api("sam", "/v1/groups")["groups"]
        summary["sam"] = {"page": page, "line": sam_line, "make": make, "groups": [g.get("name") for g in groups],
                          "folder_is_dir": (home("sam") / "svrngs" / GROUP).is_dir(), "link_on_page": link.startswith(PAGE),
                          "origin_kept": (data("sam") / "origin").read_text() if (data("sam") / "origin").exists() else None}
        summary["sam"]["share"] = [command("sam", "share", str(PORT)),
                                   command("sam", "share", str(PORT), "--kind", "llama-server")]
        summary["sam"]["home"] = sam_home
        say("sam", **{k: summary["sam"][k] for k in ("groups", "folder_is_dir", "link_on_page", "origin_kept")})
        take_over("sam")

        # 3. Bo, by the link and the one line.
        t0 = time.time(); T = round(t0 * 1000)
        shown = browser("line", {"link": LINK, "t0": T})
        if not (shown.get("line") or "").startswith("curl "): sys.exit(f"Bo's page showed no line: {shown}")
        bo_line = line("bo", shown["line"])
        bo_home = bo_line["open_asked"] or home_page("bo")
        summary["bo"] = {"shown": shown, "line": bo_line, "home": bo_home}
        for engine in ("chromium", "webkit"):
            summary["bo"][f"chat_{engine}"] = browser("chat", {"engine": engine, "home": bo_home, "link": f"Open {LEND}",
                                                               "question": QUESTION, "t0": round(time.time() * 1000)})

        # 4. Bo's program at the address the home page copies; launchd
        # restarts Bo's svrngs; the same again.
        copy = browser("copy", {"home": bo_home, "link": f"Open {LEND}", "t0": round(time.time() * 1000)})
        address = copy.get("clipboard") or ""
        lends = {f"{l['person']}/{l['name']}": l["origin"] for l in (api("bo", "/v1/lends?namespace=" + groups[0]["id"]) or {}).get("lent", [])}
        summary["program"] = {"copy": copy, "lends": lends, "first": program(address, "first")}
        t = time.time()
        kick = subprocess.run(["launchctl", "kickstart", "-k", f"gui/{UID}/{LABEL}"], capture_output=True, text=True)
        up = wait(lambda: http("GET", bo_home, timeout=2)[0] == 200 and api("bo", "/v1/groups"), 60)
        up_ms = ms(t) if up else None
        lends_after = {f"{l['person']}/{l['name']}": l["origin"] for l in (api("bo", "/v1/lends?namespace=" + groups[0]["id"]) or {}).get("lent", [])}
        again = program(address, "once")
        st, _, body = http("GET", bo_home)
        summary["restart"] = {"kickstart": {"exit": kick.returncode, "said": (kick.stdout + kick.stderr).strip()},
                              "home_answers_ms": up_ms, "home_before": bo_home, "home_after": home_page("bo"),
                              "home_status": st, "home_title": (re.search(r"<title>([^<]*)", body or "") or [None, None])[1],
                              "lends_after": lends_after, "again": again, "answered_ms_since_restart": ms(t)}
        say("restart", **{k: v for k, v in summary["restart"].items() if k != "again"})

        # 5. What the lend keeps back, asked at Bo's address for it.
        lent_at = lends_after.get(f"Sam/{LEND_NAME}") or address
        hostport = lent_at.split("//")[1].rstrip("/")
        asked = {}
        for path in kept_back() + ["/health", "/props", "/v1/models"]:
            st, hd, body = http("GET", f"http://127.0.0.1:{hostport.rsplit(':', 1)[1]}{path}", host=hostport)
            asked[path] = {"status": st, "refused": hd.get("x-svrngs-refused"),
                           "said": body.get("error") if isinstance(body, dict) and isinstance(body.get("error"), str) else str(body)[:120]}
        summary["kept_back"] = asked
        say("kept-back", **asked)

        # 6. Cy, Dee and Eve on phones; three at once; 2 set on Sam's page.
        summary["phones"] = browser("phones", {"link": LINK, "lend": LEND, "lend_name": LEND_NAME, "question": QUESTION,
                                               "long": LONG, "window_ms": 35_000, "sam_home": home_page("sam"), "py": PY,
                                               "program": str(HERE / "program.py"), "address": address,
                                               "t0": round(time.time() * 1000)}, timeout=2400)
    finally:
        summary["end"] = {"load": os.getloadavg()}
        subprocess.run(["launchctl", "bootout", f"gui/{UID}/{LABEL}"], capture_output=True)
        for name, p in procs.items():
            try: p.terminate(); p.wait(15)
            except Exception: p.kill()
        verdicts = {"full": "as many people as the lend's limit", "home": "door: guessed in this computer's home",
                    "let_in": "door: let in, and told"}
        summary["door"] = {k: (data("sam") / "svrngs.log").read_text(errors="replace").count(v) for k, v in verdicts.items()} \
            if (data("sam") / "svrngs.log").exists() else None
        (OUT / "summary.json").write_text(redact(json.dumps(summary, indent=1, ensure_ascii=False)) + "\n")
        for who in ("sam", "bo"):
            log = data(who) / "svrngs.log"
            if log.exists():
                keep = [l for l in log.read_text(errors="replace").splitlines()
                        if re.search(r"door|limit|home|group|refused|ERROR|WARN", l)]
                (OUT / f"{who}.log").write_text(redact("\n".join(keep[-4000:])) + "\n")
        say("end", load=os.getloadavg())


if __name__ == "__main__":
    build() if sys.argv[1:] == ["build"] else main()
