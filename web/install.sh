#!/bin/sh
# install.sh — svrngs on this computer from the group's link (DESIGN §7,
# "A computer gets svrngs from the group's link"), the line the group's
# page gives:
#
#   curl -fsSL <page>install.sh | sh -s -- '<link>' [--give] [--at '/#group-<id>'] [--as '<your name>']
#
# (--give: Give's program too, for a computer that keeps apps for its
# groups, the line Give's page gives with the page's address for the link;
# apps/give)
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
# or, for whoever already checked a release from its signature, with that
# release's file, before the link or after it (--give-archive is Give's
# program's, with --give):
#
#   sh install.sh --archive <file> '<link or page>' [--give --give-archive <file>]
#
# or, to take svrngs off this computer, `--uninstall` where the link goes:
#
#   curl -fsSL <page>install.sh | sh -s -- --uninstall
#
# It gets svrngs for this computer from the page's address the link names
# (else $SVRNGS_FROM), refuses it unless it matches SHA256SUMS there, or
# takes it from --archive's file as given, asking the page for no sums: a
# release checked by its signature supplies both svrngs and its keeper; the
# page then only joins and is kept for links, so it cannot choose them. It puts
# it in ~/.local/bin with no sudo, and on the PATH of the person's login
# shell when it is zsh or bash, joins the link as `svrngs join` does
# when there is one, keeps the page it came from in the data dir (where a
# group made on the home page keeps it, for its https links), keeps
# svrngs running whenever this person logs in (a LaunchAgent on
# macOS, a systemd --user unit on Linux), puts an opener that runs
# `svrngs open` in their Applications (a .desktop entry on Linux), and opens
# their home page with it, a code that becomes its session there. With
# --give it gets Give's program from the same page, checked by the same
# sums before either is installed, puts it beside svrngs and keeps `give
# keep` running at login too, waited for as it stops. Run
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

# --archive <file> and --give-archive <file> taken out wherever they stand,
# the rest kept in their order.
archive="" give_archive="" n=$#
while [ "$n" -gt 0 ]; do
  case "$1" in
    --archive | --give-archive)
      [ "$n" -gt 1 ] && [ -n "$2" ] || no "$1 wants the file of the release you checked, as svrngs-linux-x86_64.tar.gz is."
      if [ "$1" = --archive ]; then archive="$2"; else give_archive="$2"; fi
      shift 2 && n=$((n - 2)) ;;
    *) set -- "$@" "$1" && shift && n=$((n - 1)) ;;
  esac
done
link="${1:-}"
if [ $# -gt 0 ]; then shift; fi # what follows the link, `--as <name>`, is join's
uninstall=""
if [ "$link" = --uninstall ]; then
  uninstall=yes
  [ $# -eq 0 ] || no "--uninstall takes nothing after it, not $*."
  # Taken out above wherever it stood, so $# never shows it.
  [ -z "$archive$give_archive" ] || no "--uninstall installs nothing, so it takes no --archive."
fi
relay=""
if [ "${1:-}" = --relay ]; then
  relay="${2:-}"
  [ -n "$relay" ] || no "--relay wants the name the relay is reached at, as relay.example.org is."
  shift 2
  for a in "$@"; do [ "$a" = --open ] || no "a relay takes --open and nothing else, not $a."; done
fi
give=""
[ "${1:-}" != --give ] || { give=give && shift; }
[ -z "$give_archive" ] || [ -n "$give" ] ||
  no "--give-archive is the file of Give's program, which only --give installs: add --give after the link."
destination="/"
if [ "${1:-}" = --at ]; then
  destination="${2:-}"
  case "$destination" in /\#group-*) ;; *) no "the local group destination must start with /#group-." ;; esac
  shift 2
fi
from="${SVRNGS_FROM:-${link%%#*}}"
if [ -z "$uninstall" ]; then
  case "$from" in http://* | https://*) ;; *) no "paste the whole line from your group's page, its link included: it says where svrngs comes from." ;; esac
  case "$from" in */) ;; *) from="$from/" ;; esac
fi

