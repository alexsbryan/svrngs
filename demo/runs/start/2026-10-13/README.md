# What a newcomer meets — 2026-10-13

Row start-measure (`ralph/next/start/STATE.md`). The operator, 2026-10-13:
"nail down the svrn / llama-server on svrngs story and make it part of an
early demo document for anyone getting started", and "making sure our
localhost dashboard really makes the system easy to navigate for someone
who is a bit afraid technically" (`ralph/next/start/CHARTER.md`). Three
questions, measured before anything changes:

- **(a)** Does a friend's llama-server, lent, answer the programs a
  newcomer points at it — curl, Python's `openai` package, Node's `fetch`
  — at the address their computer gives it; and does that address, and
  the home page's, survive svrngs starting again?
- **(b)** What of llama-server should a lend keep back: which of its routes
  show or change another person's work or the server's state, and what
  does one member read of another's chat through a lend that keeps back
  only `/lora-adapters`?
- **(c)** Walked as someone a bit afraid technically, what can the home
  page do, what only the terminal, and which of its words would its reader
  have to look up?

This file's first commit holds the questions, the setup and the
predictions, before any data. One host, one run of each: not a rate
(AGENTS.md 7). Nobody but the driver reads the page: every word judged
"to look up" below is this run's reading, not a person's (could-not-judge
as a people bar).

## 1. Setup

- **llama-server** as `demo/runs/llama/2026-10-11/` started it: Homebrew's
  `/opt/homebrew/bin/llama-server`, version 10180 (`11b068d06`), `-hf
  Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M --host 127.0.0.1 --port 18089 -c
  4096 --parallel 2`. Its answers are not judged.
