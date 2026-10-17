# Re-pub what a downstream crate needs, from cargo's JSON: a privacy error's
# definition span, or an unresolved name's definition in another crate.
import json,sys,re
from pathlib import Path
root=Path(sys.argv[1]); fixed=set(); errs=0; unk=[]
srcs=[p for p in root.glob('crates/*/src/**/*.rs')]+list(root.glob('web/src/**/*.rs'))
def crate_of(p): 
    s=str(p); m=re.search(r'crates/([^/]+)/',s); return m.group(1) if m else 'web'
texts={p:p.read_text().split('\n') for p in srcs}
def define(name, exclude=None, only=None):
    hits=[]
    pat=re.compile(r'pub\(crate\)\s+(?:async\s+|unsafe\s+|const\s+)*(?:fn|struct|enum|trait|type|const|static|mod)\s+'+re.escape(name)+r'\b')
    upat=re.compile(r'pub\(crate\)\s+use\b.*\b'+re.escape(name)+r'\b')
    for p,ls in texts.items():
        c=crate_of(p)
        if exclude and c==exclude: continue
        if only and c!=only: continue
        for i,l in enumerate(ls):
            if pat.search(l) or upat.search(l): hits.append((str(p),i+1))
    # a multi-line `pub(crate) use x::{\n a,\n b }`
    for p,ls in texts.items():
        c=crate_of(p)
        if (exclude and c==exclude) or (only and c!=only): continue
        for i,l in enumerate(ls):
            if re.match(r'\s*pub\(crate\)\s+use\b.*\{\s*$',l):
                j=i
                while j<len(ls) and '}' not in ls[j]: 
                    j+=1
                    if re.search(r'\b'+re.escape(name)+r'\b',ls[j]): hits.append((str(p),i+1))
    return hits
def field(struct, fld):
    hits=[]
    for p,ls in texts.items():
        for i,l in enumerate(ls):
            if re.search(r'\bstruct\s+'+re.escape(struct)+r'\b',l):
                if '(' in l and 'pub(crate)' in l.split(struct,1)[1]: hits.append((str(p),i+1)); continue
                for j in range(i+1,min(i+200,len(ls))):
                    if re.match(r'^\s*\}',ls[j]): break
                    if re.search(r'pub\(crate\)\s+'+re.escape(fld)+r'\s*:',ls[j]): hits.append((str(p),j+1))
    return hits
for line in open(sys.argv[2]):
    try: m=json.loads(line)
    except: continue
    if m.get('reason')!='compiler-message': continue
    d=m['message']
    if d['level']!='error': continue
    errs+=1
    code=(d.get('code') or {}).get('code'); msg=d['message']
    efile=str((root/__import__('os').environ.get('DIR','.')/d['spans'][0]['file_name']).resolve()) if d['spans'] else ''
    ec=crate_of(efile)
    got=False
    for ch in d.get('children',[]):
        for s in ch.get('spans',[]):
            p=Path(s['file_name']); p=p if p.is_absolute() else (root/__import__('os').environ.get('DIR','.')/p).resolve()
            if 'defined here' in ch['message'] or 'private' in ch['message']:
                if p.exists(): fixed.add((str(p),s['line_start'])); got=True
    if got: continue
    mm=re.search(r"field `(\w+)` of struct `(\w+)` is private",re.sub(r"`[\w:]*::(\w+)`",r"`\1`",msg))
    m2=re.search(r"fields? (.*) of struct `(\w+)` (?:are|is) private",re.sub(r"`[\w:]*::(\w+)`",r"`\1`",msg))
    if m2 and not mm:
        for fl in re.findall(r"`(\w+)`",m2.group(1)):
            h=field(m2.group(2),fl); fixed.update(h); got=got or bool(h)
    m3=re.search(r"^type `(?:[\w:]*::)?(\w+)(?:<[^`]*>)?` is private",msg)
    if m3:
        h=define(m3.group(1)); fixed.update(h); got=got or bool(h)
    if mm:
        h=field(mm.group(2),mm.group(1)); fixed.update(h); got=bool(h)
    names=re.findall(r"`(?:crate::|[\w:]*::)?(\w+)`",msg)
    if not got and code in ('E0425','E0433','E0405','E0422','E0432','E0412','E0603','E0624','E0364','E0365','E0599','E0616','E0451','E0574','E0531','E0532'):
        for n in names:
            if n in ('crate','self','super'): continue
            h=define(n, only=ec if code in ('E0364','E0365') else None, exclude=None if code in ('E0364','E0365') else ec)
            if h: fixed.update(h); got=True
    if not got: unk.append(d['rendered'][:400])
n=0
fixed={(str(Path(f).resolve()),l) for f,l in fixed}
for f,l in sorted(fixed):
    ls=Path(f).read_text().split('\n')
    if 'pub(crate)' in ls[l-1]:
        ls[l-1]=ls[l-1].replace('pub(crate)','pub'); Path(f).write_text('\n'.join(ls)); n+=1
for u in unk[:6]: print('UNHANDLED',u)
print('errors',errs,'fixed',n)
