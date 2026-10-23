# The pantry's scope, measured on one host — 2026-10-22

Row `models-scope` (`ralph/next/models/STATE.md`). Predictions were committed
first, at `84f82cc8` (`PREDICTIONS.md`), before the first install. This is one
host and one run of each: not a rate (AGENTS.md 7). Every answer below is a
file under `data/responses/`, named beside it.

## The rule this run is judged by

The pantry's convention is a lend that answers OpenAI's `/v1/models`; a server
that does not is *unheard*, however good its other endpoint is. A server's
discovery is therefore measured by exactly that route, never by a kind this
run keeps a list of.

## Pinned and recorded

| What | Pinned | Record |
|---|---|---|
| Host | macOS 27.0.1 (26A434), arm64, 64 GiB, Apple M2 Max | `data/responses/versions.json` |
| `llama.cpp` | Homebrew 10180 (`11b068d06`) | `data/responses/versions.json` |
| `whisper.cpp` | Homebrew 1.9.5 | `data/responses/versions.json` |
| `mcp-proxy` | Homebrew 0.13.0 | `data/responses/versions.json`, `data/responses/mcp/piece.json` |
| Python MCP SDK | 1.30.0 in `data/venv/mcp` | `data/responses/mcp/piece.json` |
| `llama-swap` | release v266 (`c099777`), MIT, darwin_arm64 | `data/bin/checksums.txt`; sha256 `704db943…d600a` |
| Kokoro FastAPI | `ghcr.io/remsky/kokoro-fastapi-cpu:v0.9.0` | digest `sha256:73940a71…cc20`, `data/responses/kokoro/piece.json` |
| Open WebUI | `ghcr.io/open-webui/open-webui:v0.12.3` | digest `sha256:bd153a1f…60887`, `data/responses/interface/piece.json` |
| Qwen2.5-Coder-7B-Instruct Q4_K_M | `Qwen/Qwen2.5-Coder-7B-Instruct-GGUF@13fb94bf…426a` | sha256 `89f12054…14be` + `0183b3c8…a05e0` (two shards) |
| Qwen2.5-VL-3B-Instruct Q4_K_M + mmproj | `ggml-org/Qwen2.5-VL-3B-Instruct-GGUF@5037fcf1…8ca2` | sha256 `d02fe9b6…6c12`, `b9160fe9…d5e` |
| Qwen2.5-Coder-1.5B / 0.5B Q4_K_M (router tests) | `Qwen/…@f86cb2c1…4361`, `@ebb20151…875f` | sha256 `cc324af0…46046`, `1d961463…ff32` |
| whisper `ggml-base.en.bin` | `ggml-org/whisper.cpp` | sha256 `a03779c8…6d002` |

## 1. The pieces

### 1.1 `llama-server`, one model: the code model and the vision model — **clean for chat and vision; messy for tool calls**

Serve: `llama-server -m <coder> --jinja --alias ana/coder --host 127.0.0.1
--port 8101 -c 4096 -ngl 99`; vision adds `--mmproj
mmproj-Qwen2.5-VL-3B-Instruct-f16.gguf` (`data/responses/llama-coder/piece.json`,
`data/responses/llama-vision/piece.json`).

- **Discovery.** `/v1/models` answers one `data[].id`, the `--alias`
  (`data/responses/llama-coder/direct-models.json`). `/props` shows the real
  context and `model_path` (`direct-props.json`).
- **Routing by `model`.** JSON: the single server accepts and ignores the
  field. A request naming `someone/not-this-lend` is answered by `ana/coder`,
  with `"model":"ana/coder"` written back
  (`data/responses/llama-coder/origin-chat-wrong-model.json`). That is clean
  for the pantry, which writes back the id it read; it is a substitution only
  if something else chooses the id. The server is not the guard.
- **Multipart transcription.** N/A: `/v1/audio/transcriptions` is not a route
  here (404 at the same class of server in `whisper/direct-transcription.json`).
