#!/usr/bin/env python3
"""arch-helper (a): what Jellyfin's own helper, apps/jellyfin/mw, answers a
member who reaches it as a lend. The door's test
(crates/svrngs-node/tests/door/sign_in.rs,
a_sign_in_helper_is_never_offered_nor_lent_from_the_page) shows what the door
tells a lent helper when bo opens it (a-red.out): his own identity, every
x-svrngs- header he typed displaced, and no x-svrngs-stand-in. A stand-in
cannot say what the real helper does with that, so this asks it: scratch
Jellyfin on 8096, as v1ui's helper run made one (no films), the helper on 8097
with an API key, then the door's head as the test recorded it, and a GET.
Jellyfin's accounts are listed before and after. Everything on 127.0.0.1.

    W=/tmp/arch-helper python3 demo/runs/arch/2026-10-15-helper/mw.py
"""
import json, os, subprocess, sys, time, urllib.request, urllib.error
from pathlib import Path

REPO = Path(__file__).resolve().parents[4]
W = Path(os.environ.get("W", "/tmp/arch-helper"))
MW = REPO / "apps/jellyfin/mw/target/debug/jellyfin-mw"
JF_APP = Path("/Applications/Jellyfin.app/Contents")
JF, HELPER = "http://127.0.0.1:8096", "http://127.0.0.1:8097"
CLIENT = 'MediaBrowser Client="arch-helper", Device="run", DeviceId="arch-helper", Version="1"'
# What the door told the stand-in helper when bo opened it (a-red.out).
BO = "8139770ea87d175f56a35466c34c7ecccb8d8a91b4ee37a25df60f5b8fc9b394"
DOORS = {"x-svrngs-person": "bo", "x-svrngs-key": BO, "x-svrngs-user": "bo-8139770ea87d175f",
         "x-svrngs-device": BO, "x-svrngs-label": "umfrj1nxbj23n9mv5p0w3rv7rnghrcq9d0mbfh9wrvfadqasq7ra34qx2ccpnh",
         "x-svrngs-pages": "house=none", "content-length": "0"}


def http(method, url, body=None, headers=None):
    data = json.dumps(body).encode() if body is not None else (b"" if method == "POST" else None)
    req = urllib.request.Request(url, data=data, method=method, headers={"Content-Type": "application/json", **(headers or {})})
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()
    except OSError as e:
        return None, str(e)


def jf(method, path, body=None, token=None):
    status, text = http(method, JF + path, body, {"Authorization": CLIENT + (f', Token="{token}"' if token else "")})
    if status is None or status >= 400: sys.exit(f"Jellyfin {method} {path}: {status} {text[:200]}")
    return json.loads(text) if text.strip() else None


def main():
    for url in (JF + "/System/Info/Public", HELPER):
        if http("GET", url)[0] is not None: sys.exit(f"something answers at {url} already")
    subprocess.run(["cargo", "build", "--quiet", "--manifest-path", REPO / "apps/jellyfin/mw/Cargo.toml"], check=True)
    jw = W / "jf"
    for d in ("data", "config", "cache", "log"): (jw / d).mkdir(parents=True, exist_ok=True)
    (jw / "config/network.xml").write_text('<?xml version="1.0" encoding="utf-8"?>\n<NetworkConfiguration xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">\n  <EnableIPv6>false</EnableIPv6>\n  <LocalNetworkAddresses><string>127.0.0.1</string></LocalNetworkAddresses>\n  <EnableRemoteAccess>false</EnableRemoteAccess>\n</NetworkConfiguration>\n')
    procs = [subprocess.Popen([JF_APP / "MacOS/jellyfin", "-d", jw / "data", "-c", jw / "config", "-C", jw / "cache", "-l", jw / "log",
                               "-w", JF_APP / "Resources/jellyfin-web", "--ffmpeg", JF_APP / "MacOS/ffmpeg", "--nonetchange"],
                              stdout=open(jw / "stdout.log", "w"), stderr=subprocess.STDOUT)]
    try:
        ok, info = 0, {}
        for _ in range(120):
            status, text = http("GET", JF + "/System/Info/Public")
            ok = ok + 1 if status == 200 and "StartupWizardCompleted" in text else 0
            if ok >= 2: info = json.loads(text); break
            time.sleep(1)
        if ok < 2: sys.exit("Jellyfin did not come up")
        pw = os.urandom(12).hex()
        jf("POST", "/Startup/Configuration", {"UICulture": "en-US", "MetadataCountryCode": "US", "PreferredMetadataLanguage": "en"})
        jf("GET", "/Startup/User")
        jf("POST", "/Startup/User", {"Name": "priya", "Password": pw})
        jf("POST", "/Startup/RemoteAccess", {"EnableRemoteAccess": False, "EnableAutomaticPortMapping": False})
        jf("POST", "/Startup/Complete")
        time.sleep(3)
        token = jf("POST", "/Users/AuthenticateByName", {"Username": "priya", "Pw": pw})["AccessToken"]
        jf("POST", "/Auth/Keys?app=svrngs-mw", token=token)
        key = next(k["AccessToken"] for k in jf("GET", "/Auth/Keys", token=token)["Items"] if k["AppName"] == "svrngs-mw")
        users = lambda: sorted(u["Name"] for u in jf("GET", "/Users", token=token))
        print(json.dumps({"jellyfin": info.get("Version"), "users": users()}))
        mw = subprocess.Popen([MW, "--listen", "127.0.0.1:8097"], stdin=subprocess.PIPE, text=True,
                              stdout=open(W / "mw.out", "w"), stderr=open(W / "mw.log", "w"),
                              env={**os.environ, "NO_COLOR": "1", "RUST_LOG": "info"})
        procs.append(mw)
        mw.stdin.write(key + "\n"); mw.stdin.close()
        for _ in range(30):
            if "listening on port" in (W / "mw.out").read_text(): break
            time.sleep(1)
        else: sys.exit("the helper did not listen")
        for method, headers in (("POST", DOORS), ("GET", {})):
            status, text = http(method, HELPER + "/", None, headers)
            said = text.replace(key, "<the API key>").replace(token, "<priya's token>")
            print(json.dumps({"asked": f"{method} /", "status": status, "answer": said,
                              "holds the API key": key in text, "holds a token": token in text}))
        print(json.dumps({"users": users()}))
        print(json.dumps({"helper said": (W / "mw.log").read_text().replace(key, "<the API key>").splitlines()[-4:]}))
    finally:
        for p in reversed(procs):
            p.terminate()
            try: p.wait(15)
            except subprocess.TimeoutExpired: p.kill()


if __name__ == "__main__":
    main()
