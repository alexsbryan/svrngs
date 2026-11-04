# What still warns in a build's cargo JSON, one line each, path:line: message,
# and whether the build finished (the form of the earlier surveys' warnings files).
import json, sys
seen, done = set(), None
for l in open(sys.argv[1]):
    try: m = json.loads(l)
    except ValueError: continue
    if m.get("reason") == "build-finished": done = m["success"]
    msg = m.get("message") or {}
    if m.get("reason") != "compiler-message" or msg.get("level") != "warning" or not msg.get("spans"): continue
    s = next((s for s in msg["spans"] if s["is_primary"]), msg["spans"][0])
    seen.add(f"{s['file_name']}:{s['line_start']}: {msg['message']}")
print("\n".join(sorted(seen)))
print("build-finished success:", done)