- **Tools (`--jinja`).** Messy: Qwen2.5-Coder emits the call as text inside
  `<tools>…</tools>` in `content`, not as OpenAI `tool_calls`
  (`data/responses/llama-coder/origin-tools.json`). This model, served this
  way, cannot be the story's "coder calls the search tool" without an adapter.
- **Vision.** Clean: the `/v1/models` entry carries `"multimodal"` and an
  OpenAI image part is read (`data/responses/bridge/*-vision.json`).
- **Keep-backs.** `/slots`, `/v1/stream`, `/v1/streams`,
  `/v1/chat/completions/control`, `/lora-adapters`, `/metrics`, `/tools`,
  `/cors-proxy`, `/models` answered 200/400/404/501 directly
  (`data/responses/llama-coder/direct-*.json`) and every one of them answers
  **403** through a lend made with `--refuse` each
  (`data/responses/llama-coder/origin-*.json`). `/props` stays reachable and
  shows the model's path on disk; the well-known table already accepts that
  leak. The door is the guard.
- **One stock call through the origin.** 200, `"content":"ok"`, `"model":
  "ana/coder"` (`data/responses/llama-coder/origin-chat.json`).

### 1.2 Several models behind one lend

#### (a) `llama-server` router mode — **messy**

Serve: `llama-server --models-dir data/weights/router --models-max 2 --host
127.0.0.1 --port 8104 --jinja -ngl 99` (`data/responses/router/piece.json`).

- **Discovery.** `/v1/models` answers **six** ids, not the two in
  `--models-dir`: it also lists models found in the machine's caches —
  `unsloth/Qwen3-8B-GGUF:Q4_K_M`, `lrsbrgrn/HalluGuard-Qwen3-4B-GGUF:4B`,
  `ggml-org/Qwen3-Reranker-0.6B-Q8_0-GGUF:Q8_0`,
  `Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M`, plus the two tiny coders
  (`data/responses/router/origin-models.json`). A lend of the router leaks the
  names of every model on the lender's disk, and every entry carries the
  model's `status.args` (`--hf-repo`, port, `/opt/homebrew/...`). There is no
  `--refuse` that removes the cache entries without removing discovery.
- **Routing by `model`.** Real, and an unknown model is refused:
  `{"code":400,"message":"model name=not/a-model is not running"}`
  (`data/responses/router/origin-chat-unknown.json`), never answered by
  another. A known model answers (`origin-chat.json`).
- **Keep-backs.** `GET /models` and `POST /models` share one path: the listing
  and the loader (`direct-models-path.json`, `direct-load.json`). `--refuse
  /models` returns 403 for **both** through the door (`origin-models-path.json`,
  `origin-load.json`), which keeps the loader back but also removes the
  listing; `/v1/models` still answers (`origin-models.json`). A load of an
  unknown name answers 200 `{"success":true}` and does nothing
  (`direct-load-unknown.json`); the real refusal is at chat time.
- **Verdict:** routing is clean, but the leak of every cache model's name and
  path, plus a method-blind refusal on the one path, makes it **messy** and
  **out** as a bare multi-model lend for the minimum. A wrapper that exposes
  only `/v1/models` for the intended set is the way to use it.

#### (b) `llama-swap` v266 — **clean routing, messy keep-backs**

Serve: `llama-swap --config data/llama-swap.yaml --listen 127.0.0.1:8105`, two
`llama-server` children on `${PORT}` (`data/responses/swap/piece.json`).

- **Discovery.** `/v1/models` answers the two configured ids (`tiny-0.5b`,
  `tiny-1.5b`) (`data/responses/swap/origin-models.json`).
- **Routing.** Chat by id answers 200 (`origin-chat.json`); an unknown id is
  404 `"no router for requested model"` (`direct-chat-unknown.json`). But the
  response's `model` field is the child's model **path**, not the id asked for
  (`origin-chat.json`): the pantry writes back the lend's own id, yet the
  interface sees a filesystem path. Messy for a picker.
