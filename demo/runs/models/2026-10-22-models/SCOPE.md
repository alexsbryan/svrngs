# The curated pair, measured on one host — 2026-10-22

Row `models-pantry` (`ralph/next/models/STATE.md`), part (0). Predictions
were committed first (`PREDICTIONS.md`, `f624ca7f`), before the first server
started. One host, one run of each: not a rate (AGENTS.md 7).

The shortlist and its revisions are the operator's (CHARTER.md, "What the
operator decided at models-scope"). Each file was checked against its pinned
sha256 before it was served; the revision is the HuggingFace commit whose
tree carries that sha256. The raw answers are under `data/responses/<key>/`;
the driver is `drive.py`.

## Pinned and recorded

| Model | Role | Public repo @ revision | File | sha256 |
|---|---|---|---|---|
| Qwen3.6-35B-A3B | code | `unsloth/Qwen3.6-35B-A3B-MTP-GGUF@5bc3e238` | `Qwen3.6-35B-A3B-UD-IQ4_NL.gguf` | `d0ad7804…eae5b` |
| Qwen3-8B | code, small | `unsloth/Qwen3-8B-GGUF@a6adef13` | `Qwen3-8B-Q4_K_M.gguf` | `120307ba…49bd4` |
| Gemma-4-E4B-it | vision, small | `unsloth/gemma-4-E4B-it-GGUF@ce152932` | `gemma-4-E4B-it-Q5_K_M.gguf` + `mmproj-F16.gguf` | `8022e25d…be510`; projector `4b3bf6e1…26f67` |
| Gemma-4-26B-A4B-it | vision | `unsloth/gemma-4-26B-A4B-it-GGUF@c462057f` | `gemma-4-26B-A4B-it-UD-Q6_K_XL.gguf` + `mmproj-F16.gguf` | `5cfb7ab4…82a4`; projector `418a6d87…5a42f` |

The operator's charter names the Qwen file `Qwen3.6-35B-A3B-UD-MTP-IQ4_NL.gguf`;
the public tree at the pinned revision carries the same bytes as
`Qwen3.6-35B-A3B-UD-IQ4_NL.gguf` (no `MTP` in the file name). The local copy
is the seeded one, under the charter's name, and its sha256 is the pinned
one. Anyone pulls the public name above.

Host: macOS 27.0.1, arm64, Apple M2 Max, 64 GiB; `llama-server` Homebrew
10180 (`11b068d06`). See `data/responses/versions.json`.

## The serve command

    llama-server -m target/models/curated/<file> --jinja --alias <id> \
      --host 127.0.0.1 --port <port> -c 8192 -ngl 99
      # vision adds: --mmproj target/models/curated/<projector>.gguf

Each was lent in the run's own group with `svrngs share <port> --kind
llama-server --name <key>`, then asked through the lend's origin. The lend's
own address is in `data/responses/<key>/piece.json`; it is not repeated here
because it changes with the run.

## What each answered

| Model | `/v1/models` | chat through the lend | tools through the lend | vision through the lend |
|---|---|---|---|---|
| Qwen3.6-35B-A3B | one id, the `--alias` | `ok` (`qwen36-35b/origin-chat.json`) | **one OpenAI `tool_calls`** (`qwen36-35b/origin-tools.json`) | — |
| Qwen3-8B | one id, the `--alias` | `ok` (`qwen3-8b/origin-chat.json`) | **one OpenAI `tool_calls`** (`qwen3-8b/origin-tools.json`) | — |
| Gemma-4-E4B-it | one id, the `--alias` | `ok` (`gemma4-e4b/origin-chat.json`) | — | **`Red`** (`gemma4-e4b/origin-vision.json`) |
| Gemma-4-26B-A4B-it | one id, the `--alias` | `ok` (`gemma4-26b/origin-chat.json`) | — | **`Red`** (`gemma4-26b/origin-vision.json`) |

The tool request asked for Paris's weather with a `weather(city)` function.
Both coders answered `message.tool_calls: [{function: {name: "weather",
arguments: "{\"city\":\"Paris\"}"}}]` with empty `content`, which is the
OpenAI shape the scope's Qwen2.5-Coder did not produce. The vision request
sent a 64×64 red square as an `image_url` data URL and asked for one word;
both Gemmas answered `Red` at `finish_reason: stop`. The image is made in
`drive.py` (`png_square`), so the run needs no image library.

Load, generation and answer are in each `piece.json` and response's
`timings`; on this host load was 2–8 s from the seeded files, and the MoE
models answered in tens of tokens a second. Every ask finished; nothing was
could-not-judge.

## The pair

**The strongest pair is Qwen3.6-35B-A3B to code and Gemma-4-26B-A4B-it to
see.** They are the largest curated coder and the largest curated vision
model, both emit the shapes the story needs, and their weights plus the
26B's projector come to about 43.0 GB. With whisper.cpp's `ggml-base.en.bin`
(≈0.15 GB) and Kokoro's container (a few GB) the run stays under this host's
64 GB. The smaller pair, Qwen3-8B with Gemma-4-E4B-it, also passes every ask
and is the fallback for a smaller host; it is not v0's choice where the 64 GB
host can hold the strong pair.

The pantry is told none of these names: each was discovered because it
answered `/v1/models`, and each was asked through a lend with no pantry code.
