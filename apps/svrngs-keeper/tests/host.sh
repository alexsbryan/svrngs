#!/bin/sh
# A kept native card under this computer's own user manager: the host half of
# keeper-units' check (ralph/next/keeper/STATE.md). Run it on the host, not in
# a container, with the svrngs-keeper the toolbox built:
#
#   apps/svrngs-keeper/tests/host.sh target/debug/svrngs-keeper
#
# Its unit is svrngs-keeper-test-host<pid>.service, removed at the end. Its
# kept card is in a scratch XDG_DATA_HOME; the unit goes where the user
# manager reads units, ~/.config/systemd/user. It needs linger on already, so
# keep's linger step asks nothing. PLANT=no-restart sets Restart=no after the
# second keep, and the kill -9 step must fail.
set -eu
bin=$(realpath "$1")
name=test-host$$
unit=svrngs-keeper-$name.service
scratch=$(mktemp -d)
export XDG_DATA_HOME="$scratch/data"
folder="$scratch/folder"
mkdir -p "$folder"
echo "the program's own data" > "$folder/kept.txt"
port=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
cat > "$scratch/compose.yaml" <<EOF
x-svrngs:
  title: $name
  about: Python's http.server, on loopback
  door: { port: $port, protocol: http }
  state: []
  native:
    command: [/usr/bin/python3, -m, http.server, --bind, 127.0.0.1, "$port"]
    folder: $folder
EOF

cleanup() {
  "$bin" remove "$name" >/dev/null 2>&1 || true
  rm -f "$HOME/.config/systemd/user/$unit"
  systemctl --user daemon-reload
  systemctl --user reset-failed "$unit" 2>/dev/null || true
  rm -rf "$scratch"
}
trap cleanup EXIT

case "$(loginctl show-user "$USER" -p Linger)" in
  Linger=yes) ;;
  *) echo "never-ran  linger is off here, and keep would ask to turn it on"; exit 2 ;;
esac

show() { systemctl --user show "$unit" -p "$1" --value; }
answers() { curl -fsS -o /dev/null --max-time 1 "http://127.0.0.1:$port/kept.txt" 2>/dev/null; }
until_true() { # <seconds> <command…>
  n=$1; shift
  while [ "$n" -gt 0 ]; do "$@" && return 0; sleep 1; n=$((n - 1)); done
  return 1
}
pass() { echo "passed     $*"; }
fail() { echo "failed     $*"; exit 1; }

"$bin" keep "$scratch" --name "$name"
until_true 10 answers || fail "keep: nothing answers on $port within 10 s"
pid=$(show MainPID)
pass "keep: $unit answers on $port, pid $pid, $(show UnitFileState)"

"$bin" keep "$scratch" --name "$name"
[ "$(show MainPID)" = "$pid" ] || fail "keep twice: pid $pid became $(show MainPID)"
[ "$(show NRestarts)" = 0 ] || fail "keep twice: NRestarts $(show NRestarts)"
[ "$(cat "$folder/kept.txt")" = "the program's own data" ] || fail "keep twice: the folder changed"
pass "keep twice resets nothing: pid $pid, NRestarts 0, its folder as it was"

if [ "${PLANT:-}" = no-restart ]; then
  sed -i 's/^Restart=on-failure$/Restart=no/' "$HOME/.config/systemd/user/$unit"
  systemctl --user daemon-reload
  echo "planted    Restart=no"
fi

kill -9 "$pid"
back() { p=$(show MainPID); [ "$p" != 0 ] && [ "$p" != "$pid" ] && answers; }
until_true 10 back || fail "kill -9: not back within 10 s ($(show ActiveState)/$(show SubState))"
pass "kill -9 brings it back: pid $(show MainPID), NRestarts $(show NRestarts)"

"$bin" stop "$name"
sleep 12
[ "$(show ActiveState)" = inactive ] || fail "stop: $(show ActiveState) after 12 s"
! answers || fail "stop: still answering after 12 s"
[ "$(show UnitFileState)" = disabled ] || fail "stop: $(show UnitFileState), so a boot would start it"
pass "stop stays stopped past 10 s: inactive, disabled, nothing on $port"

"$bin" start "$name"
until_true 10 answers || fail "start: nothing answers on $port within 10 s"
pass "start: answering again, $(show UnitFileState)"

"$bin" list

"$bin" remove "$name"
[ ! -e "$HOME/.config/systemd/user/$unit" ] || fail "remove: the unit file is still there"
[ "$(show LoadState)" = not-found ] || fail "remove: the unit is $(show LoadState)"
! answers || fail "remove: still answering"
[ "$(cat "$folder/kept.txt")" = "the program's own data" ] || fail "remove: its folder changed"
[ -f "$XDG_DATA_HOME/svrngs-keeper/$name/compose.yaml" ] || fail "remove: its card is gone"
pass "remove leaves its data: the folder and the card stay, the unit is gone"
"$bin" list
