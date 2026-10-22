# svrngs models — director charter

The supervisor's resolution session decides this queue's forks instead of
waiting on the operator. `NORTH_STAR.md` and
`docs/principles/ARCH_PRINCIPLES.md` are the compass. Where they and this
charter disagree, they win, and the decision says so.

## What the operator decided, 2026-10-22

- **The design.** svrn-docs `rings/current/MODELS_ON_THE_RING.md` (a draft
  of 2026-10-21, at svrngs `cefff1b5`), read whole. A model is a lend; what
  it can do is a card; a small app on the person's computer, the pantry,
  matches. This queue builds that design's zero-platform v0 (its §6), cut
  further by a lean pass.
- **A lean lens on priority and scope.** "I want to get this planned out and
  scheduled in rows, but with a lean startup lens applied to the product
  priority and scope."
- **Scope the design's six needs first, then choose.** "Is there a way to do
  #3 but very lean? How elaborate are the runtime differences? I'd like to
  at least try to scope out #3, but if it's too messy we can fall back to
  #1." Here #3 is the design's six needs (code, embed, image, transcribe,
  tool search, video), and #1 is LLMs alone (chat and code). The first row
  measures each runtime and stops; the operator chooses the kinds.
- **No Ollama, anywhere; fully open, high-integrity software only.** "We
  don't work with ollama. Let's cut that from any plan. Fully open and high
  integrity software only." Runtimes are the ggml family (llama.cpp,
  whisper.cpp, stable-diffusion.cpp) and community projects of the same
  standing (SearXNG, an open MCP server, ComfyUI). Containers are Podman's,
  never Docker (2026-10-20).
- **Written, held.** "Write rows, hold." The operator launches the loop.

## The lean cut, and why

The design's v0 is 40–70 platform lines and 1,000–1,600 app lines, with
needs as acts, a `card` field on the declaration, a router to come, and
three runtimes. This queue builds less:

- **No platform line.** The repository is at 17,634 of 17,635 and
  svrngs-core at 3,149 of 3,150 (`scripts/lean_check.py`, 2026-10-22).
  `apps/` and `demo/runs/` are not counted, and neither is
  `well_known.txt`. Cards are made from what the runtime itself answers,
  through the lend's origin, plus the lend's name and kind, which
  `/v1/lends` already carries (the design's §6, "The zero-platform v0").
  A runtime's routes that change the lender's computer are kept back by the
  recipe's `svrngs share --refuse`, which exists.
- **No need acts.** A need is the consumer's flags on `pantry use`. A
  standing need in the record earns its place when a person asks the group
  for a model nobody lends.
- **No router, no page, no spread.** v0 prints the lend's stable origin.
  The order is this computer's own lends, then by the lender's name.
- **The kinds the scope finds clean.** LLMs alone is the floor.

Lending a model already works without any of this: `llama-server` is a
known kind with its keep-backs (`crates/svrngs-node/src/well_known.txt`),
and a stock client reads `/v1/models` from the address it is given. What v0
adds is one command for a provider, and for a consumer, which friend has
which model and one address for it, or every candidate named with why not.

## The budget

Zero. No row raises the repository's or svrngs-core's cap. A row that finds
it needs a platform line writes `NEEDS_HUMAN.md`, saying what the line
would do and why the zero-platform path cannot.

## Decide

- The recipe's format and its home in `apps/pantry/`, and the pantry's
  command shape, within the rows.
- The smallest open model for each kind a run needs, pinned by revision.
- How each chosen runtime is installed here (Homebrew, a pinned source
  build, or a pinned Podman image), recorded.
- The canonical lend name's exact form, carrying what the runtime cannot
  say (model, quant, revision).
- The smaller reversible step.

**Bars by their spirit** (operator, 2026-10-08). Ask what a bar guards and
judge by that. Where its letter and its spirit part, rewrite the letter
yourself, saying both in the commit, and keep going.

## Leave for the operator

Write `NEEDS_HUMAN.md` and stop for:

- **the kinds:** models-scope ends here every time; the operator chooses
  which kinds v0 covers;
- **a platform line:** any change counted by `scripts/lean_check.py`;
- **the trust floor:** any widening of a page's policy or a listener;
- **a runtime or tool** that is not fully open and of high integrity, or
  that cannot be pinned and recorded; Ollama is never one;
- **what a group holds:** a new act, or a change to the record;
- **the outward-facing:** a system service, a port on this host's network,
  publishing;
- **git:** pushing, history rewrites, `--no-verify`;
- **anything whose evidence you could not reproduce.**

**Always:** one decision, one commit, with the fork, the choice, the
evidence and what would falsify it in the body.
