# Sourced by run.sh. Per app: the image (pinned by its multi-arch index
# digest), how it starts in its own pod on 127.0.0.1, when it is ready, the
# setup that is part of settling, and the one light use at ~minute 12.
# Secrets (passwords) are made here with `openssl rand`, kept in files under
# the app's directory mode 600, and never printed.

N8N_IMG=ghcr.io/n8n-io/n8n@sha256:9a8d8dcea75667496695d7810d7e1fe018a65efb36cc499d347f3931b3957b5d
KUMA_IMG=mirror.gcr.io/louislam/uptime-kuma@sha256:70233f4acb5163fd2a59a49909cf01e44415cecff13dba69e3a86647a2919f83
OWUI_IMG=ghcr.io/open-webui/open-webui@sha256:03443b8de6b617da9d900edd09b4bf1cb0ddbfdcba6afb42c548336a0b821ded
TARGET_IMG=docker.io/library/python:3.12-slim@sha256:aa943ce411423c7582288f18707ff66eddcbaeb510502d747a72aeb34f30d159

secret() { [ -s "$B/.pass" ] || (umask 077; openssl rand -hex 16 > "$B/.pass"); }

# n8n, as its docs run it: the image, a named volume at /home/node/.n8n.
start_n8n() {
  podman pod create --name "$POD" -p 127.0.0.1:15678:5678
  podman run -d --pod "$POD" --name "$POD-app" -v "$VOL:/home/node/.n8n" "$N8N_IMG"
}
ready_n8n() { curl -fsS -o /dev/null http://127.0.0.1:15678/healthz; }
setup_n8n() { :; }
use_n8n() { secret; python3 -I "$S/use_n8n.py" 15678 "$B/.pass"; }

# Uptime Kuma, with a python http.server beside it in the pod for its two
# monitors to check every 20 s.
start_kuma() {
  podman pod create --name "$POD" -p 127.0.0.1:13001:3001
  podman run -d --pod "$POD" --name "$POD-target" -w /tmp "$TARGET_IMG" python3 -m http.server --bind 127.0.0.1 8000
  podman run -d --pod "$POD" --name "$POD-app" -v "$VOL:/app/data" "$KUMA_IMG"
}
ready_kuma() { curl -fsS -o /dev/null http://127.0.0.1:13001/; }
setup_kuma() {
  secret
  KUMA_PASS=$(cat "$B/.pass") podman exec -i -e KUMA_PASS -w /app "$POD-app" node - < "$S/kuma_setup.js"
}
use_kuma() { echo "kuma: no extra use; its two monitors write every 20 s"; }

# Open WebUI on its own image, the key file in the state (compose-c).
start_owui() {
  podman pod create --name "$POD" -p 127.0.0.1:18090:8080
  podman run -d --pod "$POD" --name "$POD-app" -e WEBUI_SECRET_KEY_FILE=/app/backend/data/.webui_secret_key \
    -v "$VOL:/app/backend/data" "$OWUI_IMG"
}
ready_owui() { curl -fsS -o /dev/null http://127.0.0.1:18090/health; }
setup_owui() { :; }
use_owui() { secret; python3 -I "$S/use_owui.py" 18090 "$B/.pass"; }
