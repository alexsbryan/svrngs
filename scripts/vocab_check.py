#!/usr/bin/env python3
"""The infra never learns an app's nouns (docs/DESIGN.md §5). Exit 0 holds, 1 broken.

Every app lists its nouns in a nouns.txt under apps/, one per line, `#` for a
comment. Each is refused, as a whole word in any case, in infra code: every
.rs file under crates/ and web/src/, comments and tests apart (test files by
name; an item `#[cfg(test)]` gates, to its `;` or closing brace: it once
ended the file, hiding what followed a `mod tests;`, #194), since an example
there is fine; and the pages' scripts, the home page's and web/'s but lend.js
and shim.js, whose DOM `event` is no app's (v1ui-door (c)). `--self-test`
plants a noun in each and watches it caught.
"""
import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TEST_FILE = re.compile(r"(^|/)tests?(_\w+)?\.rs$|_tests?\.rs$")


def nouns(root):
    out = {}
    for f in sorted((root / "apps").glob("**/nouns.txt")):
        for line in f.read_text().splitlines():
            w = line.split("#", 1)[0].strip().lower()
            if w:
                out[w] = f.parent.name
    return out


def code_lines(text):
    lines, past = text.splitlines(), 0
    for n, line in enumerate(lines, 1):
        past = gated_to(lines, n) if n > past and line.strip().startswith("#[cfg(test)]") else past
        code = line.split("//", 1)[0]
        if n > past and code.strip():
            yield n, code


# The last line of the item the attribute on line `at` gates: past any
# further attributes, to its `;`, or to the brace that closes its first.
def gated_to(lines, at):
    depth = None
    for n in range(at, len(lines)):
        code = lines[n].split("//", 1)[0]
        if depth is None and code.strip().startswith("#["):
            continue
        if depth is not None or "{" in code:
            depth = (depth or 0) + code.count("{") - code.count("}")
        if (depth is not None and depth <= 0) or (depth is None and code.rstrip().endswith(";")):
            return n + 1
    return len(lines)


def check(root):
    words = nouns(root)
    if not words:
        return [], 0
    pattern = re.compile(r"\b(" + "|".join(map(re.escape, words)) + r")\b", re.I)
    found = []
    pages = [*(root / "crates").glob("*/src/**/*.js"), *(root / "web").glob("*.js")]
    pages = [f for f in pages if f.relative_to(root).as_posix() not in ("web/lend.js", "web/shim.js")]
    for f in sorted([*(root / "crates").glob("*/src/**/*.rs"), *(root / "web/src").glob("**/*.rs"), *pages]):
        rel = f.relative_to(root).as_posix()
        if "/tests/" in f"/{rel}" or TEST_FILE.search(rel):
            continue
        for n, code in code_lines(f.read_text(errors="replace")):
            for m in pattern.finditer(code):
                found.append(f"{rel}:{n}: `{m.group(0)}` is {words[m.group(0).lower()]}'s noun")
    return found, len(words)


def self_test():
    with tempfile.TemporaryDirectory() as d:
        root = Path(d)
        (root / "apps/films").mkdir(parents=True)
        (root / "apps/films/nouns.txt").write_text("# comment\nfilm\n")
        src = root / "crates/x/src"
        src.mkdir(parents=True)
        (src / "lib.rs").write_text("// a film in a comment is fine\nfn f() {}\n#[cfg(test)]\nmod tests { const FILM: &str = \"film\"; }\n")
        (src / "tests_x.rs").write_text("const FILM: u8 = 1;\n")
        clean, _ = check(root)
        (src / "door.rs").write_text("pub struct Film; // planted\n")
        (src / "home.js").write_text("// planted\nconst what = { k: 'film' };\n")
        (src / "guest.rs").write_text("#[cfg(test)]\nmod tests;\n\npub struct Film; // planted, after a test's declaration\n")
        (root / "web/src").mkdir(parents=True)
        (root / "web/src/lib.rs").write_text("fn f() { let _ = \"film\"; } // planted in the page's crate\n")
        (root / "web/open.js").write_text("const what = 'film'; // planted in a page script\n")
        caught, _ = check(root)
    planted = ["crates/x/src/door.rs:1:", "crates/x/src/guest.rs:4:", "crates/x/src/home.js:2:", "web/open.js:1:", "web/src/lib.rs:1:"]
    ok = clean == [] and [c.split(" ")[0] for c in caught] == planted
    print(f"self-test: clean tree {clean}, planted tree {caught} -> {'holds' if ok else 'BROKEN'}")
    return 0 if ok else 1


def main():
    if "--self-test" in sys.argv:
        return self_test()
    found, n = check(ROOT)
    for line in found:
        print(f"BROKEN  {line}")
    print(f"{n} app nouns, {len(found)} in infra code")
    return 1 if found else 0


if __name__ == "__main__":
    sys.exit(main())
