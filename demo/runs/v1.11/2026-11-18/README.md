# V1.11 release, 18 November 2026

Released at the operator's word, 2026-11-18: "run the next release (it
can only happen on this mac) for a minor bump". The page and downloads at
`group.svrnme.sh`, from the commit tagged `v1.11`, version 1.11.0. The
resolver is not deployed: `git diff v1.10.1..HEAD -- resolver` is empty,
so `join.svrnme.sh` keeps V1.10's.

## Scope and source evidence

The release before this one is V1.10.1 at 7b2628d7; its readback is
`../../v1.10.1/2026-11-17/README.md`. Before this run, `group.svrnme.sh`
was serving V1.10.1's page: at 2026-11-18 16:06 UTC its `sw.js` named
version `d5a753aadf4c`, V1.10.1's own, and its `lend.js` came with the
policy `web/vercel.json` sets for it (`page-before.txt`).

Since `v1.10.1`, the shipped files that changed are, on a computer,
`svrngs-node/src/home.rs`, `door.rs`, `lend.rs`, `share.rs`, `update.rs`,
`surface.rs` and `ring_sync.rs`; the home page's `page/lent-row.js`,
`lent-read.js`, `lend-change.js`, `what-you-lend.js`, `update.js` and
`update-line.js`, which `home.rs` serves from the binary; and
`svrngs/src/cli.rs` and `help.txt`. On the phone's page, `web/lend.js`
alone. In the build, `scripts/build-release.sh` stamps each binary with
the commit it is built from (`SVRNGS_COMMIT`), and `scripts/deploy-page.sh`
ends by printing the GitHub Release line that keeps the version for
packages, which the operator runs. The workspace's version is 1.11.0
(`Cargo.toml`). No crate is added: `git diff v1.10.1..HEAD -- Cargo.lock
web/Cargo.lock` changes only the version of svrngs's own crates,
`svrngs`, `svrngs-core`, `svrngs-node` and `svrngs-runtime`, from 0.1.0
(b97e7dd6 and be20e26e, then this release's commit). All other changes
are tests, docs, the lean check, records and `apps/`, which no build
ships: `build-web.sh` copies `web/`, `crates/svrngs-node/src/home/` and
`docs/brand/`, and `build-release.sh` builds `svrngs` and `give`.

`git diff v1.10.1..HEAD -- scripts/trust_floor.txt` is empty.

| Release note | Source or run |
|---|---|
| A program that wants a header on every request, such as llama-server with `--api-key`, could be lent only from a terminal | `git show v1.10.1:docs/guide/lend.md`, line 20: "If a program wants a password on every request, lend it from a terminal"; e7e354cc's body |
| Its line under **What you lend** says the quoted words; **Header:** says `Authorization`; **What it says:**; **Add this header** | e7e354cc, `home/page/lent-row.js` `Headers` (128-143); 76c85758, `web/tests/pages/opens.test.mjs` 'a header the door adds is given in a lend’s row, and its value sent once, to /lent alone'; the headers run, `../../headers/2026-11-17/README.md` prediction 3 (`4-ana-adds.png`) |
| Your friends' next request carries it, with nothing restarted | the headers run, predictions 1 and 4: Bo's chat 401 before, 200 after, svrngs not restarted (one host, Fedora, one run) |
| Another header the same way; **Stop adding** takes one off | `home.rs` `change` (`lend::keep_header`, an empty value stops one, 536); `lent-row.js:138`; `crates/svrngs-node/tests/home.rs` `a_header_the_door_adds_is_given_on_the_home_page`; the headers run, prediction 7 |
| The line names the header, never what it says, and your friends never receive it | `lent-row.js:132`; `home.rs` `lent` names the kept headers alone (465-467); the headers run, prediction 5 (`grep -r` for the key finds it in no response, log or trace) |
| svrngs puts its own in place of a friend's header of that name, and drops it from an answer that repeats it | `door.rs` `rewrite_headers` (869) and `drop_echoed_secret` (1067), as before this release; `crates/svrngs-node/tests/door/main.rs` `a_forged_identity_header_is_replaced_by_the_verified_one` (a friend's `Authorization` in three spellings, then an echoed one) |
| The program sees all your friends arrive with the one key | `door.rs` `secret_headers` (890): the lend's kept headers, the same for every asker; `docs/guide/lend.md` "they all arrive as that one password" |
| A program that signs each person in itself takes no header | `share.rs` `adds_headers` (328), which `svrngs share`'s prompt and `home.rs` (531, "It signs each person in, so svrngs adds no header to it.") both read |
| `svrngs version` says its release and the commit it was built from | b97e7dd6, `cli.rs:756`; `build-release.sh:39`; `help.txt:97`; READBACK_VERSION |
| V1.11 is 1.11.0 | `Cargo.toml` `[workspace.package]`; svrn-docs `rings/current/SVRNGS_PACKAGE.md`, decision 1 ("the release called V1.10 is `1.10.0`; the next is `1.11.0`") |
| A package's line beside svrngs: it never replaces itself, `svrngs update` says the line, the home page offers no update | b97e7dd6, `update.rs` `UPDATED_BY` (39) and `packaged` (75), `home.rs:359,373` (`packaged`), `update-line.js` `SHOWN` (12, no `packaged`); `crates/svrngs-node/tests/update.rs` `a_svrngs_a_package_updates_never_replaces_itself`; `web/tests/update.test.mjs` 'a svrngs a package updates offers nothing, and runs nothing'. No package was installed by this run |
| Installed with the line, svrngs updates itself as before | `update.rs` `check` asks the page as before when no line is there; `tests/update.rs` `a_page_s_release_replaces_the_binary_read_with_no_socket` |
| svrngs no longer tries to replace a copy it can't write over, and says so before downloading | `update.rs:160-161` ("whoever installed svrngs where it is updates it"); `tests/update.rs` `a_svrngs_that_cannot_write_beside_itself_is_refused_before_the_download` |
| Spot, an example to run from a checkout | c1b933d4, 29b42dfd, 59e46ffc, 91d18af7; `apps/spot/README.md`; the spot run, `../../spot/2026-11-17/README.md` (a stand-in provider, one host). Not in the site or the downloads |
| Groups, lends and the record otherwise work as in V1.10.1 | `git diff v1.10.1..HEAD --stat -- '*.rs'`: `home.rs` (`/lent` takes and names headers; `/update` may answer `packaged`), `door.rs` (a kept header held to `key_header_fits` as the door reads it), `lend.rs`, `share.rs`, `update.rs`, `cli.rs`, and the economizing passes, behavior kept: 837ff35f, bd06f75c, 57a31e07, 7c017670 (a trace's field), bb2e9334 (`web/lend.js`). No act kind or record format changes |
| Press **Update and restart**, or run `svrngs update` and then `svrngs restart` | `home/page/update-line.js:51` (the button's words); `cli.rs:767` ("Updated. `svrngs restart` uses it.") |
| Your phone's page updates itself | `web/sw.js` sends a navigation to the network first (verify.py `network_first`, read on the live `sw.js`) |
| Nothing else changes | the other commits since `v1.10.1` change tests, docs, the lean check, records and `apps/`: 3f02df6e (V1.10.1's record), a35bd10b, c0859400, d7f0dfd5, e5085d44 (`docs/PLAN.md`), ccbaf094 (`docs/seams.tsv`), cc55495d, 0c6276ec (the lean check's caps), 4b3de073, 0f7f3d15 (the headers run), 1aef4f5b, 000c3cb1 (the spot run), ba3e9cff, 7a1ac173, aedea135, 9ddc850f, 73cd91d0, 74baf4ac, 3028f8e0, abe7bb11 (Magnets, `apps/magnets/`, and its runs), and e72a00a1 (a merge, `scripts/lean_check.py` its one conflict) |

## Validation before packaging

VALIDATION

## Publication

PUBLICATION

## Readback

READBACK

## Not verified, not done

NOT_VERIFIED
