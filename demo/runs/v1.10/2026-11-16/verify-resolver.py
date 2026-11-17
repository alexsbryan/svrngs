#!/usr/bin/env python3
"""Read back the resolver at join.svrnme.sh after its deploy: it reads
three words and a number (resolver/handler.mjs `read`), reaches its store,
and says what happened (README, "Each answer says what happened").

A never-issued phrase of the new shape answers 404 `missing`, which needs
both the new reading and a store call: the resolver before this one
answered such a phrase 503 `unavailable` (resolver-before.txt, read
2026-11-17T05:35Z), as a store it cannot reach still does. An unread word answers 404 `wrong` naming
the word, before any store call. Each answer is no-store and no-referrer.
Two lookups in all, one of them a miss against this network's budget.
The deployment serves its uploaded handler.mjs, store.mjs and words.txt
as files too (before this deploy, V1.9's bytes), so each is held equal to
the source's.

  python3 verify-resolver.py <source resolver dir> <record dir>   # writes resolver.json
  python3 verify-resolver.py --self-test
"""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import secrets
import sys
from urllib.error import HTTPError
from urllib.request import Request, urlopen

BASE = "https://join.svrnme.sh/"


def get(path, accept="application/json"):
    request = Request(BASE + path, headers={"Accept": accept, "User-Agent": "svrngs-release-readback"})
    try:
        with urlopen(request, timeout=30) as response:
            return response.status, dict(response.headers), response.read().decode()
    except HTTPError as error:
        return error.code, dict(error.headers), error.read().decode()


def judged(status, headers, body, want_status, want_state, want_words):
    """Why this answer is not the one wanted, or None."""
    if status != want_status:
        return f"status {status}, not {want_status}"
    lower = {k.lower(): v for k, v in headers.items()}
    if "no-store" not in lower.get("cache-control", ""):
        return f"cache-control {lower.get('cache-control')!r}"
    if lower.get("referrer-policy") != "no-referrer":
        return f"referrer-policy {lower.get('referrer-policy')!r}"
    try:
        answer = json.loads(body)
    except ValueError:
        return "not JSON"
    if answer.get("state") != want_state:
        return f"state {answer.get('state')!r}, not {want_state!r}"
    if want_words not in answer.get("message", ""):
        return f"message {answer.get('message')!r} lacks {want_words!r}"
    return None


GOOD = {"Cache-Control": "no-store", "Referrer-Policy": "no-referrer"}


def self_test():
    planted = [
        ("the old resolver's answer", 503, GOOD, '{"error":"unavailable","message":"joining is temporarily unavailable."}'),
        ("an unreachable store", 503, GOOD, '{"state":"unavailable","message":"Joining is temporarily unavailable"}'),
        ("a refused shape", 404, GOOD, '{"state":"wrong","message":"An invitation address is three words"}'),
        ("a cached answer", 404, {"Cache-Control": "max-age=60", "Referrer-Policy": "no-referrer"}, '{"state":"missing","message":"No invitation has this address"}'),
    ]
    for label, status, headers, body in planted:
        if judged(status, headers, body, 404, "missing", "No invitation") is None:
            raise SystemExit(f"instrument accepted {label}")
    if judged(404, GOOD, '{"state":"missing","message":"No invitation has this address."}', 404, "missing", "No invitation") is not None:
        raise SystemExit("instrument refused the wanted answer")
    print("Instrument plants rejected: the old resolver's answer, an unreachable store, a refused shape, a cached answer.")


if sys.argv[1:] == ["--self-test"]:
    self_test()
    raise SystemExit(0)

source, record = (Path(argument).resolve() for argument in sys.argv[1:])
self_test()
# Words from the resolver's own fixtures (web/tests/resolver-contracts.test.mjs),
# with a fresh number, so the phrase is all but certainly never issued.
phrase = f"acorn-basil-robin-{secrets.randbelow(10_000):04d}"
checks = {
    "missing": (phrase, 404, "missing", "No invitation"),
    "wrong": ("zzzzz-basil-robin-0042", 404, "wrong", "zzzzz"),
}
result = {"base": BASE, "checked": datetime.now(timezone.utc).isoformat(), "answers": {}}
failures = []
for name, (path, want_status, want_state, want_words) in checks.items():
    status, headers, body = get(path)
    why = judged(status, headers, body, want_status, want_state, want_words)
    # An answer other than the one wanted is not kept: for the fresh phrase
    # it could be a real invitation.
    result["answers"][name] = {"path": path, "status": status, "verdict": "passed" if why is None else "failed", **({"why": why} if why else {"body": json.loads(body)})}
    if why:
        failures.append(f"{name}: {why}")
result["files"] = {}
for name in ("handler.mjs", "store.mjs", "words.txt"):
    with urlopen(Request(BASE + name, headers={"User-Agent": "svrngs-release-readback"}), timeout=30) as response:
        live = hashlib.sha256(response.read()).hexdigest()
    built = hashlib.sha256((source / name).read_bytes()).hexdigest()
    result["files"][name] = {"live": live, "source": built, "verdict": "passed" if live == built else "failed"}
    if live != built:
        failures.append(f"{name}: live {live[:12]}, source {built[:12]}")
result["state"] = "failed" if failures else "passed"
(record / "resolver.json").write_text(json.dumps(result, indent=2) + "\n")
print(f"Resolver readback: {result['state']}")
if failures:
    raise SystemExit("; ".join(failures))
