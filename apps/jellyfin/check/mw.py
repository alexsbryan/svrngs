#!/usr/bin/env python3
"""mw-1's check (ralph/next/mw/STATE.md, its row): two people ask through
the lend of Jellyfin's middleware, each from a computer of their own, and
a process on the lender's computer asks its port with no key and with
another person's. Every byte each person receives is kept and searched.
Since mwp-3 the middleware is the lend's sign-in, which the door asks
`POST /` (apps/jellyfin/mw/src/answer.rs), and the device each person is
signed in on is theirs, named from their key (person.rs `Device::of`),
not the one the client names.

  mw.py <scratch> <out> <tag>

<scratch>/env names each person's address of the lend (ALEX_AT, BO_AT),
the middleware's port (MW_PORT) and its log (<scratch>/mw.log, where the
keys it saw are named); <scratch>/secrets holds the key the lender gave it,
the lender's password and token. Writes <out>/mw-<tag>.json, prints a
verdict per bar, and exits 0 only if every judged bar passed. What is
recorded and not judged (another person's key at the port) is printed as
`recorded`. When <scratch>/env names a library Sam made to keep back
(KEPT, check.sh up's KEEP_BACK), each person's libraries, as their own
address of the lend lists them, are judged to hold every one but it.
"""
import http.client
import json
import os
import re
import socket
import sys
import time

R, OUT, TAG = sys.argv[1:4]
ENV = dict(l.split("=", 1) for l in open(f"{R}/env").read().split() if "=" in l)
SECRETS = f"{R}/secrets"
JF = ("127.0.0.1", 8096)
ADMIN = open(f"{SECRETS}/admin_token").read().strip()
ADMIN_AUTH = f'MediaBrowser Client="mw-1 check", Device="Sam computer", DeviceId="mw1-lender", Version="1", Token="{ADMIN}"'
PEOPLE = [("Alex", ENV["ALEX_AT"]), ("Bo", ENV["BO_AT"])]
MW = ("127.0.0.1", int(ENV["MW_PORT"]))


def device(who):
    return f'MediaBrowser Client="mw-1 check", Device="{who} computer", DeviceId="mw1-{who.lower()}", Version="1"'


def ask(addr, host, method, path, headers=None, body=None):
    """One request: (status, headers as text, body bytes)."""
    c = http.client.HTTPConnection(*addr, timeout=60)
    h = {"Host": host, **(headers or {})}
    c.request(method, path, body=body, headers=h)
    r = c.getresponse()
    data = r.read()
    head = "".join(f"{k}: {v}\r\n" for k, v in r.getheaders())
    c.close()
    return r.status, head, data


def via(at, method, path, headers=None, body=None):
    """A request through a person's address of the lend: their bare
    command's loopback origin, `http://<label>.localhost:<port>/`."""
    m = re.match(r"http://([^/:]+):(\d+)/", at)
    return ask(("127.0.0.1", int(m.group(2))), f"{m.group(1)}:{m.group(2)}", method, path, headers, body)


def jf(path):
    s, _, b = ask(JF, "127.0.0.1:8096", "GET", path, {"Authorization": ADMIN_AUTH})
    if s != 200:
        raise SystemExit(f"mw.py: the lender's GET {path} answered {s}: {b[:200]!r}")
    return json.loads(b)


def users():
    return sorted((u["Name"], u["Id"]) for u in jf("/Users"))


def socket_through(at, who):
    """Jellyfin's WebSocket through the lend: the switch's answer, its
    first frame's text if it sends one within 10 s, and the socket, left
    open: Jellyfin ends a session when its socket closes (mw-1's first
    run read GET /Sessions after closing both and found neither person)."""
    m = re.match(r"http://([^/:]+):(\d+)/", at)
    s = socket.create_connection(("127.0.0.1", int(m.group(2))), timeout=10)
    s.sendall((f"GET /socket?deviceId=mw1-{who.lower()} HTTP/1.1\r\nHost: {m.group(1)}:{m.group(2)}\r\n"
               f"Upgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n"
               f"Sec-WebSocket-Version: 13\r\nAuthorization: {device(who)}\r\n\r\n").encode())
    got = b""
    try:
        while b"\r\n\r\n" not in got:
            chunk = s.recv(4096)
            if not chunk:
                break
            got += chunk
        head, _, rest = got.partition(b"\r\n\r\n")
        status = int(head.split(b" ")[1]) if head.startswith(b"HTTP/") else None
        frame = rest
        if status == 101:
            deadline = time.time() + 10
            while len(frame) < 2 or len(frame) < 2 + (frame[1] & 0x7F) and time.time() < deadline:
                chunk = s.recv(4096)
                if not chunk:
                    break
                frame += chunk
    except socket.timeout:
        status, head, frame = None, got, b""
    text = None
    if len(frame) >= 2 and (frame[1] & 0x7F) < 126:
        text = frame[2:2 + (frame[1] & 0x7F)].decode(errors="replace")
    return status, head.decode(errors="replace"), frame, text, s


