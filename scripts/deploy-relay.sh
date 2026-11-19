#!/usr/bin/env bash
# deploy-relay.sh — put svrngs's own relay on a box the operator ordered by
# hand (svrn-docs rings/current/RELAY_COMMONS.md design 1: OVH VPS-2 if its
# order confirms a US location and no fair-use limit, else Hetzner US), and
# put this build on it again at each release, so the relay is the build its
# clients are. The box is ordered by hand with IPv4 and IPv6 and a firewall
# letting in TCP 80 and 443 and UDP 7842 (crates/svrngs/src/relay.rs). Over
# SSH, as root: svrngs by install.sh as the user svrngs,
# the unit install.sh prints with its file descriptors raised (a viral
# moment is ~50,000 phones holding a dial, RELAY_COMMONS design 1, and a
# systemd unit's default is 1,024), journald capped (a full disk was part of
# n0's 2024 outage), ssh by key alone, and the OS updating itself. Then the
# name's A and AAAA at Vercel, and a wait until the relay answers — its
# certificate asked by the relay once the name leads there. "Answers" has
# one home, scripts/relay-answers.sh. Then a build whose default relay this
# is may be published (scripts/deploy-page.sh).
#
#   scripts/deploy-relay.sh --host root@<address> [--name relay.svrnme.sh]
#     [--scope svrnmesh] [--dry-run]
#
# --dry-run prints every step and runs none, ssh included. Needs ssh, scp,
# dig and the vercel CLI logged in to the scope, and build-release.sh's
# toolchain.
set -euo pipefail
HOST="" NAME="" SCOPE=svrnmesh DRY=""
while [ $# -gt 0 ]; do
  case "$1" in
    --host) HOST="${2:?--host wants user@address}"; shift 2 ;;
    --name) NAME="${2:?--name wants a name}"; shift 2 ;;
    --scope) SCOPE="${2:?--scope wants a team}"; shift 2 ;;
    --dry-run) DRY=yes; shift ;;
    *) echo "deploy-relay: unknown argument $1" >&2; exit 2 ;;
  esac
done
[ -n "$HOST" ] || { echo "deploy-relay: --host <user@address>: the box ordered by hand (RELAY_COMMONS design 1)" >&2; exit 2; }
cd "$(dirname "$0")/.."
[ -n "$NAME" ] || NAME="$(sed -n 's|^pub const DEFAULT_RELAY: &str = "https://\(.*\)/";$|\1|p' crates/svrngs/src/cli.rs)"
[ -n "$NAME" ] || { echo "deploy-relay: cannot read DEFAULT_RELAY (crates/svrngs/src/cli.rs), and --name was not given" >&2; exit 1; }
say() { printf 'deploy-relay: %s\n' "$*"; }
run() { if [ -n "$DRY" ]; then say "would run: $*"; else "$@"; fi }
SITE="$(mktemp -d)/site" BOX="$(mktemp)"
trap 'rm -rf "${SITE%/site}" "$BOX"' EXIT
# The steps on the box, one ssh run; sh, as the box has it. It prints its
# IPv4 and IPv6 last, for the name's records below.
cat >"$BOX" <<'ON_BOX'
set -eu
id svrngs >/dev/null 2>&1 || useradd --create-home svrngs
chmod -R a+rX /tmp/site
(cd /tmp/site && exec python3 -m http.server 8000 --bind 127.0.0.1 >/dev/null 2>&1) & served=$!
sleep 2
su - svrngs -c "SVRNGS_FROM=http://127.0.0.1:8000/ sh /tmp/site/install.sh https://$NAME/ --relay $NAME --open" |
  sed -n '/^\[Unit\]/,$p' >/etc/systemd/system/svrngs-relay.service