- **Keep-backs.** `/ui` (308), `/logs`, `/running`, `/unload`, `/metrics`
  answer directly (`data/responses/swap/direct-*.json`) and all answer 403
  through the door with `--refuse` (`origin-*.json`).
- **Integrity.** MIT, active, but one maintainer and a second supervisor over
  `llama-server`; judged usable, not clean.
- **Verdict:** messy (response id and a supervisor process), usable.

### 1.3 `whisper.cpp`'s server — **out as a bare OpenAI lend**

Serve: `whisper-server -m ggml-base.en.bin --host 127.0.0.1 --port 8106 -l en`
(`data/responses/whisper/piece.json`).

- **Discovery.** `/v1/models` is **404 File Not Found**
  (`data/responses/whisper/direct-models.json`). Under the convention this lend
  is *unheard*.
- **Transcription.** `/v1/audio/transcriptions` is also **404**
  (`direct-transcription.json`), with `model` in the form or not
  (`direct-transcription-other-model.json`). The OpenAI route does not exist in
  1.9.5.
- **What does work.** `/inference` with the same multipart file answers
  `{"text":" The quick brown fox jumps over the lazy dog.\n"}`
  (`direct-inference.json`).
- **Keep-backs.** `/load` (400), `/unload` (404), `/metrics` (404) directly;
  all 403 through the door with `--refuse` (`direct-load.json`,
  `origin-load.json`).
- **Verdict:** **out** as a bare lend. Speech-to-text needs a small
  OpenAI-in/out wrapper on Sam's computer (allowed as an adapter, but a second
  program), or a different server that speaks the OpenAI route.

### 1.4 Kokoro speech out — **clean**

Serve: `podman run -d --name pantry-scope-kokoro -p 127.0.0.1:8107:8880
ghcr.io/remsky/kokoro-fastapi-cpu:v0.9.0` (`data/responses/kokoro/piece.json`).

- **Discovery.** `/v1/models` answers `tts-1`, `tts-1-hd`, `kokoro`,
  `gpt-4o-mini-tts` (`data/responses/kokoro/direct-models.json`). It borrows
  OpenAI's ids; the pantry lists them as the lend answered them.
- **Speech.** `POST /v1/audio/speech` with `model:"kokoro"` answers 200,
  `audio/mpeg`, 25,004 bytes, saved at
  `data/responses/kokoro/origin-speech.mp3`.
- **Routing by `model`.** An unknown model is refused 400 `"Unsupported model:
  not/a-model"` (`origin-speech-wrong-model.json`); it never substitutes.
- **Keep-backs.** `/docs` and `/openapi.json` are 200 directly and 403 through
  the door (`direct-docs.json`, `origin-*.json` from `piece.json`).
  `/v1/audio/voices` is a read the interface needs; the byte-changing voice
  upload routes live on the same path family, so a path refusal is method-blind
  here too — refuse only what you do not need.
- **Verdict:** **clean** for the wire; messy only in that it is a container and
  a Python stack (Apache-2.0 server and weights).

### 1.5 MCP over Streamable HTTP, native and behind `mcp-proxy` — **native clean; stdio bridge messy on identity**

The stand-in server is `data/src/mcp_echo.py` (official `mcp` 1.30.0), lent
first natively and then as a stdio child behind `mcp-proxy`; the client is
`data/src/mcp_client.py` through the lend's origin
(`data/responses/mcp/piece.json`).

- **Native Streamable HTTP.** `initialize` answers `scope-echo`; `tools/list`
  answers `whoami`, `add` (both `readOnlyHint:true`) and `erase`
  (`destructiveHint:true`); annotations are present
  (`data/responses/mcp/native-through-door.txt`).
