#!/bin/sh
# install.sh — svrngs on this computer from the group's link (DESIGN §7,
# "A computer gets svrngs from the group's link"), the line the group's
# page gives:
#
#   curl -fsSL <page>install.sh | sh -s -- '<link>' [--as '<your name>']
#
# or, for someone with no link yet, who makes a group on their home page
# (start-group), the page's address where the link goes:
#
#   curl -fsSL <page>install.sh | sh -s -- '<page>'
#
# or, for a relay (`svrngs relay`; DESIGN §3, "A relay anyone runs"), the
# page's address where the link goes:
#
#   curl -fsSL <page>install.sh | sh -s -- '<page>' --relay <name> [--open]
#
# It gets svrngs for this computer from the page's address the link names
# (else $SVRNGS_FROM), refuses it unless it matches SHA256SUMS there, puts
# it in ~/.local/bin with no sudo, joins the link as `svrngs join` does
# when there is one, keeps the page it came from in the data dir (where a
# group made on the home page keeps it, for its https links), keeps
# svrngs running whenever this person logs in (a LaunchAgent on
# macOS, a systemd --user unit on Linux), and opens their home page. Run
# again, it updates in place, and svrngs joins nothing twice. With
# --relay it joins nothing and keeps `svrngs relay` running instead, beside
# any svrngs kept already: a LaunchAgent on macOS; on Linux, where only the
# system lets a program take ports 80 and 443, the system unit printed for
# whoever runs the server to install as root — never installed here. Every
# step is said; a refusal says what was refused and why (AGENTS.md 1, 6).
set -eu

say() { printf '%s\n' "$*"; }
no() {
  printf 'svrngs: %s\n' "$*" >&2
  exit 1
}

link="${1:-}"
if [ $# -gt 0 ]; then shift; fi # what follows the link, `--as <name>`, is join's
relay=""
if [ "${1:-}" = --relay ]; then
  relay="${2:-}"
  [ -n "$relay" ] || no "--relay wants the name the relay is reached at, as relay.example.org is."
  shift 2
  for a in "$@"; do [ "$a" = --open ] || no "a relay takes --open and nothing else, not $a."; done
fi
from="${SVRNGS_FROM:-${link%%#*}}"
case "$from" in
  http://* | https://*) ;;
  *) no "paste the whole line from your group's page, its link included: it says where svrngs comes from." ;;
esac
case "$from" in */) ;; *) from="$from/" ;; esac

case "$(uname -s)" in Darwin) os=darwin ;; Linux) os=linux ;; *) os="" ;; esac
case "$(uname -m)" in arm64 | aarch64) arch=aarch64 ;; x86_64 | amd64) arch=x86_64 ;; *) arch="" ;; esac
if [ -z "$os" ] || [ -z "$arch" ]; then
  no "svrngs runs on Mac and Linux computers, Intel or ARM; this one is $(uname -s) on $(uname -m), so nothing was installed."
fi
tarball="svrngs-$os-$arch.tar.gz"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
say "Getting svrngs for this computer from ${from}get/$tarball"
curl -fsSL "${from}get/$tarball" -o "$tmp/$tarball" ||
  no "could not get ${from}get/$tarball, so nothing was installed."
curl -fsSL "${from}get/SHA256SUMS" -o "$tmp/SHA256SUMS" ||
  no "could not get ${from}get/SHA256SUMS to check it against, so nothing was installed."
want="$(awk -v f="$tarball" '$2 == f || $2 == "*" f { print $1 }' "$tmp/SHA256SUMS")"
if command -v sha256sum >/dev/null 2>&1; then
  got="$(sha256sum "$tmp/$tarball" | cut -d' ' -f1)"
else
  got="$(shasum -a 256 "$tmp/$tarball" | cut -d' ' -f1)"
fi
if [ -z "$want" ] || [ "$got" != "$want" ]; then
  no "$tarball does not match its sum in SHA256SUMS, so it was refused and nothing was installed."
fi
say "Checked: $tarball matches SHA256SUMS."

dir="${SVRNGS_DIR:-$HOME/.svrngs}"
bin="$HOME/.local/bin"
label="sh.svrnme.svrngs"
unit=svrngs
run="$bin/svrngs"
if [ -n "$relay" ]; then
  label="$label.relay" unit=svrngs-relay run="$run relay $relay $*"
