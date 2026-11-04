# The pantry's scope, on one host — predictions, 2026-10-22

Row `models-scope` (`ralph/next/models/STATE.md`). Written before the first
install of this run. `SCOPE.md` beside it says what each line below met,
with the real responses in `data/`. One host, one run: not a rate
(AGENTS.md 7).

## 0. The host and what was here before the first install

- macOS 27.0.1, build 26A434, arm64, 64 GiB RAM, 149 GiB free.
- `llama.cpp 10180 (11b068d06)`, already installed by Homebrew before this
  run; its `llama-server --version` says so. No model weights were in
  `~/.cache/llama.cpp` or the Hugging Face hub cache at the start.
- Podman 6.0.1; Homebrew; Node 20.20.2; uv; ffmpeg; Playwright's Chromium
  is already in `web/node_modules` (1.59.1).
- Network reaches `huggingface.co` (HTTP 200) and GitHub.
- The group and its lends are made with this checkout's `target/debug/svrngs`
  (`cargo build -p svrngs` at the run's commit), in a data dir of the run's
  own under `data/`.

**The rule this run is judged by.** The pantry's convention is a lend that
answers OpenAI's `/v1/models`; a server that does not is *unheard*, however
good its other endpoint is. So a server's discovery is measured by exactly
that route, never by a kind this run keeps a list of. A server that answers
only `/inference`, `/props` or a vendor route is not a model for this
convention.

## 1. Pieces, what I expect, and the verdict I expect

### 1.1 `llama-server`, one model: the code model (with tools) and the vision model

Install: Homebrew `llama.cpp` 10180, already present and recorded. Weights:
`Qwen/Qwen2.5-Coder-7B-Instruct-GGUF` at revision
`13fb94bfda8c8cf22497dc57b78f391a9acb426a`, `q4_k_m` (two shards), Apache-2.0;
`ggml-org/Qwen2.5-VL-3B-Instruct-GGUF` at revision
`5037fcf163dd95d1e41d1974465f0898ed108ca2`, `Q4_K_M` weights and
`mmproj-Qwen2.5-VL-3B-Instruct-f16.gguf` (f16 projector is the smaller
projector), Apache-2.0. Both file sizes and sha256 recorded.

- Serve: `llama-server -m <coder> --jinja --alias ana/coder --host 127.0.0.1
  --port <p>`; vision: `llama-server -m <vl> --mmproj <mmproj> --alias
  ana/vision --host 127.0.0.1 --port <p>`.
- **`/v1/models`**: clean; answers one entry, its `--alias` as `id`, and
  `/props` gives the context it really has.
- **Routing by `model`** (JSON): the server has one model and no router, so
  I expect it to *accept and ignore* the JSON `model` field, answering with
  its one model whatever string is sent. This is clean for the pantry (the
  pantry writes back the id it got from `/v1/models`) and unsafe as a
  *substitution* if anything else chose the id: the server is not the guard.
  A borrowed model id is answered by the loaded model, so the pantry's
  prefix mapping must never send a lend an id that lend did not answer.
- **Multipart transcription**: N/A; I expect a 404 or 405 for
  `/v1/audio/transcriptions`.
- **Tools (`--jinja`)**: clean; Qwen2.5-Coder's template emits
  `tool_calls` for an OpenAI `tools` array. If it emits only text, the model
  is out for the story's "coder calls the search tool".
- **Vision**: clean with `--mmproj`; an OpenAI image content part is read.
- **Routes that change the lender's computer**: `/slots` (saves, erases,
  reads another person's chat), `/v1/stream`, `/v1/streams`,
  `/v1/chat/completions/control`, `/lora-adapters`, `/metrics`, `/tools`
  (runs a shell), `/cors-proxy`, `/models` (router mode). **Prediction:**
  `svrngs share --refuse` each of them answers 403 at the lend's origin and
  the route is not reached; `/props` stays reachable and shows the model's
  path on disk (the well_known table already accepts that leak).
- **One stock call through the lend's origin**: clean, streamed, with the
  door's `x-svrngs-*` headers on the request reaching `llama-server` (it
  ignores unknown headers).
- **Verdict predicted:** clean.

### 1.2 Several models behind one lend

**(a) `llama-server` router mode** (`--models-dir`, `--models-max`), same
llama.cpp 10180:

- `/v1/models` lists every discovered model; routing by `model` is real, and
  I expect an unknown model to be **refused** (`400`/`404`), never answered
  by another.
