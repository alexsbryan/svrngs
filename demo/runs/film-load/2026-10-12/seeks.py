# seeks.py <data dir>: each film run's play and seek times, as the test
# printed them ("the runtime's run: {...}"), one line per engine per run:
#   label run engine first seek1 seek2 seek3 replayed   (ms; - never played)
import json, pathlib, sys

for out in sorted(pathlib.Path(sys.argv[1]).glob('*/*/test.out'), key=lambda p: (p.parts[-3], int(p.parts[-2]))):
    for line in out.read_text(errors='replace').splitlines():
        if not line.startswith("the runtime's run: "):
            continue
        r = json.loads(line.split(': ', 1)[1])
        ms = lambda x: str(x.get('ms')) if x and x.get('ms') is not None else '-'
        engine = 'chromium' if r.get('runs') else 'webkit'
        print(out.parts[-3], out.parts[-2], engine, ms(r['first']), *[ms(s) for s in r['seeks']], ms(r.get('replayed')), sep='\t')
