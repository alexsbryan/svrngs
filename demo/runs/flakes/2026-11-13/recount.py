# The share, recounted from the binaries: every test each gate target lists,
# keyed as docs/seams.tsv keys it, classified by its row's seams.
import json, subprocess, sys, csv, collections, os
wt, tsv = sys.argv[1], sys.argv[2]
env = dict(os.environ, CARGO_INCREMENTAL='0')
def exes(args):
    out = subprocess.run(['cargo', 'test', '--workspace', '--no-run', '--message-format=json', *args],
                         cwd=wt, capture_output=True, text=True, env=env).stdout
    for line in out.splitlines():
        m = json.loads(line)
        if m.get('reason') == 'compiler-artifact' and m.get('executable') and m['profile']['test']:
            t = m['target']
            yield (t['name'].replace('-', '_'), t['kind'][0], m['executable'])
targets = set(exes([]))
for t in ['canaries', 'browser', 'join_page', 'pages', 'scenes', 'conformance', 'scale', 'n0']:
    targets |= set(exes(['--test', t]))
def listed(exe, *more):
    out = subprocess.run([exe, '--list', '--format', 'terse', *more], capture_output=True, text=True).stdout
    return [l[:-6] for l in out.splitlines() if l.endswith(': test')]
rows = {}
for r in csv.reader(open(tsv), delimiter='\t'):
    if r and r[0] != 'test': rows[r[0]] = r
SEAMS = 'BMQRPCD'
tests = []
for name, kind, exe in sorted(targets):
    ign = set(listed(exe, '--ignored'))
    for t in listed(exe):
        key = f'{name}::{t}'
        if t in ign and key != 'scale::an_idle_round_is_under_the_ceiling': continue
        if name == 'scale' and key != 'scale::an_idle_round_is_under_the_ceiling' and kind == 'test':
            # the scale target runs in the gate only as the ceiling canary
            pass
        tests.append((name, kind, key))
# the scale target: check.sh's `tests` step runs its non-ignored tests too
per = collections.Counter(); tgt = collections.Counter(); tgtf = collections.Counter()
det = frag = 0; unmapped = []
for name, kind, key in tests:
    r = rows.get(key)
    if r is None:
        if kind != 'lib': unmapped.append(key)
        s = ''
    else:
        s = r[1]
    f = any(c in s for c in SEAMS)
    tgt[name] += 1
    if f:
        frag += 1; tgtf[name] += 1
        for c in SEAMS:
            if c in s: per[c] += 1
    else:
        det += 1
tot = det + frag
print(f'gate Rust tests {tot}: deterministic {det} ({100*det/tot:.1f}%), fragile {frag}')
print('per seam', dict(per))
print('fragile by target', ', '.join(f'{t} {c}' for t, c in tgtf.most_common()))
print('unmapped integration tests (counted deterministic):', len(unmapped))
for u in unmapped: print('  ', u)
moved_listed = [k for _, _, k in tests if k in rows and rows[k][2] == 'moved']
print('rows marked moved but still listed:', moved_listed)
