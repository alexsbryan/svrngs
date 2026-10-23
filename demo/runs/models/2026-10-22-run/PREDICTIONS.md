# The story on one host — predictions, 2026-10-22

Row `models-run` (`ralph/next/models/STATE.md`), part (a). Written and
committed before the first server of the run started; `README.md` beside it
says what each line met, with the answers under `data/responses/`. One host,
one run of each: not a rate (AGENTS.md 7).

## The host and what was here before the first server

- macOS 27.0.1, arm64, Apple M2 Max, 64 GiB. `llama-server` (Homebrew
  llama.cpp 10180, `11b068d06`), `whisper-server` 1.9.5, Podman 6.0.1,
  Node 20.20.2, the checkout's `target/debug/svrngs`.
- Weights already seeded under `target/models/curated/` by the operator's
  session (CHARTER.md): Qwen3.6-35B-A3B `IQ4_NL`, Gemma-4-26B-A4B-it
  `UD-Q6_K_XL` with its projector, Qwen3-8B `Q4_K_M`. Each is checked
  against its pinned sha256 before it is served; no weight is downloaded by
  this run. The whisper `ggml-base.en.bin` is the public file the scope
  pinned; the Kokoro image is `ghcr.io/remsky/kokoro-fastapi-cpu:v0.9.0`
  at digest `sha256:73940a71…cc20`.
- One group, five people, every person a program: Ana (the GPU box), Sam
  (the Mac), Bo (the thin laptop), Cy (a server the pantry's tests do not
  use), Dee (an adapter written in the run). Each has a HOME and a
  `SVRNGS_DIR` of the run's own under `data/home/`.

## The rule this run is judged by

A lend that answers OpenAI's `/v1/models` is a model; one that answers MCP's
`initialize`/`tools/list` over Streamable HTTP is a tool. The pantry asks the
protocol. Nothing is registered, and the pantry's files do not change for
any lend or adapter here (checked by sha256).

## What I expect, bar by bar

**C1 — discovery by protocol, nothing registered.** `pantry list` names
Ana's two model lends, Sam's transcription and speech lends and his tool,
and Cy's model, each with whose computer and its address; Sam's `whoami`
reads `read-only` and `erase` reads `destructive`. Dee's adapter is not a
lend: it is a server Dee runs, and the pantry never sees it. *Predicted:
clean.*

**C2 — one address, routed by `model`, streamed.** The OpenAI SDK, pointed
at Bo's `pantry serve`, answers a streamed chat for `ana/coder` from Ana's
coder, a multipart transcription for `sam/whisper-1` from Sam's whisper, and
speech for `sam/kokoro` from Sam's voice. Each lend receives its own id
(`coder`, `whisper-1`, `kokoro`), not the prefixed one. After `pantry serve`
is stopped and started again without `--port`, the same client base URL
answers. *Predicted: clean, but the two large models take tens of seconds to
load the first time.*

**C3 — extension needs no pantry change.** Cy's model is listed and answered
through the pantry though no pantry test has ever seen llama-swap; Dee's
adapter (under 50 lines) answers through the pantry. `pantry.mjs` and
`mcp.mjs` have the same sha256 before and after. *Predicted: clean.*

**C4 — composition stays as the person.** Bo runs the bridge on Bo's
computer. A screenshot sent to `ana/coder` (which cannot see) is described
by `ana/vision` through Bo's pantry, the coder answers from the description,
and Ana's door logs Bo, not the bridge or Ana. The answer names both hops in
`x-pantry-model` and `x-pantry-bridged-by`. *Predicted: clean; the
description may miss a clipped badge, as it did in the scope.*

**C5 — tools by MCP, as the person, effects listed.** Bo's stock MCP client
reaches Sam's tool address from `list`, lists `whoami` (read-only) and
`erase` (destructive), and calls `whoami`; Sam's server receives
`x-svrngs-person: Bo`. *Predicted: clean.*

**C6 — honest failure, never substituted.** With Sam's computer asleep, a
request naming `sam/whisper-1` is a 503 naming Sam's lend and why, and no
other lend answers it. *Predicted: clean, after the record's 3 s and the
pantry's 4 s discovery window.*

**C7 — the lender stays safe.** A kept-back route (`/slots` on Ana's coder)
is 403 through the door. Two speech requests at once over Sam's voice, shared
with `--limit 1`, give one answer and one `x-svrngs-refused` refusal from the
door. *Predicted: clean.*

**The provider's two lines.** Starting the code model and sharing it, timed
from the share command to its first `/v1/models` through the lend, is under a
minute after the weights are present; the weights' download is said apart
and is zero here. *Predicted: clean.*

## Things I expect to be messy

- The first chat to each large model pays its load (the scope measured 2–8 s
  from the seeded files; this run holds two large models at once, so memory
  pressure could make it slower).
- Kokoro's container takes time to become ready; the run waits up to 5 min.
- A request over the limit must overlap the first request to be refused; the
  run sends a long input twice at once and keeps whichever verdict the door
  gives, saying could-not-judge if they do not overlap.
