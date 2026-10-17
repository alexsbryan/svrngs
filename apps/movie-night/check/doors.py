"""What each lender's door saw during Movie Night's runs (check.sh `judge`),
from its own log at RUST_LOG=info,door=debug: every request it carried
with the person it named (`door: carrying a request to the lend's
service`, crates/svrngs-node/src/door.rs:847), each count it took
(`let in, and told their share`, :838, with its `people`; `guessed in this
computer's home`, :833, uncounted), each refusal, and each sign-in ask.

Written: <out>/doors.log, those lines of Sam's, Dee's and Cy's doors, each
prefixed with its door; <out>/doors.json, per door and lend the requests
each person made, the most people counted at once, who was guessed home,
the refusals by why; and, when <out>/limit.json is there, what each door
counted on each of the second person's searches after the first's last
request (README §4, the limit: how long a searcher stays counted).

usage: python3 doors.py <scratch dir> <out dir>
"""
import json
import re
import sys
from collections import Counter, defaultdict
from datetime import datetime
from pathlib import Path

root, out = Path(sys.argv[1]), Path(sys.argv[2])
KEEP = re.compile(r"door: (carrying a request|let in|guessed in|refused|sign-in)")
FIELD = re.compile(r'(\w+)=("(?:[^"\\]|\\.)*"|\S+)')


def when(line):
    return datetime.fromisoformat(line.split()[0].replace("Z", "+00:00"))


def fields(line):
    return {k: v.strip('"') for k, v in FIELD.findall(line)}


lines, seen = [], []
for door in ("sam", "dee", "cy"):
    log = root / door / "run.log"
    if not log.exists():
        continue
    for line in log.read_text(errors="replace").splitlines():
        if KEEP.search(line):
            lines.append(f"{door} {line}")
            seen.append((door, when(line), line, fields(line)))
(out / "doors.log").write_text("\n".join(lines) + "\n")

requests = defaultdict(lambda: defaultdict(Counter))
counted = defaultdict(list)
home = defaultdict(Counter)
refused = defaultdict(Counter)
signin = defaultdict(Counter)
for door, t, line, f in seen:
    lend = f"{door}/{f.get('lend', '?')}"
    if "carrying a request" in line:
        requests[lend][f.get("person", "?")][f.get("method", "?")] += 1
    elif "let in" in line:
        counted[lend].append((t, f.get("person"), int(f.get("people", 0)), f.get("relayed")))
    elif "guessed in" in line:
        home[lend][f.get("person")] += 1
    elif "refused" in line:
        refused[door][line.split("door: refused — ")[1].split(" holder=")[0].split(" actor=")[0][:90] + (f" ({f['why']})" if "why" in f else "")] += 1
    elif "sign-in" in line:
        signin[lend][line.split("door: sign-in — ")[1].split(" lend=")[0]] += 1

report = {
    "requests": {lend: {p: dict(m) for p, m in people.items()} for lend, people in requests.items()},
    "most_counted": {lend: max(c[2] for c in v) for lend, v in counted.items()},
    "counted_by_people": {lend: dict(Counter(c[2] for c in v)) for lend, v in counted.items()},
    "relayed": {lend: dict(Counter(f"{c[1]} {c[3]}" for c in v)) for lend, v in counted.items()},
    "guessed_home": {lend: dict(c) for lend, c in home.items()},
    "refused": {door: dict(c) for door, c in refused.items()},
    "sign_in": {lend: dict(c) for lend, c in signin.items()},
}

limit = out / "limit.json"
if limit.exists():
    run = json.loads(limit.read_text())
    first, second = list(run["people"].values())[:2]
    jellyfins = [lend for lend in counted if lend.endswith("/jellyfin")]
    ends = {}
    for lend in jellyfins:
        mine = [c[0] for c in counted[lend] if c[1] == first["name"]]
        ends[lend] = max(mine).isoformat() if mine else None
    after = []
    for s in second.get("after", []):
        start, end = datetime.fromisoformat(s["started"].replace("Z", "+00:00")), datetime.fromisoformat(s["ended"].replace("Z", "+00:00"))
        row = {"after_s": s["after_s"]}
        for lend in jellyfins:
            at = [c for c in counted[lend] if c[1] == second["name"] and start <= c[0] <= end]
            last = ends[lend] and datetime.fromisoformat(ends[lend])
            row[lend] = {"people": sorted({c[2] for c in at}),
                         "since_first_last_s": round((at[0][0] - last).total_seconds(), 1) if at and last else None}
        after.append(row)
    report["limit"] = {"first": first["name"], "second": second["name"], "first_last_counted": ends, "second_after": after}

(out / "doors.json").write_text(json.dumps(report, indent=1, default=str) + "\n")
print("doors: " + json.dumps({"most_counted": report["most_counted"], "guessed_home": report["guessed_home"],
                             "refused": report["refused"], "limit": report.get("limit", {}).get("second_after")}))
