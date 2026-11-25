#!/usr/bin/env bash
# Sets the scene. A connected clinic server (outbound only, nothing
# published) runs release 0.3.0 from a signed bundle: its telemetry goes to
# the site's collector, its claims through the site's gateway to the
# clearinghouse, a stand-in payer in a container beside it. svrngs on the
# clinic lends the view to Acme; svrngs on this computer, Acme's, lends its
# candidates to the clinic. Then the clinic's EHR export fills the inbox.
#
#   ./stage.sh          (needs: podman build -t onprem-target:otel -f Containerfile .)
#   ./stage.sh down
set -euo pipefail
cd "$(dirname "$0")"
B=$PWD/vendor-cache/svrngs; V=$PWD/out/vendor-node
M=onprem-clinic
v() { HOME=$V/home SVRNGS_DIR=$V/data NO_COLOR=1 "$@"; }
c() { podman exec -w /media/usb -e HOME=/root $M "$@"; }
down() {
  podman rm -f $M clearinghouse >/dev/null 2>&1 || true
  for p in out/vendor-node/pid out/candidates.pid; do [ -f $p ] && kill "$(cat $p)" 2>/dev/null; rm -f $p; done
  return 0
}
down; [ "${1:-}" = down ] && exit 0

mkdir -p out && rm -rf out/candidates out/clinic out/vendor-node out/clearinghouse
[ -f keys/vendor.key ] || node vendor.mjs key keys
# KEEP_REPO=1 keeps the vendor repo and its branches from an earlier run.
[ -n "${KEEP_REPO:-}" ] && [ -d out/vendor-repo ] || { rm -rf out/vendor-repo; mkdir -p out/vendor-repo && cp -r service/. out/vendor-repo/ && rm -rf out/vendor-repo/__pycache__ out/vendor-repo/tests/__pycache__
git -C out/vendor-repo init -q -b main && printf '__pycache__/\n' > out/vendor-repo/.gitignore
git -C out/vendor-repo add -A && git -C out/vendor-repo -c user.name=Acme -c user.email=dev@acme.invalid commit -qm "notes-to-forms 0.3.0"; }
# The release is built from main itself, never from whatever is checked out.
rm -rf out/release-main && git -C out/vendor-repo worktree prune && git -C out/vendor-repo worktree add -q -f "$PWD/out/release-main" main
node vendor.mjs bundle out/release-main/recipe.json keys bundles/release-0.3.0 --source out/release-main --version 0.3.0 --from "main@$(git -C out/vendor-repo rev-parse --short main)"
git -C out/vendor-repo worktree remove --force "$PWD/out/release-main"
python3 make-clinic.py out/clinic/inbox out/clinic/clients.db

podman network exists onprem-net || podman network create onprem-net >/dev/null
mkdir -p out/clearinghouse
podman run -d --rm --name clearinghouse --network onprem-net -v "$PWD/clearinghouse.mjs:/ch.mjs:ro,Z" -v "$PWD/out/clearinghouse:/log:Z" onprem-target:otel node /ch.mjs >/dev/null
podman run -d --rm --name $M --network onprem-net ${CLINIC_CAPS---cap-add=SYS_ADMIN,NET_ADMIN} onprem-target:otel >/dev/null; sleep 5

podman exec $M mkdir -p /media/usb
for f in site.mjs project.mjs gateway.mjs ../../../../apps/pantry/svrngs.mjs keys/vendor.pub bundles; do podman cp "$f" $M:/media/usb/; done
c node site.mjs trust vendor.pub
c mkdir -p /srv/notes-to-forms/inbox
T=$(c node site.mjs plan bundles/release-0.3.0 | grep -o -- '--token [0-9a-f]*' | cut -d' ' -f2)
c node site.mjs apply bundles/release-0.3.0 --token "$T"
podman cp out/clinic/clients.db $M:/srv/notes-to-forms/state/clients.db
c chown notes:notes /srv/notes-to-forms/state/clients.db
c node site.mjs allow-trials notes-to-forms
c systemd-run --unit=onprem-view --quiet node /usr/local/lib/onprem/site.mjs view notes-to-forms --port 8620

# svrngs: the clinic makes a group for its vendor, and lends it the view.
c systemd-run --unit=svrngs --quiet -E HOME=/root -E RUST_LOG=info /usr/local/bin/svrngs new Acme-support --as Clinic
for i in $(seq 60); do L=$(c journalctl -u svrngs -o cat | grep -o 'svrngs://join/[A-Za-z0-9_-]*' | head -1) && [ -n "$L" ] && break; sleep 0.5; done
c bash -c 'svrngs share 8620 --group Acme-support --name clinic-view --program node </dev/null' | tail -1
# Acme's computer joins, and lends the clinic its candidates.
mkdir -p $V/home out/candidates && echo '[]' > out/candidates/index.json
v env RUST_LOG=info nohup "$B" join "$L" --as Acme > $V/join.out 2> $V/run.log & echo $! > $V/pid
python3 -m http.server 8630 --bind 127.0.0.1 -d out/candidates >/dev/null 2>&1 & echo $! > out/candidates.pid
for i in $(seq 60); do v "$B" ask /v1/groups 2>/dev/null | grep -q Acme-support && break; sleep 0.5; done
v "$B" share 8630 --group Acme-support --name acme-candidates --program python3 </dev/null | tail -1
G=$(v "$B" ask /v1/groups | python3 -c 'import json,sys; print(json.load(sys.stdin)["groups"][0]["id"])')
play() { python3 -c 'import json,sys; l=[x for x in json.load(sys.stdin)["lent"] if x["name"]==sys.argv[1]][0]; print(json.dumps({"on":{"key":l["key"],"name":l["name"]}}))' "$1"; }
for i in $(seq 60); do J=$(v "$B" ask "/v1/lends?namespace=$G" | play clinic-view 2>/dev/null) && break; sleep 1; done
v "$B" ask /v1/play "$J" >/dev/null
for i in $(seq 60); do J=$(c svrngs ask "/v1/lends?namespace=$G" | play acme-candidates 2>/dev/null) && break; sleep 1; done
c svrngs ask /v1/play "$J" >/dev/null
c systemd-run --unit=onprem-watch --quiet -E HOME=/root node /usr/local/lib/onprem/site.mjs watch acme-candidates

# The clinic's EHR export.
podman cp out/clinic/inbox/. $M:/srv/notes-to-forms/inbox/
c bash -c 'chmod 755 /srv/notes-to-forms/inbox && chmod 644 /srv/notes-to-forms/inbox/*'
sleep 8
echo "staged; through svrngs, Acme sees:"
v node acme.mjs view /service/telemetry?limit=400 | python3 -c 'import json,sys,collections; t=json.load(sys.stdin); print(collections.Counter(e["attrs"].get("outcome") for e in t if isinstance(t,list) and e.get("kind")=="span" and e.get("name")=="note"))'
