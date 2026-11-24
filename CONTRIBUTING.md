# Work on svrngs

Use the Rust toolchain pinned in [`rust-toolchain.toml`](rust-toolchain.toml)
and the Node version in [`.nvmrc`](.nvmrc).

## Prepare a checkout

From the repository root, get cr-sqlite's source and Rust dependencies once
per checkout or worktree. The build itself does not fetch them:

```sh
git submodule update --init
cargo fetch --manifest-path vendor/cr-sqlite/core/rs/bundle_static/Cargo.toml
```

## Run the checks

```sh
npm ci --prefix web
scripts/check.sh
```

See [Checks](docs/CHECKS.md) for focused commands and the
[Linux container gate](scripts/gate/README.md) for running the whole suite.
[`scripts/lean_check.py`](scripts/lean_check.py) holds the code to a line budget.

To work on the group’s page in `web/`, run `scripts/build-web.sh`, then
`node web/tests/dev.mjs`. It prints how to open the local HTTPS page in Chrome
and point a group at it. Safari can’t save there ([why](demo/runs/security/2026-11-02-webkit-partition/README.md)).
Format JavaScript with `web/node_modules/.bin/oxfmt '**/*.{js,mjs,cjs}'`.
To skip formatting-only commits in `git blame`, set
`git config blame.ignoreRevsFile .git-blame-ignore-revs` once.

Some files were forked from commonwealth-ai. The build doesn’t depend on that
checkout. To compare the originals, check it out at `~/dev/commonwealth-ai`
or set `COMMONWEALTH_AI` to its path and run `scripts/ports.py`.
`scripts/principles_check.py --sync` re-pins [`docs/principles/`](docs/principles/).

[North star](NORTH_STAR.md) · [Overview](docs/OVERVIEW.md) · [Design](docs/DESIGN.md) · [Architecture](ARCHITECTURE.md) · [Plan](docs/PLAN.md) · [Agent instructions](AGENTS.md)

The original Discord expense bot is at tag `e0-bot`.
svrngs is licensed under [AGPL-3.0-or-later](LICENSE).
