# usage: python3 -I use_n8n.py <port> <password-file>
# n8n's light use: set up the owner account (the first write any person
# makes), then create one workflow (a manual trigger and a Set node), saved
# inactive. Prints status codes and ids only, never the password or cookie.
import json
import sys
import urllib.request

port, passfile = sys.argv[1], sys.argv[2]
base = f"http://127.0.0.1:{port}"
password = "Aa1" + open(passfile).read().strip()  # n8n wants a digit and a capital
cookie = None


def parse(raw):
    try:
        return json.loads(raw or b"{}")
    except ValueError:
        return {"not_json": raw[:120].decode(errors="replace")}


def call(method, path, body=None):
    global cookie
    req = urllib.request.Request(base + path, method=method,
                                 data=json.dumps(body).encode() if body is not None else None,
                                 headers={"content-type": "application/json", "browser-id": "svrngs-churn"})
    if cookie:
        req.add_header("cookie", cookie)
    try:
        with urllib.request.urlopen(req) as r:
            for h in r.headers.get_all("set-cookie") or []:
                if h.startswith("n8n-auth="):
                    cookie = h.split(";", 1)[0]
            return r.status, parse(r.read())
    except urllib.error.HTTPError as e:
        return e.code, {"error": e.read()[:300].decode(errors="replace")}


s, _ = call("POST", "/rest/owner/setup", {"email": "churn@example.invalid", "firstName": "Churn",
                                           "lastName": "Test", "password": password})
print("owner setup", s, "cookie" if cookie else "no cookie")
wf = {"name": "churn workflow", "active": False, "settings": {}, "connections": {
          "When clicking 'Execute workflow'": {"main": [[{"node": "Set", "type": "main", "index": 0}]]}},
      "nodes": [
          {"parameters": {}, "name": "When clicking 'Execute workflow'", "type": "n8n-nodes-base.manualTrigger",
           "typeVersion": 1, "position": [0, 0], "id": "a0000000-0000-4000-8000-000000000001"},
          {"parameters": {"assignments": {"assignments": [{"id": "x1", "name": "hello", "value": "world",
                                                           "type": "string"}]}, "options": {}},
           "name": "Set", "type": "n8n-nodes-base.set", "typeVersion": 3.4, "position": [220, 0],
           "id": "a0000000-0000-4000-8000-000000000002"}]}
s, r = call("POST", "/rest/workflows", wf)
d = r.get("data", r)
print("create workflow", s, d.get("id") if isinstance(d, dict) else "", "" if s < 300 else r)
s, r = call("GET", "/rest/workflows")
print("list workflows", s, len(r.get("data", [])) if s < 300 else r)
