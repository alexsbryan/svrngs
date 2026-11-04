#!/bin/bash
# usage: run.sh <n8n|kuma|owui>
# Start the app in its own pod, let it settle, then take 7 copies 5 minutes
# apart as Give's `snapshot` does (apps/give/src/runtime.rs:358-410, copy at
# :448-470): pod paused, the state volume copied inside podman's storage with
# `cp -a --reflink=always`, pod unpaused, the copy exported to a tar, the copy
# removed. One light use runs at ~minute 12. Then the pod is stopped and the
# stopped volume is exported twice, and copied-then-exported twice, to see
# whether an export of an unchanged volume is the same bytes.
set -u
APP=$1
S=~/svrngs-churn/scripts
B=~/svrngs-churn/$APP
POD=svrngs-churn-$APP
VOL=svrngs-churn-$APP-data
COPY=$VOL-snapshot
mkdir -p "$B/snaps" "$B/det"
log() { echo "$(date -u +%FT%T.%3NZ) $*" | tee -a "$B/run.log"; }
. "$S/apps.sh"

volsize() { podman unshare du -sb "$(podman volume inspect -f '{{.Mountpoint}}' "$VOL")" | cut -f1; }

# One copy the way Give takes it; prints nothing, logs the pause and the tar.
snap() { # <out.tar> [live]
  local out=$1 live=${2:-live} t0 t1 mp cp
  t0=$(date +%s%N)
  [ "$live" = live ] && podman pod pause "$POD" >/dev/null
  mp=$(podman volume inspect -f '{{.Mountpoint}}' "$VOL")
  podman volume exists "$COPY" && podman volume rm "$COPY" >/dev/null
  podman volume create "$COPY" >/dev/null
  cp=$(podman volume inspect -f '{{.Mountpoint}}' "$COPY")
  podman unshare cp -a --reflink=always "$mp/." "$cp/" || { log "reflink copy failed"; exit 1; }
  [ "$live" = live ] && podman pod unpause "$POD" >/dev/null
  t1=$(date +%s%N)
  podman volume export --output "$out" "$COPY"
  podman volume rm "$COPY" >/dev/null
  log "copy $(basename "$out") paused_ms=$(( (t1 - t0) / 1000000 )) bytes=$(stat -c %s "$out") sha256=$(sha256sum "$out" | cut -c1-64)"
}

log "start $APP"
podman volume create "$VOL" >/dev/null
"start_$APP" >/dev/null
started=$(date +%s)
until "ready_$APP" 2>/dev/null; do
  sleep 2
  [ $(( $(date +%s) - started )) -gt 1800 ] && { log "never ready"; exit 1; }
done
log "ready after $(( $(date +%s) - started )) s"
"setup_$APP" 2>&1 | tee -a "$B/setup.log"

# Settled: at least 3 minutes after ready, and the volume grew by under 1 MB
# in the last minute (Open WebUI fetches its models into it at first start).
settled=$(date +%s); last=$(volsize)
while :; do
  sleep 60
  now=$(volsize)
  log "settling: volume ${now} bytes"
  if [ $(( $(date +%s) - settled )) -ge 180 ] && [ $(( now - last )) -lt 1000000 ]; then break; fi
  last=$now
  [ $(( $(date +%s) - settled )) -gt 1800 ] && { log "never settled"; break; }
done
log "settled"

t0=$(date +%s)
( sleep 720; log "light use begins"; "use_$APP" >> "$B/use.log" 2>&1; log "light use ends: exit $?" ) &
for k in 0 1 2 3 4 5 6; do
  while [ "$(date +%s)" -lt $(( t0 + 300 * k )) ]; do sleep 1; done
  snap "$B/snaps/$k.tar"
done
wait

# Determinism: the pod stopped, nothing writes the volume.
podman pod stop --time 30 "$POD" >/dev/null
for n in a b; do
  podman volume export --output "$B/det/export-$n.tar" "$VOL"
  log "stopped export-$n bytes=$(stat -c %s "$B/det/export-$n.tar") sha256=$(sha256sum "$B/det/export-$n.tar" | cut -c1-64)"
done
sleep 2
for n in a b; do snap "$B/det/copy-$n.tar" stopped; sleep 2; done

podman pod rm -f "$POD" >/dev/null
podman volume rm "$VOL" >/dev/null
podman volume exists "$COPY" && podman volume rm "$COPY" >/dev/null
log "done"
touch "$B/done"
