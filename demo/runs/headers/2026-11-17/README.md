# Headers the door adds, lent from the home page — one host, 2026-11-17

PLAN §1, Spotting 1. `PREDICTIONS.md` was committed before the first server
started (`4b3de073`). One host, one run: not a rate (AGENTS.md 7).

Host: Fedora, Linux 7.3.3, x86_64; `/usr/bin/llama-server` (it prints
`version: 0 (unknown)`); the model `qwen2.5-0.5b-instruct-q4_k_m.gguf`;
svrngs built at `4b3de073` and copied aside before the run, so a build of
`target/` beside it changed nothing under it (`data/responses/versions.json`).
Ana and Bo are each a program here with a HOME and an `SVRNGS_DIR` of their
own (`data/home/`, not committed), in one new group, "Header Run", met at
n0's relays as `svrngs new` and `join` meet.

- **Ana is a person at her home page.** `ana.mjs` opens it in Chromium at
  the address `svrngs open --print` gives, presses **This computer**, and
  types and presses as she would. What her row says and what `/lent`
  answers her page are in each `N-ana-*.json`, with a screenshot beside it.
- **Bo is a program,** as `docs/guide/models.md` says: the lend's address
  and his own svrngs token, one chat each time (`N-bo-*.json`).
- **llama-server** ran with `--api-key night-k7Qz`. `PREDICTIONS.md` says
  `abc123`, but the page's own example text says `abc123`, so a search for
  the key would have found the example. The first take used it and was
  thrown away for that reason alone; this take's answers matched it.

    python3 drive.py        # SVRNGS_BIN=<a copy of target/debug/svrngs>

## The predictions

1. **Lent with no header, Bo's chat is 401: passed.** llama-server answered
   `{"error":{"message":"Invalid API Key",…}}` (`2-bo-before.json`).
2. **`/lent` says `headers: {coder: []}`: passed** (`1-ana-lends.json`).
   Lending took the page's two presses. The first press drew the program's
   terms: *"Apps your friends open on their computers can send to
   llama-server at /v1/embeddings, if each says yes. To lend it with these,
   press Lend again."*
3. **Ana adds `Authorization: Bearer …`, and `/lent` names it, never its
   value: passed.** Her row says *"svrngs adds authorization to each request
   your friends make to coder. They never receive what it says."*, and
   `/lent` says `["authorization"]` (`4-ana-adds.json`, `.png`).
4. **Bo's next chat, nothing restarted, is 200: passed.** The model answered
   `"lent"` (`5-bo-after.json`).
5. **Nothing Bo receives holds the key: passed.** `grep -r night-k7Qz data/`
   finds it in no response, log or trace, Ana's and Bo's included. Once Ana
   stopped adding it, `data/home/` held no copy either.
6. **A header svrngs keeps for itself is refused, and nothing is kept:
   passed, for the name only.** Ana typed `x-svrngs-key` in **Header:**.
   Her row said *"svrngs can’t add x-svrngs-key with that: a header of its
   own, the browser’s, or a value with a line break."* and `/lent` still
   said `[]` (`3-ana-refused.json`). A line break can't be typed into the
   page's one-line box, so that refusal is the Rust test's alone
   (`crates/svrngs-node/tests/home.rs`,
   `a_header_the_door_adds_is_given_on_the_home_page`).
7. **Ana stops adding it, and Bo's next chat is 401 again: passed**
   (`6-ana-stops.json`, `7-bo-stopped.json`).

## Seen beside it

The home page's row for something it doesn't know says *"if it is kokoro or
llama-server or llama-server, say that"* (`1-ana-lends.json`, `found`).
llama-server is named twice because its file has two rows, `keeps:` and
`sends:`, and `home.rs` `running` lists each row's word. This predates the
change and is not fixed here.