- **The door's headers arrive.** The `whoami` tool received `x-svrngs-person:
  Bo`, `x-svrngs-user: Bo-9307c56b78082b74`, `x-svrngs-device`,
  `x-svrngs-app`, `x-svrngs-label`, `x-svrngs-pages`
  (`data/responses/mcp/native-through-door.txt`). A native Streamable HTTP tool
  can act as the person.
- **stdio behind stock `mcp-proxy`.** Protocol is clean: `initialize` and
  `tools/list` answer the same tools (`data/responses/mcp/stdio-through-door.txt`).
  Identity is **lost**: the child's `whoami` returned `{"env":{}, "headers":{}}`
  — the proxy terminates HTTP and forwards no header and no `HTTP_*` env. A
  stdio tool behind the stock bridge cannot know Bo. The bridge can be given an
  explicit mapping, but not by default.
- **Keep-backs.** Neither has an admin route; the proxy's own process is the
  lender's.
- **Verdict:** native **clean**; stdio bridge **messy on identity**, clean on
  protocol.

### 1.6 The interface a person runs — **Open WebUI v0.12.3, clean with two caveats**

Candidates were judged for licence and shape at the pinned tag
(`data/responses/interface/licence.txt` is Open WebUI's whole licence):

- **Open WebUI** — BSD-3 plus clause 4: the "Open WebUI" name and logo may not
  be altered or removed above **50** end users in a rolling 30 days without
  written permission. Below 50, or with permission, a person may run and brand
  it. Measured.
- **LibreChat** — MIT, the best feature fit, but a Node + MongoDB +
  Meilisearch stack; **could-not-judge** inside this run's budget, not measured.
- **AnythingLLM** — MIT; judged from its docs as no one-address STT/TTS plus
  Streamable HTTP MCP, so **out** for this shape; not measured.
- **Jan** — Apache-2.0; a desktop app, not an address, so **out**; not measured.

**The measurement.** One stand-in OpenAI address (`data/src/standin.py`) on
`host.containers.internal:8111`; Open WebUI `v0.12.3` pinned by digest
`sha256:bd153a1f…60887`; the first user signed up, then the API was used
(`data/responses/interface/piece.json`).

- **Settings.** Chat, speech-to-text and speech are **three separate settings**,
  all pointed at the one address:
  `OPENAI_API_BASE_URL`, `AUDIO_STT_OPENAI_API_BASE_URL`,
  `AUDIO_TTS_OPENAI_API_BASE_URL` (`piece.json` → `settings`). So the answer
  is: yes, it can use one address for all three, but not by one field.
- **Chat by model id.** `GET /api/models` listed `ana/coder`, `sam/stt`,
  `sam/voice` from the stand-in (`models.json`); `POST /api/chat/completions`
  reached the stand-in with `"model":"ana/coder"` and answered
  (`chat.json`, `data/responses/standin/requests.jsonl`).
- **Transcription.** `POST /api/v1/audio/transcriptions` reached the stand-in
  and answered `{"text":"stand-in heard the quick brown fox"}` (`transcription.json`).
- **Speech.** `POST /api/v1/audio/speech` reached the stand-in and returned
  audio (`speech.json`), **but the interface sent `"model":"tts-1"`, not the
  `sam/voice` asked for** (`requests.jsonl`). It has a separate TTS model
  setting; by default the picker's model is not the TTS model. One setting to
  add (`AUDIO_TTS_MODEL`) closes it, but as shipped it is a routing surprise.
- **MCP.** The pinned image has a native Streamable HTTP client:
  `open_webui/utils/mcp/client.py` imports `mcp.client.streamable_http` and
  connects with `streamablehttp_client(url, …)`
  (`data/responses/interface/mcp-client.py`). It can take an MCP Streamable
  HTTP address; no proxy is needed.
- **Verdict:** **clean** for a small group, with two caveats (three settings;
  the TTS model is rewritten) and the branding clause, which is the operator's
  call.

### 1.7 The bridge's quality — **not shippable as a promise at 3B + 7B**

Three screenshots of the run's own home page (`data/ui/index.html`) at three
widths, each with one planted defect; the answers are whole in
`data/responses/bridge/{clip,overlap,table}-{vision,coder}.json`. Rubric,
stated in `bridge/piece.json`: **(1) the bug found** — the vision answer names
the planted element and its symptom; **(2) the fix runs** — the coder answer
gives a concrete change for that element, whose selector exists in the page.

| Page | Planted defect | Vision named it? | Coder fix runs? |
|---|---|---|---|
| clip, 480 | the "Subscription required" badge is clipped (86px, `overflow:hidden`) | **No** — it never mentions the badge; says "no visible visual defects" | **No** — says no defect, proposes nothing |
| overlap, 760 | the primary button (`left:26px`) overlaps its neighbour | **No** — invents section overlap and top clipping | **No** — `.main-content` / `.first-section` do not exist in the page |
| table, 1024 | the Status column is cut off (`width:620px;overflow:hidden` around a 900px table) | **Partly** — calls the table "not properly formatted" and reads only three columns, but never names the missing Status column | **No** — `.first-section .description` does not exist; the "table borders" change does not restore the column |

The coder's selectors (`.main-content`, `.first-section`,
`.first-section .description`, `.open-room`) do not appear in
`data/ui/index.html`, so none of the three fixes can run against the page as
written. The vision model read the header and table correctly but missed both
the clipped badge and the missing column, and hallucinated defects elsewhere.
The bridge helps a coder that cannot see, but at 3B + 7B it is **not reliable
enough to ship as a promise**. It can ship as the worked example, with the
limits said plainly.

## 2. Each piece, and what v0 could adopt

| Piece | Verdict | Why, in a line | Adopt? |
|---|---|---|---|
| `llama-server` single model | clean for chat/vision, messy for tools | one id, door keeps admin back; Qwen2.5-Coder emits tool calls as text | yes, for chat and vision; the tool story needs a model whose template emits `tool_calls` (not this one with `--jinja`) |
| `llama-server` router | messy, out | `/v1/models` leaks every cached model's name and path; `/models` is both list and loader | no, not as a bare lend |
| `llama-swap` | messy, usable | clean routing, but writes a filesystem path as `model` and adds a supervisor | only if several models must sit behind one lend |
| `whisper.cpp` server | out as a bare lend | no `/v1/models`, no `/v1/audio/transcriptions`; only `/inference` | no; a wrapper or another server |
| Kokoro | clean | OpenAI speech, unknown model refused, 25 KB of audio through the door | yes |
| MCP native | clean | `initialize`, `tools/list` with annotations, `x-svrngs-person` arrives | yes |
| MCP stdio + `mcp-proxy` | messy on identity | protocol clean, but the child sees no door headers | only with an explicit header mapping |
| Open WebUI v0.12.3 | clean with caveats | one address in three settings; TTS model rewritten; branding clause | operator's call |

**The app lines, estimated from what was measured.** The pantry itself is
`list` and `serve` over `/v1/lends`: read each lend's origin, ask
`/v1/models` or MCP `initialize`/`tools/list`, merge ids with a lender prefix,
route by `model` (JSON and multipart), stream back. From this run's stand-ins
and the surface's shape, `apps/pantry/` is about **400–600 lines** of Node
including its tests' helpers; the vision bridge is about **100 lines**; a
provider's recipe is **two shell lines**. No platform line was needed: every
piece above is reached through `share`, the door's headers and `--refuse`.

## 3. Could-not-judge, said with why

- **LibreChat** was not installed: its Node + MongoDB + Meilisearch stack did
  not fit this run's budget, and the row asks one interface measured, not
  four. Its licence (MIT) and documented features were read; no live answer is
  claimed.
- **AnythingLLM and Jan** were judged from their shape and docs, not measured;
  the row asks one interface measured.
- **The bridge's "fix runs"** was judged by checking the coder's selectors
  against `data/ui/index.html`, not by re-rendering a patched page: no
  proposed selector exists, so there is nothing to apply. The answers are kept
  whole, so the judgement can be checked.
- **`llama-server` router's keep-back** cannot be made clean with the door as
  it is: `/models` is method-blind. That is a finding, not a failed install.

## 4. What the operator must decide

`NEEDS_HUMAN.md` asks it in one place: what v0 adopts from the table above,
and whether the bridge example ships. Nothing outside `demo/runs/models/` was
changed by this row.
