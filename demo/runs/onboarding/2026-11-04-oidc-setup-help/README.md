# App sign-in setup help — 2026-11-04

Unit: `onboarding-oidc-setup-help`. This corrects the generic page/command's
claim that creating a client signs people in. Both now say **Sign-in
credentials created; app setup still required**, explain the plugin/configure
step, identify the server-side issuer reachability requirement and container
loopback distinction, link to the app-owned guides, and ask for a test through
the group's app list. The saved-client row keeps the reminder after reload;
it does not claim that svrngs verified the app's setup.

`apps/jellyfin/README.md` now provides installation/settings navigation, field
mapping, proxy and Branding/login-button setup, explicit library choice with
no administrator grant, and end-to-end verification/recovery. No app-specific
code was added to the node/core and no real Jellyfin server was changed.

## Reuse and upstream evidence

Read `apps/jellyfin/README.md` at base `51dace89`, and the successful
`demo/runs/oidc/2026-10-21-jellyfin-2/README.md` and its fixture settings.
That trial used Jellyfin **10.11.11** and SSO-Auth **4.0.0.4**; its
`KnownProxies=127.0.0.1` fix made callback origins work in Chromium and
WebKit. Its disposable fixture's all-folders permission is deliberately not
the guide's default. No old fixture secret is copied into this evidence.

Upstream checked 2026-11-04:

```text
$ gh api repos/9p4/jellyfin-plugin-sso --jq '{archived,updated_at,default_branch}'
{"archived":true,"default_branch":"main","updated_at":"2026-11-02T21:33:24Z"}

$ gh api 'repos/9p4/jellyfin-plugin-sso/contents/manifest.json?ref=manifest-release' --jq .content | base64 --decode
[relevant fields from the returned manifest]
name: SSO Authentication
version: 4.0.0.4
targetAbi: 10.11.0.0
timestamp: 2026-03-20T03:13:34Z
sourceUrl: https://github.com/9p4/jellyfin-plugin-sso/releases/download/v4.0.0.4/sso-authentication_4.0.0.4.zip
```

The GitHub `releases/latest` API returned **v4.0.0.3**, whereas the manifest
includes **4.0.0.4**; the guide cites the manifest and actual successful
trial, not a claim that `latest` names the tested version.

