# models-pantry (0): the curated pair — predictions before data

Row `models-pantry` (`ralph/next/models/STATE.md`), part (0). These are
written and committed before any of the four curated models is served. One
host, one run of each: not a rate (AGENTS.md 7).

The four files are the operator's curated shortlist (CHARTER.md, "What the
operator decided at models-scope"). The local copies under
`target/models/curated/` are the seeded ones; each is checked against its
pinned sha256 before it is served, and each revision is the HuggingFace
commit whose tree carries that sha256.

## What I expect

1. **Both coders emit OpenAI `tool_calls` under `llama-server --jinja`.** The
   scope found Qwen2.5-Coder-7B writing the call into `content` as
   `<tools>…</tools>` (SCOPE.md §1.1). Qwen3.6-35B-A3B and Qwen3-8B ship
   Qwen3's chat template, which has a native tool-call branch; `--jinja`
   should therefore produce `message.tool_calls`.
2. **Both Gemma 4s see through their `mmproj`.** A red square should be named
   red, at either size, when the projector is served with `--mmproj`.
3. **The strongest pair that fits is Qwen3.6-35B-A3B (coder) with
   Gemma-4-26B-A4B (vision).** Their weights are 18.5 GB and 23.3 GB; with a
   projector (1.2 GB) and the speech pieces (whisper base ~0.15 GB, Kokoro a
   container of a few GB) the total is under this host's 64 GB. The MoE
   coders run on a few billion active parameters, so both should answer in
   under a minute per ask.
4. **The smaller pair also works, and is the fallback if the strong one does
   not fit in practice:** Qwen3-8B (5.0 GB) with Gemma-4-E4B (5.5 GB) plus
   its 1.0 GB projector.

## What would falsify it

- A coder that answers tools as text inside `content`, as Qwen2.5 did.
- A Gemma that cannot name the square, or a projector that `llama-server`
  refuses.
- A load that does not fit, or an ask that does not finish in this run's
  budget; that is could-not-judge with the command and its output kept, never
  silently replaced by the smaller pair.
