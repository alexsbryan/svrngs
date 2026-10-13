"""The home page walked as someone a bit afraid technically (README.md, row
start-home): start-measure (c)'s walk (../2026-10-13/run.py) again, on the
page at this commit, its setup trimmed to what the page shows. Each
newcomer act is tried on the page by the name of the control that would do
it, and said done there or not; those the page lacks are tried at a
terminal while svrngs runs, as it does after install.sh. Four svrngs
processes on this host, each with its own HOME and SVRNGS_DIR, and Eve's,
who joins by a link the page made, if it made one.

    W=<scratch> python3 demo/runs/start/2026-10-13-home/run.py <before|after>

Data in data/<before|after>/: events.jsonl, summary.json, each node's log
and what each command said, and shots/. A program named `open` (and
`xdg-open`) first on each svrngs's PATH writes what it was asked to open
to opened.txt and opens nothing, so no window opens on this host.
scripts/check.sh runs none of this."""
import http.client, json, os, re, shutil, signal, subprocess, sys, time, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
WHEN = sys.argv[1] if len(sys.argv) > 1 and sys.argv[1] in ("before", "after") else sys.exit("before or after?")
OUT = HERE / "data" / WHEN; SHOTS = OUT / "shots"; W = Path(os.environ.get("W", "/tmp/sh-w"))
PORT, MODEL = 18089, "Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M"
NIGHT, TASTE = 18091, 18092
LOG = "info,door=debug,origin=debug,home=debug,surface=debug,cli=debug,transport=warn"
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
                      "NO_COLOR": "1", "RUST_LOG": LOG, "PATH": f"{W / 'bin'}:{os.environ['PATH']}"}
def base(who): return (data(who) / "api").read_text().strip().rstrip("/")


def api(who, path):
    try:
        with urllib.request.urlopen(base(who) + path, timeout=5) as r: return json.loads(r.read())
    except Exception: return None


def up(who, *args):
    """svrngs on `who`'s computer, running; its words to data/<who>.out."""
    data(who).mkdir(parents=True, exist_ok=True)
    procs[who] = subprocess.Popen([W / "svrngs", *args], env=env(who), stdin=subprocess.DEVNULL,
                                  stdout=open(OUT / f"{who}.out", "a"), stderr=open(OUT / f"{who}.log", "a"))
    if not wait(lambda: (data(who) / "api").exists() and api(who, "/v1/groups") is not None, 60):
        sys.exit(f"{who}'s svrngs did not come up")
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
    return {"exit": run.returncode, "said": said}


def lends(who):
    groups = (api(who, "/v1/groups") or {}).get("groups") or [{}]
    return api(who, f"/v1/lends?namespace={groups[0].get('id')}") or {}


def origin_of(who, name):
    return next((l["origin"] for l in lends(who).get("lent", []) if l["name"] == name), None)


def label(origin): return origin.split("//")[1].split(".")[0]


def home(who): return f"http://svrngs.localhost:{base(who).rsplit(':', 1)[1]}/"


def walk(state, url, click=None):
    got = {}
    for which in ("chromium", "webkit"):
        shot = SHOTS / f"{state}-{which}.png"
        argv = ["node", str(HERE / "walk.mjs"), which, url, str(shot), *([click] if click else [])]
        r = subprocess.run(argv, capture_output=True, text=True, timeout=180)
        try: got[which] = json.loads(r.stdout.strip().splitlines()[-1])
        except Exception: got[which] = {"error": (r.stderr or r.stdout)[-400:]}
        say("walk", state=state, click=click, **{k: v for k, v in got[which].items() if k not in ("text", "after", "tab")},
            text=got[which].get("text", "")[:300], after=str(got[which].get("after"))[:300])
    summary.setdefault("walk", {})[state] = got
    return got


