#!/bin/bash
# The smoke burst on each binary, V1.1.1 then V1.1, with ticks.py beside each run on Fedora.
S=/private/tmp/claude-502/-Users-alexsbryan-dev-svrngs/bb17e709-0dcd-4285-9662-14f7287f9f8d/scratchpad/v111
for k in kit kit11; do
  ssh alexbryan@fedora "rm -f ticks-$k.jsonl; nohup python3 ticks.py /home/alexbryan/twentyfive/svrngs ticks-$k.jsonl 420 >/dev/null 2>&1 < /dev/null &"
  (cd $S/$k && python3 run.py --mode burst --layout linux --wall-every 1 --run-dir runs/ticks-linux-burst-wall1s > runs/ticks.out 2>&1; echo "exit $?" >> runs/ticks.out)
  sleep 5; ssh alexbryan@fedora "pkill -f 'ticks.py /home/alexbryan/twentyfive/svrngs' ; true"
  scp -q alexbryan@fedora:ticks-$k.jsonl $S/$k/runs/ticks-linux-burst-wall1s/ticks.jsonl
done
echo pair-done