- **Four computers on this host**, each a svrngs process with its own
  `SVRNGS_DIR` and `HOME`, svrngs's debug build at this file's commit:
  - **Sam** `svrngs new Lab --as Sam`, and lends llama-server:
    `svrngs share 18089 --name llama --kind llama --limit 4 --refuse
    /lora-adapters` — only `/lora-adapters` kept back, so (b) reads what
    the rest of llama-server shows a member; and a small page as an app,
    `share <port> --name night --kind night` (Python's `http.server`).
  - **Bo**, the newcomer, put on as `web/install.sh` puts a computer on:
    `svrngs join <link> --as Bo --no-run`, then the bare `svrngs` over the
    same data dir, as the LaunchAgent runs it (`install.sh`, the plist's
    `ProgramArguments`). Bo lends nothing.
  - **Cy** `svrngs join <link> --as Cy`, and lends a page that asks:
    `share <port> --name taste --kind taste --ask "the films you have
    marked"`.
  - **Dee** the bare `svrngs` over an empty data dir: a computer in no
    group yet.
- **The clients** run as Bo's programs, at the lend's address Bo's
  `/v1/lends` gives (`origin`): curl 8.7.1 (macOS's); Python 3.13's
  `openai` package in a venv of the run's own, `base_url` the address with
  `v1`; Node 20's `fetch`. Each asks `POST /v1/chat/completions` whole and
  streamed (`"stream": true`), and says whether its own name lookup gave
  the address's host (`getaddrinfo`, as each uses it).
- **The browsers** are Playwright's Chromium and WebKit (`web/node_modules`),
  each opening Bo's (or Sam's, or Dee's) home page in a state, screenshotting
  it, and listing its headings, its words, its buttons and its links.
- **The restart**: Bo's svrngs sent `SIGTERM`, as `launchctl bootout`
  stops it, and the bare `svrngs` started again over the same data dir, as
  `KeepAlive` starts it again.
- **What the run reads of llama-server's routes** is its source at the
  installed revision, `11b068d06` (llama.cpp, fetched to scratch, never
  changed): `tools/server/server.cpp`'s routes and the handlers in
  `server-context.cpp`, `server-stream.cpp`, `server-tools.cpp`; then each
  is asked straight and through the lend.
- **The budget** at this file's base: `scripts/lean_check.py` says the
  repository 15,828 / 15,950 and svrngs-core 3,121 / 3,130 — the cap
  already raised to 15,950 (`c27521f`, start-findings), so 122 lines are
  left for the queue's changes.

## 2. Predictions

### (a) A friend's llama-server, from Bo's computer

1. **Bo's address for it**: `/v1/lends` gives llama as
   `http://<label>.localhost:<P>/`, `P` the port Bo's record API took, and
   the home page is `http://svrngs.localhost:<P>/` — the same `P`.
2. **curl** resolves `<label>.localhost` to 127.0.0.1 itself, without
   asking the system, and is answered: 200 whole, and streamed with
   several events.
3. **Python** (`socket.getaddrinfo`, which `httpx` under `openai` uses)
   does not resolve `<label>.localhost` on macOS: `openai` raises
   `APIConnectionError`, whole and streamed, and never reaches the lend.
4. **Node's `fetch`** does not resolve it either: `TypeError: fetch failed`,
   its cause `getaddrinfo ENOTFOUND`.
5. **After the restart** Bo's record API takes another port
   (`crates/svrngs/src/cli.rs:220`, `API`, port 0): the home page and
   llama's address both move to the new port, llama's label unchanged; at
   the old addresses every client that was answered before is refused the
   connection; at the new ones, answered as before.

### (b) What a lend of llama-server should keep back

6. **The routes**, at `11b068d06` (`tools/server/server.cpp:226-356`):
   of those on by default, `GET /slots` shows each slot's state — whether
   it is working, its task's id, its prompt's and its answer's token
   counts and its sampling settings — and not its words, which only
   `LLAMA_SERVER_SLOTS_DEBUG` adds (`server-context.cpp:631-661, 2471`);
   `GET /props` shows the server's settings and the model's path on the
   lender's disk; `GET` and `POST /lora-adapters` show and change the
   adapters' scale for everyone; `GET /v1/stream?conv_id=`,
   `POST /v1/streams/lookup` and `DELETE /v1/stream?conv_id=` read, find
   and stop a stream by the conversation's id
   (`server-stream.cpp:456-570`); `POST /v1/chat/completions/control`
   ends another completion's reasoning by its id. Off unless started so,
   and refused by llama-server itself as this run starts it: `/metrics`
   (`--metrics`), `POST /props` (`--props`), `POST /slots/<id>`
   (`--slot-save-path`), `/tools` (`--tools`, which can run a shell on the
   lender's computer, `server-tools.cpp:583`), `/cors-proxy`
   (`--webui-mcp-proxy`), and the `/models` load and unload (router mode).
7. **Through the lend**, `/lora-adapters` is refused by svrngs (`403`,
   `x-svrngs-refused: path`), `GET` and `POST`; every other route is
   carried to llama-server.
8. **While Cy's chat runs** (streamed, through Cy's own address for the
   lend, with an `X-Conversation-Id`), Bo's `GET /slots` through Bo's
   address answers 200 and shows one slot working, its token counts
   rising and Cy's sampling settings, but none of Cy's words.
9. **Given its id**, Bo's `GET /v1/stream?conv_id=<Cy's>` through the lend
   reads Cy's stream — Cy's model's words — and Bo's `DELETE` of it stops
   Cy's answer before its end. Without the id Bo finds none: lookup
   answers only the ids asked (`server-stream.h`, its comment).
10. **`GET /props`** through the lend shows `model_path`, a path in Sam's
    account's home.

### (c) The home page, walked

11. **Done on the page**: open what the group lends (a link each); see who
    is in; see where the group's folder is (its path as words, nothing to
    open it); let in more or fewer at once, or stop lending, of what this
    computer lends; say yes or stop to an app. **Only in a terminal**:
    make a group (`svrngs new <name>`), invite someone (`svrngs link
    <group>`), add a phone or another computer (`svrngs link <group>
    --device`), share something, llama-server among them (`svrngs share
    <port> …`), remove someone (`svrngs remove <name> --group <group>`).
    **Neither**: leave a group — no command does it (`cli.rs` `USAGE`).
12. **The page names no terminal line** for any act it lacks.
13. **No group yet** (Dee): the page says only that this computer lends
    nothing, and "What apps use" with nothing under it; it does not say
    how to make or join a group.
14. **One group with lends** (Bo; Sam): the group, its people, the folder's
    path, and a link per lend ("Sam’s llama", "Sam’s night", "Cy’s
    taste"); on Sam's, llama with its limit, a box, "Let in this many",
    "Let everyone in" and "Stop lending".
15. **An app asking** (Bo at `/ask?lend=<taste>&app=<night>`): "Let Sam’s
    night use Cy’s taste?" around Cy's words, with "Yes" and "Not now".
16. **A computer asleep** (Cy's stopped): "Cy's computer did not answer, so
    what they lend is not listed."
17. **Removed** (Bo, by Sam's `svrngs remove Bo --group Lab`): "You are no
    longer in Lab. What you already have stays on this computer; nothing
    new reaches it."
18. **None of the fence's words** (key, member, keeper, ring, act,
    journal, port, namespace, origin) is in the page's own text; the
    address bar shows a port, and each lend's link a 62-letter label.
19. **After the restart** the home page is not found at its old address:
    each browser shows its own error for a refused connection.
20. **Each state opens in Chromium and in WebKit** alike.

The ranked list of what to change, with its estimates, is written from
the data, after it.
