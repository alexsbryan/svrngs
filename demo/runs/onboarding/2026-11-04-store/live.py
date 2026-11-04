#!/usr/bin/env python3
"""Live Upstash Free smoke test for the onboarding resolver.

Reads the token from Keychain (service svrngs-upstash, account alexsbryan)
and never prints it. Deletes unique fixture keys, retains the namespace's
permanent generation. Initialization is an explicit commissioning action.
"""
import concurrent.futures
import argparse
import json
from pathlib import Path
import subprocess
import sys
import urllib.error
import urllib.request

URL = "https://true-pheasant-225442.upstash.io"
SERVICE = "svrngs-upstash"
ACCOUNT = "alexsbryan"
PREFIX = "svrngs:onboarding:proof:"
HERE = Path(__file__).resolve().parent
GENERATION_KEY = "svrngs:{onboarding}:generation"
GENERATION = "onboarding-2026-11-04-v1"


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


def generation_proof(initialize):
    if initialize:
        # Explicit one-time commissioning only. Never call from a resolver or
        # automatically on restart; absent state must stop service, not heal it.
        result = one("SET", GENERATION_KEY, GENERATION, "NX")
        print("generation initialization:", "created" if result == "OK" else "already exists")
    assert one("GET", GENERATION_KEY) == GENERATION, "missing/mismatched generation; reconcile, do not reseed"
    assert one("PTTL", GENERATION_KEY) == -1, "generation must be permanent"
    print("live namespace generation: matched; TTL=-1")
    run = f"svrngs:{{onboarding}}:proof:{__import__('uuid').uuid4().hex}"
    keys = [f"{run}:reservation", f"{run}:secret", GENERATION_KEY]
    missing_key = f"{run}:missing-generation"
    guard = (HERE / "generation.lua").read_text()
    sources = {name: (HERE / f"{name}.lua").read_text() for name in ("create", "close", "lookup")}

    def operation(name, generation_key=GENERATION_KEY, expected=GENERATION):
        import time
        return one("EVAL", guard + sources[name], 3, *keys[:2], generation_key,
                   "owner-a/id-a/digest-a", "fixture-invite", int(time.time() * 1000) + 60000, expected)

    try:
        # Never delete the real generation to simulate reset in a live database.
        assert one("EXISTS", missing_key) == 0
        for name in ("create", "lookup", "close"):
            result = operation(name, missing_key)
            print(f"live missing generation: {name}={result}")
            assert result == "unavailable"
        assert one("EXISTS", *keys[:2], missing_key) == 0, "refusal changed store"
        for name in sources:
            assert operation(name, expected="wrong-generation") == "unavailable"
        print("live mismatched generation: all operations unavailable")
        assert operation("create") == "created"
        assert operation("lookup") == "fixture-invite"
        assert operation("close") == "closed"
        assert operation("create") == "closed"
        assert operation("lookup") == "ended"
        assert one("PTTL", keys[0]) == -1
        print("live guarded lifecycle: created, resolved, closed, replay refused; reservation TTL=-1")
    finally:
        one("DEL", *keys[:2])
        print("cleaned fixture keys; permanent namespace generation retained")
    print("live generation proof passed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--generation-proof", action="store_true")
    parser.add_argument("--initialize-generation", action="store_true")
    options = parser.parse_args()
    if options.initialize_generation and not options.generation_proof:
        parser.error("initialization requires --generation-proof")
    if options.generation_proof:
        generation_proof(options.initialize_generation)
        return
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
