#!/usr/bin/env python3
"""The dry run's one probe beyond demo/c.md, again (PREDICTIONS.md here,
10; demo/runs/c/dry-2026-10-12/name.py, its finding 1): a second computer
whose login is `ana` runs the line the group's page gives, for a group Ana
already stands in; and a member opens Ana's Navidrome, set as `svrngs share`
prints, through the lend. Sam and Ana are two data dirs on this host
running the binary run.py unpacked (W/sam/.local/bin/svrngs); the second
`ana` gets its own from the page, run.py's site (W/site) served again.

    W=<run.py's scratch> MEDIA=<run.py's media> python3 demo/runs/c/dry-2026-10-12-rerun/name.py

Prints one JSON line, also written to data/name.json, the token never in
it. Run after run.py, which leaves the one LaunchAgent label free."""
import json, os, re, shutil, signal, subprocess, sys, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent; REPO = HERE.parents[3]
W = Path(os.environ["W"]) / "name"; MEDIA = Path(os.environ["MEDIA"])
B = Path(os.environ["W"]) / "sam/.local/bin/svrngs"; SITE = Path(os.environ["W"]) / "site"
ND = "http://127.0.0.1:4534"; LABEL = "sh.svrnme.svrngs"; UID = os.getuid()
procs, out, hidden = [], {}, []


def wait(cond, bound, every=0.3):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def get(url, host=None, body=None, headers=None):
    req = urllib.request.Request(url, data=body and json.dumps(body).encode(),
                                 headers={"content-type": "application/json", **(headers or {}), **({"Host": host} if host else {})})
    try:
        with urllib.request.urlopen(req, timeout=10) as r: return r.status, r.read().decode(errors="replace")
    except urllib.error.HTTPError as e: return e.code, e.read().decode(errors="replace")
    except Exception as e: return None, str(e)


def env(who): return {**os.environ, "SVRNGS_DIR": str(W / who), "HOME": str(W / who), "NO_COLOR": "1", "RUST_LOG": "info"}
def api(who, path):
    try: s, b = get((W / who / "api").read_text().strip().rstrip("/") + path)
    except Exception: return None
    return json.loads(b) if s == 200 else None


def node(who, *args):
    (W / who).mkdir(parents=True, exist_ok=True)
    p = subprocess.Popen([B, *args], env=env(who), stdin=subprocess.DEVNULL, stdout=open(W / f"{who}.out", "w"),
                         stderr=open(W / f"{who}.log", "w"))
    procs.append(p)
    if not wait(lambda: api(who, "/v1/groups"), 60): sys.exit(f"{who} did not come up")


