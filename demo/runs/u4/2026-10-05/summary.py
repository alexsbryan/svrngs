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
    ("the wasm, fetches a run", lambda r: [r.get("origin_wasm")]),
    ("link opened → name asked", lambda r: [r.get("open_to_name_ms")]),
    ("link → list, untyped", lambda r: [sum_ms(r.get("open_to_name_ms"), r.get("name_to_list_ms"))]),
    ("name typed → list", lambda r: [r.get("name_to_list_ms")]),
    ("link again → list", lambda r: [r.get("again_to_list_ms")]),
    ("lend opened → <video>", lambda r: [r.get("open_lend_ms")]),
    ("play → first frame", lambda r: [(r.get("first") or {}).get("frame_ms")]),
    ("lend opened → first frame", lambda r: [sum_ms(r.get("open_lend_ms"), (r.get("first") or {}).get("frame_ms"))]),
    ("play → plays on", lambda r: [(r.get("first") or {}).get("plays_ms")]),
    ("play → first seek asked", lambda r: [first_seek(r)]),
    ("seek → seeked", lambda r: [s.get("seeked_ms") for s in r.get("seeks") or []]),
    ("seek → its frame", lambda r: [s.get("frame_ms") for s in r.get("seeks") or []]),
    ("seek → plays on", lambda r: [s.get("plays_ms") for s in r.get("seeks") or []]),
]


def first_seek(r):
    """How long after play the first seek was asked: u4-9's early seek."""
    first, seeks = r.get("first") or {}, r.get("seeks") or []
    return seeks[0]["t"] - first["t"] if seeks and "t" in seeks[0] and "t" in first else None


def parts(log):
    """What the lent service answered in a pass (lend.mjs's lines): each
    range the worker asked as a 2 MiB part (u4-9, web/sw.js PART), the ones
    sent whole and their milliseconds, and the longest any answer ran."""
    whole, longest = [], 0
    for l in open(log):
        f = l.split()
        if len(f) != 7 or not f[6].endswith("ms"):
            continue
        took = int(f[6][:-2])
        longest = max(longest, took)
        a, _, b = f[3].removeprefix("bytes=").partition("-")
        sent, of = f[5].split("/")
        if a and b and int(b) - int(a) + 1 == 2 * 1024 * 1024 and sent == of:
            whole.append(took)
    return whole, longest


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
    for f in sys.argv[2:]:
        log = f.replace("runs-", "lend-").replace(".jsonl", ".log")
        try:
            whole, longest = parts(log)
        except FileNotFoundError:
            continue
        if whole:
            print(f"{log.rsplit('/', 1)[-1]}: {len(whole)} parts of 2 MiB sent whole, median {statistics.median(whole):,.0f} ms"
                  f"  range {min(whole):,}-{max(whole):,}; the longest answer {longest:,} ms")
        else:
            print(f"{log.rsplit('/', 1)[-1]}: no part of 2 MiB; the longest answer {longest:,} ms")
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
            unit = "bytes" if "bytes" in name else "" if "fetches" in name else "ms"
            print(f"  {name:<26} n {len(done)}/{len(vals)}  median {statistics.median(done):>9,.0f} {unit}"
                  f"  range {min(done):,}-{max(done):,}")
        late = sum(1 for r in rs for s in r.get("seeks") or [] if ms(s.get("plays_ms")) is None or s["plays_ms"] > 5000)
        print(f"  seeks not playing on within 5 s: {late}")
        for r in rs:
            if r.get("stopped"):
                print(f"  run {r.get('run')} stopped: {r['stopped']}")


if __name__ == "__main__":
    main()
