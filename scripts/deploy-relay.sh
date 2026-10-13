#!/usr/bin/env bash
# deploy-relay.sh — put up svrngs's own relay, where every computer's svrngs
# meets its groups unless a group keeps its own, once the build ships it
# (`DEFAULT_RELAY` and `SHIPPED`, crates/svrngs/src/cli.rs; DESIGN §3,
# "Where a computer meets its groups"). Parked at the gate until the
# operator starts paying for it (DESIGN §9; the operator, 2026-10-13); the
# switch, theirs, in order: `SHIPPED` flipped to `MeetAt::Svrngs`, this
# script, then scripts/deploy-page.sh, which publishes a build meeting its
# groups at svrngs's only once this relay answers. Written by relay-default
# (c), never run there. In order:
#  1. this commit's svrngs for Linux, built here (scripts/build-release.sh);
#  2. a DigitalOcean droplet in US West (DESIGN §10), a cloud firewall
#     letting in SSH and what `svrngs relay` serves: TCP 80 and 443, UDP
#     7842 (crates/svrngs/src/relay.rs);
#  3. svrngs there by install.sh as the user svrngs, from the build, and
#     the system unit install.sh prints for a relay installed as root and
#     started: `svrngs relay <name> --open`;
#  4. the name's A and AAAA records at Vercel, svrnme.sh's DNS, in place of
#     its wildcard's answer;
#  5. a wait until the relay answers (scripts/relay-answers.sh), Let's
#     Encrypt's certificate asked by the relay once the name leads there.
# Only then may a build that meets its groups there be published:
# deploy-page.sh refuses before. Needs doctl and vercel logged in, an SSH key on the
# DigitalOcean account, and build-release.sh's toolchain. Run again, it
# makes a second droplet: delete the first.
#
#   scripts/deploy-relay.sh --ssh-key <fingerprint> [--name relay.svrnme.sh]
#     [--region sfo3] [--size s-1vcpu-1gb] [--scope svrnmesh]
set -euo pipefail
NAME=relay.svrnme.sh REGION=sfo3 SIZE=s-1vcpu-1gb SCOPE=svrnmesh KEY=""
while [ $# -gt 0 ]; do
  case "$1" in
    --ssh-key) KEY="${2:?--ssh-key needs a fingerprint}"; shift 2 ;;
    --name) NAME="${2:?--name needs a name}"; shift 2 ;;
    --region) REGION="${2:?--region needs a region}"; shift 2 ;;
    --size) SIZE="${2:?--size needs a size}"; shift 2 ;;
    --scope) SCOPE="${2:?--scope needs a team}"; shift 2 ;;
    *) echo "deploy-relay: unknown argument $1" >&2; exit 2 ;;
  esac
done
[ -n "$KEY" ] || { echo "deploy-relay: --ssh-key, one of \`doctl compute ssh-key list\`" >&2; exit 2; }
cd "$(dirname "$0")/.."
SITE="$(mktemp -d)/site"
scripts/build-release.sh --out "$SITE"
cp web/install.sh "$SITE/install.sh"
read -r ID IP4 IP6 < <(doctl compute droplet create svrngs-relay --region "$REGION" --size "$SIZE" \
  --image ubuntu-24-04-x64 --ssh-keys "$KEY" --enable-ipv6 --wait \
  --format ID,PublicIPv4,PublicIPv6 --no-header)
echo "deploy-relay: droplet $ID at $IP4 and $IP6"
any="address:0.0.0.0/0,address:::/0"
doctl compute firewall create --name svrngs-relay --droplet-ids "$ID" \
  --inbound-rules "protocol:tcp,ports:22,$any protocol:tcp,ports:80,$any protocol:tcp,ports:443,$any protocol:udp,ports:7842,$any" \
  --outbound-rules "protocol:tcp,ports:all,$any protocol:udp,ports:all,$any protocol:icmp,$any" >/dev/null
until ssh -o StrictHostKeyChecking=accept-new "root@$IP4" true 2>/dev/null; do sleep 5; done
scp -rq "$SITE" "root@$IP4:/tmp/site"
# install.sh fetches from an address, so the build is served on the
# droplet's loopback while it runs; its unit is what it prints from [Unit].
ssh "root@$IP4" "NAME=$NAME sh -s" <<'ON_THE_DROPLET'
set -eu
id svrngs >/dev/null 2>&1 || useradd --create-home svrngs
chmod -R a+rX /tmp/site
(cd /tmp/site && exec python3 -m http.server 8000 --bind 127.0.0.1 >/dev/null 2>&1) & served=$!
sleep 2
su - svrngs -c "SVRNGS_FROM=http://127.0.0.1:8000/ sh /tmp/site/install.sh https://$NAME/ --relay $NAME --open" |
  sed -n '/^\[Unit\]/,$p' >/etc/systemd/system/svrngs-relay.service
kill "$served"
grep -q '^ExecStart=' /etc/systemd/system/svrngs-relay.service || { echo "install.sh printed no unit" >&2; exit 1; }
systemctl daemon-reload && systemctl enable --now svrngs-relay
ON_THE_DROPLET
vercel dns add "${NAME#*.}" "${NAME%%.*}" A "$IP4" --scope "$SCOPE"
vercel dns add "${NAME#*.}" "${NAME%%.*}" AAAA "$IP6" --scope "$SCOPE"
echo "deploy-relay: waiting for https://$NAME/ to answer — the name, then its certificate"
until scripts/relay-answers.sh "https://$NAME/" 2>/dev/null; do sleep 15; done
echo "deploy-relay: https://$NAME/ answers; a build whose default it is may be published now (scripts/deploy-page.sh)"
