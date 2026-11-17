# V1.10 release, 16 November 2026

Released at the operator's word, 2026-11-16: "Do the release", in the
order set out to them that day: push, a native release carrying
`c327032f`, then the resolver at `join.svrnme.sh`.

## Scope and source evidence

The release before this one is V1.9 at 16b7eb46; its readback is
`../../v1.9/2026-11-16/README.md`. Before this run, `group.svrnme.sh` was
serving V1.9's page: at 2026-11-17 05:33 UTC its `sw.js` named version
`fb5ab80cf0a6`, V1.9's own, and `join.svrnme.sh` served V1.9's
`handler.mjs` (sha256 `0d6d9f01c8b6…`, equal to `v1.9:resolver/handler.mjs`)
and answered a phrase of three words and a number 503 `unavailable`
(`resolver-before.txt`).

Since `v1.9`, the shipped files that changed are the home page's
`page/group-links.js`, `page/group-link.js`, `page/home-page.js`,
`ui/view.js` and `look.css`; the phone's page's `web/app.js` and
`web/page/`'s `computer.js`, `computer-line.js`, `enter.js`, `join.js`,
`name-form.js` and `page.js`; of Rust, `svrngs-node/src/publication.rs`
and `svrngs-core/src/act_set.rs` (`lib.rs` adds only its test module);
and the resolver's `handler.mjs` and
`store.mjs`. One crate is added, `qrcodegen` `=1.8.0`, for the QR code
(65074824). All other changes are tests, docs, the lean check and records.
`git diff v1.9..HEAD -- scripts/trust_floor.txt` is empty.

