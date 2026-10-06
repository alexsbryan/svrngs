"""One line per pair run (drive.mjs's JSON), read against README.md's bars.

usage: python3 summary.py --line <pair json>     one line, for measure.sh's log
       python3 summary.py <out dir>              every pair-*.json there, a table
"""
import json
import sys
from pathlib import Path


def line(path):
    try:
        d = json.loads(Path(path).read_text())
    except (OSError, ValueError) as e:
        return f"never ran: {e}"
    if d.get("stopped"):
        return f"stopped: {d['stopped']}"
    ppl = d["people"]
    bits = []
    for u, p in ppl.items():
        if p.get("stopped"):
            bits.append(f"{u} stopped: {p['stopped'][:160]}")
            continue
        sock = p.get("socket", {}).get("message_shown_ms")
        bits.append(f"{u}: open {p.get('open_ms')} ms, signed in {p.get('signed_in_ms')} ms, plays {p.get('play_ms')} ms{' after a second tap' if p.get('second_tap') else ''}, "
                    f"socket message {sock} ms, signed in after: {p.get('signed_in_after')}")
    for k in ("first_seeks", "then_seeks"):
        ph = d.get(k)
        if ph:
            w = ph["watch"]
            bits.append(f"{ph['seeker']} seeks {[s['ms'] for s in ph['seeks']]} ms ({ph['within_5s']}/{len(ph['seeks'])} within 5 s)"
                        + (f"; {ph['watched']} left alone: {w['left_alone']} (emptied {w['emptied']}, loadstart {w['loadstart']}, "
                           f"new element {w['element']}, back {w['back_s']} s, still {w['still_ms']} ms)" if w else ""))
    bits.append(f"devices apart: {d.get('devices_apart')}")
    return "; ".join(bits)


if sys.argv[1] == "--line":
    print(line(sys.argv[2]))
else:
    for f in sorted(Path(sys.argv[1]).glob("pair-*.json")):
        print(f"{f.stem}: {line(f)}")
