# models-scope: the operator's choice

The row ends here: the survey is measured, the predictions stand beside it,
and v0's servers are the operator's choice (`CHARTER.md`, "Leave for the
operator"; `ralph/next/models/STATE.md`, row `models-scope`). The evidence is
`SCOPE.md` in this directory and every raw answer under `data/responses/`.
One host, one run of each: not a rate.

## Ask 1 — what v0 adopts

The measured table is `SCOPE.md` §2. In short:

- **Chat + vision:** `llama-server` with one model per lend is clean. The
  Qwen2.5-Coder-7B we measured does **not** emit OpenAI `tool_calls` under
  `--jinja`; it writes the call into `content` as `<tools>…</tools>`
  (`data/responses/llama-coder/origin-tools.json`). Adopt it for chat and
  vision, but the story's "the coder calls Sam's search tool" needs either a
  different model/template or an adapter. **Choose: keep this coder and ship
  the tool story with a wrapper, or pick a tool-calling model?**
- **Several models behind one lend:** `llama-server` router is **out** as a
  bare lend — `/v1/models` listed every cached model on this machine, with
  each one's `--hf-repo` and paths (`data/responses/router/origin-models.json`),
  and `/models` is both the listing and the loader. `llama-swap` routes
  cleanly but writes a filesystem path as `model` and adds a supervisor.
  **Choose: one lend per model (the minimum, recommended), or `llama-swap`
  for people who need several behind one address?**
- **Speech to text:** `whisper.cpp` 1.9.5's server has neither `/v1/models`
  nor `/v1/audio/transcriptions`; it only has `/inference`
  (`data/responses/whisper/direct-models.json`,
  `direct-transcription.json`, `direct-inference.json`). **Choose: ship a
  small OpenAI-in/out wrapper as a recipe (a second program on Sam's
  computer), or find/measure another open STT server that speaks the route?**
- **Speech out:** Kokoro v0.9.0 is clean (`data/responses/kokoro/`). **Adopt?
 **
- **Tools:** native MCP Streamable HTTP is clean, including `x-svrngs-person`
  arriving at the tool (`data/responses/mcp/native-through-door.txt`). A stdio
  server behind stock `mcp-proxy` loses identity (`stdio-through-door.txt`).
  **Adopt native MCP; do you want the stdio bridge in the recipes with its
  limit stated, or left out until the proxy maps headers?**
- **The interface:** Open WebUI v0.12.3 is measured and clean for a small
  group, with caveats: chat, STT and TTS use three settings pointed at one
  address, and TTS rewrites the model to `tts-1` unless `AUDIO_TTS_MODEL` is
  set (`data/responses/interface/piece.json`). Its licence adds a branding
  clause above 50 users (`data/responses/interface/licence.txt`). **Choose:
  Open WebUI as the interface a person runs, or leave the interface to the
  community?** The branding clause is the operator's call, not ours.
- **No platform line was needed** for any of the above. A platform need would
  be a separate ask; none was found.

## Ask 2 — does the vision bridge example ship?

The bridge's quality is `SCOPE.md` §1.7: three screenshots, vision alone and
the coder given the vision description, both answers kept whole. The 3B vision
model missed the clipped badge and the cut-off table column; the 7B coder
proposed selectors that do not exist in the page (`data/ui/index.html`). The
bridge helps, but at these sizes it is **not a promise**. The row
`models-bridge` closes early if the answer is no.

**Choose: ship `apps/pantry/examples/bridge/` as the worked example with these
limits said plainly, or drop it and let the community write the first
adapter?**

## Evidence, if a line is challenged

- Row: `ralph/next/models/STATE.md`, `models-scope`.
- Predictions before the first install: `PREDICTIONS.md`, commit `84f82cc8`.
- Host and pinned versions: `data/responses/versions.json`.
- Weights and their sha256: `SCOPE.md`'s pinned table; files in
  `data/weights/` (not committed).
- Every status quoted in `SCOPE.md` is a file under `data/responses/`.
- The bridge's answers and the page: `data/responses/bridge/`, `data/ui/`.
- No server was left running and nothing was lent when the row ended:
  `data/responses/stop/`.
