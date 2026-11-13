#!/usr/bin/env bash
# In the gate image at /work (as launch.sh mounts it): the publication test
# alone N times, then N times beside a port prober over Linux's ephemeral
# range, as share::listening probes 1024-49151 from another test of the
# same binary (home.rs `looking_for_what_runs_here_…`, GET /running).
cd /work; n=${N:-5}
scan() { python3 -c "
import socket, threading
def run(lo, hi):
    while True:
        for p in range(lo, hi):
            s = socket.socket(); s.settimeout(0.05)
            try: s.connect((\"127.0.0.1\", p))
            except Exception: pass
            s.close()
ts = [threading.Thread(target=run, args=(a, a+1000), daemon=True) for a in range(32768, 61000, 1000)]
[t.start() for t in ts]; [t.join() for t in ts]"; }
for mode in alone probed; do
  for i in $(seq $n); do
    [ $mode = probed ] && { scan & sp=$!; }
    cargo test -q --test home publication_refuses_redirects -- --exact publication::publication_refuses_redirects_oversized_bodies_and_timeouts > /tmp/o 2>&1; rc=$?
    [ $mode = probed ] && kill $sp
    echo "$mode $i exit=$rc $(grep -o "invitation must never follow[^\"]*\|test result: [a-z]*" /tmp/o | head -2 | tr "\n" " ")"
  done
done
