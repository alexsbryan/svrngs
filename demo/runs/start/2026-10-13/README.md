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

## 3. What the run gave

`PY=/tmp/sm-py/bin/python W=/tmp/sm-w python3
demo/runs/start/2026-10-13/run.py` at `c193e5b`, exit 0, in 90.9 s
(`data/events.jsonl`, the first and last `t`; the data commit `e45fbda`
says 81 s and 19 screenshots, wrongly: `data/shots/` holds 22). The host's
1-minute load was 17.0 at the start and 14.9 at the end, on 12 cores
(`events.jsonl`, `start` and `end`). Every number below is in
`data/summary.json` or `data/events.jsonl` unless another file is named.
Each prediction, by AGENTS.md 6's verdicts:

### (a) A friend's llama-server, from Bo's computer

1. **Passed.** Bo's `/v1/lends` gave llama at
   `http://97kf…a1jd432f8.localhost:64525/` and the home page was
   `http://svrngs.localhost:64525/`, the same port (`before`).
2. **Passed, its reason not told apart.** curl reached 127.0.0.1: 200
   whole, 200 streamed with 23 `data:` lines (`clients_before.curl`).
   Whether curl resolved the name itself or asked the system is not told
   apart: the system resolves it too (3).
3. **Failed.** Python resolved `<label>.localhost` to `127.0.0.1` and `::1`,
   and `openai` 3.29.0 was answered: a `chat.completion` whole in
   153.6 ms, 22 chunks streamed (`clients_before.python`). On this macOS
   the system's lookup gives `*.localhost`; Linux is unmeasured.
4. **Failed, and worse than predicted.** Node resolved it too (`::1`,
   `127.0.0.1`) and reached svrngs, which **refused it**: 403, whole and
   streamed, `x-svrngs-refused: closed`, "a page reaches another lend only
   by fetch, which says where it is from" (`clients_before.node`). Node's
   `fetch` sends `Sec-Fetch-Mode: cors` with no `Sec-Fetch-Site` and no
   `Origin`; Bo's log reads it as `site=None mode=Some("cors")
   origin=None app=None`, then "refused — from another page that names no
   origin" (`data/bo.log:24-27`). The rule is `app_of`
   (`crates/svrngs-node/src/origin.rs:74-93`): only a request with
   neither `Sec-Fetch-Site` nor `Sec-Fetch-Mode` is taken as a program's.
5. **Passed.** After the restart Bo's record API took port 49821 in place
   of 64525: the home page moved to `http://svrngs.localhost:49821/` and
   llama to `…a1jd432f8.localhost:49821/`, its label unchanged (`before`,
   `after`). At the old address every client was refused the connection:
   curl `(7) … Couldn't connect to server`, Python `APIConnectionError`
   (`[Errno 61] Connection refused`), Node `fetch failed`
   (`ECONNREFUSED`) (`clients_old_address`). At the new one curl and
   Python were answered as before, and Node refused as before
   (`clients_new_address`). The terminal said each address as it started
   (`data/bo.out`): before, no lend, since none was lent yet when the bare
   command started; after, both of Sam's with their new addresses and
   "Cy's computer did not answer".

### (b) What a lend of llama-server should keep back