- Auto-load changes the lender's computer; a model load is a route
  (`POST /models`) that must be kept back. I expect `GET /models` (the
  listing twin) and `/v1/models` to stay, and `POST /models`,
  `/models/unload`, `/slots*`, `/props` per-model to be the keep-backs.
- **Verdict predicted:** clean for routing, messy for keep-backs (the
  listing and the load share one path, so a method-scoped refusal is
  needed; `RefusedPath::covers` refuses every method alike, so `--refuse
  /models` would also remove the listing). This is the row's most likely
  "messy" finding.

**(b) `llama-swap`**, release `v266` (2026-10-22), MIT, mostlygeek, one
maintainer, active; downloaded arm64 tarball and sha256 recorded. It runs
`llama-server` children from a YAML config.

- `/v1/models` lists configured models; routing by `model` is real and I
  expect an unknown id to be refused.
- Admin surface: the UI (`/ui`), logs (`/logs`), `/running`, `/unload`,
  `/health`, `/metrics`. **Prediction:** routing clean; keep-backs messy
  (a UI and a log route, plus a supervisor process). Integrity: MIT and
  active, but a single maintainer and a second supervisor over
  `llama-server`, which DESIGN's "do not supervise another component"
  warns about. Judge it clean-enough, messy on keep-backs.
- **Verdict predicted:** messy (keep-backs), usable.

### 1.3 `whisper.cpp`'s server, speech to text

Install: Homebrew `whisper.cpp` 1.9.5 (MIT), already not installed at the
start; model `ggml-base.en.bin` from `ggml-org/whisper.cpp` at a pinned
revision, MIT weights.

- `/v1/audio/transcriptions`: I expect it present and clean (OpenAI-shaped
  multipart, `model` ignored).
- `/v1/models`: **I expect it absent.** whisper.cpp's server is not an
  OpenAI model server; it has `/inference` and `/load`. If absent, the
  pantry's own convention says *unheard*: the story's speech-to-text is not
  a model lend as it is, and needs a tiny OpenAI-in/out wrapper (which is
  allowed as an adapter, but is a second program on Sam's computer).
- **Multipart `model` routing:** one loaded model; I expect `model` to be
  ignored, never refused.
- Admin: `/load` (loads a model from disk), `/unload`, `/metrics`.
  Keep-backs: `/load`, `/unload`, `/metrics`. `/inference` stays if it works
  on the loaded model, but it is not the OpenAI wire.
- **Verdict predicted:** messy (discovery fails the convention); out as a
  bare lend for `pantry list`, clean behind a wrapper.

### 1.4 One open speech server, speech out (Kokoro)

Install: `remsky/Kokoro-FastAPI` pinned at a commit, Apache-2.0, with the
`hexgrad/Kokoro-82M` weights (Apache-2.0); `espeak-ng` from Homebrew for
phonemes. Install by `uv` in a venv of the run's own, not a container, so
the binary versions are recorded.

- `/v1/models`: I expect it present (`kokoro`), and `/v1/audio/speech` with
  `model: "kokoro"`, `voice`, `input`, streamed audio.
- **Routing by `model`:** I expect the request's `model` to be validated
  loosely (accepted if it names a known model, else 400) — clean.
- Admin: `/v1/audio/voices` (list is needed; any POST is an upload),
  `/v1/models` history/download routes, `/docs`, `/openapi.json`. Watch
  which of these change the lender's computer. Keep-backs predicted:
  anything that writes voices/models/history.
- **Verdict predicted:** clean for the wire, messy for install (Python,
  espeak, ~350 MB of weights); licence Apache-2.0 for both server and
  weights.

### 1.5 MCP: one Streamable HTTP server, one stdio server behind `mcp-proxy`

Install: `mcp-proxy` 0.13.0 (Homebrew, MIT, sparfenyuk); Node packages
pinned by exact version from npm: `@modelcontextprotocol/server-everything`
(official, MIT) over its Streamable HTTP transport, and
`@modelcontextprotocol/server-time` (official, MIT) run as stdio behind the
proxy. Npx is pinned to the package version, and the resolved package
version is recorded.

- **Streamable HTTP:** `initialize` and `tools/list` clean; annotations
  (`readOnlyHint`, `destructiveHint`) present on the tools. **Headers:** the
  door's `x-svrngs-*` headers arrive at the HTTP server (it is the same
  request, the door only adds headers). `server-everything` does not show
  them, so the run also lends a stand-in echo MCP server of a few lines to
  prove the headers' arrival; the stand-in is test scaffolding in this run,
  never a product.
