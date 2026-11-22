#!/bin/sh
# `svrngs-keeper keep <port>` under this computer's own user manager: the
# host half of keeper-promote's check (ralph/next/keeper/STATE.md). Run it on
# the host, not in a container (a toolbox's programs are not the host's),
# with the svrngs-keeper the toolbox built:
#
#   apps/svrngs-keeper/tests/promote.sh target/debug/svrngs-keeper [case…]
#
# The cases are relative, tilde, wrapper, renamed and planted; all run when
# none is named. Each starts a program the way a person would in a terminal
# (a session of its own, led by a shell), keeps it with `keep <port>`, and
# reads what runs after. Their units are svrngs-keeper-test-promote-*,
# removed at the end; their cards are in a scratch XDG_DATA_HOME. The model
# is the 0.5B Qwen llama.cpp's cache holds. It needs linger on already, so
# keep's linger step asks nothing.
set -eu
bin=$(realpath "$1")
shift
cases=${*:-relative tilde wrapper renamed planted}
scratch=$(mktemp -d)
export XDG_DATA_HOME="$scratch/data"
model=$HOME/.cache/llama.cpp/Qwen_Qwen2.5-0.5B-Instruct-GGUF_qwen2.5-0.5b-instruct-q4_k_m.gguf
names=""

cleanup() {
  for name in $names; do
    "$bin" remove "$name" >/dev/null 2>&1 || true
    systemctl --user reset-failed "svrngs-keeper-$name.service" 2>/dev/null || true
  done
  for pid in $(cat "$scratch"/*.pids 2>/dev/null); do kill "$pid" 2>/dev/null || true; done
  rm -rf "$scratch"
}
trap cleanup EXIT

case "$(loginctl show-user "$USER" -p Linger)" in
  Linger=yes) ;;
  *) echo "never-ran  linger is off here, and keep would ask to turn it on"; exit 2 ;;
esac
[ -f "$model" ] || { echo "never-ran  no model at $model"; exit 2; }

free_port() { python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])'; }
holder() { ss -ltnpH "sport = :$1" | grep -o 'pid=[0-9]*' | head -1 | cut -d= -f2; }
# Any HTTP answer, as keep <port> takes one: this llama-server says 404 at /.
answers() { curl -sS -o /dev/null --max-time 1 "http://127.0.0.1:$1$2" 2>/dev/null; }
until_true() { # <seconds> <command…>
  n=$1; shift
  while [ "$n" -gt 0 ]; do "$@" && return 0; sleep 1; n=$((n - 1)); done
  return 1
}
pass() { echo "passed     $*"; }
fail() { echo "failed     $*"; exit 1; }
argv() { tr '\0' ' ' < "/proc/$1/cmdline"; }

# start <dir> <command…>: run it as a terminal would, in a session its shell
# leads, and wait for its port to answer.
start() {
  dir=$1; shift
  (cd "$dir" && setsid sh -c "$*; true" </dev/null >"$scratch/$name.log" 2>&1 &)
  until_true 30 answers "$port" / || fail "$name: what it started never answered on $port"
  before=$(holder "$port")
  echo "$before" >> "$scratch/$name.pids"
}

# keep_it [flag]: answer yes to keep <port> (--name, or the flag given), and
# check its unit holds the port now.
keep_it() {
  echo y | "$bin" keep "$port" "${1:---name}" "$name" > "$scratch/$name.out" 2>&1 || {
    cat "$scratch/$name.out"; fail "$name: keep $port exited $?"
  }
  cat "$scratch/$name.out"
  card="$XDG_DATA_HOME/svrngs-keeper/$name/compose.yaml"
  now=$(holder "$port")
  [ -n "$now" ] && [ "$now" != "$before" ] || fail "$name: port $port is held by ${now:-nothing}, not a new process"
  grep -q "svrngs-keeper-$name.service" "/proc/$now/cgroup" || fail "$name: pid $now is not its unit's"
  kill -0 "$before" 2>/dev/null && fail "$name: the one started by hand, pid $before, still runs"
  true
}

model_case() { # <how the model is named> <folder> <case>
  name=test-promote-$3$$; names="$names $name"; port=$(free_port)
  start "$2" llama-server -m "$1" --host 127.0.0.1 --port "$port" -c 2048 -t 2
  keep_it
  grep -q '^  about: "llama-server with your model ' "$card" || fail "$name: not composed: $(cat "$card")"
  grep -q "Yours was also started with -t: this one starts without it" "$scratch/$name.out" || fail "$name: -t was not said to be left out"
  until_true 30 sh -c "curl -fs -o /dev/null --max-time 1 http://127.0.0.1:$port/health" || fail "$name: the kept llama-server never got ready"
  reported=$(curl -fsS "http://127.0.0.1:$port/props" | python3 -c 'import json,sys; print(json.load(sys.stdin)["model_path"])')
}

for c in $cases; do case $c in
relative)
  mkdir -p "$scratch/m/models" && ln -s "$model" "$scratch/m/models/q.gguf"
  model_case models/q.gguf "$scratch/m" relative
  grep -q -- "--model, \"$scratch/m/models/q.gguf\"" "$card" || fail "relative: the card's model is not its folder's: $(cat "$card")"
  [ "$reported" = "$scratch/m/models/q.gguf" ] || fail "relative: the kept one reports $reported"
  pass "relative: models/q.gguf from $scratch/m, composed from /props as $reported; the kept one answers"
  ;;
tilde)
  model_case "$model" "$scratch" tilde
  rest=${model#"$HOME"/}
  grep -q -- "--model, \"~/$rest\"" "$card" || fail "tilde: the card's model is not under ~/: $(cat "$card")"
  grep -q "\"%h/$rest\"" "$HOME/.config/systemd/user/svrngs-keeper-$name.service" || fail "tilde: the unit has no %h path"
  [ "$reported" = "$model" ] || fail "tilde: the kept one reports $reported"
  pass "tilde: ~/$rest composed, the unit's %h/$rest, and the kept one reports $reported"
  ;;
wrapper)
  name=test-promote-wrapper$$; names="$names $name"; port=$(free_port)
  mkdir -p "$scratch/w" && echo kept > "$scratch/w/index.html"
  printf 'echo serving\npython3 -m http.server --bind 127.0.0.1 "$1"\n' > "$scratch/w/serve.sh"
  start "$scratch/w" sh ./serve.sh "$port"
  wrapper=$(ps -o ppid= -p "$before" | tr -d ' ')
  echo "$wrapper" >> "$scratch/$name.pids"
  keep_it
  grep -q '"./serve.sh"' "$card" && grep -q '^    argv0: "sh"' "$card" || fail "wrapper: the script was not adopted: $(cat "$card")"
  kill -0 "$wrapper" 2>/dev/null && fail "wrapper: its script, pid $wrapper, still runs"
  case "$(argv "$(ps -o ppid= -p "$now" | tr -d ' ')")" in
    "sh ./serve.sh $port ") ;;
    *) fail "wrapper: the kept python's parent is not sh ./serve.sh" ;;
  esac
  pass "wrapper: sh ./serve.sh adopted, not its python; under its unit, sh ./serve.sh starts python again"
  ;;
renamed)
  name=test-promote-renamed$$; names="$names $name"; port=$(free_port)
  start "$scratch" exec -a coder-py python3 -m http.server --bind 127.0.0.1 "$port"
  # --lend names it and then runs svrngs approve: a stand-in svrngs says so.
  mkdir -p "$scratch/bin"
  printf '#!/bin/sh\necho "$@" > %s/approved\n' "$scratch" > "$scratch/bin/svrngs"
  chmod +x "$scratch/bin/svrngs"
  PATH="$scratch/bin:$PATH" keep_it --lend
  [ "$(cat "$scratch/approved" 2>/dev/null)" = "approve $name" ] || fail "renamed: svrngs approve $name was not run"
  grep -q '^    argv0: "coder-py"' "$card" || fail "renamed: argv0 was not kept: $(cat "$card")"
  case "$(argv "$now")" in
    "coder-py -m http.server --bind 127.0.0.1 $port ") ;;
    *) fail "renamed: the kept one runs as $(argv "$now")" ;;
  esac
  pass "renamed: copied as coder-py, the kept one runs as $(argv "$now"), and --lend ran svrngs approve $name"
  ;;
planted)
  name=test-promote-planted$$; names="$names $name"; port=$(free_port)
  printf 'import http.server, os, sys\nif "KEEPER_PLANT" not in os.environ: sys.exit("KEEPER_PLANT is not set")\nhttp.server.test(http.server.SimpleHTTPRequestHandler, port=int(sys.argv[1]), bind="127.0.0.1")\n' > "$scratch/needs.py"
  start "$scratch" KEEPER_PLANT=1 python3 needs.py "$port"
  if echo y | "$bin" keep "$port" --name "$name" > "$scratch/$name.out" 2>&1; then
    cat "$scratch/$name.out"; fail "planted: keep $port succeeded without KEEPER_PLANT"
  fi
  cat "$scratch/$name.out"
  grep -q "none of the settings your terminal added: .*KEEPER_PLANT" "$scratch/$name.out" || fail "planted: KEEPER_PLANT was not named"
  grep -q "So yours is started again as it was" "$scratch/$name.out" || fail "planted: not rolled back"
  again=$(holder "$port")
  echo "$again" >> "$scratch/$name.pids"
  [ -n "$again" ] && [ "$again" != "$before" ] && answers "$port" / || fail "planted: the original does not answer again"
  tr '\0' '\n' < "/proc/$again/environ" | grep -qx KEEPER_PLANT=1 || fail "planted: started again without its environment"
  [ ! -e "$HOME/.config/systemd/user/svrngs-keeper-$name.service" ] || fail "planted: its unit is still there"
  pass "planted: the copy without KEEPER_PLANT failed, its unit is gone, and the original answers again as pid $again with KEEPER_PLANT=1"
  ;;
*) fail "no case $c" ;;
esac; done
