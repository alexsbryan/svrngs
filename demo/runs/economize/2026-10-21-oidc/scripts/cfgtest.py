# Items under a test cfg in files lean_check counts: what each costs by its rule.
import re,subprocess
from pathlib import Path
TEST=re.compile(r"(^|/)tests?(_\w+)?\.rs$|_tests?\.rs$")
fs=subprocess.run(['git','ls-files','crates/*/src/**.rs','crates/*/src/*.rs','web/src/*.rs'],capture_output=True,text=True).stdout.split()
def code(l):
    s=l.strip(); return bool(s) and not s.startswith(("//","#","/*","*"))
tot=0
for f in sorted(set(fs)):
    if TEST.search(f) or '/tests/' in f: continue
    ls=open(f).read().split('\n'); i=0
    while i<len(ls):
        if re.match(r'\s*#\[cfg\((any\()?(all\()?test|\s*#\[cfg\(any\(test',ls[i]) or re.match(r'\s*#\[cfg\(.*feature = "test-support"',ls[i]):
            j=i+1
            while j<len(ls) and (ls[j].strip().startswith(('#[','//')) or not ls[j].strip()): j+=1
            if re.match(r'\s*(pub(\(crate\))?\s+)?mod \w+;',ls[j]): i=j+1; continue
            # item extent: to ; at depth 0 or matching brace
            d=0;k=j;started=False
            while k<len(ls):
                d+=ls[k].count('{')-ls[k].count('}')
                if '{' in ls[k]: started=True
                if (started and d<=0) or (not started and ls[k].rstrip().endswith((';',','))): break
                k+=1
            n=sum(code(x) for x in ls[j:k+1])
            tot+=n; print(f"{f}:{i+1}-{k+1} {n}  {ls[j].strip()[:70]}")
            i=k+1; continue
        i+=1
print('total',tot)
