# arch-peer (a): the bridge's exposure, shown (2026-10-15, this host)

The row's first part (`ralph/next/arch/STATE.md`, arch-peer; AUDIT §2.1):
what reaches a friend's door through a loopback port on this computer, read
at HEAD 965b52b with the bridge as it is. One host, one run of each: not a
rate.

## How

`crates/svrngs-node/tests/peer.rs`, a test binary of its own, its four
tests one at a time. Alex founds a group and lets Sam in, two nodes in one
process on 127.0.0.1, no relay. Sam lends `pad`, which asks nobody, and
`book`, which asks before an app uses it. Alex's computer dials Sam's as it
does on its own: a round, and Sam's pad opened at its origin. Then every
TCP port the process listens on, as `lsof` lists them, is asked:

- by a program holding no key, speaking no svrngs and carrying no pass:
  `GET /lend/pad/probe-<port>`, `GET /choices/book`, and `POST
  /choices/book` saying yes for an app nobody named;
- by a page on another origin (`web/tests/bridge.mjs`), in Chromium 147.0.7727.15
  and WebKit 26.4 (Playwright 1.59.1): the GET, read if the browser lets it
  be, and the POST sent as a form posts, `no-cors` with `text/plain`, so no
  preflight asks first.

What reached Sam's door is read at the far end: the pad's service told the
probe (and as whom), and a yes kept in Sam's choices for the book.

```
cargo test -p svrngs-node --test peer -- --ignored     # a-red.out
```

## What it found

All four red, each through port 30029 alone, the port Alex's bridge to Sam
prefers (`iroh.rs` `preferred_bridge_port`). No other port reached Sam's
door; Alex's lend origin answers only a lend's host.

| who asks | Sam's pad, as Alex | Alex's choices read | a yes kept as Alex | the answer read by the asker |
|---|---|---|---|---|
| a program, no key | yes | yes, `{"choices":{}}` | yes | yes |
| a page on loopback, Chromium | yes | not asked | yes | no (CORS) |
| a page on loopback, WebKit | yes | not asked | yes | no (CORS) |
| a page on the internet, Chromium, the person's leave given | yes | not asked | yes | no (CORS) |
| a page on the internet, Chromium, no leave | no: refused, "Permission was denied for this request to access the `loopback` address space" (`chromium-public-unasked.out`, a stand-in listener, `listen.mjs`) | | | |
| a page on the internet, WebKit | could-not-judge: WebKit has no flag that makes a page on loopback count as one on the internet, and no host off this computer is used (fence 8) | | | |

Chromium is told a page is on the internet by
`--ip-address-space-overrides=127.0.0.1:<its port>=public`; the person's
leave is Playwright's `grantPermissions(['local-network-access'])`, which a
person gives Chromium by its prompt.

## What it means

The yes is the worst of it. A lend that asks lets an app in only on its
person's yes (DESIGN §3, call 6; `door/choices.rs` `judge`). Through the
bridge any program on Alex's computer, and any page Alex's browser lets
through, keeps a yes on Sam's computer as Alex for an app Alex never named;
and every lend that asks nobody is opened as Alex with no app named, past
the per-app scoping a lend's origin keeps (`origin.rs:20-28`). Finding #175
in `ralph/REVIEW_FINDINGS.md`. arch-peer (d) takes the bridge out, and the
four tests go green there.