- **stdio behind `mcp-proxy`:** `initialize` and `tools/list` clean.
  **Headers:** I expect `mcp-proxy` to terminate HTTP and speak stdio, and
  to forward *no* arbitrary HTTP header to the child. So Sam's stdio tool
  cannot see Bo's `x-svrngs-person` (or anything) unless the proxy is given
  an explicit env/args mapping, which stock `mcp-proxy` does not do. That
  is C5's identity line: a native Streamable HTTP server can act as Bo; a
  stdio server behind a stock bridge cannot. **Verdict predicted:** native
  clean; stdio bridge messy on identity, clean on protocol.
- Admin: none on either; the bridge's own process is the lender's, no route.

### 1.6 The interface a person runs

Candidates, licence read whole at the pinned tag: Open WebUI (BSD-3 plus a
2025 branding clause: keep the "Open WebUI" name and logo, and for more
than 50 users do not remove them), LibreChat (MIT), AnythingLLM (MIT), Jan
(Apache-2.0 in recent releases; older AGPL-3.0 — read the pinned tag).
Judge each for openness and integrity, then measure one against a stand-in
OpenAI address: chat, transcription and speech by model id, and an MCP
Streamable HTTP address for a tool.

- **Open WebUI**, single container from a pinned image digest: I expect
  chat clean (one base URL and model ids), transcription and speech
  configurable but through *separate* base-URL settings for STT and TTS,
  not one address, and no native Streamable HTTP MCP client (its MCP story
  is a proxy/tool server). **Prediction:** messy; the branding clause is
  the operator's call, not ours.
- **LibreChat**, MIT: I expect chat, `speech.stt`/`speech.tts` against an
  OpenAI-compatible base URL with model ids, and MCP `streamable-http`
  servers — the best feature fit — but a heavy stack (Node image, MongoDB,
  Meilisearch) to pin and run here. **Prediction:** clean features, messy
  install; live measurement could-not-judge within this run if the stack
  cannot be pinned cleanly.
- **AnythingLLM**, MIT: I expect chat clean, STT/TTS by model id absent or
  partial, MCP partial. **Prediction:** messy/out.
- **Jan**: I expect a desktop app, not a server, and no one-address
  STT/TTS + Streamable HTTP. **Prediction:** out for this shape.
- **The measured interface:** Open WebUI, against one stand-in address
  written in this run (its responses logged, never a model). If its STT,
  TTS and MCP cannot all use that one address, that is the finding: the
  interface, not the pantry, is the limit on "one address".

### 1.7 The bridge's quality

Three screenshots of a real UI in this repository (the home page, served by
this run, at three widths; each with one bug a person would report: a
clipped label, an overlapping button, a table column cut off). For each:

- the vision model (`Qwen2.5-VL-3B-Instruct`) alone, asked to describe the
  screenshot for a programmer who cannot see it;
- the code model (`Qwen2.5-Coder-7B-Instruct`) given that description under
  one fixed prompt, asked for the fix.

Rubric, stated before the answers: **(1) the bug found** (the answer names
the same element and the same symptom as the planted bug); **(2) the fix
runs** (the proposed CSS/property, applied to the page, removes the symptom
in a fresh screenshot). Each of the three, two answers, kept whole.

**Prediction:** the vision model names the symptom and some nearby text in
two of three, and misses the third (the off-screen button) because it is
absent from the frame; the coder given the description proposes a plausible
selector change in two of three, and "runs" in one or two; the bridge helps
a coder that cannot see, but at 3B+7B it is not reliable enough to ship as
a promise. One host, three screenshots: not a rate.

## 2. The bars this run expects to leave for the operator

- What v0 adopts: a code model, a vision model, a speech-to-text server, a
  speech server, an MCP bridge, an interface — each only if clean or
  acceptably messy, and each fully open and high integrity.
- Whether the bridge example ships.
- **Predicted shakiest piece:** whisper.cpp's missing `/v1/models`, which
  decides whether speech-to-text is a bare lend or needs a wrapper.
- **Predicted second:** the stdio MCP bridge dropping the door's identity
  headers, which decides whether a stdio tool can act as the person.
- **Predicted third:** llama-server router mode's `/models` being both the
  listing and the loader, which decides whether one lend may carry several
  models without a method-scoped refusal the door does not have.

If any of those holds, it is said in `SCOPE.md` with the response beside
it, never dropped and never rounded to a pass.
