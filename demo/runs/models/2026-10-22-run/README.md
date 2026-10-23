# Your friends' local AI, in the app you already use — one host, 2026-10-22

Row `models-run` (`ralph/next/models/STATE.md`), part (a). `PREDICTIONS.md`
was committed before the first server (`d48e5ce3`, corrected by `7a2fbd88`
and `4c6316d1` before the kept take). One host, one run of each: not a rate
(AGENTS.md 7). Every answer is under `data/responses/`; every command is
under `data/commands/`; the driver is `drive.py` and the stock clients are
`story.py`. The group is `46fcfc6c…` (`data/responses/setup/group.txt`), one
of five programs: ana, sam, bo, cy, dee, each with a HOME and an `SVRNGS_DIR`
of its own under `data/home/` (ignored).

Host: macOS 27.0.0, arm64, Apple M2 Max; `llama-server` Homebrew 10180
(`11b068d06`); `whisper-server` 1.9.5; Podman 6.0.1; Node 20.20.2
(`data/responses/versions.json`). No weight was downloaded: every curated
file was already under `target/models/curated/` and its pinned sha256 was
checked before it was served (`versions.json`, `pins`). The whisper model is
`ggml-base.en.bin` sha256 `a03779c8…6d002`; llama-swap is the scope's pinned
binary sha256 `5bd81982…5a65`; Kokoro is
`ghcr.io/remsky/kokoro-fastapi-cpu:v0.9.0`.

## The bars

**C1 — discovery by protocol, nothing registered: passed.**
`pantry list` (`data/responses/story/list.txt`) names six lends, each with
whose computer and its address, and none was registered:

    ana's computer — … — model ana/coder
    ana's computer — … — model ana/vision
    cy's computer  — … — model cy/cy-model
    sam's computer — … — model sam/whisper-1
    sam's computer — … — tool whoami (read-only), erase (destructive)
    sam's computer — … — model sam/tts-1, sam/tts-1-hd, sam/kokoro, sam/gpt-4o-mini-tts

**C2 — one address, routed by `model`, streamed: passed.**
Bo's `pantry serve` answered at `http://127.0.0.1:8280/v1`
(`data/responses/story/address.txt`). The stock OpenAI SDK's streamed chat
for `ana/coder` answered `"ok"` in 3 chunks
(`story/c2-chat.json`); the multipart transcription for `sam/whisper-1`
answered `"The quick brown fox jumps over the lazy dog."`
(`story/c2-transcribe.json`); speech for `sam/kokoro` returned an mp3 of
25,004 bytes, sha256 `92f112f7…68de0` (`story/c2-speech.json`). Each lend
received its own id, not the prefixed one. After `pantry serve` was stopped
and started again with no `--port`, the same base URL answered `"again"`
(`story/c2-after-restart.json`), and the address it kept was 8280.

**C3 — extension needs no pantry change: passed.**
Cy's model, through llama-swap, a server no pantry test uses, answered
`"cy"` for `cy/cy-model` (`story/c3-cy.json`). Dee's adapter (`dee.mjs`, 46
lines) answered `"I am not an adapter."` through Bo's pantry
(`story/c3-dee.json`). `apps/pantry/pantry.mjs` and `mcp.mjs` have the same
sha256 before and after (`versions.json`, `pantry`; `story/pantry-hashes.json`).

**C4 — composition stays as the person: passed.**
Bo's bridge sent a 480-wide screenshot of a clipped page (`data/ui/clip.png`,
sha256 `8e55b9f2…47ab2`) to `ana/coder`, which cannot see; `ana/vision`
described it through Bo's pantry; the coder answered with a diagnosis
(`story/c4-bridge.json`, 200). The answer names both hops:
`x-pantry-model: ana/coder`, `x-pantry-bridged-by: ana/vision`. Ana's own
door log names `bo` as the person for the bridged request
(`story/c4-door.txt`), not the bridge or another name.

**C5 — tools by MCP, as the person, effects listed: passed.**
The stock MCP SDK listed `whoami` with `readOnlyHint` and `erase` with
`destructiveHint`, and called `whoami`; Sam's tool received
`"x-svrngs-person": "bo"` (`story/c5-tool.json`).

**C6 — honest failure, never substituted: passed.**
With sam's computer stopped, a request for `sam/whisper-1` was `503`:
"sam's computer did not answer (asleep), so sam/whisper-1 cannot be
answered" (`story/c6-asleep.json`), and no other lend answered it.

**C7 — the lender stays safe: the kept-back route passed; the limit is
could-not-judge on one host.**
`GET /slots` on ana's coder through the door was `403` with
`x-svrngs-refused: path` (`story/c7-slots.json`). Two people (bo through
Bo's pantry, dee at dee's own origin) asked the same coder at once, and both
were let in (`story/c7-limit.json`), because the door takes a path that ends
at a private address as the lender's home and does not count it
(`crates/svrngs-node/src/door/limit.rs`, "Who is not";
`a_viewer_on_a_private_address_is_not_counted`). Every viewer in a one-host
run is home. The limit itself is proven by the door's own tests
(`a_fourth_is_refused_naming_three`, `one_stops_and_after_the_window_the_fourth_is_let_in`),
not by this run: could-not-judge, said with why, never a pass.

**A provider's two lines: passed.** From starting `llama-server` to its
first `/v1/models` was 20,288 ms; the `svrngs share` line took 9 ms; the
pair 20,297 ms (`story/provider-two-lines.json`). The download is said
apart and was none: the weights were already under `target/models/curated/`
and each sha256 was checked.

## What was messy

The first take listed `ana/ana/coder` (the serve command's `--alias` already
carried the person) and read the speech response the old way. The second
take spent Qwen3's token budget in `reasoning_content`, and the vision
decode ran out of Metal memory beside the coder. The kept take asks the
models not to think, unloads Cy's 8B before the bridge, keeps the vision
server's layers at `-ngl 90` with `-ub 512`, and scales the screenshot to
480. The first two takes are in `data/logs/` (ignored), not kept as runs.

## Nothing left running

The driver stops every process it started and removes the Kokoro container
in its `finally` (`drive.py`, `stop`); the pid files under `data/logs/` are
gone, `podman ps` is empty, and `data/responses/story/ps-after.txt` has none
of the run's commands.
