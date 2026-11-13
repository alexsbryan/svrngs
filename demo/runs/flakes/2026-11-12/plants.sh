#!/usr/bin/env bash
# Each plant applied in the clean worktree, the tests it must turn red run
# under the gate lock, the planted file restored from its copy after.
# Output: $S/mac/plant-<name>.log
set -u
S=/private/tmp/claude-502/-Users-alexsbryan-dev-svrngs/ec058323-fd43-47eb-b700-2cdd0033b6ca/scratchpad
cd $S/flake-wt
export CARGO_INCREMENTAL=0
keep() { for f in "$@"; do cp "$f" "$f.unplanted"; done; }
back() { for f in "$@"; do mv "$f.unplanted" "$f"; done; }
run() { # run <name> <cargo test args...>
  local name=$1; shift
  { uptime; lockf -k /tmp/svrngs-gate.lock sh -c 'ulimit -Sn 10240; rm -rf target/could-not-judge target/playback; cargo test --workspace "$@"; echo exit=$?; cat target/could-not-judge/*.txt target/playback/*.txt 2>/dev/null' _ "$@"; uptime; } > $S/mac/plant-$name.log 2>&1
  echo "== $name"; grep -E "^test .*(ok|FAILED)$|^exit=|^could-not-judge|failed|panicked at" $S/mac/plant-$name.log | grep -v "^test result" | cut -c1-220 | head -30
}
case ${1:-} in
undecodable)
  # The film's first 64 KiB zeroed after ffmpeg makes it: its bytes are
  # still answered as kept, but no decoder can play it.
  f=crates/svrngs-node/tests/browser/main.rs; keep $f
  perl -0pi -e 's/(    std::fs::metadata\(path\)\.unwrap\(\)\.len\(\)\n\}\n)/    { let mut b = std::fs::read(path).unwrap(); b[..65536].fill(0); std::fs::write(path, b).unwrap(); }\n$1/' $f
  grep -c "fill(0)" $f
  run undecodable --test browser -- playback:: a_lent_film_comes_under a_lend_worker_that_ guests::sam_opens
  back $f ;;
range)
  # Every range answered from one byte past where it was asked.
  f=crates/svrngs-node/tests/door/origins.rs; keep $f
  sed -i '' 's/f.seek(std::io::SeekFrom::Start(start)).await?;/f.seek(std::io::SeekFrom::Start(start + 1)).await?;/' $f
  grep -c "start + 1" $f
  run range --test browser -- a_lent_film_comes_under a_lend_worker_that_ guests::sam_opens
  back $f ;;
rate-cookie)
  # The door counting no reads; the signer leaving its handoff cookie.
  keep crates/svrngs-node/src/door/limit.rs web/sign.js
  perl -0pi -e 's/(pub fn read\(&mut self, at: \(LendName, String, Label\), now: Instant\) -> Option<Duration> \{\n)/$1        if true { return None; }\n/' crates/svrngs-node/src/door/limit.rs
  perl -pi -e 's/^    document\.cookie = `\$\{name\}=; Max-Age=0.*$/    void name;/' web/sign.js
  grep -c "if true" crates/svrngs-node/src/door/limit.rs; grep -c "void name" web/sign.js
  run rate-cookie --test browser -- acts::a_phone_s_page_reading_past carried::an_app_s_entry_is_approved
  back crates/svrngs-node/src/door/limit.rs web/sign.js ;;
seeks)
  # The canary's audio never there to play.
  f=apps/web-conformance/checks.js; keep $f
  perl -pi -e "s/        src: 'big.wav',/        src: 'not-big.wav',/" $f
  grep -c "not-big" $f
  run seeks --test conformance seeks_back_to_back::
  back $f ;;
esac
