#!/usr/bin/env bash
# The channel: a site node and a vendor node on this computer, in a lab
# group of two. The site lends its projected view (site-view) and the raw
# control (site-raw); the vendor turns Play on for both.
#   ./lab.sh up | down
set -euo pipefail
cd "$(dirname "$0")"
B=/home/alexbryan/dev/svrngs/target/release/svrngs
s() { HOME=$PWD/out/site-node/home SVRNGS_DIR=$PWD/out/site-node/data NO_COLOR=1 "$@"; }
v() { HOME=$PWD/out/vendor/home SVRNGS_DIR=$PWD/out/vendor/data NO_COLOR=1 "$@"; }
if [ "$1" = down ]; then for p in out/site-node/pid out/vendor/pid out/viewP.pid out/viewR.pid; do [ -f $p ] && kill "$(cat $p)" 2>/dev/null; rm -f $p; done; exit 0; fi
rm -rf out/site-node out/vendor; mkdir -p out/site-node/home out/vendor/home
node siteview.mjs out/site/raw.jsonl --port 8721 --fresh > out/viewP.log 2>&1 & echo $! > out/viewP.pid
node siteview.mjs out/site/raw.jsonl --port 8722 --raw --fresh > out/viewR.log 2>&1 & echo $! > out/viewR.pid
for p in 8721 8722; do for i in $(seq 40); do (exec 3<>/dev/tcp/127.0.0.1/$p) 2>/dev/null && break; sleep 0.5; done; done
s env RUST_LOG=info nohup $B new Bench --as Site > out/site-node/new.out 2> out/site-node/run.log & echo $! > out/site-node/pid
for i in $(seq 60); do L=$(grep -o 'svrngs://join/[A-Za-z0-9_-]*' out/site-node/new.out | head -1) && [ -n "$L" ] && break; sleep 0.5; done
v env RUST_LOG=info nohup $B join "$L" --as Vendor > out/vendor/join.out 2> out/vendor/run.log & echo $! > out/vendor/pid
for i in $(seq 60); do v $B ask /v1/groups 2>/dev/null | grep -q Bench && break; sleep 0.5; done
s $B share 8721 --group Bench --name site-view --program node </dev/null | tail -1
s $B share 8722 --group Bench --name site-raw --program node </dev/null | tail -1
G=$(v $B ask /v1/groups | python3 -c 'import json,sys; print(json.load(sys.stdin)["groups"][0]["id"])')
for n in site-view site-raw; do
  for i in $(seq 60); do J=$(v $B ask "/v1/lends?namespace=$G" | python3 -c 'import json,sys; l=[x for x in json.load(sys.stdin)["lent"] if x["name"]==sys.argv[1]][0]; print(json.dumps({"on":{"key":l["key"],"name":l["name"]}}))' $n 2>/dev/null) && break; sleep 1; done
  v $B ask /v1/play "$J" >/dev/null
done
echo "lab up: $(./view-P.sh / | head -c 120)"
