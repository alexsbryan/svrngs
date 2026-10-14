#!/usr/bin/env python3
"""The infra never learns an app's nouns (docs/DESIGN.md §5). Exit 0 holds, 1 broken.

Every app lists its nouns in apps/<app>/nouns.txt, one per line, `#` for a
comment. Each is refused, as a whole word in any case, in infra code: every
.rs file under crates/, with comments and tests apart — test files by name,
and a file's inline tests from its first `#[cfg(test)]` on — since an example
in a comment or a test is fine; and the pages' own scripts, the home page's
under crates/ and the group's page, web/app.js, so neither types a kind's
noun of its own (v1ui-door (c)). `--self-test` plants a noun in each and
watches it caught.
"""
import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TEST_FILE = re.compile(r"(^|/)tests?(_\w+)?\.rs$|_tests?\.rs$")


def nouns(root):
    out = {}
    for f in sorted((root / "apps").glob("*/nouns.txt")):
        for line in f.read_text().splitlines():
            w = line.split("#", 1)[0].strip().lower()
            if w:
                out[w] = f.parent.name
    return out


def code_lines(text):
    for n, line in enumerate(text.splitlines(), 1):
        if line.strip().startswith("#[cfg(test)]"):
            return
        code = line.split("//", 1)[0]
        if code.strip():
            yield n, code


def check(root):
    words = nouns(root)
    if not words:
        return [], 0
    pattern = re.compile(r"\b(" + "|".join(map(re.escape, words)) + r")\b", re.I)
    found = []
    pages = [*(root / "crates").glob("*/src/**/*.js"), *(root / "web").glob("app.js")]
    for f in sorted([*(root / "crates").glob("*/src/**/*.rs"), *pages]):
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
        caught, _ = check(root)
    ok = clean == [] and [c.split(" ")[0] for c in caught] == ["crates/x/src/door.rs:1:", "crates/x/src/home.js:2:"]
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
