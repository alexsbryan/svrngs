#!/bin/bash
# observe.sh <port>: from a listening TCP port to the process behind it, as the
# person, no root, no lsof. Never reads /proc/<pid>/environ.
port=$1
echo "## ss -ltnpH sport = :$port"
ss -ltnpH "sport = :$port"
pid=$(ss -ltnpH "sport = :$port" | grep -o 'pid=[0-9]*' | head -1 | cut -d= -f2)
echo "pid from ss: ${pid:-NONE}"
echo "## without ss: /proc/net/tcp{,6} inode -> /proc/*/fd socket:[inode]"
hexport=$(printf '%04X' "$port")
inodes=$(awk -v p=":$hexport" '$4=="0A" && $2 ~ p"$" {print $10}' /proc/net/tcp /proc/net/tcp6)
for ino in $inodes; do
  for fd in /proc/[0-9]*/fd/*; do
    [ "$(readlink "$fd" 2>/dev/null)" = "socket:[$ino]" ] && echo "inode $ino -> $fd"
  done
done | sort -u
awk -v p=":$hexport" '$4=="0A" && $2 ~ p"$" {print FILENAME": local "$2" uid "$8}' /proc/net/tcp /proc/net/tcp6
[ -z "$pid" ] && exit 1
echo "## owner"; stat -c 'uid=%u user=%U' /proc/$pid
echo "## cmdline"; tr '\0' ' ' < /proc/$pid/cmdline; echo
echo "## cwd"; readlink /proc/$pid/cwd
echo "## exe"; readlink /proc/$pid/exe
ppid=$(awk '/^PPid:/{print $2}' /proc/$pid/status)
echo "## parent"; echo "ppid=$ppid comm=$(cat /proc/$ppid/comm) cmd=$(tr '\0' ' ' < /proc/$ppid/cmdline | head -c 200)"
echo "## ancestors"; p=$pid; while [ "$p" -gt 1 ]; do p=$(awk '/^PPid:/{print $2}' /proc/$p/status); printf '%s:%s ' "$p" "$(cat /proc/$p/comm)"; done; echo
echo "## cgroup (systemd unit, if any)"; cat /proc/$pid/cgroup
echo "## started"; ps -o lstart= -p $pid
echo "## threads/children"; ps --ppid $pid -o pid=,comm= | head
echo "## files open for writing (fdinfo flags & O_ACCMODE in {1,2}), regular files only"
for fd in /proc/$pid/fd/*; do
  t=$(readlink "$fd" 2>/dev/null) || continue
  case "$t" in /*) ;; *) continue;; esac
  [ -f "$t" ] || [ "${t% (deleted)}" != "$t" ] || continue
  fl=$(awk '/^flags:/{print $2}' "/proc/$pid/fdinfo/${fd##*/}")
  acc=$(( 0$fl & 3 ))
  [ $acc -ne 0 ] && echo "fd ${fd##*/} flags=$fl acc=$acc $t"
done
echo "## mmapped files under cwd/home (read side, first 15)"
awk '$6 ~ /^\// {print $6}' /proc/$pid/maps | sort -u | grep -v -E '^/(usr|lib|lib64)/' | head -15