6. **Passed, as read and as asked.** At `11b068d06`, of the routes that
   answer as this run started llama-server, these show or change another
   person's work or the server's state:

   | route | what it shows or does | as started here | why |
   |---|---|---|---|
   | `GET /slots` | each slot: working or not, its task's id, its prompt's and answer's token counts, its sampling settings; never its words unless `LLAMA_SERVER_SLOTS_DEBUG` is set | on (`--slots` is the default, `common/common.h:665`); 200 straight and lent | another person's chat: when, how long, how |
   | `POST /slots/<id>?action=` | saves, restores or erases a slot's cache as a file on the lender's disk | 501, `--slot-save-path` unset | the server's state, the lender's disk |
   | `GET /v1/stream?conv_id=`, `POST /v1/streams/lookup`, `DELETE /v1/stream?conv_id=` | read a live or finished stream from its start, say whether one is live, stop it — each by its conversation id (`server-stream.cpp:456-570`) | on; lookup 200 | another person's chat, given its id |
   | `POST /v1/chat/completions/control` | ends another completion's reasoning, by its id (`server-context.cpp:4689-4720`) | on; 200 `no active completion for this id` | another person's work, given its id (`chatcmpl-` and a random string, `server-common.cpp:80`) |
   | `GET /lora-adapters`, `POST /lora-adapters` | lists the adapters with their paths; sets their scale for everyone (`server-context.cpp:4995-5055`) | on; 200 straight, 403 `path` lent | the server's state |
   | `GET /props` | the server's settings, its chat template, and `model_path`, a file on the lender's disk | on; 200 both | the lender's disk; the chat page reads it (the 2026-10-11 run's page asked it three times) |
   | `POST /props` | changes the server's settings | 501, `--props` unset | the server's state |
   | `GET /metrics` | the server's load and counts | 501, `--metrics` unset | the server's state |
   | `/tools` | runs the server's tools, among them `exec_shell_command`, `write_file` and `edit_file` on the lender's computer (`server-tools.cpp:583-709`) | 403 `feature_disabled`, `--tools` unset | the lender's computer |
   | `/cors-proxy` | fetches any address from the lender's computer for the page's MCP servers | 403, `--webui-mcp-proxy` unset | the lender's computer and network |
   | `/models`, `/models/load`, `/models/unload`, `/models/sse` | loads and unloads models | 404, router mode only (`server.cpp:226-230`) | the server's state |

7. **Passed.** Through the lend, `GET` and `POST /lora-adapters` were
   refused by svrngs, 403, `x-svrngs-refused: path`, "llama is lent
   without /lora-adapters"; straight, 200 and `{"success": true}`. Every
   other of the 14 routes asked gave the same status through the lend as
   straight (`routes`).
8. **Passed.** While Cy's answer ran (302 events in 1.79 s through Cy's
   address), Bo read `/slots` five times through Bo's: the first, at
   7 ms, before Cy's task began; then slot 1 working on task 95, its
   tokens decoded rising 26, 61, 93, 122, and Cy's settings — seed 7,
   `max_tokens` 300, temperature 0, `stream` true (`cy_chat.bo_slots`).
   No `prompt` or `generated` key. Its `generation_prompt` is the chat
   template's `<|im_start|>assistant\n`, not Cy's words.
9. **Passed.** Bo's lookup of `cy-conv-1` through the lend said it was live,
   33,945 bytes so far; Bo's `GET /v1/stream?conv_id=cy-conv-1` gave 303
   events whose words were Cy's, all 1,417 characters, identical
   (`cy_chat.bo_stream`, `same_words`). Bo's `DELETE` of `cy-conv-2` was
   answered 204, and Cy's second answer, the same question, ended after 14
   events against the first's 302: "In the ancient land of Zhenjiang,
   there was a river" (`cy_stopped`). Asked for an id nobody used, lookup
   answered `[]` (`routes`). The id is what Cy's program sent as
   `X-Conversation-Id`; whatever program sends one makes its chat as
   readable as its id is guessable.
10. **Passed.** `GET /props` through the lend gave `model_path`
    `/Users/alexsbryan/.cache/huggingface/hub/…/qwen2.5-0.5b-instruct-q4_k_m.gguf`
    (`routes`): the lender's account name, to everyone lent the model.

### (c) The home page, walked

