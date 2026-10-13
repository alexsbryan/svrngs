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
