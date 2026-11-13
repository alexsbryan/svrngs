#!/usr/bin/env bash
# One measuring round inside the gate image: check.sh's own step() for the
# named slow steps (FLAKE_STEPS, newline separated), each step's whole log kept,
# per-test lines (no --quiet), load sampled every 5 s.
set -u
cd /work
out=$1
mkdir -p "$out"
npm ci --prefix web --no-audit --no-fund >"$out/npm.log" 2>&1 || echo "npm ci failed" >>"$out/steps.txt"
sed -e 's/^unset SVRNGS_STEPS SVRNGS_TESTS.*/export SVRNGS_STEPS="$FLAKE_STEPS"/' \
    -e 's/wait \$running; status=\$?;/wait $running; status=$?; cp \/tmp\/svrngs-check.$$ "$FLAKE_OUT\/$name.log";/' \
    -e 's/cargo test --quiet/cargo test/g' \
    -e 's/^if \[ "\$failed" != 0 \] || \[ "\${1:-}" = --fast \]; then exit "\$failed"; fi$/:/' \
    scripts/check.sh > scripts/flake-check.sh
if [ "${FLAKE_BUILD:-}" ]; then s=$SECONDS; { cargo test --workspace --no-run && cargo test --workspace --no-run --test browser --test conformance --test canaries --test join_page --test pages --test scenes && scripts/build-web.sh; } >"$out/build.log" 2>&1; echo "build $? $((SECONDS-s))" >>"$out/steps.txt"; fi
(while :; do echo "$(date +%s) $(cut -d' ' -f1-4 /proc/loadavg) $(awk '/MemAvailable/{print $2}' /proc/meminfo)"; sleep 5; done) >"$out/load.txt" & lp=$!
FLAKE_OUT=$out bash scripts/flake-check.sh 2>&1 | tee "$out/check.txt"
kill $lp
cp -r target/could-not-judge target/conformance target/scenes "$out/" 2>/dev/null
cp target/seeks-webkit.log "$out/" 2>/dev/null
echo done >"$out/DONE"
