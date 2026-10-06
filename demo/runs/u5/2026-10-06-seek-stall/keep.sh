#!/bin/bash
# keep.sh <dir>, run from the repo after `cargo test -p svrngs-node --test conformance`: the conformance run's seeks, worker RANGE lines, door carriage lines and verdicts
set -e
D=$1; mkdir -p $D
python3 - "$D" <<'PY'
import json,sys,glob,os
d=sys.argv[1]
for f in sorted(glob.glob('target/conformance/*.json')):
    r=json.load(open(f))
    out={w:{'at':r['lends'][w].get('at'),'seeks':r['lends'][w]['seeks'],'trace':[l for l in r['lends'][w].get('trace','').splitlines() if 'RANGE' in l or 'part' in l]} for w in ('sam','bo')}
    json.dump(out,open(os.path.join(d,os.path.basename(f)),'w'),indent=1)
    for w in ('sam','bo'): print(os.path.basename(f)[:-5], w, r['lends'][w]['seeks'].get('again'), r['lends'][w]['seeks']['ok'])
PY
gzip -c target/conformance/carriage.log > $D/carriage.log.gz; cat target/conformance/*.txt > $D/verdicts.txt
