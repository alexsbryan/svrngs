#!/usr/bin/env python3
"""Live Upstash Free smoke test for the onboarding resolver.

Reads the token from Keychain (service svrngs-upstash, account alexsbryan)
and never prints it. Uses a unique test prefix and deletes it afterwards.
"""
import concurrent.futures
import json
import subprocess
import sys
import urllib.error
import urllib.request

URL = "https://true-pheasant-225442.upstash.io"
SERVICE = "svrngs-upstash"
ACCOUNT = "alexsbryan"
PREFIX = "svrngs:onboarding:proof:"


def token():
    return subprocess.check_output(
        ["security", "find-generic-password", "-s", SERVICE, "-a", ACCOUNT, "-w"],
        text=True,
    ).strip()


def call(commands):
    body = json.dumps(commands).encode()
    request = urllib.request.Request(
        URL,
        data=body,
        headers={
            "Authorization": f"Bearer {token()}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as answer:
            return answer.status, json.loads(answer.read())
    except urllib.error.HTTPError as error:
        return error.code, json.loads(error.read() or b"{}")


def one(*commands):
    status, answer = call(list(commands))
    if status != 200 or "error" in answer:
        raise RuntimeError(f"Upstash refused: status={status} answer={answer}")
    return answer.get("result")


CREATE = """
if redis.call('SET', KEYS[1], ARGV[1], 'NX') then return 1 else return 0 end
"""
CLOSE = """
if redis.call('GET', KEYS[1]) == ARGV[1] then
  redis.call('SET', KEYS[1], ARGV[2])
  return 1
else
  return 0
end
"""


def race(script, key, *args):
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        futures = [pool.submit(one, "EVAL", script, "1", key, *args) for _ in range(2)]
        return sorted(f.result() for f in futures)


def main():
    run = f"{PREFIX}{__import__('uuid').uuid4().hex}"
    created = f"{run}:create"
    closed = f"{run}:close"
    permanent = f"{run}:permanent"
    try:
        print("ping:", one("PING"))
        result = race(CREATE, created, "owner-a")
        print("create race:", result)
        assert result == [0, 1], "create-if-absent must acknowledge one owner"
        result = race(CLOSE, closed, "open", "closed")
        print("close race before create:", result)
        assert result == [0, 0], "close must not act on a missing key"
        assert one("SET", closed, "open") == "OK"
        result = race(CLOSE, closed, "open", "closed")
        print("close race:", result)
        assert result == [0, 1], "compare-and-close must acknowledge one close"
        assert one("SET", permanent, "reserved") == "OK"
        assert one("GET", permanent) == "reserved"
        ttl = one("TTL", permanent)
        print("permanent TTL:", ttl)
        assert ttl == -1, "a reservation must not expire"
        # A closed key stays closed; a retry is refused.
        assert one("EVAL", CLOSE, "1", closed, "open", "closed") == 0
        print("retry after close: refused")
        # An unknown command is an error the client can map to unavailable.
        status, answer = call(["NOPE"])
        print("unknown command status:", status, "error:", "error" in answer)
        assert status != 200 or "error" in answer
        print("live proof passed")
    finally:
        one("DEL", created, closed, permanent)
        print("cleaned up")


if __name__ == "__main__":
    sys.exit(main())
