# svrngs models — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so.

## What the operator decided, 2026-10-22

- **The design.** svrn-docs `rings/current/MODELS_ON_THE_RING.md` (a draft
  of 2026-10-21, at svrngs `cefff1b5`), read whole. A model is a lend; an
  app on the person's computer, the pantry, reaches it. This queue builds
  far less than that design (below).
- **A lean lens.** "with a lean startup lens applied to the product
  priority and scope".
- **The main point is the community's.** "What is the absolute minimum we
  can offer that makes it near obvious for the community to extend and add
  what they want near trivially to the architecture? That's the main point
  for this." Then, of the answer below: "Ok that's pretty good", and "yes,
  rewrite the queue with that, still held".
- **Two jobs, and tools.** These are the story the minimum is tested
  against:
  - "take screenshot of UI, send it to local model, local model does code
    to fix (the problem is some very strong local models don't have built
    in multimodal so this would have to be quietly orchaestrated)";
  - "the person who does a voice to text I/O to their model interface";
  - of tools, "opening as much of the open source world as possible for
    people, but just through the right protocol adoption and simple
    machinery".
- **Scope first.** "I'd like to at least try to scope out #3, but if it's
  too messy we can fall back to #1". The first row measures and stops.
- **No Ollama, anywhere; fully open, high-integrity software only.** "We
  don't work with ollama. Let's cut that from any plan. Fully open and high
  integrity software only." The runtimes are the ggml family (llama.cpp,
  whisper.cpp) and community projects of the same standing. Containers are
  Podman's, never Docker (2026-10-20).
- **Written, held.** The operator launches the loop.

## What the operator decided at models-scope, 2026-10-22

Asked by the scope (`demo/runs/models/2026-10-22-scope/NEEDS_HUMAN.md`,
068d0439):
- **The models are the operator's own.** "Please tell me we're leveraging
  the models here /Users/alexsbryan/dev/commonwealth-ai/sovereign/models".
  The scope's Qwen2.5-Coder-7B and its 3B vision model are not v0's.
  - Six candidates are hard-linked under `target/models/sovereign/` in this
    tree, sharing their bytes, so fence 11 holds and nothing was copied:
    `Qwen3.8-27B-UD-Q6_K_XL`, `Qwen3.6-35B-A3B-UD-MTP-IQ4_NL`,
    `Qwopus3.5-9B-Coder-MTP-Q6_K` and `Qwen3-8B-Q4_K_M` to code;
    `gemma-4-26B-A4B-it-UD-Q6_K_XL` and `gemma-4-E4B-it-Q5_K_M` to see.
  - A Gemma 4's vision projector (`mmproj`) is fetched into `LLAMA_CACHE`
    from its own HuggingFace repository.
  - The host has 64 GB. Pull another model only when none of these does the
    job, and say why.
- **Speech to text: a small wrapper recipe** ("A small wrapper recipe").
  - It is an OpenAI-in, OpenAI-out server of about 40 lines, in front of
    whisper.cpp's server, answering `/v1/models` and
    `/v1/audio/transcriptions`. Try whisper-server's `--inference-path`
    first.
  - It is lent as the transcription model, and lives with the recipes. It is
    the extension pattern on the provider's side, and the pantry learns
    nothing of it.
- **The interface is the community's** ("Leave it to the community"). The
  README says how to point any OpenAI-compatible interface at the address,
  and names none as ours. The runs use stock SDK clients.
- **The bridge ships, its limits stated** ("Ship it, limits stated").
- **Shown with the ask as recommendations, and not challenged:**
  - one lend per model, with llama-swap waiting in STATE.md, since llama.cpp's
    router mode is out;
  - Kokoro v0.9.0, pinned by digest, for speech;
  - native MCP over Streamable HTTP for tools;
  - the stdio bridge left out of the recipes and named in the README with
    its limit: a stdio tool behind `mcp-proxy` cannot tell who is calling
    (`data/responses/mcp/stdio-through-door.txt`).

## The minimum: two protocols, discovery, one address

