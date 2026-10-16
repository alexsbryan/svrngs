#!/usr/bin/env python3
"""arch-view (a): every journal left under /private/tmp, the bytes its lines
give to their writers' views, counted as the scale instrument counts them
(`,"view":` and the value as written). Usage: survey.py [root]"""
import json, sys
from pathlib import Path

root = Path(sys.argv[1] if len(sys.argv) > 1 else "/private/tmp")
rows, unread = [], []
for f in sorted(root.glob("**/ring_oplog.jsonl")):
    try:
        text = f.read_text()
    except OSError as e:
        unread.append(f"{f}: {e}")
        continue
    total, view, lines, keys = len(text.encode()), 0, 0, 0
    for n, line in enumerate(text.splitlines(), 1):
        try:
            op = json.loads(line)
        except ValueError as e:
            unread.append(f"{f}:{n}: {e}")
            continue
        lines += 1
        if "view" in op:
            view += len(',"view":') + len(json.dumps(op["view"], separators=(",", ":")))
            keys = max(keys, len(op["view"].get("entries", {})))
    rows.append({"journal": str(f), "bytes": total, "lines": lines, "view_bytes": view,
                 "share": view / total if total else None, "most_keys_in_a_view": keys})
shares = sorted(r["share"] for r in rows if r["share"] is not None)
print(json.dumps({
    "journals": len(rows),
    "bytes": sum(r["bytes"] for r in rows),
    "view_bytes": sum(r["view_bytes"] for r in rows),
    "share_of_all_bytes": sum(r["view_bytes"] for r in rows) / max(1, sum(r["bytes"] for r in rows)),
    "share_min": shares[0] if shares else None,
    "share_median": shares[len(shares) // 2] if shares else None,
    "share_max": shares[-1] if shares else None,
    "most_keys_in_a_view": max((r["most_keys_in_a_view"] for r in rows), default=None),
    "unread": unread,
    "rows": rows,
}, indent=1))