case "$(uname -s)" in Darwin) os=darwin ;; Linux) os=linux ;; *) os="" ;; esac
case "$(uname -m)" in arm64 | aarch64) arch=aarch64 ;; x86_64 | amd64) arch=x86_64 ;; *) arch="" ;; esac
[ -n "$os" ] && [ -n "$arch" ] ||
  no "svrngs runs on Mac and Linux computers, Intel or ARM; this one is $(uname -s) on $(uname -m), so nothing was installed."

dir="${SVRNGS_DIR:-$HOME/.svrngs}"
bin="$HOME/.local/bin"
label="sh.svrnme.svrngs"
unit=svrngs
run="$bin/svrngs"
[ -z "$relay" ] || label="$label.relay" unit=svrngs-relay run="$run relay $relay $*"
# What runs at login finds svrngs and Podman (its installer's place, then
# Homebrew's) here, since launchd and systemd give it no shell's PATH.
paths="$bin:/opt/podman/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
# $label's (launchd) or $unit's (systemd) file, and stopping it if kept.
if [ "$os" = darwin ]; then
  at() { kept="$HOME/Library/LaunchAgents/$label.plist"; }
  # bootout returns before launchd lets go, and until then a bootstrap fails
  # with 5. No limit: launchd ends a service, and Give (ExitTimeOut 0) finishes.
  stop() { launchctl bootout "gui/$(id -u)/$label" 2>/dev/null && while launchctl print "gui/$(id -u)/$label" >/dev/null 2>&1; do sleep 0.1; done; }
else
  at() { kept="$HOME/.config/systemd/user/$unit.service"; }
  stop() { systemctl --user is-active --quiet $unit.service && systemctl --user stop $unit.service; }
fi
stopped() { at && if [ -f "$kept" ] && stop; then restart=yes; say "Stopped $unit, to update it."; fi; }
# The opener, in the person's Applications on macOS and their applications
# menu on Linux (below).
if [ "$os" = darwin ]; then opener="$HOME/Applications/svrngs.app"
else opener="${XDG_DATA_HOME:-$HOME/.local/share}/applications/svrngs.desktop"; fi
# A svrngs started by hand holds its data dir's lock, and no launchd or
# systemd stops it: stopped by that lock, waited for, for a limit.
by_hand() {
  for pid in $(lsof -t "$dir/lock" 2>/dev/null); do kill "$pid" && say "Stopped the svrngs running here$1."; tries=0; while [ "$tries" -lt 50 ] && lsof -t "$dir/lock" >/dev/null 2>&1; do tries=$((tries + 1)); sleep 0.1; done; done
}

# --uninstall takes away what this line put here, each step said: what runs
# at login (svrngs, a relay's, Give's), the programs, the opener. It keeps
# the data dir, the person's key and their groups' copies, and the profile's
# PATH line, which other programs in $bin may need: each said, with how to
# remove it. A relay's Linux system unit is root's, so said and left.
if [ -n "$uninstall" ]; then
  for kept_as in sh.svrnme.svrngs:svrngs sh.svrnme.svrngs.relay:svrngs-relay sh.svrnme.give:give; do
    label="${kept_as%%:*}" unit="${kept_as#*:}"
    at
    [ -f "$kept" ] || continue
    stop || :
    [ "$os" = darwin ] || systemctl --user disable --quiet "$unit.service" 2>/dev/null || :
    rm -f "$kept" && say "Stopped $unit, and took it off what runs when you log in ($kept)."
  done
  [ "$os" = darwin ] || systemctl --user daemon-reload 2>/dev/null || :
  by_hand ""
  for program in svrngs svrngs-keeper give; do
    [ ! -e "$bin/$program" ] || { rm -f "$bin/$program" && say "Removed $program from $bin."; }
  done
  [ ! -e "$opener" ] || { rm -rf "$opener" && say "Removed the svrngs opener, $opener."; }
  relay_unit=/etc/systemd/system/svrngs-relay.service
  [ ! -f "$relay_unit" ] || say "A relay runs from $relay_unit, which is root's: as root, run systemctl disable --now svrngs-relay and remove it."
  for profile in "$HOME/.zshrc" "$HOME/.bash_profile" "$HOME/.bashrc"; do
    ! grep -qF "export PATH=\"$bin:\$PATH\"" "$profile" 2>/dev/null ||
      say "$profile still adds $bin to where your terminal looks for programs, since others may live there; remove that line if none do."
  done
  if [ -d "$dir" ]; then
    say "Kept $dir: your key and your groups' copies. To delete them too: rm -rf '$dir'"
  fi
  say "Apps you told svrngs to run are not stopped or removed. Native apps can still start as before; managed containers on Linux need svrngs-keeper to start and stop. To stop running an app and remove its setup, use svrngs-keeper remove <name> before uninstalling, or reinstall first. Your apps' files stay where they were."
  say "svrngs is uninstalled. It left none of your groups: to leave one, run svrngs leave <group> before you uninstall."
  exit 0
