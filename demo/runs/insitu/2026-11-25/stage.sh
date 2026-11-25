#!/usr/bin/env bash
# Sets the scene: the clinic's air-gapped server runs release 0.2.0 from a
# signed bundle, its EHR export fills the inbox, the admin has said yes to
# trials once, and the view and the delivery watcher run. Also makes the
# vendor's repo, at 0.2.0, for the agent to work in.
#
#   ./stage.sh      (needs: podman build -t onprem-target:f45 -f Containerfile .)
set -euo pipefail
cd "$(dirname "$0")"
M=onprem-clinic
mkdir -p out
[ -f keys/vendor.key ] || node vendor.mjs key keys
rm -rf out/vendor-repo && cp -r service out/vendor-repo && git -C out/vendor-repo init -q -b main \
  && git -C out/vendor-repo add -A && git -C out/vendor-repo -c user.name=Acme -c user.email=dev@acme.invalid commit -qm "notes-to-forms 0.2.0"
node vendor.mjs bundle vendor/recipe.json keys bundles/release-0.2.0 --source out/vendor-repo --version 0.2.0 --from "$(git -C out/vendor-repo rev-parse --short HEAD)"
rm -rf out/inbox && python3 make-inbox.py out/inbox
podman rm -f $M >/dev/null 2>&1 || true
podman run -d --rm --name $M --network none onprem-target:f45 >/dev/null; sleep 4
podman exec $M mkdir -p /media/usb /media/delivery
for f in site.mjs keys/vendor.pub bundles; do podman cp "$f" $M:/media/usb/; done
c() { podman exec -w /media/usb $M "$@"; }
c node site.mjs trust vendor.pub
c mkdir -p /srv/notes-to-forms/inbox
T=$(c node site.mjs plan bundles/release-0.2.0 | grep -o -- '--token [0-9a-f]*' | cut -d' ' -f2)
c node site.mjs apply bundles/release-0.2.0 --token "$T"
podman cp out/inbox/. $M:/srv/notes-to-forms/inbox/
c bash -c 'chmod 755 /srv/notes-to-forms/inbox && chmod 644 /srv/notes-to-forms/inbox/*'
c node site.mjs allow-trials notes-to-forms
c systemd-run --unit=onprem-view --quiet node /media/usb/site.mjs view notes-to-forms --port 8620
c systemd-run --unit=onprem-watch --quiet node /media/usb/site.mjs watch /media/delivery
sleep 3; echo "staged: $(c curl -s 127.0.0.1:8620/service/state)"
