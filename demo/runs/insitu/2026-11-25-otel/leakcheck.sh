#!/usr/bin/env bash
# Reads every route of the clinic's view through svrngs, as Acme's computer
# does, and fails on any client name, client id, plan id or note phrase the
# clinic holds. Plant: `--plant` reads the collector's raw file on the clinic
# instead, before the projection, which must fail.
set -uo pipefail
cd "$(dirname "$0")"
V=$PWD/out/vendor-node
if [ "${1:-}" = --plant ]; then all=$(podman exec onprem-clinic cat /var/lib/onprem/telemetry/notes-to-forms/raw.jsonl /var/lib/onprem/effects/notes-to-forms.jsonl)
else all=$(for r in / /site /contract '/service/telemetry?limit=1000' '/service/effects?limit=1000' /trials; do HOME=$V/home SVRNGS_DIR=$V/data node acme.mjs view "$r"; done); fi
words=$(python3 - <<'PY'
import os, re
names = set(); ids = set()
for f in os.listdir("out/clinic/inbox"):
    names.update(re.sub(r"^[\d-]+ | - session\.txt$", "", f).split())
    for l in open(os.path.join("out/clinic/inbox", f)):
        ids.update(re.findall(r"TEST-\d+|pl-\d+", l))
import sqlite3
ids.update(r[0] for r in sqlite3.connect("out/clinic/clients.db").execute("select plan_id from clients where plan_id is not null"))
print("\n".join(sorted(names | ids | {"week was hard", "grounding", "telehealth", "50 min"})))
PY
)
hits=0
while read -r w; do grep -qF -- "$w" <<<"$all" && { echo "LEAK: $w"; hits=$((hits+1)); }; done <<<"$words"
n=$(wc -l <<<"$words")
[ $hits = 0 ] && echo "passed     none of $n names, ids or phrases in the view ($(wc -c <<<"$all") bytes read)" || { echo "failed     $hits of $n leaked"; exit 1; }