fi

tmp="$(mktemp -d)"
restart=""
cleanup() {
  status=$?
  trap - EXIT
  if [ -n "$restart" ]; then
    say "Restarting $unit after the interrupted installation. You can retry the same installation line."
    if [ "$os" = darwin ]; then launchctl bootstrap "gui/$(id -u)" "$kept" || status=1
    else systemctl --user enable --now "$unit.service" || status=1; fi
  fi
  rm -rf "$tmp"
  exit "$status"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM
sum=sha256sum && command -v sha256sum >/dev/null 2>&1 || sum="shasum -a 256"
# Programs $3 onward, unpacked from the archive named by $3, its sum in $want:
# from the file $1 when given, copied first so the bytes summed are
# the bytes installed; else from the page's get/ (scripts/build-release.sh),
# refused unless it is the one SHA256SUMS there lists.
get() {
  file=$1 flag=$2 program=$3; shift 2
  tarball="$program-$os-$arch.tar.gz"
  if [ -n "$file" ]; then
    [ -f "$file" ] && [ -r "$file" ] && cp -- "$file" "$tmp/$tarball" ||
      no "$file is not a file you can read, so nothing was installed: give --${flag}archive the release you checked."
    say "Using $program from $file, the file you gave, in place of the page's."
    want="$($sum "$tmp/$tarball" | cut -d' ' -f1)"
  else
    [ -f "$tmp/SHA256SUMS" ] || curl -fsSL "${from}get/SHA256SUMS" -o "$tmp/SHA256SUMS" ||
      no "could not get ${from}get/SHA256SUMS to check it against, so nothing was installed."
    say "Getting $program for this computer from ${from}get/$tarball"
    curl -fsSL "${from}get/$tarball" -o "$tmp/$tarball" ||
      no "could not get ${from}get/$tarball, so nothing was installed."
    want="$(awk -v f="$tarball" '$2 == f || $2 == "*" f { print $1 }' "$tmp/SHA256SUMS")"
    got="$($sum "$tmp/$tarball" | cut -d' ' -f1)"
    [ -n "$want" ] && [ "$got" = "$want" ] ||
      no "$tarball is not the one the release lists, so it was refused and nothing was installed."
    say "Checked: $tarball is the one the release lists, unchanged on its way here."
  fi
  (cd "$tmp" && tar -xzf "$tarball" "$@" && chmod 755 "$@") ||
    no "${file:-$tarball} must hold each of these programs: $*. Nothing was installed."
}
# Every archive is checked before anything is installed.
get "$archive" "" svrngs svrngs-keeper
svrngs_sum="$want"
[ -z "$give" ] || get "$give_archive" give- give

# A svrngs kept running holds its data, which joining needs: stopped
# first, started again below on the new one. One started by hand holds it
# too, and no launchd or systemd stops it: stopped here, kept at login below.
# Waited for by its lock, for a limit: if it stays, joining says so.
stopped
[ -n "$relay" ] || by_hand ", to update it"

mkdir -p "$bin" "$dir"
# The page it came from, by the name the node reads (svrngs-node invite.rs
# `ORIGIN_FILE`; tests/install.rs reads it by that constant).
printf '%s' "$from" >"$dir/origin"
put() {
  if [ -e "$bin/$1" ]; then done="Updated"; else done="Installed"; fi
  mv -f "$tmp/$1" "$bin/$1" && say "$done $1 in $bin."
}
put svrngs
put svrngs-keeper
# What this computer installed, for the home page's update (svrngs-node
# update.rs `RELEASE_FILE`; tests/install.rs reads it by that constant).
printf '%s' "$svrngs_sum" >"$dir/release"
[ -z "$give" ] || put give
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

# $run kept running at login as $label (launchd) or $unit (systemd), with $hold.
keep() {
  mkdir -p "$(dirname "$kept")"
  if [ "$os" = darwin ]; then
    cat >"$kept" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>Label</key><string>$label</string>
  <key>ProgramArguments</key><array>$(for a in $run; do printf '<string>%s</string>' "$a"; done)</array>
  <key>EnvironmentVariables</key><dict><key>PATH</key><string>$paths</string><key>SVRNGS_DIR</key><string>$dir</string></dict>
  <key>RunAtLoad</key><true/>
  ${hold:-}
  <key>KeepAlive</key><true/>
  <key>StandardOutPath</key><string>$dir/$unit.log</string>
  <key>StandardErrorPath</key><string>$dir/$unit.log</string>
</dict></plist>
EOF
    launchctl bootstrap "gui/$(id -u)" "$kept" || no "$unit could not be kept running at login: launchctl refused $kept."
  else
    cat >"$kept" <<EOF
[Unit]
Description=$unit
[Service]
ExecStart=$run
Environment=SVRNGS_DIR=$dir
Environment=PATH=$paths
Restart=on-failure
${hold:-}
[Install]
WantedBy=default.target
EOF
    { systemctl --user daemon-reload && systemctl --user enable --now $unit.service; } ||
      no "$unit could not be kept running at login: systemd refused $kept."
  fi
  restart=""
  say "$unit runs now, and whenever you log in ($kept)."
}
keep
[ -z "$relay" ] || exit 0
# Give's program, asked for: `give keep` beside svrngs, waited for as it
# stops, since it copies each app it keeps to the group then (apps/give).
if [ -n "$give" ]; then
  label=sh.svrnme.give unit=give run="$bin/give keep" hold="<key>ExitTimeOut</key><integer>0</integer>"
  [ "$os" = darwin ] || hold="TimeoutStopSec=infinity"
  stopped && keep
fi

# The opener, in the person's Applications on macOS and their applications
# menu on Linux, where "Open svrngs from your computer" sends them: it runs
# `svrngs open`, so the home page opens again with no terminal (give-session).
if [ "$os" = darwin ]; then
  app="$opener/Contents" where=Applications && mkdir -p "$app/MacOS"
  printf '<plist version="1.0"><dict><key>CFBundleExecutable</key><string>svrngs</string></dict></plist>\n' >"$app/Info.plist"
  printf '#!/bin/sh\nSVRNGS_DIR="%s" exec "%s/svrngs" open\n' "$dir" "$bin" >"$app/MacOS/svrngs" && chmod +x "$app/MacOS/svrngs"
else
  where="applications menu" && mkdir -p "$(dirname "$opener")"
  printf '[Desktop Entry]\nType=Application\nName=svrngs\nExec=env "SVRNGS_DIR=%s" "%s/svrngs" open\n' "$dir" "$bin" >"$opener"
fi
# The home page opened, once svrngs answers there: `svrngs open` asks it as
# any program asks it, with the token beside the address it writes to its
# data and keeps at each start after (start-address, give-token).
tries=0
until said="$(SVRNGS_DIR="$dir" "$bin/svrngs" open "$destination" 2>"$tmp/open")"; do
  # 1 is svrngs's refusal, said in its words; else it has not started yet.
  [ $? -ne 1 ] || { cat "$tmp/open" >&2; exit 1; }
  tries=$((tries + 1))
  [ "$tries" -lt 30 ] || no "svrngs did not start within 30 seconds; $dir/svrngs.log says why."
  sleep 1
done
api="$(cat "$dir/api")"
say "Your home page, the same address each time svrngs starts: http://svrngs.localhost:${api##*:}/" "$said"
say "To open it again, open svrngs from your $where."
