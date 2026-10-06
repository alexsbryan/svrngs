"""Search every byte each person's browser received (drive.mjs's dump),
and both computers' svrngs output, for every secret of this run: the
lender's password and token, each person's password, each person's token
as Jellyfin handed it to their browser, every token a plant made the lend
keep (check.sh `plant`), and whatever Sam's lends keep now. A person's own
token in what they received is counted apart: Jellyfin answers a sign-in
with it, and it is theirs. Forked from u5-3's
(demo/runs/u5/2026-10-05-jellyfin/scan.py).

usage: python3 scan.py <scratch dir> <out dir>    writes <out>/scan.json
"""
import json
import sys
from pathlib import Path

root, out = Path(sys.argv[1]), Path(sys.argv[2])
base = {f.name: f.read_text().strip() for f in sorted((root / "secrets").iterdir()) if f.is_file()}
# What Sam's lends keep now, by `share`'s own store (svrngs_node::lend).
kept = {f"lend {f.parent.name} {f.name}": f.read_text().strip()
        for f in sorted((root / "sam/secrets/lends").glob("*/*")) if f.is_file()}
base.update(kept)


def hits(blob, secrets):
    return {k: blob.count(v.encode()) for k, v in secrets.items() if v and v.encode() in blob}


report = {"secrets_searched": sorted(base), "lend_kept": sorted(kept), "pairs": {}, "logs": {}}
for pair in sorted(p for p in (root / "dump").iterdir() if p.is_dir()):
    people = sorted(p for p in pair.iterdir() if p.is_dir())
    tokens = {f"{p.name}_token": (p / "token").read_text().strip() for p in people if (p / "token").exists()}
    report["pairs"][pair.name] = {}
    for p in people:
        secrets = {**base, **tokens}
        own = f"{p.name}_token"
        found, where, scanned = {}, {}, {"responses": 0, "bodies": 0, "bytes": 0, "frames": 0}
        index = (p / "index.jsonl").read_text().splitlines() if (p / "index.jsonl").exists() else []
        parts = []
        for line in index:
            e = json.loads(line)
            scanned["responses"] += 1
            parts.append((e["url"], json.dumps(e["headers"]).encode()))
            if e["file"]:
                b = (p / e["file"]).read_bytes()
                scanned["bodies"] += 1
                scanned["bytes"] += len(b)
                parts.append((e["url"], b))
        if (p / "frames.txt").exists():
            b = (p / "frames.txt").read_bytes()
            scanned["frames"] = b.count(b"\n")
            parts.append(("the live socket's frames", b))
        for url, blob in parts:
            for k, n in hits(blob, secrets).items():
                found[k] = found.get(k, 0) + n
                where.setdefault(k, [])
                if len(where[k]) < 4:
                    path = url.split("/", 3)[-1] if "://" in url else url
                    where[k].append("/" + path.split("?")[0][:80])
        report["pairs"][pair.name][p.name] = {
            "scanned": scanned,
            "own_token": found.pop(own, 0),
            "own_token_in": where.pop(own, []),
            "others_secrets": found,
            "others_secrets_in": {k: where[k] for k in found},
            "holds_none_of_anothers": not found,
        }
everyone = dict(base)
for pair in (root / "dump").iterdir():
    for t in pair.glob("*/token"):
        everyone[f"{pair.name}/{t.parent.name}_token"] = t.read_text().strip()
for log in sorted(list((root / "sam").glob("*")) + list((root / "alex").glob("*"))):
    if log.is_file() and log.suffix in (".log", ".out"):
        report["logs"][str(log.relative_to(root))] = hits(log.read_bytes(), everyone)
(out / "scan.json").write_text(json.dumps(report, indent=1) + "\n")
print(json.dumps({k: {u: v["holds_none_of_anothers"] for u, v in d.items()} for k, d in report["pairs"].items()}))
