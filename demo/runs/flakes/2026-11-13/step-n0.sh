set -u; failed=0; set -m
slowest() { case $1 in tests) echo 307 ;; browser) echo 246 ;; scenes) echo 544 ;; "conformance canary") echo 178 ;; playback) echo 225 ;; *) echo 60 ;; esac; }
reached() { [ -z "${SVRNGS_STEPS+x}" ] || printf '%s\n' "$SVRNGS_STEPS" | grep -qxF "$1"; }
step() {  # step <name> <command...>
  reached "$1" || { echo "untouched  $1: nothing changed reaches it (scripts/touched.py)"; return; }
  local name=$1 began=$SECONDS limit=$((5 * $(slowest "$1"))) status watch unjudged="target/could-not-judge/$1.step"; shift
  "$@" >/tmp/svrngs-check.$$ 2>&1 & running=$!
  (sleep "$limit"; kill -TERM -$running) 2>/dev/null & watch=$!
  wait $running; status=$?; { kill -TERM -$watch; kill -TERM -$running && sleep 2; kill -KILL -$running; } 2>/dev/null
  if [ $status = 0 ] && [ -s "$unjudged" ]; then echo "could-not-judge $name ($((SECONDS - began)) s): $(cat "$unjudged")"
  elif [ $status = 0 ]; then echo "passed     $name ($((SECONDS - began)) s)"
  elif ((SECONDS - began >= limit)); then echo "failed     $name: still running at its limit of $limit s, so it and all it started were ended; its last output:"; tail -n 20 /tmp/svrngs-check.$$ | sed 's/^/           /'; failed=1
  else echo "failed     $name ($((SECONDS - began)) s)"; sed 's/^/           /' /tmp/svrngs-check.$$; failed=1; fi
}
rm -rf target/could-not-judge
step "n0" sandbox-exec -f "$1" cargo test -p svrngs --test n0
echo "failed=$failed"
rm -rf target/could-not-judge
step "n0" cargo test -p svrngs --test n0
echo "failed=$failed"
