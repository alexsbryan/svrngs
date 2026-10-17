"""Bo's program in the story (run.py): Python's `openai` package given, as
its `base_url`, the address the home page copied, nothing added to it.
With `first`, as a newcomer would first write it, with no `api_key`, then
with one; with `once`, with one alone. A chat streamed each time it is
let ask. One JSON line. start-measure's client.py (../2026-10-13/), changed
in the address it is given and the key.

    <venv>/bin/python program.py <address> <first|once>"""
import json, os, sys, time

import openai

address, how = sys.argv[1], sys.argv[2]
os.environ.pop("OPENAI_API_KEY", None); os.environ.pop("OPENAI_BASE_URL", None)
out = {"address": address, "openai": openai.__version__}
ask = dict(model="any", messages=[{"role": "user", "content": "Name three rivers in Spain."}], max_tokens=48,
           temperature=0, seed=1)


def why(e): return {"error": f"{type(e).__name__}: {e}"[:400], "status": getattr(e, "status_code", None)}


if how == "first":
    try:
        openai.OpenAI(base_url=address, max_retries=0, timeout=60)
        out["no_key"] = {"made": True}
    except Exception as e:
        out["no_key"] = {"made": False, **why(e)}
client = openai.OpenAI(base_url=address, api_key="none", max_retries=0, timeout=60)
t0 = time.perf_counter()
try:
    n, first, said = 0, None, []
    for chunk in client.chat.completions.create(**ask, stream=True):
        n += 1
        if chunk.choices and chunk.choices[0].delta.content:
            first = first or round((time.perf_counter() - t0) * 1000, 1)
            said.append(chunk.choices[0].delta.content)
    out["streamed"] = {"reached": True, "chunks": n, "first_ms": first, "ms": round((time.perf_counter() - t0) * 1000, 1),
                       "words": "".join(said)[:200]}
except Exception as e:
    out["streamed"] = {"reached": False, "ms": round((time.perf_counter() - t0) * 1000, 1), **why(e)}
print(json.dumps(out))