def main():
    if subprocess.run(["launchctl", "print", f"gui/{UID}/{LABEL}"], capture_output=True).returncode == 0:
        sys.exit(f"a svrngs is kept running on this host already ({LABEL}): this run would replace it")
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    try:
        page = subprocess.Popen(["node", REPO / "web/tests/serve.mjs", SITE], stdout=subprocess.PIPE, text=True)
        procs.append(page)
        PAGE = f"http://localhost:{page.stdout.readline().split()[1]}/"
        # Ana's Navidrome, set as run.py's `svrngs share 4533` printed.
        nd = W / "nd"; nd.mkdir()
        procs.append(subprocess.Popen(["navidrome"], env={**{k: v for k, v in os.environ.items() if not k.startswith("ND_")},
                     "ND_MUSICFOLDER": str(MEDIA / "music"), "ND_DATAFOLDER": str(nd), "ND_ADDRESS": "127.0.0.1", "ND_PORT": "4534",
                     "ND_EXTAUTH_USERHEADER": "X-Svrngs-User", "ND_EXTAUTH_TRUSTEDSOURCES": "127.0.0.1/32"},
                     stdout=open(nd / "nd.log", "w"), stderr=subprocess.STDOUT))
        wait(lambda: get(ND + "/ping")[0] == 200, 60)
        pw = os.urandom(12).hex()
        get(ND + "/auth/createAdmin", body={"username": "ana", "password": pw})
        node("sam", "new", "Lake", "--as", "Sam", "--origin", PAGE)
        link = wait(lambda: re.search(re.escape(PAGE) + r"#([A-Za-z0-9_-]+)", (W / "sam.out").read_text()), 60)
        hidden.append(link[1]); link = link[0]
        node("ana", "join", link, "--as", "ana")
        out["ana_joined"] = wait(lambda: re.search(r"You're in.*", (W / "ana.out").read_text()), 60)
        out["ana_joined"] = out["ana_joined"] and out["ana_joined"][0]
        group = api("sam", "/v1/groups")["groups"][0]["id"]
        wait(lambda: len((api("sam", f"/v1/rail/people?namespace={group}") or {}).get("people", [])) == 2, 30)

        # The second `ana`: the line the page gives, a HOME of its own, its
        # login `ana`, `open` stubbed.
        line = json.loads(subprocess.run(["node", HERE / "c.mjs", "line", link], cwd=REPO, capture_output=True,
                                         text=True).stdout.strip().splitlines()[-1])["line"]
        second = W / "second"; (second / "stub").mkdir(parents=True)
        (second / "stub/open").write_text('#!/bin/sh\necho "$1" >"$HOME/opened"\n'); (second / "stub/open").chmod(0o755)
        e = {k: v for k, v in os.environ.items() if k != "SVRNGS_DIR"}
        t = time.time()
        r = subprocess.run(["sh", "-c", line], env={**e, "HOME": str(second), "USER": "ana", "NO_COLOR": "1",
                                                    "PATH": f"{second / 'stub'}:{e['PATH']}"}, capture_output=True, text=True)
        out["second_at_the_line"] = {"exit": r.returncode, "ms": round((time.time() - t) * 1000),
                                     "said": (r.stdout + r.stderr).strip().splitlines(),
                                     "kept_running": subprocess.run(["launchctl", "print", f"gui/{UID}/{LABEL}"],
                                                                    capture_output=True).returncode == 0,
                                     "opened": (second / "opened").exists()}
        out["people"] = sorted(p["person"] for p in (api("sam", f"/v1/rail/people?namespace={group}") or {}).get("people", []))
        sam_user = [f"{p['person']}-{p['person_key'][:16]}" for p in api("sam", f"/v1/rail/people?namespace={group}")["people"]
                    if p["person"] == "Sam"]

        # A member, Sam, opens Ana's Navidrome through the lend.
        r = subprocess.run([B, "share", "4534", "--kind", "navidrome"], env=env("ana"), input="", capture_output=True, text=True)
        out["share"] = {"exit": r.returncode, "said": (r.stdout + r.stderr).strip()}
        origin = wait(lambda: next((l["origin"] for l in (api("sam", f"/v1/lends?namespace={group}") or {}).get("lent", [])
                                    if l["name"] == "navidrome"), None), 180, 2)
        if not origin: sys.exit("Sam's computer never listed Ana's navidrome")
        host = origin.split("//")[1].rstrip("/")
        s, html = get(f"http://127.0.0.1:{host.rsplit(':', 1)[1]}/app/", host=host)
        conf = re.search(r"window.__APP_CONFIG__ = (\"(?:[^\"\\]|\\.)*\")", html or "")
        auth = json.loads(json.loads(conf[1])).get("auth", {}) if conf else None
        out["sam_opens_navidrome"] = {"status": s, "signed_in_as": auth and auth.get("username"),
                                      "is_admin": auth and auth.get("isAdmin"), "x_svrngs_user": sam_user}
        tok = json.loads(get(ND + "/auth/login", body={"username": "ana", "password": pw})[1])["token"]
        s, users = get(ND + "/api/user", headers={"x-nd-authorization": f"Bearer {tok}"})
        out["navidrome_users"] = sorted(u["userName"] for u in json.loads(users)) if s == 200 else {"status": s, "said": users}
    finally:
        subprocess.run(["launchctl", "bootout", f"gui/{UID}/{LABEL}"], capture_output=True)
        for p in procs: p.send_signal(signal.SIGTERM)
        for p in procs:
            try: p.wait(10)
            except Exception: p.kill()
        text = json.dumps(out)
        hidden.extend(re.findall(r"svrngs://join/([A-Za-z0-9_-]+)", (W / "sam.out").read_text() if (W / "sam.out").exists() else ""))
        for h in hidden: text = text.replace(h, "<the token>")
        (HERE / "data").mkdir(exist_ok=True)
        (HERE / "data/name.json").write_text(text + "\n")
        print(text)


if __name__ == "__main__":
    main()
