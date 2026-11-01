#!/usr/bin/env bash
# Parity (ralph/next/page/CHARTER.md, "What is settled" 6; page-switch (c)):
# every target that includes tests/browser/harness.rs and that
# scripts/check.sh runs, run with the arguments check.sh gives it, but on
# v2 (SVRNGS_UI=v2), the command's binary built with its test-support so it
# serves v2's home page. Each test's verdict goes to target/parity/
# verdicts.tsv: passed, failed, could-not-judge (a pass the files check.sh
# reads say it could not judge) or never-ran (`-- --list` names it on v1
# and v2 gave it no verdict, or its target did not build). Prints the
# counts and each test not passed, and exits 1 on any. A pass the baseline
# (demo/runs/page/2026-10-31-baseline/verdicts.tsv, the empty pages) also
# had is counted apart: v2 did not earn it. xenia is not here: check.sh
# does not run it. Targets named as arguments run alone. Under the host's
# one gate lock (PROMPT fence 12).
set -uo pipefail
[ -n "${PARITY_LOCKED:-}" ] || PARITY_LOCKED=1 exec lockf -k /tmp/svrngs-gate.lock "$0" "$@"
cd "$(dirname "$0")/.."
# The open files check.sh raises (check.sh:7-13), raised here too: under
# launchd's soft limit of 256, a loop's await, the browser target's 94
# tests in one process fail at spawning node ("Too many open files",
# drive.rs:96) and four hang, by the launcher, not the page.
soft=$(ulimit -n) && { ulimit -Sn 10240 2>/dev/null || ulimit -Sn "$(ulimit -Hn)"; }
echo "parity: open files: $soft, raised to $(ulimit -n)"
out=target/parity base=demo/runs/page/2026-10-31-baseline/verdicts.tsv
rm -rf "$out" target/could-not-judge target/conformance target/scenes && mkdir -p "$out"
# Each target, and what check.sh gives its tests after `--`.
for t in browser conformance join_page scenes:--test-threads=2 home_group install leave_page phone_dry relay; do
  name=${t%%:*} after=${t#"$name"}
  [ $# -eq 0 ] || [[ " $* " == *" $name "* ]] || continue
  run=(cargo test --workspace --features svrngs/test-support --test "$name")
  if ! env -u SVRNGS_UI "${run[@]}" -- --list >"$out/$name.list" 2>"$out/$name.build"; then
    printf '%s\t*\tnever-ran\tit did not build: %s\n' "$name" "$out/$name.build" >>"$out/notes"; continue
  fi
  env -u SVRNGS_UI "${run[@]}" -- --list --ignored 2>/dev/null | sed -n 's/: test$//p' | sort >"$out/$name.ignored"
  sed -n 's/: test$//p' "$out/$name.list" | sort | comm -23 - "$out/$name.ignored" | sed "s/^/$name /" >>"$out/listed"
  echo "parity: $name on v2"
  SVRNGS_UI=v2 "${run[@]}" -- ${after#:} 2>&1 | tee "$out/$name.log" | sed -nE "s/^test (.*) \.\.\. (ok|FAILED)\$/$name \1 \2/p" >>"$out/ran"
done
# What check.sh reads as could-not-judge and never-ran, by the test it names.
sed -nE 's/^([a-z_]+::[a-z]+)[: ].*/browser	\1	could-not-judge	browser.txt/p' target/could-not-judge/browser.txt >>"$out/notes" 2>/dev/null
awk '/could-not-judge/ {printf "conformance\t%s::%s\tcould-not-judge\t%s\n", $1 == "computer" ? "on_a_computer" : "on_a_phone_with_the_wildcard", $2, FILENAME}' target/conformance/*.txt >>"$out/notes" 2>/dev/null
for f in target/scenes/*/could-not-judge.txt target/scenes/*/never-ran.txt; do
  [ -f "$f" ] && d=${f%/*} d=${d##*/} && printf 'scenes\t%s::%s\t%s\t%s\n' "${d%-*}" "${d##*-}" "$(basename "$f" .txt)" "$f" >>"$out/notes"
done
touch "$out/listed" "$out/ran" "$out/notes"
awk -F'\t' -v out="$out/verdicts.tsv" '
  FILENAME ~ /listed$/ { split($0, w, " "); k = w[1] "\t" w[2]; v[k] = "never-ran"; next }
  FILENAME ~ /ran$/ { split($0, w, " "); k = w[1] "\t" w[2]; v[k] = w[3] == "ok" ? "passed" : "failed"; next }
  FILENAME ~ /notes$/ { if ($2 == "*") { v[$1 "\t*"] = $3; why[$1 "\t*"] = $4 }
    for (k in v) if (index(k, $1 "\t") == 1 && (substr(k, length($1) + 2) == $2 || k ~ ("::" $2 "$")) && ($3 == "never-ran" || v[k] == "passed")) { v[k] = $3; why[k] = $4 }; next }
  { was[$2 "\t" $3] = $1 }
  END { for (k in v) { printf "%s\t%s\t%s\n", v[k], k, why[k] > out; n[v[k]]++; if (v[k] == "passed" && was[k] == "passed") apart++ }
    printf "parity: %d passed (%d of them passed with no v2, in the baseline), %d failed, %d could-not-judge, %d never-ran\n", n["passed"], apart, n["failed"], n["could-not-judge"], n["never-ran"] }
' "$out/listed" "$out/ran" "$out/notes" $( [ -f "$base" ] && echo "$base")
sort -o "$out/verdicts.tsv" "$out/verdicts.tsv"
grep -v '^passed' "$out/verdicts.tsv" | sed 's/^/parity: /'
[ -s "$out/verdicts.tsv" ] && ! grep -qv '^passed' "$out/verdicts.tsv"
