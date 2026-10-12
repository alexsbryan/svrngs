#!/usr/bin/env python3
"""The dry run's one probe beyond demo/c.md (README.md here, "A name is not
a person"): a second member who joins as `ana` — a name Ana's own is — opens
Ana's Navidrome, which takes the person from X-Svrngs-Person alone. Sam,
Ana and the second `ana` are three data dirs on this host running the
published binary run.py unpacked (W/sam/.local/bin/svrngs); no page.

    W=<run.py's scratch> MEDIA=<run.py's media> python3 demo/runs/c/dry-2026-10-12/name.py

Prints one JSON line, also written to data/name.json, the link never in it."""
import json, os, re, shutil, signal, subprocess, sys, time, urllib.error, urllib.request
from pathlib import Path

HERE = Path(__file__).resolve().parent
W = Path(os.environ["W"]) / "name"; MEDIA = Path(os.environ["MEDIA"])
B = Path(os.environ["W"]) / "sam/.local/bin/svrngs"; ND = "http://127.0.0.1:4534"
procs, out = [], {}


def wait(cond, bound, every=0.3):
    t0 = time.time()
    while time.time() - t0 < bound:
        got = cond()
        if got: return got
        time.sleep(every)
    return None


def get(url, host=None, body=None):
    req = urllib.request.Request(url, data=body and json.dumps(body).encode(),
                                 headers={"content-type": "application/json", **({"Host": host} if host else {})})
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
    shutil.rmtree(W, ignore_errors=True); W.mkdir(parents=True)
    try:
        nd = W / "nd"; nd.mkdir()
        procs.append(subprocess.Popen(["navidrome"], env={**os.environ, "ND_MUSICFOLDER": str(MEDIA / "music"), "ND_DATAFOLDER": str(nd),
                     "ND_ADDRESS": "127.0.0.1", "ND_PORT": "4534", "ND_EXTAUTH_USERHEADER": "X-Svrngs-Person",
                     "ND_EXTAUTH_TRUSTEDSOURCES": "127.0.0.1/32"}, stdout=open(nd / "nd.log", "w"), stderr=subprocess.STDOUT))
        wait(lambda: get(ND + "/ping")[0] == 200, 60)
        get(ND + "/auth/createAdmin", body={"username": "ana", "password": os.urandom(12).hex()})
        node("sam", "new", "Lake", "--as", "Sam")
        link = wait(lambda: re.search(r"svrngs://join/\S+", (W / "sam.out").read_text()), 60)[0]
        for who in ("ana", "second"):
            node(who, "join", link, "--as", "ana")
            out[f"{who}_joined"] = wait(lambda: re.search(r"You're in.*|svrngs: .*", (W / f"{who}.out").read_text() + (W / f"{who}.log").read_text()), 60)
            out[f"{who}_joined"] = out[f"{who}_joined"] and out[f"{who}_joined"][0]
        r = subprocess.run([B, "share", "4534", "--kind", "navidrome"], env=env("ana"), input="\n", capture_output=True, text=True)
        out["share_exit"] = r.returncode
        group = api("sam", "/v1/groups")["groups"][0]["id"]
        out["people"] = sorted(p["person"] for p in (api("sam", f"/v1/rail/people?namespace={group}") or {}).get("people", []))
        origin = wait(lambda: next((l["origin"] for l in (api("second", f"/v1/lends?namespace={group}") or {}).get("lent", [])
                                    if l["name"] == "navidrome"), None), 180, 2)
        if not origin: sys.exit("the second ana's computer never listed Ana's navidrome")
        host = origin.split("//")[1].rstrip("/")
        s, page = get(f"http://127.0.0.1:{host.rsplit(':', 1)[1]}/app/", host=host)
        conf = re.search(r"window.__APP_CONFIG__ = (\"(?:[^\"\\]|\\.)*\")", page or "")
        auth = json.loads(json.loads(conf[1])).get("auth", {}) if conf else None
        out["second_opens_navidrome"] = {"status": s, "signed_in_as": auth and auth.get("username"), "is_admin": auth and auth.get("isAdmin")}
    finally:
        for p in procs: p.send_signal(signal.SIGTERM)
        for p in procs:
            try: p.wait(10)
            except Exception: p.kill()
        text = json.dumps(out)
        for t in re.findall(r"svrngs://join/([A-Za-z0-9_-]+)", (W / "sam.out").read_text() if (W / "sam.out").exists() else ""):
            text = text.replace(t, "<the token>")
        (HERE / "data/name.json").write_text(text + "\n")
        print(text)


if __name__ == "__main__":
    main()
