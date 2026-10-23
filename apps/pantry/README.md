# The pantry

Your friends lend models and tools on their own computers. The pantry is the
one place on yours that shows them, and the one address your interface talks
to.

It has no API of its own. A server that answers OpenAI's `/v1/models` is a
model. A server that answers MCP's `initialize` over Streamable HTTP is a
tool. There is nothing to register, no card and no matcher: the pantry asks
each lend its protocol.

## Run it

    node apps/pantry/pantry.mjs list
    node apps/pantry/pantry.mjs serve

`list` shows what every group you hold lends now: each model with its
`<lender>/<model>` id, each tool with its read-only or destructive
annotation, and each person whose computer did not answer. `serve` prints
one address; keep it running.

The pantry reads this computer's record at `$SVRNGS_DIR/api`, else
`~/.svrngs/api`. `--record URL` names another record, and `--port N` names
a port (the default is any free one). It listens on 127.0.0.1 only, so it is
yours, not your network's.

## Point your interface at it

In any OpenAI-compatible interface, set the OpenAI base URL to the address
`serve` printed, with `/v1` (for example `http://127.0.0.1:8787/v1`). Your
interface's model picker is the matcher: it lists `ana/coder`, `sam/voice`
and the rest, and a request goes to the friend whose id it names.

Some interfaces keep speech-to-text and speech in settings of their own.
Point those at the same address; if they ask for a model, name the
`sam/stt` or `sam/voice` id you see in `list`.

For a friend's tool, open your interface's MCP settings and paste their
tool's address from `list`, with `/mcp` on the end
(`http://…/mcp`). The tool runs on their computer as them, not as you.

## Recipes: two shell lines per server

Each row serves on 127.0.0.1, then lends the port. The `--refuse` paths are
the routes that change the lender's computer; the door answers them 403.
Add a row with a pull request, not a pantry change.

| What | Serve on your computer | Lent to your groups |
|---|---|---|
| Chat and vision (the curated pair) | `llama-server -m Qwen3.6-35B-A3B-UD-IQ4_NL.gguf --jinja --alias ana/coder --host 127.0.0.1 --port 8101 -c 8192 -ngl 99`<br>(vision: add `--mmproj mmproj-F16.gguf`, the Gemma-4-26B projector) | `svrngs share 8101 --kind llama-server --name ana-coder --refuse /slots --refuse /v1/stream --refuse /v1/streams --refuse /v1/chat/completions/control --refuse /lora-adapters --refuse /metrics --refuse /tools --refuse /cors-proxy --refuse /models` |
| Speech out (Kokoro v0.9.0) | `podman run --rm -p 127.0.0.1:8107:8880 ghcr.io/remsky/kokoro-fastapi-cpu:v0.9.0` | `svrngs share 8107 --kind kokoro --name sam-voice --refuse /docs --refuse /openapi.json` |
| Speech to text (whisper.cpp + wrapper) | `whisper-server -m ggml-base.en.bin --host 127.0.0.1 --port 8106 -l en` then `node apps/pantry/examples/whisper/server.mjs --upstream http://127.0.0.1:8106 --port 8110` | `svrngs share 8110 --kind whisper --name sam-stt` |
| Tools (any native MCP Streamable HTTP server) | serve it on `127.0.0.1:8108` with its endpoint at `/mcp` | `svrngs share 8108 --kind mcp --name sam-tools` |

The curated pair is `unsloth/Qwen3.6-35B-A3B-MTP-GGUF` and
`unsloth/gemma-4-26B-A4B-it-GGUF`; the public file, revision and sha256 are
in `demo/runs/models/2026-10-22-models/SCOPE.md`. Pull the same bytes before
you serve them. The full measured recipe, with every refused route, is there
and in `demo/runs/models/2026-10-22-scope/SCOPE.md`.

A stdio MCP server goes behind a bridge to Streamable HTTP, but a stock
bridge passes none of the door's headers: the tool cannot tell who is
calling. Use a native Streamable HTTP server when the tool must act per
person.

## To add something

| You want to lend | Do this |
|---|---|
| A model, of any runtime or modality | Run any server that answers OpenAI's `/v1/models` and `/v1/chat/completions` (or transcription, speech, embeddings, images) on 127.0.0.1, then `svrngs share <port> --kind <anything> --name <name>`. The pantry asks it; nobody adds it to a list. |
| A tool | Run any MCP Streamable HTTP server on 127.0.0.1, then `svrngs share <port> --kind <anything> --name <name>`. `list` shows its tools and annotations. |
| Behaviour: vision bridging, a voice pipeline, retrieval, guardrails | Write a small OpenAI-in, OpenAI-out server that calls your own pantry's address, and point your interface at it. The vision bridge in `examples/bridge/` is the worked example. |

## The trust lines

A lent tool acts on the lender's computer with the lender's permissions.
The door decides who may reach it and how many at once; it does not decide
what the tool does. Read a tool's annotation before you let a model call it.

A friend's tool output enters your model's context, and a friend's model
sees what you send it. Send what you are willing to share with that person.

The pantry names an asleep, full or unknown lender and why. It never
quietly puts another model in its place. When a lend does not answer,
`list` names that person and the reason.
