# usage: python3 -I use_owui.py <port> <password-file>
# Open WebUI's light use: the first account (it becomes admin), one chat saved
# with a user message and no model, and one settings change. Prints status
# codes and ids only, never the password or token.
import json
import sys
import urllib.request

port, passfile = sys.argv[1], sys.argv[2]
base = f"http://127.0.0.1:{port}"
password = open(passfile).read().strip()
token = None


def parse(raw):
    try:
        return json.loads(raw or b"{}")
    except ValueError:
        return {"not_json": raw[:120].decode(errors="replace")}


def call(method, path, body=None):
    req = urllib.request.Request(base + path, method=method,
                                 data=json.dumps(body).encode() if body is not None else None,
                                 headers={"content-type": "application/json"})
    if token:
        req.add_header("authorization", "Bearer " + token)
    try:
        with urllib.request.urlopen(req) as r:
            return r.status, parse(r.read())
    except urllib.error.HTTPError as e:
        return e.code, {"error": e.read()[:300].decode(errors="replace")}


s, r = call("POST", "/api/v1/auths/signup", {"name": "Churn", "email": "churn@example.invalid",
                                              "password": password})
token = r.get("token")
print("signup", s, "role", r.get("role"), "token" if token else "no token")
msg = {"id": "m1", "role": "user", "content": "hello from the churn test", "timestamp": 0}
s, r = call("POST", "/api/v1/chats/new", {"chat": {"title": "churn chat", "models": [],
                                                    "messages": [msg], "history": {"messages": {"m1": msg},
                                                                                   "currentId": "m1"}}})
print("new chat", s, r.get("id") if s < 300 else r)
s, r = call("POST", "/api/v1/users/user/settings/update", {"ui": {"theme": "dark"}})
print("settings", s, "" if s < 300 else r)
s, r = call("GET", "/api/v1/chats/")
print("list chats", s, len(r) if isinstance(r, list) else r)
