#!/usr/bin/env python3
"""B7's stop, asked as it happens (gf-1; README §4, B7): a 1 KiB range of a
hash at the meeting point's endpoint every STEP s, each at an offset no
earlier one held, so each is fetched from a holder anew, for SECONDS s.
One JSON line per ask: its time, offset, status and the refusal's body.
Run beside `unshare`: a 206 is the holder serving, a 502 her provider
refusing (or no holder answering), a 404 the meeting point's record
having dropped the hash.

    probe.py ENDPOINT HASH SIZE SECONDS OUT_JSONL [STEP]
"""
import json
import sys
import time
import urllib.error
import urllib.request

endpoint, hash_, size, seconds, out = sys.argv[1:6]
step = float(sys.argv[6]) if len(sys.argv) > 6 else 0.02
size, seconds = int(size), float(seconds)
asks = int(seconds / step)
stride = max(64 * 1024, size // max(asks, 1) // 16384 * 16384)
t_end = time.time() + seconds
with open(out, "a", buffering=1) as log:
    i = 0
    while time.time() < t_end:
        at = (i * stride) % max(size - 1024, 1)
        i += 1
        req = urllib.request.Request(f"{endpoint}{hash_}", headers={
            "Range": f"bytes={at}-{at + 1023}", "User-Agent": "gf-probe"})
        t0 = time.time()
        try:
            with urllib.request.urlopen(req, timeout=10) as r:
                status, body = r.status, len(r.read())
        except urllib.error.HTTPError as e:
            status, body = e.code, e.read().decode(errors="replace")[:600]
        except Exception as e:  # a refusal mid-body, or no answer at all
            status, body = None, repr(e)[:300]
        log.write(json.dumps({"t": round(t0, 3), "t_end": round(time.time(), 3),
                              "at": at, "status": status, "body": body}) + "\n")
        time.sleep(max(0.0, step - (time.time() - t0)))
