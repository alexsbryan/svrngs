# Completes Jellyfin's first-run wizard over HTTP, adds the Movies library
# (default LibraryOptions), waits for the scan to finish, and writes
# state.json (token, ids, scan window). Throwaway identity only.
# usage: jf_setup.py BASE_URL STATE_OUT
import json, sys, time, urllib.request, urllib.parse

BASE, OUT = sys.argv[1].rstrip("/"), sys.argv[2]
AUTH = 'MediaBrowser Client="strm-exp", Device="strm-exp", DeviceId="strm-exp-setup", Version="1.0"'
USER, PW = "exp", "exp-throwaway-pw"


def call(method, path, body=None, token=None, q=None):
    url = BASE + path + ("?" + urllib.parse.urlencode(q, doseq=True) if q else "")
    data = json.dumps(body).encode() if body is not None else None
    hdr = {"Content-Type": "application/json",
           "Authorization": AUTH + (f', Token="{token}"' if token else "")}
    req = urllib.request.Request(url, data=data, method=method, headers=hdr)
    with urllib.request.urlopen(req, timeout=60) as r:
        raw = r.read()
        return json.loads(raw) if raw.strip() else None


def log(*a):
    print(time.strftime("%H:%M:%S"), *a, flush=True)


info = call("GET", "/System/Info/Public")
log("server", info["Version"], "wizard_done=", info["StartupWizardCompleted"])
if not info["StartupWizardCompleted"]:
    call("POST", "/Startup/Configuration", {"UICulture": "en-US", "MetadataCountryCode": "US",
                                             "PreferredMetadataLanguage": "en"})
    call("GET", "/Startup/User")
    call("POST", "/Startup/User", {"Name": USER, "Password": PW})
    call("POST", "/Startup/RemoteAccess", {"EnableRemoteAccess": True})
    call("POST", "/Startup/Complete")
    log("wizard completed")
auth = call("POST", "/Users/AuthenticateByName", {"Username": USER, "Pw": PW})
tok, uid = auth["AccessToken"], auth["User"]["Id"]
sid = auth["ServerId"]
log("authenticated user", uid)

import re
from datetime import datetime, timezone


def ts(iso):
    m = re.match(r"(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d)", iso or "")
    return datetime.fromisoformat(m.group(1)).replace(tzinfo=timezone.utc).timestamp() if m else 0


def scan_task():
    for t in call("GET", "/ScheduledTasks", token=tok, q={"isHidden": "false"}):
        if t.get("Key") == "RefreshLibrary":
            return t
    return None


def wait_idle():
    for _ in range(600):
        t = scan_task()
        if t and t["State"] == "Idle":
            return t
        time.sleep(1)
    sys.exit("scan task never went idle")


# Completing the wizard starts a scan of an EMPTY library; a library added
# while it runs is not picked up by it (observed on 12.1.0). Let it finish.
wait_idle()
log("pre-existing scan idle")
folders = call("GET", "/Library/VirtualFolders", token=tok)
scan_start = time.time()
if not any(f["Name"] == "Movies" for f in folders):
    call("POST", "/Library/VirtualFolders", {"LibraryOptions": {}}, tok,
         {"name": "Movies", "collectionType": "movies", "paths": "/media/Movies",
          "refreshLibrary": "true"})
    log("library added, refreshLibrary=true")
else:
    call("POST", "/Library/Refresh", token=tok)
    log("library existed; refresh triggered")

done_at, kicked = None, False
for i in range(600):
    t = scan_task()
    last = (t or {}).get("LastExecutionResult") or {}
    if t and t["State"] == "Idle" and ts(last.get("StartTimeUtc")) >= int(scan_start) - 1:
        done_at = time.time()
        log("scan ran", last.get("StartTimeUtc"), "->", last.get("EndTimeUtc"), last.get("Status"))
        break
    if i == 5 and t and t["State"] == "Idle" and not kicked:
        call("POST", "/Library/Refresh", token=tok)
        kicked = True
        log("no scan started within 5 s of the add; POST /Library/Refresh")
    time.sleep(1)
if done_at is None:
    sys.exit("scan did not finish in 600 s")

items = []
for _ in range(60):
    r = call("GET", "/Items", token=tok, q={"userId": uid, "Recursive": "true",
                                            "IncludeItemTypes": "Movie",
                                            "Fields": "Path,MediaSources,MediaStreams"})
    items = r["Items"]
    if len(items) >= 2:
        break
    time.sleep(1)
log("movies:", [(i["Name"], i.get("Path")) for i in items])
# Grace period so any post-scan work (providers, segment/keyframe passes)
# lands inside the scan window before the browsers start.
time.sleep(20)
scan_end = time.time()
state = {"base": BASE, "server_id": sid, "user_id": uid, "token": tok, "user": USER, "pw": PW,
         "version": info["Version"], "scan_window": [scan_start, scan_end],
         "scan_task_idle_at": done_at, "refresh_kicked": kicked,
         "items": {i["Name"]: {"id": i["Id"], "path": i.get("Path"),
                               "media_sources_pre_play": i.get("MediaSources")} for i in items}}
json.dump(state, open(OUT, "w"), indent=1)
log("state written", OUT)
