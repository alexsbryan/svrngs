# Every `pub` in a non-test source of the four crates and web/src made
# `pub(crate)`, each `pub use a::{b, c}` split to one name a line; the two
# glob re-exports and the page's #[wasm_bindgen] items stay `pub`
# (start-economize's survey, §1, done by hand there, by this here).
import re, sys
from pathlib import Path
root = Path(sys.argv[1])
TEST = re.compile(r"(^|/)tests?(_\w+)?\.rs$|_tests?\.rs$")
PUB = re.compile(r"\bpub\b(?!\s*\()")

def flat(prefix, body):
    out, depth, cur = [], 0, ""
    for ch in body:
        if ch == "{": depth += 1
        if ch == "}": depth -= 1
        if ch == "," and depth == 0:
            out.append(cur.strip()); cur = ""
        else:
            cur += ch
    if cur.strip(): out.append(cur.strip())
    names = []
    for item in out:
        m = re.match(r"^([\w:]*?)::?\{(.*)\}$", item, re.S)
        if m: names += flat(prefix + "::" + m.group(1).rstrip(":"), m.group(2))
        elif item == "self": names.append(prefix)
        else: names.append(prefix + "::" + item)
    return names

files = [p for p in root.glob("crates/*/src/**/*.rs") if not TEST.search(p.as_posix()) and "/tests/" not in p.as_posix()]
files += [p for p in root.glob("web/src/**/*.rs")]
for p in files:
    text = p.read_text()
    # split grouped re-exports
    def split(m):
        ind, prefix, body = m.group(1), m.group(2), m.group(3)
        return "\n".join(f"{ind}pub use {n};" for n in flat(prefix, body))
    text = re.sub(r"(?m)^([ \t]*)pub use ([\w:]+)::\{(.*?)\};", split, text, flags=re.S)
    lines, out, keep_impl, prev_bindgen = text.split("\n"), [], False, False
    for l in lines:
        s = l.strip()
        if re.match(r"pub use [\w:]+::\*;", s):
            out.append(l); continue
        if s.startswith("#[wasm_bindgen"):
            prev_bindgen = True; out.append(l); continue
        if prev_bindgen and s.startswith("impl"):
            keep_impl = True
        if keep_impl and l.startswith("}"):
            keep_impl = False; out.append(l); continue
        if prev_bindgen or keep_impl or s.startswith("///") or s.startswith("#["):
            if not (s.startswith("///") or s.startswith("#[")):
                prev_bindgen = False
            out.append(l); continue
        out.append(PUB.sub("pub(crate)", l))
    p.write_text("\n".join(out))
print(len(files), "files")
