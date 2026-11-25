#!/usr/bin/env bash
# Rehearses an install on a fresh, air-gapped Fedora 45 server (a systemd
# container with no network), as the clinic's admin would run it. The bundle
# and the site tool go in as they would on a USB stick. Each scenario prints
# what the admin sees; out/transcript.txt keeps all of it.
#
#   ./rehearse.sh            (needs: podman build -t onprem-target:f45 -f Containerfile .)
set -uo pipefail
cd "$(dirname "$0")"
mkdir -p out; exec > >(tee out/transcript.txt) 2>&1
M=onprem-clinic
podman rm -f $M >/dev/null 2>&1
podman run -d --rm --name $M --network none onprem-target:f45 >/dev/null; sleep 4
podman exec $M mkdir -p /media/usb
for f in site.mjs keys/vendor.pub bundles; do podman cp "$f" $M:/media/usb/; done
x() { echo; echo "\$ $*"; podman exec -w /media/usb $M "$@"; echo "[exit $?]"; }
token() { podman exec -w /media/usb $M node site.mjs plan "$@" | grep -o -- '--token [0-9a-f]*' | cut -d' ' -f2; }
scene() { echo; echo "=================== $*"; }

scene "1. An untrusted bundle: nothing is read"
x node site.mjs plan bundles/v010

scene "2. Pin the vendor's key, then plan: nothing changes yet"
x node site.mjs trust vendor.pub
x node site.mjs plan bundles/v010

scene "3. Something already holds the port (planted: another program on 8610)"
podman exec -d $M python3 -m http.server 8610 --bind 0.0.0.0 >/dev/null; sleep 1
x node site.mjs plan bundles/v010
x node site.mjs plan bundles/v010 --auto

scene "4. Approved, then the machine changes before apply: nothing runs"
T=$(token bundles/v010 --auto)
podman exec -d $M python3 -m http.server 8611 --bind 0.0.0.0 >/dev/null; sleep 1
x node site.mjs apply bundles/v010 --token "$T" --auto

scene "5. A tampered artifact: refused before anything is done"
podman exec $M bash -c 'cp -r /media/usb/bundles/v010 /media/usb/bundles/tampered && printf x >> /media/usb/bundles/tampered/service.tar.gz'
x node site.mjs plan bundles/tampered --auto

scene "6. The install, as approved"
T=$(token bundles/v010 --auto)
START=$(date +%s.%N)
x node site.mjs apply bundles/v010 --token "$T" --auto
echo "apply took $(echo "$(date +%s.%N) - $START" | bc) s"
x curl -s http://127.0.0.1:8612/health

scene "7. Run it again: nothing to do"
x node site.mjs plan bundles/v010 --auto

scene "8. The machine restarts: the service comes back by itself"
podman restart $M >/dev/null; sleep 6
x curl -s http://127.0.0.1:8612/health

scene "9. Upgrade to 0.1.1: only what changed runs"
T=$(token bundles/v011 --set port=8612)
x node site.mjs apply bundles/v011 --token "$T" --set port=8612
x curl -s http://127.0.0.1:8612/health

scene "10. A release that crashes at start: failed, with the unit's state and no log line"
T=$(token bundles/crash --set port=8612)
x node site.mjs apply bundles/crash --token "$T" --set port=8612

scene "11. Back to 0.1.1"
T=$(token bundles/v011 --set port=8612)
x node site.mjs apply bundles/v011 --token "$T" --set port=8612

scene "12. What the service can reach: no network, and nothing outside its folder"
x systemctl show notes-to-forms -p User,ProtectSystem,ReadWritePaths,NoNewPrivileges
x ip -br link
x node site.mjs status notes-to-forms

podman rm -f $M >/dev/null
