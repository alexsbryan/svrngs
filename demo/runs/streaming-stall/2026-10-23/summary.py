"""streaming-stall (a): one line per run and engine from run.sh's data.

    python3 -I summary.py data/<cell> [...]

Each: the load at start and end (1-minute), the range's status or error,
its ms and what the page had drained when it timed out (drive.mjs `drain`),
the other steps' ms; then, for the run, every door line for the 9 MB range's
end (`carriage`, door.rs `Carriage`), which the log cannot attribute to an
engine, with how far the door had sent it.
"""

import json
import re
import sys
from pathlib import Path


def load(text, at):
    m = re.search(rf"^{at}: .*load averages?: ([\d.]+)", text, re.M)
    return m.group(1) if m else "?"


def step(s):
    if s is None:
        return "never-ran"
    if "error" in s:
        d = s.get("drained")
        had = f" drained {d['bytes']} B in {d['chunks']} chunks, last at {d['lastMs']} ms" if d else ""
        return f"{s['error']}@{s['ms']}{had}"
    return f"{s['status']}@{s['ms']}"


def main():
    for cell in map(Path, sys.argv[1:]):
        for out in sorted(cell.glob("*.out"), key=lambda p: int(p.stem)):
            n = out.stem
            text = out.read_text(errors="replace")
            lt = (cell / f"{n}.load").read_text(errors="replace")
            loads = f"load {load(lt, 'start')}->{load(lt, 'end')}"
            seen = {}
            for m in re.finditer(r"streaming::(\w+): (\{.*)$", text, re.M):
                try:
                    seen[m.group(1)] = json.loads(m.group(2))
                except ValueError:
                    pass
            for engine in ("chromium", "webkit"):
                h = seen.get(engine)
                if h is None:
                    print(f"{cell.name:>14} {n:>2} {engine:8} {loads} | never-ran: no streaming line")
                    continue
                red = "red  " if "error" in h["range"] or h["range"].get("status") != 206 else "green"
                print(f"{cell.name:>14} {n:>2} {engine:8} {loads} | {red} range={step(h['range'])} "
                      f"past={step(h.get('past'))} chunked={step(h.get('chunked'))} post={step(h.get('post'))}")
            log = cell / f"{n}.relay.log"
            ends = [l for l in log.read_text(errors="replace").splitlines()
                    if "8999999" in l and ("cut off" in l or "to its end" in l or "no answer" in l)] if log.exists() else []
            for l in ends:
                when = l[11:23]
                sent = re.search(r"sent=(\d+)", l)
                ms = re.search(r" ms=(\d+)", l)
                what = "cut off" if "cut off" in l else "to its end" if "to its end" in l else "no answer"
                print(f"{'':>14} {n:>2} door     {when} {what}, sent {sent and sent.group(1)} in {ms and ms.group(1)} ms")


main()
