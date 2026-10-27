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
# it in ~/.local/bin with no sudo, and on the PATH of the person's login
# shell when it is zsh or bash, joins the link as `svrngs join` does
# when there is one, keeps the page it came from in the data dir (where a
# group made on the home page keeps it, for its https links), keeps
# svrngs running whenever this person logs in (a LaunchAgent on
# macOS, a systemd --user unit on Linux), puts an opener that runs
# `svrngs open` in their Applications (a .desktop entry on Linux), and opens
# their home page with it, a code that becomes its session there. Run
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
[ -n "$os" ] && [ -n "$arch" ] ||
  no "svrngs runs on Mac and Linux computers, Intel or ARM; this one is $(uname -s) on $(uname -m), so nothing was installed."
tarball="svrngs-$os-$arch.tar.gz"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
say "Getting svrngs for this computer from ${from}get/$tarball"
curl -fsSL "${from}get/$tarball" -o "$tmp/$tarball" ||
  no "could not get ${from}get/$tarball, so nothing was installed."
curl -fsSL "${from}get/SHA256SUMS" -o "$tmp/SHA256SUMS" ||
  no "could not get ${from}get/SHA256SUMS to check it against, so nothing was installed."
want="$(awk -v f="$tarball" '$2 == f || $2 == "*" f { print $1 }' "$tmp/SHA256SUMS")"
sum=sha256sum && command -v sha256sum >/dev/null 2>&1 || sum="shasum -a 256"
got="$($sum "$tmp/$tarball" | cut -d' ' -f1)"
[ -n "$want" ] && [ "$got" = "$want" ] ||
  no "$tarball is not the one the release lists, so it was refused and nothing was installed."
say "Checked: $tarball is the one the release lists, unchanged on its way here."

dir="${SVRNGS_DIR:-$HOME/.svrngs}"
bin="$HOME/.local/bin"
label="sh.svrnme.svrngs"
unit=svrngs
run="$bin/svrngs"
[ -z "$relay" ] || label="$label.relay" unit=svrngs-relay run="$run relay $relay $*"
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
# The page it came from, by the name the node reads (svrngs-node invite.rs
# `ORIGIN_FILE`; tests/install.rs reads it by that constant).
printf '%s' "$from" >"$dir/origin"
tar -xzf "$tmp/$tarball" -C "$tmp" svrngs
chmod 755 "$tmp/svrngs"
if [ -e "$bin/svrngs" ]; then done="Updated"; else done="Installed"; fi
mv -f "$tmp/svrngs" "$bin/svrngs"
say "$done svrngs in $bin."
# Where the person's terminal looks for programs gets $bin, once: the profile
# their login shell ($SHELL) reads in a new window, as rustup and uv add
# theirs; any other shell is told the whole path. A `|` keeps $SHELL's part
# apart from $PATH's.
case "${SHELL:-}|$os|:$PATH:" in
  *":$bin:"*) ;;
  */zsh\|*) profile="$HOME/.zshrc" ;;
  */bash\|darwin\|*) profile="$HOME/.bash_profile" ;;
  */bash\|*) profile="$HOME/.bashrc" ;;
  *) say "To run svrngs yourself, type $bin/svrngs: your terminal does not look in $bin. Nothing else needs it." ;;
esac
again="Open a new terminal window, and svrngs runs there by name."
if [ -z "${profile:-}" ]; then :
elif grep -qF "$bin:" "$profile" 2>/dev/null; then say "$profile already adds $bin to where your terminal looks for programs. $again"
else printf '\nexport PATH="%s:$PATH"\n' "$bin" >>"$profile" && say "Added $bin to where your terminal looks for programs, in $profile. $again"; fi

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

# The opener, in the person's Applications on macOS and their applications
# menu on Linux, where "Open svrngs from your computer" sends them: it runs
# `svrngs open`, so the home page opens again with no terminal (give-session).
if [ "$os" = darwin ]; then
  app="$HOME/Applications/svrngs.app/Contents" where=Applications && mkdir -p "$app/MacOS"
  printf '<plist version="1.0"><dict><key>CFBundleExecutable</key><string>svrngs</string></dict></plist>\n' >"$app/Info.plist"
  printf '#!/bin/sh\nSVRNGS_DIR="%s" exec "%s/svrngs" open\n' "$dir" "$bin" >"$app/MacOS/svrngs" && chmod +x "$app/MacOS/svrngs"
else
  app="${XDG_DATA_HOME:-$HOME/.local/share}/applications" where="applications menu" && mkdir -p "$app"
  printf '[Desktop Entry]\nType=Application\nName=svrngs\nExec=env "SVRNGS_DIR=%s" "%s/svrngs" open\n' "$dir" "$bin" >"$app/svrngs.desktop"
fi
# The home page opened, once svrngs answers there: `svrngs open` asks it as
# any program asks it, with the token beside the address it writes to its
# data and keeps at each start after (start-address, give-token).
tries=0
until said="$(SVRNGS_DIR="$dir" "$bin/svrngs" open 2>/dev/null)"; do
  tries=$((tries + 1))
  [ "$tries" -lt 30 ] || no "svrngs did not start within 30 seconds; $dir/svrngs.log says why."
  sleep 1
done
api="$(cat "$dir/api")"
say "Your home page, the same address each time svrngs starts: http://svrngs.localhost:${api##*:}/" "$said"
say "To open it again, open svrngs from your $where."