11. **Passed.** Across every state, the page's only buttons and links
    (`walk.<state>.chromium.buttons`, `.links`) are each lend's link,
    Sam's box with "Let in this many", "Let everyone in" and "Stop
    lending", the ask's "Yes" and "Not now", and "Stop" beside an app's
    use. So:

    | a newcomer's act | where | the line, if a terminal's |
    |---|---|---|
    | see what their groups lend, and open it | the page: Bo's link "Sam’s llama" opened llama-server's chat page, `llama-ui`, "Hello there", in Chromium and WebKit (`walk.lends-bo.*.after`); it replaces the home page in the tab | — |
    | invite someone | terminal only | `svrngs link Lab` |
    | add their phone or another computer | terminal only | `svrngs link Lab --device` |
    | share something, llama-server among them | terminal only | `svrngs share 18089 --name llama --kind llama --refuse /lora-adapters …` |
    | find the group's folder | the page, as words: "Lab’s folder is on this computer at /tmp/sm-w/bo/svrngs/Lab"; nothing opens it | — |
    | see who is in | the page: "People: Bo, Cy, Sam" | — |
    | let in more or fewer at once, or stop lending | the page, on the lender's: the box and its three buttons | — |
    | say yes or stop to an app | the page: "Yes" took both browsers to night's page; Bo's page then said "Sam’s night uses Cy’s taste. Stop" (`walk.after-yes-bo`) | — |
    | remove someone | terminal only | `svrngs remove Bo --group Lab` (Sam's, exit 0, `remove`) |
    | leave a group | neither: no command does it (`crates/svrngs/src/cli.rs` `USAGE`) | none |
    | make a group | terminal only | `svrngs new <name>` |

12. **Passed.** No state's words name a line to type
    (`walk.<state>.*.text`); the one `svrngs` the page says is in the
    folder's path. The terminal does: Dee's bare command said "This
    computer is in no group yet: make one with `svrngs new <name>`"
    (`data/dee.out`), while Dee's page did not.
13. **Passed.** Dee's page, in no group: "What this computer lends", "This
    computer lends nothing.", "What apps use", and nothing else
    (`walk.no-group-dee`; `shots/no-group-dee-*.png`).
14. **Passed.** Bo's: "Lab", "People: Bo, Cy, Sam", the folder's path, and
    "Cy’s taste", "Sam’s llama", "Sam’s night" as links; Sam's adds
    "llama: up to 4 people at once, 0 using it now." with the box at 4 and
    three buttons, and "night: everyone in your groups at once" with two
    (`walk.lends-bo`, `walk.lends-sam`; `shots/lends-*.png`). The box's
    name, "How many at once", is for a screen reader only: nothing on the
    page says what the number is.
15. **Passed.** "Let Sam’s night use Cy’s taste?", "the films you have
    marked", "Yes", "Not now" (`walk.asking-bo`; `shots/asking-bo-*.png`).
16. **Passed, and slow.** "Cy's computer did not answer, so what they lend
    is not listed." The page took 3,571 ms to load in Chromium and 3,610
    in WebKit with Cy's computer stopped, and 3,536 and 3,612 after the
    restart, Cy's still stopped, against 516-608 ms in every state where
    each computer answered (`walk.<state>.*.opened.ms`): for those seconds it says
    only "Reading your groups…", the whole page held for the one computer
    that does not answer (`home.js`, `groups`, every group's lends awaited
    before any is shown).
17. **Passed.** "You are no longer in Lab. What you already have stays on
    this computer; nothing new reaches it.", under the heading "Lab"
    (`walk.removed-bo`).
18. **Passed.** None of the fence's words in any state's words
    (`walk.<state>.*.fence`, empty each). The address bar says
    `svrngs.localhost:49821`, and each lend's link a 62-letter label and
    the port.
19. **Passed.** The old home page, after the restart: Chromium
    `net::ERR_CONNECTION_REFUSED`, WebKit "Could not connect to the
    server." (`walk.restart-old-home-bo.*.opened`). Chromium's screenshot
    is blank: headless Chromium draws no error page.
20. **Passed.** Every state opened 200 in both browsers but the old
    address, refused in both, and each state's words were the same in
    both but for a trailing blank line.

**Words its reader would have to look up**, by this run's reading (not a
person's: could-not-judge as a people bar): "lends", "lending" and "What
this computer lends", DESIGN §1's verb but not a word §1 gives a person;
"What apps use", over nothing in every state but one, and in that one
"Sam’s night uses Cy’s taste", two lends of which one is an app;
"llama", the lender's word for its kind; the folder's path; "0 using it
now"; "everyone in your groups"; a box with no words beside it.

## 4. Findings, a sentence each

- **The address moves**: Bo's home page and llama's address went from port
  64525 to 49821 across one restart, and every program set to the old one
  was refused the connection (prediction 5), so a bookmark, the page
  install.sh opened, and a tool's `base_url` each break at the next login.
- **Node programs are refused**: Node's own `fetch` reached svrngs and was
  refused 403 `closed`, before the restart and after, because svrngs takes
  a request with `Sec-Fetch-Mode` but no `Sec-Fetch-Site` for a page's
  (prediction 4, `data/bo.log:24-27`, `origin.rs:74-93`). Not in any row
  of this queue as written; it is the address's story (start-address (b),
  "whatever start-measure (a) found programs cannot … given a form they
  can"). OpenAI's Node package, which uses `fetch`, was not run.
- **`*.localhost` resolves for programs here**: curl, Python and Node each
  resolved it on this macOS (predictions 2-4), so no second form is needed
  on this host; Linux, where it rests on the system's resolver, is
  could-not-judge.
- **A lend of llama-server shows a member's chat to the others**: through a
  lend keeping back only `/lora-adapters`, Bo saw when Cy chatted, how
  long, and with what settings at `/slots`, and, with the conversation's
  id, read Cy's whole answer and stopped another (predictions 8, 9).
- **What llama-server's lend should keep back**, by the reading (6) and the
  run (7-10): `/slots`, `/v1/stream`, `/v1/streams`, `/v1/chat/completions/control`,
  `/lora-adapters`, `/metrics`, `/tools`, `/cors-proxy` and `/models/`
  always; `POST /props`, but not `GET /props`, which the chat page reads —
  and svrngs's refusal is by path for every method (`crates/svrngs-node/src/lend.rs:94-110`,
  `covers`), so it cannot keep back the one without the other; and
  `GET /props`'s `model_path`, which names the lender's account, only
  by changing an answer, which svrngs does not do.
