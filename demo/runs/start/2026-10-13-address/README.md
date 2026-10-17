# The address after a restart, again — 2026-10-13

Row start-address (`ralph/next/start/STATE.md`), its check: "start-measure
(a)'s restart step run again, every client answered at the address it was
set to". start-measure watched Bo's home page and a friend's llama-server,
lent, move from port 64525 to 49821 across one restart of Bo's svrngs, every
program set to the old address refused the connection, and Node's `fetch`
refused 403 by svrngs before and after
(`demo/runs/start/2026-10-13/README.md` §3 (a) 4-5). start-address kept the
address in the data dir (82c7dd1), gave it in one sentence (a39c8d4), and
took a request with no `Sec-Fetch-Site` for a program's (4f7d9a5).

This file's first commit holds the question, the setup and the
predictions, before any data. One host, one run: not a rate (AGENTS.md 7).

## 1. Setup

As start-measure's (a), on the svrngs debug build at this file's commit:

- **llama-server** `-hf Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M --host
  127.0.0.1 --port 18089 -c 4096 --parallel 2`, Homebrew's, version 10180.
  Its answers are not judged.
- **Sam** `svrngs new Lab --as Sam`, and lends it: `svrngs share 18089
  --name llama --kind llama --limit 4 --refuse /lora-adapters`.
- **Bo** put on as `web/install.sh` puts a computer on: `svrngs join <link>
  --as Bo --no-run`, then the bare `svrngs` over the same data dir.
- **The clients**, start-measure's own, by path
  (`../2026-10-13/client.py`, `../2026-10-13/fetch.mjs`, and the driver's
  curl): curl, Python's `openai` package in a venv of its own, Node 20's
  `fetch`, each asking `POST /v1/chat/completions` whole and streamed at
  Bo's address for llama.
- **The home page**, asked as a typed address asks it (`Sec-Fetch-Site:
  none`, `Sec-Fetch-Mode: navigate`) at Bo's address for it.
- **The restart**: Bo's svrngs sent `SIGTERM`, then the bare `svrngs`
  again over the same data dir. Unlike start-measure's driver, this one
  never deletes `<data dir>/api` before a start.
- **A port held**: Bo's svrngs stopped again, a Python socket listening on
  127.0.0.1 at Bo's port, and the bare `svrngs` started over Bo's data dir.

`PY=<venv with openai>/bin/python W=<scratch> python3
demo/runs/start/2026-10-13-address/run.py`; data in `data/`.

## 2. Predictions

1. **Before**: Bo's home page and llama's address share one port `P`;
   curl, Python and Node are each answered 200 whole and streamed (Node
   now, where start-measure saw 403 `closed`).
2. **After the restart**: Bo's `<data dir>/api`, home page and llama's
   address are byte for byte as before, `P` again; the home page answers
   200 at the address it was set to; curl, Python and Node are each
   answered whole and streamed at the address they were set to before the
   restart.
3. **Bo's terminal** says, as each start begins, "Your home page, the same
   address each time svrngs starts: http://svrngs.localhost:P/", the same
   both times.
4. **A port held**: the bare `svrngs` ends with exit 3, saying "Another
   program on this computer holds 127.0.0.1:P, where svrngs keeps your home
   page, http://svrngs.localhost:P/, …", and `<data dir>/api` still names
   `P`.

## 3. What the run gave

`PY=/tmp/sm-py/bin/python W=/tmp/sa-w python3
demo/runs/start/2026-10-13-address/run.py` at `f87215d`, exit 0, in 7.8 s
(`data/events.jsonl`, the first and last `t`). The host's 1-minute load was
9.7 at the start and 9.2 at the end, on 12 cores. The first run, at
`c10b914`, ended in the driver at the held-port step (Python's bind without
`SO_REUSEADDR`, `Errno 48`; `f87215d` says it), its data not kept; its
restart step had passed as this one's did. Every number below is in
`data/summary.json` or `data/events.jsonl` unless another file is named.
By AGENTS.md 6's verdicts:

1. **Passed.** Bo's home page was `http://svrngs.localhost:56398/` and
   llama `http://wuzw…a1jd432f8.localhost:56398/`, one port (`before`). The
   home page answered 200 with the page (`home_before`). curl 200 whole and
   200 streamed, 23 `data:` lines; Python's `openai` 3.29.0 a
   `chat.completion` whole in 112.1 ms and 22 chunks streamed; Node's
   `fetch` 200 whole in 99 ms and 200 streamed, 23 events, where
   start-measure saw 403 `closed` (`clients_before`).
2. **Passed.** After `SIGTERM` and the bare command again, Bo's
   `<data dir>/api` read `http://127.0.0.1:56398` as before, and the home
   page and llama's address were the same strings (`same`: `kept`, `home`,
   `llama` each `true`). At the address set before the restart the home
   page answered 200 with the page (`home_after`); curl 200 and 200 (23
   events), Python whole in 104.8 ms and 22 chunks, Node 200 in 102 ms and
   200 (23 events) (`clients_after`). Each name resolved as start-measure
   saw it: Python `127.0.0.1`, `::1`; Node `::1`, `127.0.0.1`.
3. **Passed, by Bo's terminal; the driver's read of it raced.** `data/bo.out`
   holds both starts' words, each "Your home page, the same address each
   time svrngs starts: http://svrngs.localhost:56398/" and "Sam's llama, in
   your browser: http://wuzw…a1jd432f8.localhost:56398/". The driver's
   `after.said` is empty, so `same.said` reads `false`: it read `bo.out`
   once llama was listed at the API, before the command had printed —
   the bare command prints after asking every computer what it lends.
4. **Passed.** With a Python socket on 127.0.0.1:56398, the bare `svrngs`
   over Bo's data dir ended with exit 3, saying "svrngs: Another program on
   this computer holds 127.0.0.1:56398, where svrngs keeps your home page,
   http://svrngs.localhost:56398/, and what your groups lend: stop that
   program, then start svrngs again. Or delete /tmp/sa-w/bo/.svrngs/api,
   and svrngs takes a new address as it starts, which bookmarks and
   programs set to this one do not reach"; `<data dir>/api` still named
   56398 (`held`).

## 4. Findings, a sentence each

- **The address stays**: across one restart Bo's home page and a friend's
  model kept port 56398, and every program set to it before — curl,
  Python's `openai`, Node's `fetch` — was answered after, where
  start-measure watched all three refused at the old port.
- **Node's programs are answered**, whole and streamed, before and after
  (4f7d9a5).
- **A port another program holds is said**, with what to do, and svrngs
  stops rather than moving; on a computer whose svrngs runs at login,
  those words reach its log (`web/install.sh`'s `$dir/svrngs.log`), not a
  person — start-home's and start-run's to watch.