The current [upstream README](https://github.com/9p4/jellyfin-plugin-sso)
confirms the archived status, repository URL, plugin installation, Branding
button, username default and permission-overwrite caveat. The versioned form
and serialization were read directly:

```text
$ gh api 'repos/9p4/jellyfin-plugin-sso/contents/SSO-Auth/Config/configPage.html?ref=v4.0.0.4' --jq .content | base64 --decode
$ gh api 'repos/9p4/jellyfin-plugin-sso/contents/SSO-Auth/Config/config.js?ref=v4.0.0.4' --jq .content | base64 --decode
[relevant returned form text]
SSO Settings:
Making changes to this configuration requires a restart of Jellyfin.
Add / Update Provider Configuration
OpenID Endpoint:
OpenID Client ID:
OpenID client secret:
Enable Authorization by Plugin
Enable All Folders
Enabled Folders:
Set default username claim:
Disable OpenID HTTPS Discovery (Insecure)
Do Not Validate OpenID Endpoints (Insecure)
Do Not Validate OpenID Issuer Name (Insecure)
```

Compatibility outside the recorded pairing and the operator's own
configuration walk remain unjudged; neither a credential toggle nor this
wording change is a new successful server sign-in trial.

## Named red plants

Plant: base `51dace89`'s unchanged product wording, with the extended existing
checks. The CLI and immediate-page assertions require truthful credential
state plus plugin and server-side setup guidance; the read-again assertion
requires the saved-client reminder rather than a successful-login claim.

```text
$ lockf -k /tmp/svrngs-gate.lock cargo test -q -p svrngs --test share share_turns_on_signing_people_in_with_svrngs -- --exact
running 1 test
share_turns_on_signing_people_in_with_svrngs --- FAILED
credential creation must explain the remaining app setup
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.97s
exit 101

$ lockf -k /tmp/svrngs-gate.lock cargo test -q -p svrngs-node --test browser the_home_page_turns_on_signing_people_in_with_svrngs
running 1 test
the_home_page_turns_on_signing_people_in_with_svrngs::chromium --- FAILED
credential creation must explain the remaining app setup
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 98 filtered out; finished in 7.74s
exit 101

$ lockf -k /tmp/svrngs-gate.lock cargo test -q -p svrngs --test pages the_secret_stays_shown_when_the_page_is_read_again
running 1 test
index::the_secret_stays_shown_when_the_page_is_read_again::chromium --- FAILED
not so: ["credentials created, app setup still required, its button gone"]
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 36 filtered out; finished in 14.67s
exit 101
```

Failure excerpts omit the page driver's full fixture dump, which includes a
throwaway test-generated client secret. No operator credential was read or
used. The existing secret-not-sent/not-linked/not-traced assertions remain.

## Green after the correction

```text
$ lockf -k /tmp/svrngs-gate.lock sh -c 'cargo test -q -p svrngs --test share && cargo test -q -p svrngs-node --test browser the_home_page_turns_on_signing_people_in_with_svrngs && cargo test -q -p svrngs --test pages the_secret_stays_shown_when_the_page_is_read_again'
running 11 tests
...........
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.69s

running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 11.39s

running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 36 filtered out; finished in 14.61s
exit 0

$ web/node_modules/.bin/oxfmt --check crates/svrngs-node/src/home/page/signing-on.js crates/svrngs-node/src/home/page/lent-row.js && cargo fmt --all --check
All matched files use the correct format.
exit 0

$ python3 scripts/lean_check.py
crates/svrngs-core     3486 / 3494 lines
.                     19789 / 19833 lines
js                     8066 / 8090 lines
swift                     0 / 1 lines
exit 0
```

No counted product growth or cap change; the generic text/link replacements
reuse existing render/command surfaces. A final punctuation-only correction
removed a duplicated period after the guide link after the focused green run.
The locked full `scripts/check.sh` closing gate is still owed at this
checkpoint; it will be run by the loop via `ralph-result await` and its actual
output must be recorded before reporting done.

The inherited untracked V1.6 publish artifacts were inspected and preserved
untouched, without staging or publication.

## Resumed closing gate and vocabulary correction

The loop's first locked closing gate at `39638b89` exited **1** after
1,327 seconds. Its full output is in
`ralph/next/onboarding/ctl/log/sessions/onboarding-oidc-setup-help-1.await.log`.
Every stage passed except the app-noun check:

```text
failed     app nouns (0 s)
           BROKEN  crates/svrngs/src/cli.rs:1409: `attendee` is events's noun
           BROKEN  crates/svrngs-node/src/home/page/signing-on.js:15: `attendee` is events's noun
           89 app nouns, 2 in infra code
```

The generic guidance now says “the computer of the person opening the app”,
preserving the issuer/server distinction without borrowing an app's noun.
The existing vocabulary gate is unchanged; the run above is its actual red
input, not a new check. Focused checks after the wording correction:

```text
$ python3 scripts/vocab_check.py && python3 scripts/lean_check.py && cargo fmt --all --check && web/node_modules/.bin/oxfmt --check crates/svrngs-node/src/home/page/signing-on.js
89 app nouns, 0 in infra code
crates/svrngs-core     3486 / 3494 lines
.                     19789 / 19833 lines
js                     8066 / 8090 lines
swift                     0 / 1 lines
Checking formatting...
All matched files use the correct format.
exit 0

$ lockf -k /tmp/svrngs-gate.lock sh -c 'cargo test -q -p svrngs --test share && cargo test -q -p svrngs-node --test browser the_home_page_turns_on_signing_people_in_with_svrngs && cargo test -q -p svrngs --test pages the_secret_stays_shown_when_the_page_is_read_again'
running 11 tests
...........
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.40s

running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 98 filtered out; finished in 11.53s

running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 36 filtered out; finished in 14.72s
exit 0
```

No product line growth or budget adjustment. A second locked full gate is
owed after this correction and is handed to the loop with `ralph-result
await`; do not count the first run as a passing close.

## Second closing gate: ceiling failure and isolated rerun

The loop's second locked full gate at `f43684f1` exited **1** after
1,276 seconds. Its output is in
`ralph/next/onboarding/ctl/log/sessions/onboarding-oidc-setup-help-2.await.log`.
Every stage passed except the existing ceiling canary; the vocabulary,
browser, pages, scenes and conformance stages all passed. The actual failure:

```text
$ lockf -k /tmp/svrngs-gate.lock scripts/check.sh
failed     ceiling canary (33 s)
CEILING idle round at 25 x 10,000: 0.2943 s of processor, the median of [0.29426700000000494, 0.19435000000000002, 0.7404319999999984], against 0.1 s
an_idle_round_is_under_the_ceiling --- FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 3.80s
note       took 1276 s in all
exit 1
```

Reran the same ceiling test alone under the gate lock, without a product,
instrument or threshold change:

```text
$ lockf -k /tmp/svrngs-gate.lock cargo test --release --quiet -p svrngs-node --test scale -- --ignored --exact an_idle_round_is_under_the_ceiling --nocapture
running 1 test
CEILING idle round at 25 x 10,000: 0.0294 s of processor, the median of [0.23146799999999956, 0.029366000000003112, 0.024200000000000443], against 0.1 s; the last round's account {"acts_moved":0,"asleep":0,"cpu_s":0.024200000000000443,"dialled":24,"folds":{"digest":{"kept":96,"rebuilt":0},"offer":{"kept":48,"rebuilt":0},"register":{"kept":27,"rebuilt":0},"write_index":{"kept":126,"rebuilt":0}},"reached":24,"refused":0,"wall_s":0.003913375}
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 3.68s
exit 0
```

The failure did not reproduce in this isolated run. This is not a diagnosis
of the timing difference or a passing full gate. A fresh locked full gate is
still owed and is handed to the loop with `ralph-result await`. The protected
V1.6 publish artifacts remain untracked and untouched.

## Completed closing gate

The loop's third locked full gate at `2ab07496` exited **0** after **1,240
seconds**. Read the complete output in
`ralph/next/onboarding/ctl/log/sessions/onboarding-oidc-setup-help-3.await.log`.
Every stage passed, including the previously failing ceiling canary. No
product, check or threshold changed between the second and third runs.
Actual output below omits only detailed ceiling accounting, individual scene
and conformance rows, and the historical seek-rate table; those remain in
the complete log.

```text
$ lockf -k /tmp/svrngs-gate.lock scripts/check.sh
note       open files: 256, raised to 10240
note       kept awake while this runs (caffeinate, pid 64165)
passed     principles pinned (0 s)
passed     lean and RUST_ARCH (1 s)
passed     trust floor (0 s)
passed     advisories (1 s)
passed     app nouns gate (0 s)
passed     app nouns (0 s)
passed     lint (1 s)
passed     format (0 s)
passed     page laws (16 s)
passed     rustfmt (1 s)
passed     clippy (1 s)
passed     tests (514 s)
passed     canaries (2 s)
passed     ceiling canary (5 s)
passed     web rustfmt and clippy (10 s)
passed     browser (168 s)
passed     types (2 s)
passed     typed lint (2 s)
passed     joining in a browser (51 s)
passed     pages (63 s)
passed     scenes (262 s)
passed     conformance canary (99 s)
passed     app laws (41 s)
note       took 1240 s in all
exit 0
```

The ceiling median was **0.0307 s** against **0.1 s** (samples
`[0.19298200000000065, 0.030742000000000047, 0.022975000000000634]`).
Together with the named red plants and focused green runs above, this
completes the row's automated proof. The operator's real Jellyfin
configuration walk remains separate, as the row requires. The inherited
V1.6 publish artifacts were inspected again and remain untouched and
unstaged.
