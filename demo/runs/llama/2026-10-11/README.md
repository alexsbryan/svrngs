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

## 3. What the run gave

`W=<scratch> python3 demo/runs/llama/2026-10-11/run.py` at `5e4c1c6`, exit
0, its data in `run/` (`events.jsonl`, `summary.json`, `sam.log`, `bo.log`,
`llama.log`); the host's 1-minute load 14.4 at its start and 11.3 at its
end, on 12 cores (`events.jsonl`, `start` and `end`), the queue v1's
`scripts/check.sh` having just ended. Each prediction, by AGENTS.md 6's
verdicts:

1. **The page: passed.** Bo's Chromium opened the lend's address 200 in
   976 ms, titled `llama-ui`, its words "Hello there … Type a message or
   upload files to get started"; its script's `/props` (three times) and
   `/v1/models` answered 200 through the lend, its bundle and styles too,
   and no page error (`summary.json`, `page`). One request answered 403:
   `/tools`, by llama-server itself — the door's log carries it and says
   "the lend's service answered … status=403" (`run/sam.log:35-37`);
   llama-server's tools are off unless started with them.
2. **The API, whole: passed.** Five each way, every one 200: straight
   72.9 ms at the median, through the lend 75.8 ms — 2.9 ms more, inside
   30 ms (`summary.json`, `whole`). The answer through the lend is an
   OpenAI-shaped `chat.completion`. Its words name the Garonne, which is in
   France: a 0.5B model's quality, not judged here.
3. **The API, streamed: passed.** 23 events each, every run, both ways; the
   first event at 6.4 ms straight and 9.5 ms through the lend at the
   median, 3.1 ms more, inside 50 ms; the span from first to last 65.4 ms
   straight and 66.2 ms through the lend, 101% of it, so the lend carried
   each event as it was made, held none to the end (`summary.json`,
   `streamed`).
4. **What the lend keeps back: passed.** `GET /slots` straight 200; through
   the lend 403, svrngs's own refusal, `{"error": "llama is lent without
   /slots", "kind": "path"}`, in 2.5 ms (`summary.json`, `slots`).
5. **Nothing typed, no account, no key: passed.** Nothing asked for any:
   llama-server keeps none, and the lend asked none (the page's words above;
   no `401` or sign-in in `asked`).
6. **The person on each request: never-ran at llama-server**, as
   predicted — it shows no request headers. svrngs's side says it: the
   door's line for each request it carried names `person="Bo"`
   (`run/sam.log:35`).

**So at its simplest, shared inference is a lend.** A stock llama-server,
one `svrngs share` with two `--refuse`s, and another member's computer has
its chat page and its API, streamed, 3 ms from straight on one host.
Unmeasured here: two computers on two networks and a relay (Demo C's two
homes, DESIGN §8); a phone, through the group's page; several members at
once against `--limit 4` and `--parallel 2`.
