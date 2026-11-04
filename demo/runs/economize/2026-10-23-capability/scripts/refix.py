# cap-economize's error-driven re-publication, run beside repub.py: for each
# refusal cargo reports, make the named field, type or method `pub` in the
# scratch clone. repub.py (the earlier surveys' unchanged) covers most; this
# covers the E0451/E0616 fields and the 'type X is private' / 'method X is
# private' messages it does not parse. It reads the cargo JSON on stdin's
# path ($1) and edits /tmp/cap-econ-scratch. Not a product check, kept so
# the survey's pub set is reproducible.
import json, re, sys
from pathlib import Path
root = Path('/tmp/cap-econ-scratch')
b = Path(sys.argv[1])
srcs = [p for p in (root/'crates').glob('*/src/**/*.rs')]
srcs += list((root/'web/src').rglob('*.rs')) if (root/'web/src').exists() else []
def crate_of(p): 
    m=re.search(r'crates/([^/]+)/', str(p)); return m.group(1) if m else 'web'
def allpub(name, kinds=r'(?:struct|enum|type|fn|const|static|mod)', exclude=None, only=None):
    n=0
    pat=re.compile(r'pub\(crate\)(\s+(?:async\s+|unsafe\s+)?'+kinds+r'\s+'+re.escape(name)+r'\b)')
    for p in srcs:
        c=crate_of(p)
        if (exclude and c==exclude) or (only and c!=only): continue
        t=p.read_text(); t2=pat.sub(r'pub\1', t)
        if t2!=t: p.write_text(t2); n+=1
    return n
def fields(struct, names):
    n=0
    for p in srcs:
        t=p.read_text(); o=t
        for m in re.finditer(r'\bstruct\s+'+re.escape(struct)+r'\b[^\n{]*\{', t):
            i=m.end(); j=t.index('\n}', i)
            body=t[i:j]
            nb=body
            for f in names:
                nb=re.sub(r'(?m)^(\s*)pub\(crate\)(\s+'+re.escape(f)+r'\s*:)', r'\1pub\2', nb)
            if nb!=body: t=t[:i]+nb+t[j:]
        if t!=o: p.write_text(t); n+=1
    return n
def methods(name, exclude=None):
    n=0
    pat=re.compile(r'pub\(crate\)(\s+(?:async\s+|unsafe\s+|const\s+)*fn\s+'+re.escape(name)+r'\b)')
    for p in srcs:
        if exclude and crate_of(p)==exclude: continue
        t=p.read_text(); t2=pat.sub(r'pub\1', t)
        if t2!=t: p.write_text(t2); n+=1
    return n
names=set(); work=[]
for l in b.open():
    try: m=json.loads(l)
    except: continue
    d=m.get('message') or {}
    if m.get('reason')!='compiler-message' or d.get('level')!='error': continue
    code=(d.get('code') or {}).get('code'); msg=d['message']
    spans=d.get('spans') or [{}]
    ec=crate_of(root/spans[0]['file_name']) if spans and spans[0].get('file_name') else None
    mm=re.search(r"fields? (.*?) of struct `(?:[\w:]*::)?(\w+)` (?:are|is) private", re.sub(r"`[\w:]*::(\w+)`", r"`\1`", msg))
    if mm:
        fl=re.findall(r"`(\w+)`", mm.group(1))
        fields(mm.group(2), fl); print('fields', mm.group(2), fl); continue
    m3=re.search(r"type `(?:[\w:]*::)?(\w+)(?:<[^`]*>)?` is private", msg)
    if m3:
        print('type', m3.group(1)); allpub(m3.group(1)); continue
    m4=re.search(r"(?:method|function|associated function) `(\w+)` is private", msg)
    if m4:
        print('method', m4.group(1)); methods(m4.group(1), exclude=ec); continue
    if code in ('E0425','E0433','E0405','E0422','E0432','E0412','E0603','E0599','E0574','E0531','E0532','E0282','E0220'):
        for nm in re.findall(r"`(?:crate::|[\w:]*::)?(\w+)`", msg):
            if nm in ('crate','self','super'): continue
            allpub(nm); names.add(nm)
        print('names', msg[:120])
        continue
    work.append(msg[:160])
print('left', len(work))
for w in work[:8]: print(' ', w)
