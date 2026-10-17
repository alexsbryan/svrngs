"""arch-stream (a): one line per run and engine from run.sh's data.

    python3 -I summary.py [--retakes] data/<cell> [...]

Each: the load at start and end (1-minute), the assert the engine failed
(main.rs's line at fb5023bf), each step's status or error and its ms, the
page's last line, and, for the run, how often the relay set bo's key aside
("multiple connections found", iroh-relay's clients.rs:85-88).
"""

import json
from datetime import datetime
import re
import sys
from pathlib import Path

# main.rs at fb5023bf: the assert at each line, as a reader names it.
ASSERTS = {
    512: "range not 206",
    522: "range bytes",
    527: "range sha",
    533: "past not 416",
    541: "chunked not 200",
    542: "chunked text",
    551: "post not 200",
    552: "post held",
    556: "post text",
}
BO = "ea4a6c63e2"


def load(text, at):
    m = re.search(rf"^{at}: .*load averages?: ([\d.]+)", text, re.M)
    return m.group(1) if m else "?"


def main():
    for cell in map(Path, sys.argv[1:]):
        runs = sorted(cell.glob("*.out"), key=lambda p: int(p.stem))
        for out in runs:
            text = out.read_text(errors="replace")
            loads = (out.parent / f"{out.stem}.load").read_text()
            relay = (out.parent / f"{out.stem}.relay.log").read_text(errors="replace")
            aside = sum(
                1
                for l in relay.splitlines()
                if "multiple connections found" in l and BO in l
            )
            panics = dict(
                re.findall(r"streaming_both_ways::(\w+)' \(\d+\) panicked at [^:]+:(\d+)", text)
            )
            seen = {
                m.group(1): json.loads(m.group(2))
                for m in re.finditer(r"streaming::(\w+): (\{.*\})$", text, re.M)
            }
            for engine in ("chromium", "webkit"):
                failed = f"{engine} --- FAILED" in text or engine in panics
                line = panics.get(engine)
                verdict = (
                    f"red :{line} {ASSERTS.get(int(line), '?')}" if line
                    else ("red (no JSON)" if failed else "green")
                )
                http = seen.get(engine, {})
                steps = []
                for k in ("range", "past", "chunked", "post"):
                    s = http.get(k, {})
                    said = s.get("status") if "error" not in s else "ERR " + s["error"][:70]
                    steps.append(f"{k}={said}@{s.get('ms')}")
                last = (http.get("lines") or ["-"])[-1][:110]
                print(
                    f"{cell.name:>10} {out.stem:>2} {engine:8} load {load(loads, 'start')}->{load(loads, 'end')}"
                    f" aside {aside} | {verdict:28} | {' '.join(steps)} | last: {last}"
                )




def retakes():
    """Each run's times, from the relay's first setting bo's key aside (a
    driver binding), of every later one: a third or fourth is the page's own
    endpoint taking the key back."""
    for cell in map(Path, sys.argv[1:]):
        for log in sorted(cell.glob("*.relay.log"), key=lambda p: int(p.name.split(".")[0])):
            at = [
                datetime.fromisoformat(l[:26])
                for l in log.read_text(errors="replace").splitlines()
                if "multiple connections found" in l and BO in l
            ]
            print(f"{cell.name:>10} {log.name.split('.')[0]:>2} aside at +s: "
                  + " ".join(f"{(t - at[0]).total_seconds():.1f}" for t in at))


if sys.argv[1] == "--retakes":
    sys.argv.pop(1)
    retakes()
else:
    main()
