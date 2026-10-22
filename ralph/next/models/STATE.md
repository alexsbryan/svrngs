# svrngs models — the pantry's lean v0, built for two jobs and tools (operator, 2026-10-22)

The design is svrn-docs `rings/current/MODELS_ON_THE_RING.md` (a draft of
2026-10-21, at svrngs `cefff1b5`), read whole. A model is a lend; what it
can do is a card; the pantry, an app on the person's computer, routes. The
operator asked for it "with a lean startup lens applied to the product
priority and scope", with "No ollama" and "Fully open and high integrity
software only", and named the jobs it is for (2026-10-22):

- **A screenshot to a code fix.** "take screenshot of UI, send it to local
  model, local model does code to fix (the problem is some very strong
  local models don't have built in multimodal so this would have to be
  quietly orchaestrated)".
- **Voice in and out.** "the person who does a voice to text I/O to their
  model interface".
- **Tools.** "opening as much of the open source world as possible for
  people, but just through the right protocol adoption and simple
  machinery".

Written and held: "Write rows, hold".

**The shape** (CHARTER.md, "The lean cut"):
- One local address on the consumer's computer, `pantry serve`, that the
  person's own interface points at. It routes each OpenAI request by path
  and model to a friend's lend: chat, `/v1/audio/transcriptions`,
  `/v1/audio/speech`.
- When an image goes to a model that cannot see, it quietly sends the image
  to a vision model a friend lends and swaps in the description. The
  response names both hops.
- Tools speak MCP over Streamable HTTP, lent like any app. The agent loop
  stays in the person's interface.

No platform line: the repository stays at 17,634 of 17,635 and
svrngs-core at 3,149 of 3,150 (`scripts/lean_check.py`, 2026-10-22).
`apps/` and `demo/runs/` are not counted. There are four rows:

1. the scope: the jobs' open runtimes, the bridge's quality and MCP's
   bridge measured here, then the operator chooses;
2. `pantry up`: one command lends a model or an MCP server;
3. `pantry serve` and `pantry list`: one local address, the vision bridge,
   and every lend named;
4. the run: both jobs and a friend's tool, end to end on one host; then a
   run with real people.

**Waits, each with its trigger** (PLAN §2):
- the `card` field on the lend declaration (MODELS §6): a fact must travel
  that neither the runtime, MCP nor the lend's name can say, counted;
- need acts, standing in the record (§2): a person asks the group for a
  model nobody lends, counted;
- a pantry page on the home page: a provider or consumer who does not use
  a terminal asks, counted;
- the deterministic spread by the person's key (§3 5): two consumers' loads
  collide on one provider while another sits idle, counted;
- image generation, embeddings and the space check, rerank and video (§3,
  §4): a group lends one and someone asks to reach it through the pantry,
  counted; each is reachable today as a lend's own address;
- one MCP address gathering every friend's tools: a person adds more than a
  handful of tool lends and asks for one, counted;
- per-person permission on a tool beyond the door's who and how many: a
  lender asks to let one friend use a tool that writes and not another;
- OpenAPI tool servers: an interface that speaks no MCP is asked for;
- A2A agents, card signatures, and the rest of MODELS §12, each with its
  trigger there.

