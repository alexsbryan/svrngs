#!/usr/bin/env python3
"""E0's instrument: measured values for the svrngs-e0 bars, and the predicate.

  e0_demo.py value <bar>   last stdout line {"value": N, "commit": .., "artifact": ..}
  e0_demo.py predicate     exit 0 true, 1 false, 2 cannot judge

It reports values only. Floors and verdicts live in the campaign that reads it
(commonwealth-ai quality/campaigns/svrngs-e0.toml, via co-lineage.py), so a
threshold is decided in one place. Exit 3 means the thing a bar measures does
not exist yet: no recorded run, or no test. That reads as could-not-judge,
never as a pass or a failure.
"""
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RUNS = ROOT / "demo" / "runs"

# A people bar reads the newest recorded run of its demo (demo/README.md).
RUN_BARS = {"n-plus-one": "d1", "host-setup": "d2"}
# A structural bar reads one test, by the file that must exist and the command
# that runs it. None of them exists before its rung lands.
TEST_BARS = {
    "double-confirm": "crates/svrngs/tests/double_confirm.rs",
    "removal-latency": "crates/svrngs/tests/removal_latency.rs",
    "bridge-holds-no-app": "crates/svrngs/tests/bridge_holds_no_app.rs",
    "origin-gate": "crates/svrngs/tests/origin_gate.rs",
    "five-laws": "apps/expenses/laws.test.mjs",
    "plain-words": "crates/svrngs/tests/plain_words.rs",
}


def head():
    r = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT,
                       capture_output=True, text=True)
    return r.stdout.strip() or None


def newest_run(demo):
    runs = sorted((RUNS / demo).glob("*.json"))
    return (runs[-1], json.loads(runs[-1].read_text())) if runs else (None, None)


def run_value(demo, rec):
    """The fraction of people who did the thing unaided, or 0 if the run broke
    a condition the demo promises (an account, a key prompt, help)."""
    people = rec["participants"]
    if not people:
        return 0.0
    if rec.get("accounts_created", 0) or rec.get("key_prompts", 0) or rec.get("assisted"):
        return 0.0
    return rec["succeeded_in_time"] / people


def test_value(path):
    cmd = (["node", "--test", str(ROOT / path)] if path.endswith(".mjs")
           else ["cargo", "test", "--quiet", "--test", Path(path).stem])
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    return 1.0 if r.returncode == 0 else 0.0


def value(bar):
    if bar in RUN_BARS:
        path, rec = newest_run(RUN_BARS[bar])
        if rec is None:
            print(f"no recorded run in demo/runs/{RUN_BARS[bar]}/", file=sys.stderr)
            return 3
        v, artifact = run_value(RUN_BARS[bar], rec), str(path.relative_to(ROOT))
    elif bar in TEST_BARS:
        if not (ROOT / TEST_BARS[bar]).exists():
            print(f"{TEST_BARS[bar]} does not exist yet", file=sys.stderr)
            return 3
        v, artifact = test_value(TEST_BARS[bar]), TEST_BARS[bar]
    else:
        print(f"unknown bar {bar!r}", file=sys.stderr)
        return 2
    print(json.dumps({"value": v, "commit": head(), "artifact": artifact}))
    return 0


def predicate():
    """Five people settle through the bot unaided; the balances the bot posted
    equal the fold; the bridge imports no app code; nothing key-written renders
    into the chat. Each clause is read, never assumed."""
    path, rec = newest_run("d1")
    clauses = []
    if rec is None:
        clauses.append(("a recorded D1 run", None))
    else:
        settled = run_value("d1", rec) * rec["participants"]
        clauses.append(("four of five settled unaided", settled >= 4 and rec["participants"] == 5))
        clauses.append(("posted balances equal the fold", rec.get("balances_equal_fold")))
    for bar in ("bridge-holds-no-app", "origin-gate"):
        p = TEST_BARS[bar]
        clauses.append((bar, test_value(p) == 1.0 if (ROOT / p).exists() else None))
    for name, ok in clauses:
        print(f"{'TRUE ' if ok else 'FALSE' if ok is False else 'NO CLAIM'}  {name}")
    if any(ok is False for _, ok in clauses):
        return 1
    if any(ok is None for _, ok in clauses):
        return 2
    return 0


if __name__ == "__main__":
    args = sys.argv[1:]
    if args[:1] == ["value"] and len(args) == 2:
        sys.exit(value(args[1]))
    if args == ["predicate"]:
        sys.exit(predicate())
    print(__doc__, file=sys.stderr)
    sys.exit(2)
