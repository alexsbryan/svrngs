import json, sys
d = json.load(sys.stdin)
for s, v in sorted(d.items()):
    print(f"{s:28} spans {v['spans']:5} errors {v['errors']:4} metrics {v['metrics']:4} logs {v['logs']}")