- **The home page lists; it leads nowhere**: of eleven acts, five are done
  on it, five only in a terminal, and one, leaving a group, nowhere; it
  names no line to type for any of them (11, 12), and with no group yet
  it says less than the terminal does (13).
- **One silent computer holds the whole page**: 3.6 s of "Reading your
  groups…" with Cy's computer stopped, against 0.5-0.6 s (16).

## 5. What to change for that reader, ranked

Each estimate is lines of code as `scripts/lean_check.py` counts them, by
reading the code it would change, not by writing it. Left at this run's
base: 15,950 − 15,828 = 122.

| # | change | for | row | estimate | running total |
|---|---|---|---|---|---|
| 1 | the home page's and each lend's address the same across restarts, a taken port said in words (`cli.rs` `API`, install.sh) | a bookmark and a tool's address that keep working | start-address (b) | 25 | 15,853 |
| 2 | a request with no `Sec-Fetch-Site` taken as a program's (`origin.rs:76`) | Node programs answered | start-address (b), as its spirit reads | 2 | 15,855 |
| 3 | what a lend of llama-server keeps back named once in the well-known table, and svrngs knowing it not by port | a lender who does not read llama-server's source | start-llama (a) | 35 | 15,890 |
| 4 | at the top, what this page is for and what can be done here; with no group yet, how to make or join one | someone opening it the first time | start-home | 11 | 15,901 |
| 5 | each act the page cannot do named, with its one line to type | someone who would otherwise be stuck | start-home | 10 | 15,911 |
| 6 | for a lend a program uses, the address to give it, with a copy button | a newcomer pointing a tool at a friend's model | start-home | 12 | 15,923 |
| 7 | each lend's link saying what it is and whose | "Sam’s llama" read as a riddle | start-home | 4 | 15,927 |
| 8 | "What apps use" hidden when empty; the box's words shown; a lend opened beside the page, not in its place | fewer things to puzzle over | start-home | 5 | 15,932 |
| 9 | each group shown as it answers, not all held for the slowest | 3.6 s of "Reading your groups…" | start-home | 10 | 15,942 |
| 10 | a link to invite someone made on the page, as `svrngs link` makes it | the act most asked of a newcomer's page | start-home | 30 | 15,972 — past the cap |
| 11 | leaving a group | an act nothing does now | none; no verb exists, and who may remove whom is the core's (DESIGN §6) | not estimated | — |
| 12 | the group's folder opened from the page | a path read as words | start-home | 15 | — |

Rows 1-9 fit, 114 lines, to 15,942. Row 10 does not by 22: the charter
sends it to `NEEDS_HUMAN.md` with its estimate if start-home cannot make
room, and the rows that fit go on. Row 11 is a decision before an
estimate.
