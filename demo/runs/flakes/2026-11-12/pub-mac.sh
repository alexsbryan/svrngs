#!/usr/bin/env bash
# The publication test on the Mac beside a prober over the Mac's ephemeral
# range (49152-65535), where its stubs listen: without the fix, with it, and
# with it and the plant (the resolver client following redirects).
set -u
S=/private/tmp/claude-502/-Users-alexsbryan-dev-svrngs/ec058323-fd43-47eb-b700-2cdd0033b6ca/scratchpad
cd $S/flake-wt; export CARGO_INCREMENTAL=0
T=crates/svrngs-node/tests/home/publication.rs; P=crates/svrngs-node/src/publication.rs
probe() { python3 -c "
import socket, threading
def run(lo, hi):
    while True:
        for p in range(lo, hi):
            s = socket.socket(); s.settimeout(0.05)
            try: s.connect(('127.0.0.1', p))
            except Exception: pass
            s.close()
ts = [threading.Thread(target=run, args=(a, min(a+1000, 65536)), daemon=True) for a in range(49152, 65536, 1000)]
[t.start() for t in ts]; [t.join() for t in ts]" >/dev/null 2>&1 & echo $!; }
once() { local mode=$1 pid; pid=$(probe); cargo test -q --test home -- --exact publication::publication_refuses_redirects_oversized_bodies_and_timeouts > /tmp/pub-mac.o 2>&1; rc=$?; kill $pid; echo "$mode exit=$rc $(grep -o 'invitation must never follow[^"]*\|test result: [a-z]*' /tmp/pub-mac.o | head -2 | tr '\n' ' ')"; }
cp $T $T.fixed
git show HEAD:$T > $T
for i in 1 2 3; do once "unfixed,probed"; done
cp $T.fixed $T
for i in 1 2 3; do once "fixed,probed"; done
cp $P $P.unplanted; sed -i '' 's/reqwest13::redirect::Policy::none()/reqwest13::redirect::Policy::limited(2)/' $P
for i in 1 2; do once "fixed,planted,probed"; done
mv $P.unplanted $P; rm $T.fixed
