# V1.11.1, published and read back

V1.11.1 is a security release: a device its person removes stays removed
(`df2e2ee9`, Eudoxia `560b0c2`). It is main at `24b6ee7c`, so it also
carries what main gained since V1.11's `08441174`. RELEASES.md says what
a person sees.

## Scope

The operator shipped it without the whole check: "no need waiting around
for this check" (2026-11-18). What ran instead:

- `cargo test -p svrngs-core` with Eudoxia `560b0c2`, every test passed,
  including `tests/counter_remove.rs` 4 of 4. Three of the four failed at
  the pin before, `e995acd`.
- web's `cargo build --target wasm32-unknown-unknown` on the new pin.
- The pre-commit tier (`check.sh --fast`) on each commit.

The last whole check, on `3e84f438` before two merges of origin/main,
failed only `relay_fallback::a_fresh_contribution_syncs_after_the_public_reach_is_cut`,
which `docs/seams.tsv:548` calls fragile. Run alone, it passed 3 of 3.

Not run: the whole check on `24b6ee7c`, a Linux gate, the scenes and
browsers on this commit, an installed package, and a phone against the
published page.

## Publication

`publish.py` ran `deploy-page.sh` in a clean detached worktree at
`24b6ee7c`, after `npm ci --prefix web`, under the gate lock, with
CARGO_TARGET_DIR unset: `publish.json` passed, exit 0, from 20:16 to
20:23 UTC. The page built as version `783da285934f`, replacing V1.11's
`095d48338161`.

## Readback

`verify.py` passed at 20:27 UTC (`verification.json`, `verify.out`). It
rejected its ten planted faults first. Then 79 public files at
`group.svrnme.sh` equaled the built site, and the live `SHA256SUMS`
equaled the built one (`live-sums.txt`, `SHA256SUMS`). The live `sw.js`
named `783da285934f` and was network-first.

## The GitHub release

It waits for the release key. The operator chose on 2026-11-18 that
releases are OpenPGP-signed for packagers, and that only the operator,
typing the key's passphrase, signs. `scripts/sign-release.sh v1.11.1
24b6ee7c <site>` does the rest:

- it signs each download and `SHA256SUMS`;
- it signs the tag;
- it creates the GitHub release with the signatures and the public key.

The site is the directory `publish.json` names.