kill "$served"
grep -q '^ExecStart=' /etc/systemd/system/svrngs-relay.service || { echo "install.sh printed no unit" >&2; exit 1; }
sed -i '/^\[Service\]$/a LimitNOFILE=65536' /etc/systemd/system/svrngs-relay.service
systemctl daemon-reload
systemctl enable svrngs-relay >/dev/null
systemctl restart svrngs-relay
mkdir -p /etc/systemd/journald.conf.d
printf '[Journal]\nSystemMaxUse=200M\n' >/etc/systemd/journald.conf.d/10-svrngs.conf
systemctl restart systemd-journald
mkdir -p /etc/ssh/sshd_config.d
printf 'PasswordAuthentication no\nKbdInteractiveAuthentication no\nPermitRootLogin prohibit-password\n' >/etc/ssh/sshd_config.d/10-svrngs.conf
sshd -t
systemctl reload ssh 2>/dev/null || systemctl reload sshd
if command -v apt-get >/dev/null 2>&1; then
  DEBIAN_FRONTEND=noninteractive apt-get update -qq
  DEBIAN_FRONTEND=noninteractive apt-get install -y -qq unattended-upgrades
  printf 'APT::Periodic::Update-Package-Lists "1";\nAPT::Periodic::Unattended-Upgrade "1";\n' >/etc/apt/apt.conf.d/20auto-upgrades
  systemctl enable --now unattended-upgrades
elif command -v dnf >/dev/null 2>&1; then
  dnf install -y -q dnf-automatic
  sed -i 's/^apply_updates *= *no/apply_updates = yes/' /etc/dnf/automatic.conf
  systemctl enable --now dnf-automatic.timer
else
  echo "this box runs neither apt nor dnf: turn its own updates on, then run this again" >&2
  exit 1
fi
echo "ADDRESS4=$(ip -4 route get 1.1.1.1 2>/dev/null | sed -n 's/.* src \([0-9.]*\).*/\1/p')"
echo "ADDRESS6=$(ip -6 route get 2606:4700:4700::1111 2>/dev/null | sed -n 's/.* src \([0-9a-f:]*\).*/\1/p')"
ON_BOX
say "building $(git rev-parse --short HEAD) for Linux"
run scripts/build-release.sh --out "$SITE"
run cp web/install.sh "$SITE/install.sh"
say "checking $HOST answers over ssh"
run ssh -o ConnectTimeout=10 "$HOST" true
run ssh "$HOST" 'rm -rf /tmp/site'
run scp -rq "$SITE" "$HOST:/tmp/site"
if [ -n "$DRY" ]; then
  say "would run on $HOST, as root:"
  sed 's/^/  /' "$BOX"
  IP4=203.0.113.7 IP6=2001:db8::7 # the dry-run's stand-ins; the box's own are read from it
else
  # shellcheck disable=SC2029 # NAME is the box's own name: expanding it here is the point
  out="$(ssh "$HOST" "NAME=$NAME sh -s" <"$BOX")"
  printf '%s\n' "$out"
  IP4="$(printf '%s\n' "$out" | sed -n 's/^ADDRESS4=//p')" IP6="$(printf '%s\n' "$out" | sed -n 's/^ADDRESS6=//p')"
  [ -n "$IP4" ] && [ -n "$IP6" ] || { echo "deploy-relay: $HOST did not say its IPv4 and IPv6 (ip route get); its records cannot be made" >&2; exit 1; }
fi
add() { # add <A|AAAA> <address> — the name's record at Vercel, once
  if [ -n "$DRY" ] || [ "$(dig +short "$1" "$NAME" @ns1.vercel-dns.com 2>/dev/null)" != "$2" ]; then
    run vercel dns add "${NAME#*.}" "${NAME%%.*}" "$1" "$2" --scope "$SCOPE"
  else
    say "$NAME already has its $1 record ($2)"
  fi
}
add A "$IP4"
add AAAA "$IP6"
say "waiting for https://$NAME/ to answer — the name, then its certificate"
if [ -n "$DRY" ]; then
  say "would wait: scripts/relay-answers.sh https://$NAME/ until it answers"
else
  tries=0
  until scripts/relay-answers.sh "https://$NAME/" 2>/dev/null; do
    tries=$((tries + 1))
    [ "$tries" -lt 120 ] || { echo "deploy-relay: https://$NAME/ does not answer after 30 minutes; its records and journalctl -u svrngs-relay on the box say why" >&2; exit 1; }
    sleep 15
  done
  say "https://$NAME/ answers; a build whose default it is may be published now (scripts/deploy-page.sh)"
fi
