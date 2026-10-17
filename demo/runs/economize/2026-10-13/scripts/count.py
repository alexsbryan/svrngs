import sys,subprocess,re
from pathlib import Path
sys.path.insert(0,'scripts')
import importlib.util; N=int(sys.argv.pop(1)) if len(sys.argv)>1 else 60
spec=importlib.util.spec_from_file_location('lc','scripts/lean_check.py');lc=importlib.util.module_from_spec(spec);spec.loader.exec_module(lc)
from collections import Counter
c=Counter()
ROOT=lc.ROOT
for f in lc.files(ROOT):
    rel=f.relative_to(ROOT).as_posix()
    if (f.suffix not in lc.CODE or not f.is_file() or "/tests/" in f"/{rel}" or "/node_modules/" in f"/{rel}" or rel.startswith(("apps/","target/","demo/runs/")) or re.search(r"(^|/)tests?(_\w+)?\.rs$|_tests?\.rs$|\.test\.m?js$", rel)): continue
    n=sum(1 for l in f.read_text(errors='replace').splitlines() if l.strip() and not l.strip().startswith(("//","#","/*","*")))
    c[rel]=n
for k,v in sorted(c.items(),key=lambda x:-x[1])[:N]: print(v,k)
print(sum(c.values()))