def main():
    run = {"tag": TAG, "plant": os.environ.get("JELLYFIN_MW_PLANT") or None, "people": {}, "bars": {}}
    received = {}
    sockets = []
    run["users_before"] = users()
    for who, at in PEOPLE:
        p = run["people"][who] = {"at": at}
        got = received[who] = []
        before = users()
        s, h, b = via(at, "GET", "/Users/Me", {"Authorization": device(who)})
        got.append(h.encode() + b)
        p["first"] = {"status": s, "user": json.loads(b) if s == 200 else b.decode(errors="replace")[:300]}
        after_first = users()
        s, h, b = via(at, "GET", "/Users/Me", {"Authorization": device(who)})
        got.append(h.encode() + b)
        p["second"] = {"status": s, "user": json.loads(b) if s == 200 else b.decode(errors="replace")[:300]}
        after_second = users()
        p["made_by_first"] = sorted(set(after_first) - set(before))
        p["made_by_second"] = sorted(set(after_second) - set(after_first))
        # The libraries Jellyfin lists to them, asked through their lend.
        if p["first"]["status"] == 200:
            s, h, b = via(at, "GET", f"/UserViews?userId={p['first']['user']['Id']}", {"Authorization": device(who)})
            got.append(h.encode() + b)
            p["views"] = [v["Name"] for v in json.loads(b)["Items"]] if s == 200 else f"{s}: {b[:200]!r}"
        # A body sent: the session's capabilities, as jellyfin-web posts them.
        s, h, b = via(at, "POST", "/Sessions/Capabilities/Full",
                      {"Authorization": device(who), "Content-Type": "application/json"},
                      json.dumps({"PlayableMediaTypes": ["Video"], "SupportsMediaControl": False}).encode())
        got.append(h.encode() + b)
        p["body_sent"] = s
        # A body streamed back: a range of film One, as it is.
        s, h, b = via(at, "GET", f"/Videos/{ENV['ONE']}/stream?static=true",
                      {"Authorization": device(who), "Range": "bytes=0-65535"})
        got.append(h.encode() + b)
        p["range"] = {"status": s, "bytes": len(b)}
        # A request naming no device: jellyfin-web's page itself.
        s, h, b = via(at, "GET", "/web/")
        got.append(h.encode() + b)
        p["page"] = {"status": s, "bytes": len(b)}
        ws = socket_through(at, who)
        got.append(ws[1].encode() + ws[2])
        p["socket"] = {"status": ws[0], "first_frame": ws[3]}
        sockets.append(ws[4])
    run["users_after"] = users()
    run["sessions"] = sorted((s.get("UserName"), s.get("DeviceId"), s.get("DeviceName"), s.get("Client"))
                             for s in jf("/Sessions"))
    for s in sockets:
        s.close()

    # The middleware's own port, asked by a process on this computer as the
    # door asks it, its stand-in named as the door names it since si-1. What
    # it answers another's key is kept to whether a secret came back, never
    # the secret.
    s, h, b = ask(MW, f"localhost:{MW[1]}", "POST", "/")
    run["no_key"] = {"status": s, "body": b.decode(errors="replace")[:300]}
    keys = dict(re.findall(r"key=([0-9a-f]{64}) person=(\w+)", open(f"{R}/mw.log").read()))
    alex_key = next((k for k, n in keys.items() if n == "Alex"), None)
    if alex_key:
        s, h, b = ask(MW, f"localhost:{MW[1]}", "POST", "/", {"x-svrngs-key": alex_key, "x-svrngs-person": "Alex",
                                                          "x-svrngs-user": f"Alex-{alex_key[:16]}", "x-svrngs-device": alex_key,
                                                          "x-svrngs-stand-in": "svrngs-stand-in"})
        run["anothers_key"] = {"status": s, "answered_a_secret": s == 200 and "secret" in json.loads(b)}
    else:
        run["anothers_key"] = {"status": None, "user": "never-ran: Alex's key is not in the middleware's log"}
    run["sessions_after_anothers_key"] = sorted((s.get("UserName"), s.get("DeviceId"), s.get("DeviceName"))
                                                for s in jf("/Sessions"))

    # Every byte each person received, searched for the lender's secrets.
    secrets = {"the key the lender gave the middleware": open(f"{SECRETS}/mw_key").read().strip(),
               "the lender's token": ADMIN, "the lender's password": open(f"{SECRETS}/sam_pw").read().strip()}
    run["found"] = {who: [name for name, v in secrets.items() if any(v.encode() in g for g in got)]
                    for who, got in received.items()}
    run["received_bytes"] = {who: sum(len(g) for g in got) for who, got in received.items()}

    a, b_ = (run["people"][w] for w in ("Alex", "Bo"))
    ids = [p["first"]["user"].get("Id") if p["first"]["status"] == 200 else None for p in (a, b_)]
    bars = run["bars"]

    def bar(name, ok, why):
        bars[name] = {"verdict": "could-not-judge" if ok is None else ("passed" if ok else "failed"), "why": why}

    if None in ids:
        bar("each a user of their own", None, f"a first /Users/Me did not answer 200: {a['first']['status']}, {b_['first']['status']}")
    else:
        own = ids[0] != ids[1] and all(
            p["made_by_first"] and p["made_by_first"][0][1] == i and p["first"]["user"]["Policy"]["IsAdministrator"] is False
            for p, i in zip((a, b_), ids))
        bar("each a user of their own", own,
            f"Alex {a['first']['user']['Name']} ({ids[0]}), made by the first ask {a['made_by_first']}; "
            f"Bo {b_['first']['user']['Name']} ({ids[1]}), made by the first ask {b_['made_by_first']}")
    bar("a second ask makes none", all(not p["made_by_second"] and p["second"]["status"] == 200
                                       and p["second"]["user"].get("Id") == p["first"]["user"].get("Id")
                                       for p in (a, b_) if p["first"]["status"] == 200) and None not in ids,
        f"made by the second asks: Alex {a['made_by_second']}, Bo {b_['made_by_second']}")
    names = {w: run["people"][w]["first"]["user"].get("Name") for w in ("Alex", "Bo") if run["people"][w]["first"]["status"] == 200}
    devices = {w: sorted({d for u, d, _, _ in run["sessions"] if u == names.get(w)}) for w in ("Alex", "Bo")}
    theirs = {w: f"svrngs-{k[:16]}" for k, w in keys.items() if w in ("Alex", "Bo")}
    bar("each person's own device", len(theirs) == 2 and all(devices[w] == [theirs[w]] for w in ("Alex", "Bo")),
        f"Jellyfin's sessions by user: {devices}; each person's, named from their key: {theirs}")
    bar("neither receives the lender's key", not any(run["found"].values()),
        f"found: {run['found']}, in {run['received_bytes']} bytes")
    bar("no key is refused", run["no_key"]["status"] == 403 and "svrngs's door" in run["no_key"]["body"],
        f"{run['no_key']['status']}: {run['no_key']['body'][:120]!r}")
    bar("carried", all(p["body_sent"] == 204 and p["range"] == {"status": 206, "bytes": 65536}
                       and p["page"]["status"] == 200 and p["socket"]["status"] == 101 for p in (a, b_)),
        "; ".join(f"{w}: body sent {p['body_sent']}, range {p['range']}, page {p['page']}, socket {p['socket']}"
                  for w, p in run["people"].items()))
    if ENV.get("KEPT"):
        views = {w: p.get("views") for w, p in run["people"].items()}
        judged = all(isinstance(v, list) for v in views.values())
        bar(f"{ENV['KEPT']} kept back from each", None if not judged else
            all(ENV["KEPT"] not in v and "Movies" in v for v in views.values()),
            f"each person's libraries, as their lend lists them: {views}")
    run["recorded"] = {"another person's key at the middleware's port": run["anothers_key"]}

    with open(f"{OUT}/mw-{TAG}.json", "w") as f:
        json.dump(run, f, indent=1)
    for name, b in bars.items():
        print(f"{b['verdict']:<16}{name}: {b['why']}")
    print(f"recorded        another person's key at the middleware's port: {run['anothers_key']}")
    return 0 if all(b["verdict"] == "passed" for b in bars.values()) else 1


sys.exit(main())