- [ ] models-scope — depends [] — ONE ROW, a survey and a measured run, nothing to product, its predictions committed before any data, ending in NEEDS_HUMAN.md for the operator's choice (the operator, 2026-10-22: "I'd like to at least try to scope out #3, but if it's too messy we can fall back to #1"; the two jobs and tools, STATE.md's opening): (a) (Predictions, nothing to product.) WRITE `demo/runs/models/<date>-scope/PREDICTIONS.md` before any install: for each piece below, what you expect it to answer about itself, what its bar is, and clean, messy or out (b) (A measured run, nothing to product.) MEASURE on this host, each installed at a pinned version (Homebrew, a pinned source build, or a pinned Podman image, recorded), each served on 127.0.0.1 with the smallest open model that does the job, then lent with `svrngs share <port> --kind <runtime> --name <name>` in a data dir of the run's own: llama.cpp's `llama-server` with a strong text-only code model with tool calls (`--jinja`), and with a vision model through its multimodal support (`--mmproj`); whisper.cpp's server for `/v1/audio/transcriptions`; one open text-to-speech server for `/v1/audio/speech` (a Kokoro server, judged for integrity: licence, maintainers, history); one front per provider, llama.cpp's router mode and llama-swap, each judged for integrity the same way; one MCP server that speaks Streamable HTTP and one stdio server behind a pinned open stdio-to-Streamable-HTTP bridge (`mcp-proxy`), called through the lend by a stock MCP client. For each, record in `demo/runs/models/<date>-scope/SCOPE.md`, with the real responses beside it: the serve command; which facts it answers about itself and where (`/v1/models`, `/props` and its `modalities`, `tools/list` and its annotations); what only a recipe knows and so travels in the lend's name; every route that changes the lender's computer (loading or unloading a model, logs, a UI, uploads) that a recipe's `--refuse` must keep back; whether the door's `x-svrngs-*` headers reach an MCP server; one stock call through the lend's origin, answered or not (c) (A measured comparison, nothing to product.) THE BRIDGE'S QUALITY: three real UI screenshots, each with a bug a person would ask to fix; for each, the vision model alone, and the code model given the vision model's description under one fixed prompt; both answers kept in the run, judged by a stated rubric (the bug found, the fix runs), one host, not a rate (d) (A decision for the operator.) WRITE SCOPE.md's end: each piece clean, messy or out, with the reason in a line; the bridge's verdict; the front to adopt or none; the app lines estimated from what was measured. Then NEEDS_HUMAN.md asking the operator which runtimes, front and TTS server v0 adopts, and whether the bridge is worth building, and stop. A runtime that cannot be installed or pinned here is could-not-judge, said with why, never dropped — read: MODELS_ON_THE_RING whole; CHARTER.md; `crates/svrngs-node/src/well_known.txt` (llama-server's `keeps`); `crates/svrngs/src/cli.rs` `share`'s help (`--refuse`, `--name`, `--kind`, `--limit`); `crates/svrngs-node/src/surface.rs` (`/v1/lends`) — check: PREDICTIONS.md committed before the first install; every piece measured or could-not-judge with why; every response cited in SCOPE.md in the run's data; no file outside `demo/runs/models/` changed; no runtime left running and nothing lent when the row ends (`ps`, the run's `/v1/lends`)
- [ ] models-up — depends [models-scope] — ONE ROW, its parts in order, each its own commit, each part's own check proven as it lands with its red run first, and `scripts/check.sh` once, at the close; what v0 adopts is the operator's choice at models-scope, recorded in this row's first commit: (a) (A test, in `apps/pantry/`.) A RECIPE AND ITS LEND: a test, red at HEAD since there is no pantry, where `pantry up <recipe>` with a stand-in server (a few lines of HTTP, no model) serves on 127.0.0.1, waits until it answers, lends it with the recipe's kind, name, limit and every `--refuse` path, and on Ctrl-C stops it and unshares, leaving no process running and nothing lent; the same for an MCP recipe whose stand-in speaks stdio behind the bridge (b) (Adds, `apps/pantry/`; nothing counted.) `pantry up`: a recipe is one small file: the runtime and its pinned version, for a model the pinned HuggingFace repo, revision and file with its sha256 and the serve arguments, for an MCP server its command and whether it needs the bridge, then the lend's kind, its canonical name carrying what the server cannot say, its limit, and its `--refuse` paths from SCOPE.md; a model is pulled through the runtime's own pull (`llama-server -hf` and its kin), never a downloader of the pantry's; one foreground child, or the front SCOPE.md chose; a missing runtime refused with its install line, a hash that differs refused by name; recipes for the jobs' models (code, vision, transcription, speech) and for one MCP server (c) (Docs.) `apps/pantry/README.md` in the house voice: one command to lend a model or a tool, what the group can and cannot do with it, that a lent tool acts on the lender's computer with the lender's permissions, and how to stop — read: SCOPE.md and the operator's choice; CHARTER.md; MODELS §5 "Provider", §7 (supervision, weights); `apps/grapevine/` for an existing Node app with tests; `crates/svrngs/tests/share.rs` (`Program`: a child stopped on drop, 1c94aa4b) — check: (a) red first and green after; a plant that skips the unshare on Ctrl-C turns it red; the test leaves no process (`ps` before and after); each recipe brought up once by hand here, its lend listed by `/v1/lends` and one stock call answered, recorded in the commit; `python3 scripts/lean_check.py` unchanged
- [ ] models-serve — depends [models-up] — ONE ROW, its parts in order, each its own commit, red first, and `scripts/check.sh` once, at the close (MODELS §3 rules 4-6, §5 "Consumer" v1, §10; STATE.md's opening): (a) (Tests, `apps/pantry/`.) ONE ADDRESS: tests, red at HEAD, over stand-in servers lent by stand-in people: a chat request to `pantry serve`'s address for a model reaches the lend serving it, streamed, with `tools` passed through untouched; `/v1/audio/transcriptions` and `/v1/audio/speech` reach the lends that serve them; a request with an image for a model whose runtime says it cannot see is answered by that model with the image swapped for a vision lend's description, `tools` intact, and the response's `x-pantry-model`, `x-pantry-provider` and `x-pantry-bridged-by` name both hops; with no lend that fits, the error names every candidate and the first fact it failed, or why it did not answer (asleep, full, could-not-judge), and never substitutes another model; `/v1/models` lists what the group lends now (b) (Adds, `apps/pantry/`.) `pantry serve`: one OpenAI-compatible address on 127.0.0.1, for the person's own interface; it reads `/v1/lends` for every group this computer holds, makes each lend's card from what its runtime answers through the lend's origin (SCOPE.md's endpoints) and its name, and routes by path and model; own computer first, then by the lender's name; `--from <person or lend>` to pin; the vision bridge with one fixed prompt for UI screenshots (layout, every visible string, the element hierarchy, errors), its prompt in the README; no score, no history, no agent loop (c) (Adds, `apps/pantry/`.) `pantry list`: each lend of a model, with whose computer, which model and what it can do; and each lend of an MCP server, with its tools and whether each is read-only or destructive, from MCP's own `tools/list` annotations, and the one address to paste into an interface's MCP settings (d) (Docs.) the README's consumer half: point your interface at one address for chat, transcription and speech, paste a friend's tool address into its MCP settings (Open WebUI, Zed, Goose), and that a friend's tool output enters your model's context — read: MODELS §3, §5, §10; SCOPE.md; `crates/svrngs-node/src/surface.rs` (`/v1/lends`, `unanswered`, `unheard`, `refused`, `older`) — check: (a) red first and green after; a plant that drops `tools` in the bridge turns it red; a plant that leaves a non-answering lend out of the error turns it red; `python3 scripts/lean_check.py` unchanged
- [ ] models-run — depends [models-serve] — ONE ROW, a recorded run on one host, its predictions and driver committed before any data, then the docs from what it found, and `scripts/check.sh` once, at the close (MODELS §11, its bars kept to the jobs): (a) (Runs, `demo/runs/models/<date>/`.) RUN, every person a program on this host, each with a data dir and a `HOME` of its own, in one group: Ana lends the code model and the vision model with `pantry up`; Sam lends transcription, speech and one MCP tool, and is asleep for one ask; Bo points a stock OpenAI client at `pantry serve` and a stock MCP client at Sam's tool. Bars: a provider from nothing to serving with one command, timed against ten minutes, the download said apart; Bo's screenshot, sent to the code model, answered with a fix, both hops named; Bo's recorded speech transcribed, answered and spoken back, each hop named; Sam's tool listed with its effects and called as Bo; with Sam asleep, the error names Sam's lends and why; a kept-back route answered refused through the door (b) (Docs.) WRITE PLAN §1 and §2 from what landed: the pantry's v0 in §1, and this STATE's "Waits" in §2, each with its trigger; DESIGN §3, "Lent apps", one paragraph: a model or a tool is lent like any app, and the pantry is an app; the run's README with every number in its data. Then the queue stops for a run with real people, whose bars the operator sets — read: MODELS §11; `demo/runs/grapevine/2026-10-16/` for a run's shape; CHARTER.md — check: (a) predictions and driver committed before data; every number in the README in the run's data; no process from the run left running (`ps`); (b) PLAN and DESIGN rewritten in place, no section added to override; `python3 scripts/lean_check.py` unchanged
