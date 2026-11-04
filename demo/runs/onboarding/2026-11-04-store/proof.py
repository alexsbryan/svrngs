#!/usr/bin/env python3
"""Local Redis experiment; no external service, package, or product import.

Owns a foreground Redis child (no TCP listener) and always waits for its exit.
Plants use barriers/events to force the bad interleaving, rather than hoping
that load produces it. Exit 1 = invariant failed; 2 = instrument could not run.
"""

import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import threading
import time


HERE = Path(__file__).resolve().parent
SCRIPTS = {name: (HERE / f"{name}.lua").read_text() for name in ("create", "close")}


class Redis:
    def __init__(self, directory):
        self.directory = directory
        self.socket = str(Path(directory) / "redis.sock")
        self.process = None

    def start(self):
        self.process = subprocess.Popen([
            "redis-server", "--port", "0", "--unixsocket", self.socket,
            "--unixsocketperm", "700", "--dir", self.directory,
            "--appendonly", "yes", "--appendfsync", "always",
            "--save", "", "--maxmemory-policy", "noeviction",
        ], stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                raise RuntimeError("Redis exited during startup")
            try:
                if self.call("PING") == "PONG":
                    return
            except (RuntimeError, json.JSONDecodeError):
                pass
            time.sleep(0.02)
        raise RuntimeError("Redis startup exceeded 5 seconds")

    def stop(self):
        if self.process and self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()

    def call(self, *args):
        result = subprocess.run([
            "redis-cli", "-s", self.socket, "--json", *map(str, args),
        ], capture_output=True, text=True, timeout=5)
        if result.returncode:
            raise RuntimeError(result.stderr.strip())
        value = json.loads(result.stdout)
        if isinstance(value, dict) and "error" in value:
            raise RuntimeError(value["error"])
        return value

    def keys(self, phrase):
        return (f"proof:{{{phrase}}}:reservation", f"proof:{{{phrase}}}:secret")

    def operation(self, name, phrase, identity, secret="invite", expires=None):
        args = [identity]
        if name == "create":
            args += [secret, expires or int(time.time() * 1000) + 60000]
        return self.call("EVAL", SCRIPTS[name], 2, *self.keys(phrase), *args)


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def create_race(redis, planted):
    reservation, secret = redis.keys("create-race")
    barrier = threading.Barrier(2)

    def contender(identity):
        if planted:
            absent = redis.call("EXISTS", reservation) == 0
            barrier.wait(timeout=5)
            if absent:
                redis.call("HSET", reservation, "identity", identity, "state", "open")
                redis.call("SET", secret, identity)
                return "created"
            return "conflict"
        barrier.wait(timeout=5)
        return redis.operation("create", "create-race", identity)

    with ThreadPoolExecutor(2) as pool:
        results = list(pool.map(contender, ("owner-a/id-a/digest-a", "owner-b/id-b/digest-b")))
    print(f"create race: {sorted(results)}", flush=True)
    require(results.count("created") == 1, "create-if-absent acknowledged two owners")


def close_race(redis, planted):
    phrase, identity = "close-race", "owner-a/id-a/digest-a"
    reservation, secret = redis.keys(phrase)
    require(redis.operation("create", phrase, identity) == "created", "setup create")
    retry_read = threading.Event()
    closed = threading.Event()

    def delayed_retry():
        if planted:
            state = redis.call("HGET", reservation, "state")
            retry_read.set()
            require(closed.wait(5), "close did not run")
            if state == "open":
                redis.call("SET", secret, "invite")
                redis.call("HSET", reservation, "state", "open")
            return "replayed"
        retry_read.set()
        require(closed.wait(5), "close did not run")
        return redis.operation("create", phrase, identity)

    with ThreadPoolExecutor(1) as pool:
        retry = pool.submit(delayed_retry)
        require(retry_read.wait(5), "retry did not read")
        require(redis.operation("close", phrase, identity) == "closed", "close refused")
        closed.set()
        result = retry.result(timeout=5)
    state = redis.call("HGET", reservation, "state")
    payload = redis.call("GET", secret)
    print(f"close race: retry={result}, state={state}, secret_present={payload is not None}", flush=True)
    require(state == "closed" and payload is None, "delayed non-atomic retry reopened closed phrase")


def lifecycle(redis):
    phrase, identity = "lifecycle", "owner-a/id-a/digest-a"
    reservation, secret = redis.keys(phrase)
    expires = int(time.time() * 1000) + 1500
    require(redis.operation("create", phrase, identity, expires=expires) == "created", "first create")
    ttl = redis.call("PTTL", secret)
    require(redis.operation("create", phrase, identity) == "replayed", "lost-response retry")
    require(0 < redis.call("PTTL", secret) <= ttl, "retry extended secret lifetime")
    require(redis.operation("create", phrase, "other-owner") == "conflict", "replacement accepted")
    require(redis.operation("close", phrase, "other-owner") == "conflict", "stale close accepted")
    require(redis.call("GET", secret) == "invite", "stale close erased current invite")
    print("passed: idempotent create, immutable binding, no TTL refresh, compare-and-close", flush=True)
    deadline = time.monotonic() + 5
    while redis.call("EXISTS", secret) and time.monotonic() < deadline:
        time.sleep(0.02)
    require(redis.call("GET", secret) is None, "secret did not expire")
    require(redis.call("PTTL", reservation) == -1, "reservation expires")
    require(redis.operation("create", phrase, identity) == "expired", "expired replay reopened")
    require(redis.operation("create", phrase, "new-group") == "conflict", "expired phrase reused")
    print("passed: actual TTL expiry erases secret; permanent reservation refuses replay/reuse", flush=True)
    require(redis.operation("close", "close-first", identity) == "closed", "early close")
    require(redis.operation("create", "close-first", identity) == "closed", "late create reopened")
    require(redis.operation("close", "close-first", identity) == "closed", "close retry")
    require(redis.operation("create", "newer-phrase", "new-id") == "created", "newer invite")
    require(redis.operation("close", "newer-phrase", identity) == "conflict", "old close hit new invite")
    require(redis.call("GET", redis.keys("newer-phrase")[1]) == "invite", "new invite erased")
    require(redis.operation("create", "late-expired", identity, expires=1) == "expired", "backdated create")
    print("passed: close-before-create, duplicate close, stale close vs newer invitation, late expiry", flush=True)
    # The child uses appendfsync=always. SIGKILL avoids a graceful save masking
    # a missing persistent reservation. This is one local crash, not an HA claim.
    redis.process.kill()
    redis.process.wait(timeout=5)
    redis.start()
    require(redis.call("PTTL", reservation) == -1, "reservation lost on crash")
    require(redis.operation("create", phrase, "new-group") == "conflict", "reuse after crash")
    require(redis.operation("create", "close-first", identity) == "closed", "close lost on crash")
    print("passed: AOF reload after SIGKILL retains expired and closed reservations", flush=True)
    print(f"sample reservation MEMORY USAGE: {redis.call('MEMORY', 'USAGE', reservation)} bytes", flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("plant-create", "plant-close", "atomic"))
    options = parser.parse_args()
    if not all(shutil.which(name) for name in ("redis-server", "redis-cli")):
        print("could-not-judge: redis-server and redis-cli required")
        return 2
    print(subprocess.check_output(["redis-server", "--version"], text=True).strip(), flush=True)
    for name, source in SCRIPTS.items():
        print(f"{name}.lua sha256={hashlib.sha256(source.encode()).hexdigest()}", flush=True)
    with tempfile.TemporaryDirectory(prefix="store-proof-", dir="/tmp") as directory:
        redis = Redis(directory)
        try:
            redis.start()
            if options.mode == "plant-create":
                create_race(redis, True)
            elif options.mode == "plant-close":
                close_race(redis, True)
            else:
                create_race(redis, False)
                close_race(redis, False)
                lifecycle(redis)
        except AssertionError as error:
            print(f"failed: {error}", flush=True)
            return 1
        except (OSError, RuntimeError, subprocess.SubprocessError, json.JSONDecodeError) as error:
            print(f"could-not-judge: {error}", flush=True)
            return 2
        finally:
            redis.stop()
    print("passed: local atomic store proof", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
