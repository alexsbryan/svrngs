#!/usr/bin/env python3
"""tally.py <round dir>...: per-test verdicts across rounds, from each step's
log (cargo's `test X ... ok|FAILED`, prefixed by its binary), node's TAP, and
check.sh's scene lines. Prints rows of test whose verdicts differ or failed."""
import re, sys, os, collections, json
res = collections.defaultdict(dict)  # test -> round -> verdict
load = {}
for rd in sys.argv[1:]:
    r = os.path.basename(rd.rstrip('/'))
    for f in os.listdir(rd):
        if not f.endswith('.log') or f in ('build.log', 'npm.log'): continue
        step = f[:-4]; binary = ''
        for line in open(os.path.join(rd, f), errors='replace'):
            m = re.match(r'\s*Running (?:unittests )?(\S+)', line)
            if m: binary = os.path.basename(m.group(1)).replace('.rs', ''); continue
            m = re.match(r'test (\S+) \.\.\. (ok|FAILED|ignored)', line)
            if m and m.group(2) != 'ignored':
                res[f'{step}|{binary}|{m.group(1)}'][r] = 'ok' if m.group(2) == 'ok' else 'FAIL'
            m = re.match(r'(not ok|ok) \d+ - (.*)', line)
            if m: res[f'{step}|node|{m.group(2)}'][r] = 'ok' if m.group(1) == 'ok' else 'FAIL'
    cj = os.path.join(rd, 'could-not-judge', 'browser.txt')
    if os.path.exists(cj):
        for line in open(cj): res['browser|could-not-judge|' + line.split()[0]][r] = 'CNJ'
    for d in ('scenes',):
        p = os.path.join(rd, d)
        if os.path.isdir(p):
            for s in os.listdir(p):
                for k in ('never-ran', 'could-not-judge'):
                    if os.path.exists(os.path.join(p, s, k + '.txt')):
                        res[f'scenes|{k}|{s}'][r] = k
    lf = os.path.join(rd, 'load.txt')
    if os.path.exists(lf):
        ls = [l.split() for l in open(lf) if len(l.split()) >= 6]
        load[r] = (max(float(l[1]) for l in ls), min(int(l[5]) for l in ls) // 1024) if ls else None
rounds = [os.path.basename(x.rstrip('/')) for x in sys.argv[1:]]
print('load (peak 1-min, min MemAvailable MB):', load)
total = 0
for t in sorted(res):
    v = res[t]; total += 1
    if all(x == 'ok' for x in v.values()) : continue
    print(f"{sum(x=='FAIL' for x in v.values())}/{len(v)}\t{t}\t" + ' '.join(v.get(r, '-') for r in rounds))
print('tests seen:', total)