| Release note | Source or run |
|---|---|
| One Invite people press shows a link, a short address and a QR code; Close joining ends all three; Invite someone to is gone | 57c170bb (`home/page/group-links.js` `ways`, `PublicationControls`), 65074824 (`publication.rs` `shown` carries `link` and `qr` while it admits); `crates/svrngs-node/tests/home/publication.rs` |
| The focus moves to Copy the link | 45fbaab7; `web/tests/pages/publication.test.mjs` 'the answer to a press takes the focus its button lost' |
| It works on every computer: svrngs's own resolver unless a `join-resolver` file names another | 51f5e669, `publication.rs` `resolver`; `src/publication_tests.rs` `a_computer_naming_no_resolver_publishes_to_svrngs_s_own` |
| If the resolver doesn't answer, the link and QR code still work | 57c170bb, `group-links.js` `publicationInfo` (pending); 65074824 |
| Three words and a number; no two words a letter apart; a slip still finds it; capitals, spaces, dots; an unread word named; five-word addresses keep working | c327032f, 02ec3ce3, `resolver/handler.mjs` `read`; `publication.rs:296` takes four or five parts; `web/tests/resolver-contracts.test.mjs` 'an address is read forgivingly, …', 'an address that cannot be read names what is wrong, …', 'phrase source, pruned list, space and Rust signing-key vector are pinned' |
| Guessing held back: 30 misses an hour, 100 a day, a network; a hit counts for nothing | 2241e350, `resolver/store.mjs`; `web/tests/resolver.integration.test.mjs` 'misses, not lookups, spend the guessing budget, …'; `resolver-contracts.test.mjs` 'a guesser spending every miss for a month finds an open invitation one time in a million or fewer'; `docs/ONBOARDING.md` §5 |
| Adding your phone asks first | 57c170bb, `group-link.js` `ending`; `home-acts.test.mjs` 'a link for your own device, made, shown and copied, said first to end an open invitation' |
| A phone at a short address joins in its browser | 03e7ac52, `web/page/join.js`; `web/tests/pages/native-arrival.test.mjs` 'a phone at a short address joins in the browser, …', 'a phone in the group already, …' |
| Windows is told what to do | 77d355a9, `web/app.js`, `web/page/computer.js` `lines`, `computer-line.js`; `group-page.test.mjs` 'a Windows computer is offered no line, …'; `native-arrival.test.mjs` 'a Windows computer at a short address is offered no line, …'. Seen in Chromium with a Windows user agent, never on Windows |
| Plainer words; the welcome's hello is a photo or any file | 4454de24, edde0665; `native-arrival.test.mjs`, `web/tests/pages/opens.test.mjs` |
| Groups, lends and the record work as in V1.9 | `git diff v1.9..HEAD --stat -- '*.rs'`: `publication.rs` and its tests, and `act_set.rs`, whose `ActSet::of` collects its keyed lines with the same set either way (1cc00b07, behavior kept; `act_set_tests.rs`, d893b559). No route, act kind or record format changes |
| Nothing else changes | the other commits since `v1.9` change tests, docs, the lean check and records: 35c53cc3, a79d1d5b (economizing), cf9749a9 (the caps down), 7dc3ffe4, 551d6a1c, d7a7fe75, d4e8bedd (the resolver's types, reasons and records), 937f9462, bcc9126f, d39afd3a, 071ffb80, 24205d87, 46f002f3, 6be96134, e77f6f6b |

## Validation before packaging

`scripts/check.sh` ran whole on the Mac at 46f002f3 under the gate lock:
exit 1 in 1,316 s, 44 of its 45 steps passed, none could-not-judge or
never-ran (`gate-46f002f3.txt`). App laws failed on one law, the grapevine
canary's "Waller's log names Sam". Every step of the scene had held. The
check was a substring search over Waller's log, whose base64 spells a
three-letter name about 1 run in 270. e77f6f6b fixed the check, with plants,
and `node web/tests/run.mjs apps`, the gate's app laws step, then passed
175 of 175 under the gate lock (`app-laws-e77f6f6b.txt`;
`../../grapevine/2026-11-16-names/README.md`).

The test audit's whole gate after its own commits, ending at 46f002f3,
passed 45 of 45 in 1,449 s (`../../test-audit/2026-11-16/gate.txt`). So
every step passed whole at 46f002f3 in one run or the other. The only file
e77f6f6b changes is the canary's, which no build ships. 6be96134 adds only
that audit's records. The whole check was not run a third time.

The release commit, tagged `v1.10`, changes no runtime file either: only
`RELEASES.md` and this record. Its pre-commit `scripts/check.sh --fast`
passed.

Not run on Linux: V1.8.2's whole Linux run (`gate-3da19985-linux.txt`) is
still the last one.

## Publication

`publish.py` ran `scripts/deploy-page.sh` under the gate lock in a clean
detached worktree at 18d3b04e (`v1.10`), with a target directory of its
own, from 2026-11-17 06:02 to 06:09 UTC, exit 0 (`publish.json`,
`publish.log`). Free disk was 16 GiB at the start. The record was written
outside the worktree and copied here after. The build meets its groups at
n0's relays (`SHIPPED`), so no relay was asked. The page built as version
660baf7db628, and `svrngs` and `give` built for all four platforms. Vercel
deployment `dpl_4UVhJgrj875JKoMkjiSNvs6WnoBP` reached production, aliased
to `https://group.svrnme.sh`. The CLI wrote `.vercel/`, `.env.local` and
`.gitignore` into the upload directory. All three answered 404 live, and
the directory was deleted after.

## Readback

`verify.py` passed at 2026-11-17 06:09 UTC (`verification.json`). Its six
planted faults were rejected first. Then:

- 78 public files fetched from `group.svrnme.sh` each equaled the built
  site's bytes.
- The live `get/SHA256SUMS` equaled the built one (`SHA256SUMS`,
  `live-sums.txt`) and named exactly the eight downloads.
- The live page is version 660baf7db628, not V1.9's fb5ab80cf0a6, and the
  live `sw.js` names it and sends a navigation to the network first.
- The live `install.sh` equals the built one.
- Both frame policies were as `vercel.json` sets them.
- `release-18d3b04e.group.svrnme.sh` served the same page.
- `.env.local`, `.vercel/project.json` and `vercel.json` answered 404.
- The downloaded darwin-aarch64 `svrngs` and `give` started.

No other platform's program was run here, and no home page was opened
from the published build: the home page's own evidence is the gates
above.

## The resolver

After the page's readback, as the order asks: a V1.10 computer reads
both five-word phrases and the new ones (`publication.rs:296`), so the
downloads went first.

`deploy-resolver.sh v1.10` ran at 2026-11-17 06:09:55 UTC, exit 0
(`resolver-deploy.log`). It uploaded the six runtime files from
`git archive v1.10 resolver`, their digests in `resolver-upload.txt`, to
project `svrngs-join`. Deployment `dpl_8wrJw4Z2rAXgCVJrc1wbA3JK4t8R` was
READY and aliased to `https://join.svrnme.sh`. The production
`SVRNGS_RESOLVER_TOKEN` was the project's already (`vercel env ls
production`, a Secret, 12 days old). No provider data, generation or
reservation was touched. The store's keys are V1.9's: the new counters are
fields in the same quota records, and `lookupMonth` keeps its count under
a cap of 30,000 instead of 20,000 (`git diff v1.9..v1.10 --
resolver/store.mjs`). The upload directory was deleted after.

`verify-resolver.py` passed at 06:10 UTC (`resolver.json`), after its
four plants were rejected:

- `acorn-basil-robin-8982`, never issued, answered 404 `missing`, "No
  invitation has this address…". The resolver before answered 503 to the
  same shape (`resolver-before.txt`). So the new reading is live and it
  reaches its store.
- `zzzzz-basil-robin-0042` answered 404 `wrong`, naming “zzzzz”.
- Both answers were `no-store` and `no-referrer`.
- The live `handler.mjs`, `store.mjs` and `words.txt` equal the tag's.
- `join.svrnme.sh/.env.local`, `.vercel/project.json` and `.gitignore`
  reach the resolver, not a file (404 JSON).

The two lookups spent one miss of this network's budget.

## Not verified, not done

- No real phone, real Windows computer, or the people scene
  (`docs/ONBOARDING.md` §6) was run. A phone at a short address and
  Windows are held by tests in Chromium and WebKit's emulation.
- No host published to the deployed resolver here: the new phrase from
  the live service is held by `resolver.integration.test.mjs` against a
  local Redis, not read from `join.svrnme.sh`. The first **Invite people**
  on an updated computer is that check.
- A V1.9 computer with a `join-resolver` file now refuses the phrase the
  resolver gives (`publication.rs:283` at `v1.9` reads five words only)
  until it updates. A V1.9 computer without one never reached the
  resolver.
- Safari with VoiceOver, and Firefox, were not run.
- No platform's program but darwin-aarch64's was run.
