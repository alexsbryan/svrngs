"""Bo's Python program in the start-measure run (run.py): Python's `openai`
package with the lend's address as its `base_url`, whole and streamed, and
whether Python's own name lookup gives the address's host. One JSON line.

    <venv>/bin/python client.py <origin>"""
import json, socket, sys, time
from urllib.parse import urlsplit

import openai

origin = sys.argv[1]
u = urlsplit(origin)
out = {"origin": origin, "openai": openai.__version__}
try:
    out["resolved"] = sorted({a[4][0] for a in socket.getaddrinfo(u.hostname, u.port)})
except Exception as e:
    out["resolved"] = None; out["resolve_error"] = f"{type(e).__name__}: {e}"

client = openai.OpenAI(base_url=origin.rstrip("/") + "/v1", api_key="none", max_retries=0, timeout=60)
ask = dict(model="any", messages=[{"role": "user", "content": "Name three rivers in Spain."}], max_tokens=32,
           temperature=0, seed=1)
t0 = time.perf_counter()
try:
    r = client.chat.completions.create(**ask)
    out["whole"] = {"reached": True, "ms": round((time.perf_counter() - t0) * 1000, 1), "object": r.object,
                    "words": r.choices[0].message.content[:80]}
except Exception as e:
    out["whole"] = {"reached": False, "error": f"{type(e).__name__}: {e}"[:300],
                    "cause": repr(e.__cause__)[:300] if e.__cause__ else None}
t0 = time.perf_counter()
try:
    n = 0
    for _ in client.chat.completions.create(**ask, stream=True): n += 1
    out["streamed"] = {"reached": True, "events": n, "ms": round((time.perf_counter() - t0) * 1000, 1)}
except Exception as e:
    out["streamed"] = {"reached": False, "error": f"{type(e).__name__}: {e}"[:300],
                       "cause": repr(e.__cause__)[:300] if e.__cause__ else None}
print(json.dumps(out))
