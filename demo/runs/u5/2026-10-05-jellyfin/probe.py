"""u5-3's (c) on a lend holding a token, and (d), what zero sign-in would
take (README.md). Run by measure.sh `probe`, with the scratch Jellyfin,
Sam's computer and Alex's up.

1. The lender's key makes a Jellyfin user for a person, `zed`, as zero
   sign-in would make one per person: once with the lender's own token,
   once with an API key the lender makes (`/Auth/Keys`), the kind of key a
   lender would give once.
2. Sam's computer signs zed in with a device of the lend's own, and lends
   Jellyfin again as `keyed`, carrying zed's token: the door then signs
   whoever opens it in as zed, as it would sign each person in as theirs.
3. Alex's computer's browser opens `keyed`, typing nothing (drive.mjs
   `typed: false`): in Chromium and WebKit, once as it comes, and once with
   the placeholder stored sign-in U3's dry run wrote, playing the film
   Jellyfin converts and seeking it. Every response is dumped for scan.py,
   which searches it for zed's token: the token the lend holds.
4. Everything made is removed; Jellyfin's users are listed before, during
   and after.

env: SECRETS, JF, B (svrngs), R (scratch), OUT; and measure.sh's env file.
Writes OUT/probe.json and OUT/probe-<engine>-<how>.json.
"""
import json
import os
import re
import subprocess
import urllib.request
from pathlib import Path

S, JF, B, R, OUT = (Path(os.environ["SECRETS"]), os.environ["JF"], os.environ["B"], Path(os.environ["R"]), Path(os.environ["OUT"]))
ENV = dict(line.split("=", 1) for line in (R / "env").read_text().split())
HERE = Path(__file__).resolve().parent
ADMIN = (S / "admin_token").read_text().strip()


def auth(token=None, device="u5-3-setup"):
    a = f'MediaBrowser Client="u5-3 measure", Device="Sam computer", DeviceId="{device}", Version="1"'
    return a + (f', Token="{token}"' if token else "")


def call(method, url, body=None, headers=None):
    req = urllib.request.Request(url, method=method, data=json.dumps(body).encode() if body is not None else None,
                                 headers={"content-type": "application/json", **(headers or {})})
    try:
        with urllib.request.urlopen(req, timeout=20) as r:
            text = r.read().decode()
            return r.status, (json.loads(text) if text.strip().startswith(("{", "[")) else text)
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()[:200]


def jf(method, path, body=None, token=ADMIN):
    return call(method, JF + path, body, {"authorization": auth(token)})


def users():
    return sorted(u["Name"] for u in jf("GET", "/Users")[1])


def secret(name, value):
    p = S / name
    p.write_text(value)
    p.chmod(0o600)


report = {"users_before": users()}

# --- 1. a user per person, made with the lender's key ----------------------
st, zed = jf("POST", "/Users/New", {"Name": "zed"})
report["made_with_the_lenders_token"] = {"status": st, "is_admin": zed.get("Policy", {}).get("IsAdministrator") if isinstance(zed, dict) else None}
jf("POST", "/Auth/Keys?app=u5-3-probe")
key = next(k["AccessToken"] for k in jf("GET", "/Auth/Keys")[1]["Items"] if k["AppName"] == "u5-3-probe")
secret("probe_api_key", key)
st, yan = call("POST", JF + "/Users/New", {"Name": "yan"}, {"authorization": f'MediaBrowser Token="{key}"'})
report["made_with_an_api_key"] = {"status": st, "is_admin": yan.get("Policy", {}).get("IsAdministrator") if isinstance(yan, dict) else None}
report["users_during"] = users()

# --- 2. the lend carries zed's token -----------------------------------------
LEND = 'MediaBrowser Client="svrngs lend", Device="Sam computer", DeviceId="svrngs-lend-zed", Version="1"'
st, signed = call("POST", JF + "/Users/AuthenticateByName", {"Username": "zed", "Pw": ""}, {"authorization": LEND})
token = signed["AccessToken"]
secret("zed_lend_token", token)
share = subprocess.run([B, "share", "8096", "--name", "keyed", "--limit", "8"], input=f'{LEND}, Token="{token}"\n',
                       capture_output=True, text=True, env={**os.environ, "SVRNGS_DIR": str(R / "sam"), "NO_COLOR": "1"})
report["shared_keyed"] = [line for line in share.stdout.splitlines() if line.startswith("Lent")]
api = re.search(r"http://127\.0\.0\.1:\d+", (R / "alex/run.out").read_text()).group(0)
ns = call("GET", f"{api}/v1/groups")[1]["groups"][0]["id"]
origin = None
for _ in range(30):
    lent = call("GET", f"{api}/v1/lends?namespace={ns}")[1]["lent"]
    origin = next((l["origin"] for l in lent if l["name"] == "keyed"), None)
    if origin:
        break
    subprocess.run(["sleep", "1"])
report["keyed_opens_on_alexs_computer"] = bool(origin)
st, me = call("GET", origin + "Users/Me")
report["keyed_users_me"] = {"status": st, "name": me.get("Name") if isinstance(me, dict) else me}

# --- 3. Alex's browser on `keyed`, typing nothing ----------------------------
runs = {}
for engine in ("chromium", "webkit"):
    for how in ("as-it-comes", "placeholder"):
        person = {"name": "Zed", "user": "zed", "film": ENV["TWO"], "seeks": [50, 15], "typed": False}
        if how == "placeholder":
            person["seed"] = zed["Id"]
        dump = R / "dump" / f"keyed-{engine}-{how}"
        subprocess.run(["rm", "-rf", str(dump)])
        run = subprocess.run(["node", str(HERE / "drive.mjs"), json.dumps({"way": "computer", "engine": engine, "at": origin, "server": ENV["SERVER"], "people": [person]})],
                             capture_output=True, text=True, env={**os.environ, "DUMP": str(dump), "ADMIN": ADMIN}, timeout=300)
        (OUT / f"probe-{engine}-{how}.json").write_text(run.stdout)
        runs[f"{engine} {how}"] = subprocess.run(["python3", str(HERE / "summary.py"), "--line", str(OUT / f"probe-{engine}-{how}.json")],
                                                 capture_output=True, text=True).stdout.strip()
report["keyed_runs"] = runs
# The token the lend holds, after the runs: still alive, or logged out by one.
report["zed_token_alive_after"] = jf("GET", "/Users/Me", token=token)[0]

# --- 4. everything made, removed --------------------------------------------
subprocess.run([B, "unshare", "keyed"], capture_output=True, env={**os.environ, "SVRNGS_DIR": str(R / "sam")})
for u in (zed, yan):
    if isinstance(u, dict) and u.get("Id"):
        jf("DELETE", f"/Users/{u['Id']}")
jf("DELETE", f"/Auth/Keys/{key}")
report["users_after"] = users()
(OUT / "probe.json").write_text(json.dumps(report, indent=1) + "\n")
print(json.dumps(report, indent=1))