The pantry has no API of its own. Everything beyond it is a server that
speaks a protocol the open-source world already writes all day.

1. **A convention, not a registry.** A lend that answers OpenAI's
   `/v1/models` is a model. A lend that answers MCP's `initialize` over
   Streamable HTTP is a tool. The pantry knows a lend by asking the
   protocol. There are no kinds to register, no card, no facts and no
   matcher.
2. **`pantry list`.** Every model and tool your groups lend now, with whose
   computer and the address to paste. For a tool, its tools and whether each
   is read-only or destructive come from MCP's own `tools/list`
   annotations.
3. **`pantry serve`.** One OpenAI-compatible address on 127.0.0.1. It merges
   every model lend's `/v1/models`, each id prefixed by its lender
   (`ana/qwen3-coder`). It routes each request by its `model` field, written
   back to the lend's own id, and passes everything else through, streamed.
   - Every OpenAI endpoint carries `model`: chat, transcription, speech,
     embeddings and images. So one rule covers them all, with no code per
     kind. A transcription is multipart, so its `model` is read from the
     form.
   - The person's own interface's model picker is the matcher.
   - A request for a model whose lender did not answer is an error naming
     the lend and why: asleep, full or unknown. No other model is ever put
     in its place.

**How the community extends it, with nothing to learn:**
- A model, runtime or modality: any OpenAI-compatible server, lent.
- A tool: any MCP server, lent. A stdio server goes behind an open bridge
  to Streamable HTTP.
- Behaviour (bridging, voice pipelines, retrieval, guardrails, prompt
  packs): a small OpenAI-in, OpenAI-out server that calls the pantry's
  address. You run it on your own computer and point your interface at it.

**The vision bridge is the worked example of that third kind**, in
`apps/pantry/examples/`, not a pantry feature. It runs on the consumer's
own computer, so every friend's lend is reached as that person, and the
lender's door counts them. A pipeline lent to others would reach Sam's lend
as Ana on Bo's behalf, which is the line DESIGN §3 "The room" draws: no lend
carried for anyone else.

**A provider's recipe is two shell lines**: serve on 127.0.0.1, then
`svrngs share <port> --kind <server> --name <name> --refuse <path>...`. The
recipes are a table in the README that the community adds rows to. There is
no `pantry up`.

## The budget

Zero. The repository is at 17,634 of 17,635 and svrngs-core at 3,149 of
3,150 (`scripts/lean_check.py`, 2026-10-22). `apps/` and `demo/runs/` are
not counted, and the pantry needs nothing from svrngs that isn't there
already: `/v1/lends`, a lend's origin, the door's headers, and `share`'s
`--refuse`. A row that finds it needs a platform line writes
`NEEDS_HUMAN.md`, saying what the line would do and why the zero-platform
path cannot.

## Decide

- The pantry's language and layout within `apps/` (an existing Node app
  with tests is `apps/grapevine/`), and its exact commands and output.
- The prefix's exact form, and how a lend's own id is written back.
- The smallest open model for each part of the story, pinned by revision.
- How each server is installed here (Homebrew, a pinned source build, or a
  pinned Podman image), recorded.
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08). Ask what a bar guards and
judge by that. Where its letter and its spirit part, rewrite the letter
yourself, saying both in the commit, and keep going.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- **what v0 adopts:** models-scope ends here every time. The operator
  chooses the servers for the story (the front, the speech server, the MCP
  bridge, the interface a person runs), and whether the bridge example
  ships;
- **a platform line:** any change counted by `scripts/lean_check.py`;
- **the trust floor:** any widening of a page's policy or a listener;
- **a server, tool or interface** that is not fully open and of high
  integrity, or that cannot be pinned and recorded. Ollama is never one. An
  interface whose licence restricts its use is the operator's call;
- **what a group holds:** a new act, or a change to the record;
- **the outward-facing:** a system service, a port on this host's network,
  publishing;
- **git:** pushing, history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the
evidence and what would falsify it in the body.
