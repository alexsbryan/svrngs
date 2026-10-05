#!/usr/bin/env python3
"""U4's dry run, read (dry.sh): each pass and engine's runs, median and range.

  summary.py <bytes.json> <runs-*.jsonl>...   the table
  summary.py --line < one run's line     that run on one line

A step a run never reached counts as missing, never as a time: `n` is how
many runs reached it, out of the runs made.
"""
import json
import statistics
import sys

STEPS = [
    ("origin's bytes, the run", lambda r: [r.get("origin_bytes")]),
    ("link opened → name asked", lambda r: [r.get("open_to_name_ms")]),
    ("link → list, untyped", lambda r: [sum_ms(r.get("open_to_name_ms"), r.get("name_to_list_ms"))]),
    ("name typed → list", lambda r: [r.get("name_to_list_ms")]),
    ("link again → list", lambda r: [r.get("again_to_list_ms")]),
    ("lend opened → <video>", lambda r: [r.get("open_lend_ms")]),
    ("play → first frame", lambda r: [(r.get("first") or {}).get("frame_ms")]),
    ("lend opened → first frame", lambda r: [sum_ms(r.get("open_lend_ms"), (r.get("first") or {}).get("frame_ms"))]),
    ("play → plays on", lambda r: [(r.get("first") or {}).get("plays_ms")]),
    ("seek → seeked", lambda r: [s.get("seeked_ms") for s in r.get("seeks") or []]),
    ("seek → its frame", lambda r: [s.get("frame_ms") for s in r.get("seeks") or []]),
    ("seek → plays on", lambda r: [s.get("plays_ms") for s in r.get("seeks") or []]),
]


def sum_ms(*vs):
    return sum(vs) if all(isinstance(v, int) for v in vs) else None


def ms(v):
    return v if isinstance(v, int) else None


def line(r):
    first = r.get("first") or {}
    seeks = [s.get("plays_ms") for s in r.get("seeks") or []]
    return (f"origin {r.get('origin_bytes')} bytes, name {r.get('open_to_name_ms')} ms, list {r.get('name_to_list_ms')} ms, "
            f"again {r.get('again_to_list_ms')} ms, lend {r.get('open_lend_ms')} ms, "
            f"frame {first.get('frame_ms')} ms, seeks {seeks}"
            + (f"; stopped: {r['stopped']}" if r.get("stopped") else ""))


def main():
    if sys.argv[1:] == ["--line"]:
        print(line(json.loads(sys.stdin.read())))
        return
    sizes = json.load(open(sys.argv[1]))
    runs = [json.loads(l) for f in sys.argv[2:] for l in open(f) if l.strip()]
    t = sizes["total"]
    print(f"The page, what it fetches: {t['raw']:,} bytes raw, {t['gzip']:,} gzip -9, {t['brotli']:,} brotli 11")
    for f in sizes["files"]:
        print(f"  {f['file']:<28} {f['raw']:>10,} {f['gzip']:>10,} {f['brotli']:>10,}")
    groups = {}
    for r in runs:
        groups.setdefault((r.get("pass"), r.get("engine")), []).append(r)
    for (p, engine), rs in groups.items():
        browser = next((r.get("browser") for r in rs if r.get("browser")), "?")
        print(f"\n{p} relay, {engine} {browser}: {len(rs)} cold runs")
        for name, get in STEPS:
            got = [get(r) for r in rs]
            vals = [ms(v) for vs in got for v in vs]
            done = [v for v in vals if v is not None]
            if not vals:
                print(f"  {name:<26} never reached")
                continue
            if not done:
                print(f"  {name:<26} n 0/{len(vals)}")
                continue
            unit = "bytes" if "bytes" in name else "ms"
            print(f"  {name:<26} n {len(done)}/{len(vals)}  median {statistics.median(done):>9,.0f} {unit}"
                  f"  range {min(done):,}-{max(done):,}")
        late = sum(1 for r in rs for s in r.get("seeks") or [] if ms(s.get("plays_ms")) is None or s["plays_ms"] > 5000)
        print(f"  seeks not playing on within 5 s: {late}")
        for r in rs:
            if r.get("stopped"):
                print(f"  run {r.get('run')} stopped: {r['stopped']}")


if __name__ == "__main__":
    main()
