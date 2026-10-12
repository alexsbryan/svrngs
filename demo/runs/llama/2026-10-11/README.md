# llama-server, lent — 2026-10-11

The operator, 2026-10-11, on shared inference at its simplest ("So at it's
simplest level, shared inference could just be someone hosting a
llama-server?"), then "Let's also do the llama server PoC". The question:
**does a stock llama-server, lent with one `svrngs share`, serve another
member's computer — its own chat page in a browser, and its OpenAI-compatible
API to a program, streamed — with nothing of svrngs or llama.cpp changed?**
svrn-docs `rings/current/SVRN_IN_SVRNGS.md` §4 names it as usable before svrn
changes at all, by prediction; this run is what makes it a record.

This file's first commit holds the question, the setup and the predictions,
before any data. One host, one run of each: not a rate (AGENTS.md 7).

## 1. Setup

- **The model**: Qwen2.5-0.5B-Instruct, Q4_K_M, fetched by llama-server's own
  `-hf Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M` (Homebrew's
  `/opt/homebrew/bin/llama-server`). Its quality is not judged; it is small so
  the run stays light beside the loop's checks on this host.
- **Sam's computer** runs `llama-server --host 127.0.0.1 --port 18089 -c 4096
  --parallel 2` and lends it:
  `svrngs share 18089 --name llama --kind llama --limit 4 --refuse /slots
  --refuse /lora-adapters` — the two routes of llama-server's that change or
  read the server's working state rather than answer a person.
- **Bo's computer** is a member of the same group, made by `svrngs new Lab`
  on Sam's and `svrngs join <link>` on Bo's, each node a process with its
  own `SVRNGS_DIR` (as `demo/runs/u7/2026-10-11/run.py` runs them), svrngs's
  debug build at this file's commit.
- **The driver** (`run.py`) asks from Bo's computer as a program does, at the
  lend's address there (`http://<label>.localhost:<port>/`, from Bo's
  `/v1/lends`), and the same requests straight at llama-server for the
  comparison; `browser.mjs` opens the lend's address in Playwright's
  Chromium on Bo's computer.

## 2. Predictions

1. **The page**: Bo's Chromium opens llama-server's own chat page through the
   lend, 200 with its HTML, and its script's first requests to the server
   (its properties and models) answered 200 through the same lend.
2. **The API, whole**: `POST /v1/chat/completions` (32 tokens at most)
   through the lend answers 200 with an OpenAI-shaped body; its median time
   within 30 ms of the same request straight at llama-server (five each).
3. **The API, streamed** (`"stream": true`): the server's events arrive
   through the lend as they are made, not held to the end — the first
   within 50 ms of the first straight at the server, the same number of
   events, and the span from first to last at least 80% of the span
   straight.
4. **What the lend keeps back**: `GET /slots` through the lend refused by
   svrngs; straight at the server, answered.
5. **Nothing typed, no account, no key**: llama-server has none, and the
   lend asks for none.
6. **The person on each request**: not read here — llama-server shows no
   request headers, so whether `X-Svrngs-Person` reached it is never-ran in
   this run (svrngs's own tests hold the door's header).