fi
if [ "$os" = darwin ]; then
  kept="$HOME/Library/LaunchAgents/$label.plist"
  stop() { launchctl bootout "gui/$(id -u)/$label" 2>/dev/null; }
else
  kept="$HOME/.config/systemd/user/$unit.service"
  stop() { systemctl --user is-active --quiet $unit.service && systemctl --user stop $unit.service; }
fi
# A svrngs kept running holds its data, which joining needs: stopped
# first, started again below on the new one.
if [ -f "$kept" ] && stop; then say "Stopped $unit, to update it."; fi

mkdir -p "$bin" "$dir"
printf '%s' "$from" >"$dir/origin"
tar -xzf "$tmp/$tarball" -C "$tmp" svrngs
chmod 755 "$tmp/svrngs"
if [ -e "$bin/svrngs" ]; then done="Updated"; else done="Installed"; fi
mv -f "$tmp/svrngs" "$bin/svrngs"
say "$done svrngs in $bin."
case ":$PATH:" in
  *":$bin:"*) ;;
  *) say "$bin is not on your PATH, so to run svrngs yourself type $bin/svrngs; nothing else needs it." ;;
esac

if [ -n "$relay" ] && [ "$os" = linux ]; then
  say "A relay takes ports 80 and 443, which on Linux only the system gives a program. As root, put this in /etc/systemd/system/svrngs-relay.service, then run systemctl enable --now svrngs-relay:"
  printf '%s\n' "" "[Unit]" "Description=svrngs relay" "After=network-online.target" "[Service]" \
    "ExecStart=$run" "Environment=SVRNGS_DIR=$dir" "User=$(id -un)" \
    "AmbientCapabilities=CAP_NET_BIND_SERVICE" "Restart=on-failure" "[Install]" "WantedBy=multi-user.target"
  exit 0
fi
# A link has its token after a `#`; the page's address alone joins nothing.
if [ "${link#*#}" != "$link" ] && [ -z "$relay" ]; then
  SVRNGS_DIR="$dir" "$bin/svrngs" join "$link" --no-run "$@" ||
    no "svrngs is in $bin, and did not join the group: it said why above."
fi

mkdir -p "$(dirname "$kept")"
if [ "$os" = darwin ]; then
  cat >"$kept" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>$label</string>
  <key>ProgramArguments</key><array>$(for a in $run; do printf '<string>%s</string>' "$a"; done)</array>
  <key>EnvironmentVariables</key><dict><key>SVRNGS_DIR</key><string>$dir</string></dict>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>$dir/$unit.log</string>
  <key>StandardErrorPath</key><string>$dir/$unit.log</string>
</dict></plist>
EOF
  launchctl bootstrap "gui/$(id -u)" "$kept" || no "svrngs could not be kept running at login: launchctl refused $kept."
else
  cat >"$kept" <<EOF
[Unit]
Description=svrngs
[Service]
ExecStart=$run
Environment=SVRNGS_DIR=$dir
Restart=on-failure
[Install]
WantedBy=default.target
EOF
  { systemctl --user daemon-reload && systemctl --user enable --now $unit.service; } ||
    no "svrngs could not be kept running at login: systemd refused $kept."
fi
say "$unit runs now, and whenever you log in ($kept)."
[ -z "$relay" ] || exit 0

# Where its home page is: the port svrngs writes to its data, once it
# answers there, and keeps at each start after (start-address).
tries=0
until api="$(cat "$dir/api" 2>/dev/null)" && curl -fsS -o /dev/null "$api/v1/groups" 2>/dev/null; do
  tries=$((tries + 1))
  [ "$tries" -lt 30 ] || no "svrngs did not start within 30 seconds; $dir/svrngs.log says why."
  sleep 1
done
home="http://svrngs.localhost:${api##*:}/"
say "Your home page, the same address each time svrngs starts: $home"
if [ "$os" = darwin ]; then opener=open; else opener=xdg-open; fi
"$opener" "$home" 2>/dev/null || say "Open it in your browser: $opener did not."