def main():
    shutil.rmtree(W, ignore_errors=True); (W / "bin").mkdir(parents=True)
    shutil.rmtree(OUT, ignore_errors=True); OUT.mkdir(parents=True); SHOTS.mkdir()
    shutil.copy(REPO / "target/debug/svrngs", W / "svrngs")
    for opener in ("open", "xdg-open"):
        stub = W / "bin" / opener
        stub.write_text(f'#!/bin/sh\necho "{opener} $*" >> "{W}/opened.txt"\n'); stub.chmod(0o755)
    head = subprocess.run(["git", "-C", str(REPO), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    say("start", when=WHEN, host=host(), head=head)
    try:
        procs["llama"] = subprocess.Popen(["llama-server", "-hf", MODEL, "--host", "127.0.0.1", "--port", str(PORT),
                                           "-c", "4096", "--parallel", "2"], stdin=subprocess.DEVNULL,
                                          stdout=open(OUT / "llama.log", "a"), stderr=subprocess.STDOUT)

        def healthy():
            try:
                c = http.client.HTTPConnection("127.0.0.1", PORT, timeout=2); c.request("GET", "/health")
                return c.getresponse().status == 200
            except Exception: return False
        if not wait(healthy, 900, 1): sys.exit("llama-server did not come up")
        for port, name in ((NIGHT, "night"), (TASTE, "taste")):
            page = W / name; page.mkdir()
            (page / "index.html").write_text(f"<!doctype html><title>{name}</title><h1>{name}</h1>")
            procs[name] = subprocess.Popen([sys.executable, "-m", "http.server", str(port), "--bind", "127.0.0.1",
                                            "--directory", str(page)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

        # The group: Sam makes it; Bo is put on as install.sh puts a
        # computer on; Cy joins. Sam lends llama-server as start-llama
        # lends it, with nothing kept back typed.
        up("sam", "new", "Lab", "--as", "Sam")
        link = wait(lambda: re.search(r"svrngs://join/\S+", (OUT / "sam.out").read_text()), 60)[0]
        summary["bo_join"] = command("bo", "join", link, "--as", "Bo", "--no-run")
        up("bo")
        up("cy", "join", link, "--as", "Cy")
        if not wait(lambda: "You're in" in (OUT / "cy.out").read_text(), 60): sys.exit("cy did not join")
        summary["shares"] = {
            "llama-server": command("sam", "share", str(PORT), "--kind", "llama-server", "--limit", "4"),
            "night": command("sam", "share", str(NIGHT), "--name", "night", "--kind", "night"),
            "taste": command("cy", "share", str(TASTE), "--name", "taste", "--kind", "taste",
                             "--ask", "the films you have marked"),
        }
        names = {"llama-server", "night", "taste"}
        if not wait(lambda: names <= {l["name"] for l in lends("bo").get("lent", [])}, 180, 1):
            sys.exit(f"Bo's computer never listed all three: {lends('bo')}")

        # One group with lends: Bo's page, each act tried on it by name.
        walk("lends-bo", home("bo"), click="Sam’s llama")
        walk("invite-bo", home("bo"), click="invite someone")
        walk("device-bo", home("bo"), click="phone or another computer")
        walk("folder-bo", home("bo"), click="folder")
        walk("copy-bo", home("bo"), click="Copy")
        summary["opened"] = (W / "opened.txt").read_text() if (W / "opened.txt").exists() else None
        say("opened", said=summary["opened"])
        # The link the page made, if it made one, joined by Eve.
        made = re.search(r"svrngs://join/\S+", summary["walk"]["invite-bo"]["chromium"].get("after", {}).get("text", "") or "")
        summary["eve_join"] = command("eve", "join", made[0], "--as", "Eve", "--no-run") if made else None
        say("eve", joined=summary["eve_join"])
        walk("lends-sam", home("sam"))

        # What the page lacks, at a terminal on Bo's computer while its
        # svrngs runs, as it runs after install.sh.
        summary["terminal"] = {
            "link": command("bo", "link", "Lab"),
            "link --device": command("bo", "link", "Lab", "--device"),
            "new": command("bo", "new", "Second", "--as", "Bo"),
        }

        # An app asking: Sam's night asks to use Cy's taste, on Bo's page.
        taste, night = origin_of("bo", "taste"), origin_of("bo", "night")
        walk("asking-bo", f"{home('bo')}ask?lend={label(taste)}&app={label(night)}&back=/", click="Yes")
        walk("after-yes-bo", home("bo"))

        # A computer asleep: Cy's stopped.
        stop("cy")
        wait(lambda: "Cy" in lends("bo").get("unanswered", []), 60, 1)
        walk("asleep-bo", home("bo"))

        # The restart, as launchd restarts it: the page found again.
        old_home = home("bo")
        stop("bo")
        up("bo")
        summary["restart"] = {"before": old_home, "after": home("bo")}
        walk("restart-bo", old_home)

        # Removed: Sam removes Bo.
        summary["remove"] = command("sam", "remove", "Bo", "--group", "Lab")
        wait(lambda: (api("bo", "/v1/groups") or {"groups": [{}]})["groups"][0].get("removed") is True, 120, 1)
        walk("removed-bo", home("bo"))

        # No group yet: Dee, the bare command over an empty data dir.
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
