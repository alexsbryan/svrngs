# Spot, lent as its README says — one host, 2026-11-17

PLAN §1, Spotting 2. `PREDICTIONS.md` was committed before the first server
started (`1aef4f5b`). One host, one run: not a rate (AGENTS.md 7).

Host: Fedora, Linux 7.3.3, x86_64; Node and svrngs's commit in
`data/responses/versions.json` (svrngs built at `4b3de073`, whose crates
`91d18af7` leaves as they were, copied aside before the run). Ana and Bo are each a
program here with a HOME and an `SVRNGS_DIR` of their own (`data/home/`,
not committed), in one new group, "Spot Run", met at n0's relays.

- **The provider is a stand-in** (`provider.mjs`): OpenAI's chat API on
  127.0.0.1, answering only `Bearer spot-run-k9Wq`, streaming three pieces
  of Markdown and reporting 64 tokens. A provider's real key isn't this
  run's to spend; a run against one is its lender's.
- **Ana** runs `apps/spot/server.mjs` with `--budget 60` and `--svrngs`,
  then `svrngs share … --program spot`, the key given on `share`'s stdin
  where it asks for a password (`1-share.txt`).
- **Bo** is a person at his home page in Chromium, at a Pixel 7's size
  (`bo.mjs`), opened as `svrngs open --print` says: his group, Ana's lend,
  then two messages on Spot's page. This is a computer's way to a lend, at
  a phone's size; a phone's own way, through the group's page and its lend
  worker, did not run here.

    python3 drive.py        # SVRNGS_BIN=<a copy of target/debug/svrngs>

## The predictions

1. **Bo's home page lists Ana's lend, and it opens as Spot's page with her
   name: passed.** Listed *"ana’s deepseek-flash (spot)"*
   (`b1-home.png`); the page's title *"ana’s deepseek-flash"* (`2-bo.json`).
2. **His first message is answered as Markdown, streamed: passed.** *"Hello,
   Bo. This came from the provider through Ana’s account."*, `Bo` in
   bold (`b2-chat.png`); the page asked `stream: true`, and Spot asked the
   provider for its usage (`include_usage: true`, the stand-in's log).
3. **The provider got the provider's id, the key, and nothing of Bo:
   passed.** `/v1/openai/chat/completions`, `deepseek-ai/DeepSeek-V4.1-
   Flash-1017`, `authorization: Bearer <the key>`, and no `x-svrngs-*` and
   no cookie (`3-provider-saw.json`). Its other headers were `accept` and
   `content-type`, which Spot sends on, and `accept-encoding`,
   `accept-language`, `connection`, `content-length`, `host`,
   `sec-fetch-mode` and `user-agent`, which Node's own `fetch` adds to
   every request it makes: none was Bo's.
4. **Ana's terminal names Bo and his tokens: passed.** *"spot: bo,
   deepseek-flash, 64 tokens (64 of 60)"* (`4-ana-terminal.txt`).
5. **The key is in nothing Bo's browser holds: passed.** Neither the page's
   text nor its localStorage held it (`2-bo.json`, `keyInBrowser: false`).
6. **Once the 60 tokens were spent, the next message was refused and the
   provider not asked: passed.** The page said *"its 60 tokens are spent"*,
   in the line under the title and as the answer (`b3-chat.png`); Ana's
   terminal *"spot: bo, refused: its 60 tokens are spent"*; the stand-in
   was asked once in all.

## Seen beside it

The budget let one chat past it: 64 of 60. A chat's tokens are known only
when it ends, and the budget is checked as each begins
(`apps/spot/README.md`, "What it does not do", now says so).
