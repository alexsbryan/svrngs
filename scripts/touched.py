#!/usr/bin/env python3
"""The touched tier: which of check.sh's slow steps a change can reach, for
`scripts/check.sh --touched [base | --files <path>...]`, which runs those and says each other one
`untouched`. The cheap steps always run, and the whole check.sh before a push
stays the gate. Changed is every file differing from `base` (default: where
HEAD left its upstream) and every untracked one. A file reaches:

- a crate's own files: that crate and each workspace crate depending on it,
  every test binary of each, whole (cargo metadata);
- a test binary's file or folder: that binary; a crate's shared test file:
  every binary of that crate;
- the phone's site (scripts/build-web.sh: web/ but web/tests, docs/brand/, the
  script, and the crates the wasm compiles, by web/Cargo.toml's paths and
  web/src's `#[path]`s): every binary that builds the site;
- a driver in web/tests: each binary naming it, or naming a driver that
  imports it, statically or by a literal `import()`; a driver with any other
  `import()` is taken to import everything;
- any other file: each binary whose sources, and each step whose command,
  name it; docs/, ralph/ and demo/runs/ and *.md the cheap steps alone read.

A file nothing above places reaches every step: what the graph cannot prove
unaffected runs. Prints `export` lines for check.sh to eval, the reasons on
stderr.
"""
import fnmatch
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
QUIET = ("docs/", "ralph/", "demo/runs/", ".githooks/")
EVERYTHING = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "scripts/check.sh", "scripts/touched.py"}


def run(*cmd):
    return subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, check=True).stdout


def text(path):
    try:
        return path.read_text(errors="replace")
    except (IsADirectoryError, FileNotFoundError):
        return ""


def main(base, given=None):
    if not base and given is None:
        up = subprocess.run(["git", "merge-base", "HEAD", "@{upstream}"], cwd=ROOT, capture_output=True, text=True)
        base = up.stdout.strip() or "HEAD"
    listed = given if given is not None else (run("git", "diff", "--name-only", base) + run("git", "ls-files", "--others", "--exclude-standard")).splitlines()
    files = set(filter(None, listed))
    meta = json.loads(run("cargo", "metadata", "--format-version", "1", "--no-deps"))
    home = {p["name"]: Path(p["manifest_path"]).parent.relative_to(ROOT).as_posix() for p in meta["packages"]}
    # Each test binary: (crate, name) -> its own path, its sources' text.
    bins = {}
    for p in meta["packages"]:
        targets = [t for t in p["targets"] if "test" in t["kind"]]
        owns = {t["name"]: Path(t["src_path"]) for t in targets}
        owns = {n: (s.parent if s.name == "main.rs" else s.with_suffix("")) for n, s in owns.items()}
        tests = ROOT / home[p["name"]] / "tests"
        # A file under tests/ in no binary's own folder is every binary's (common/).
        shared = [f for f in tests.glob("*/**/*") if not any(f.is_relative_to(o) for o in owns.values())]
        for t in targets:
            own = owns[t["name"]]
            body = "".join(text(f) for f in [Path(t["src_path"]), *own.glob("**/*"), *shared])
            bins[(p["name"], t["name"])] = (own.relative_to(ROOT).as_posix(), body, t["test"])
    check = (ROOT / "scripts/check.sh").read_text()
    every = r'^(?:.*then )?step "([^"]+)"\s+(.*)$'
    steps = dict(re.findall(every, check[check.index('"${1:-}" = --fast'):], re.M))  # the slow ones
    wasm = {f"web/{p}".replace("web/../", "") for p in re.findall(r'path = "([^"]+)"', text(ROOT / "web/Cargo.toml"))}
    wasm |= {f"web/src/{p}".replace("web/src/../../", "") for f in (ROOT / "web/src").glob("*.rs") for p in re.findall(r'#\[path = "([^"]+)"\]', text(f))}
    # What the pages' types check (scripts/types.mjs): the two jsconfigs' includes.
    typed = ["jsconfig*.json"] + [g for j in ROOT.glob("jsconfig*.json") for g in re.findall(r'"([^"]+)"', text(j).split('"include"')[1])]
    site_users = {b for b, (_, body, _) in bins.items() if "build-web" in body or "site()" in body}
    # What each web/tests file reaches: each one whose name is in its text (an
    # import, a script it runs, a file it reads), or any, for an `import()`
    # its text cannot name: neither a literal, nor loaded in a page from the
    # served site (`new URL(..., app)`, `/...`), nor of a folder outside web/tests.
    tests = {d.relative_to(ROOT).as_posix(): text(d) for d in (ROOT / "web/tests").glob("**/*") if d.is_file()}

    def unknown(f, src):
        literal = re.findall(r"""import\(\s*(new URL\(\s*)?[`'"]([^`'"]*)""", src)
        heads = [(ROOT / f).parent.joinpath(h.split("${")[0]).resolve() for u, h in literal if "${" in h and not u and not h.startswith("/")]
        return len(literal) < src.count("import(") or any(h.is_relative_to(ROOT / "web/tests") or not h.is_relative_to(ROOT) for h in heads)

    imports = {f: ({g for g in tests if g != f and Path(g).name in src}, unknown(f, src)) for f, src in tests.items()}

    def reached_from(start):  # the files named, and whether it may reach any
        seen, todo, more = set(), [start], False
        while todo:
            d = todo.pop()
            if d not in seen:
                seen.add(d)
                deps, unsure = imports[d]
                todo, more = todo + list(deps), more or unsure
        return seen, more

    # Each binary's web/tests files: from the drivers it names, None for any.
    drives = {}
    for b, (_, body, _) in bins.items():
        froms = [reached_from(d) for d in tests if Path(d).name in body]
        drives[b] = None if any(more for _, more in froms) else set().union(*(seen for seen, _ in froms))

    crates, reached, why = set(), set(), {}

    def reach(what, file, how):
        for b in what:
            if b not in reached:
                reached.add(b)
                why[b] = f"{file}: {how}"

    for f in sorted(files):
        crate = max((c for c, h in home.items() if f.startswith(h + "/")), key=lambda c: len(home[c]), default=None)
        if f in EVERYTHING or f.startswith(".cargo/"):
            print(f"note       {f} reaches every step", file=sys.stderr)
            return
        if f.startswith(QUIET) or (f.endswith(".md") and not f.startswith("apps/")):
            continue
        if crate and f.startswith(home[crate] + "/tests/"):
            own = [b for b, (path, _, _) in bins.items() if b[0] == crate and (f == path + ".rs" or f.startswith(path + "/"))]
            reach(own or [b for b in bins if b[0] == crate], f, "its test binary" if own else "its crate's tests")
            continue
        if crate:
            crates.add(crate)
            if any(f.startswith(w) for w in wasm):
                reach(["web rustfmt and clippy"], f, "compiled into the page's wasm")
            continue
        if f.startswith("web/tests/"):
            if f.endswith(".test.mjs"):
                continue
            named = {b for b, seen in drives.items() if seen is None or f in seen}
            laws = any(f in reached_from(d)[0] for d in tests if d.endswith(".test.mjs"))
            if named or not laws:  # a law's helper alone runs with the laws
                reach(named or site_users, f, "a driver that reaches it" if named else "nothing reaches it, so every site's")
            continue
        if f.startswith(("web/", "docs/brand/")) or f == "scripts/build-web.sh":
            reach(site_users, f, "the phone's site")
            if f.startswith(("web/src/", "web/Cargo")) or f == "scripts/build-web.sh":
                reach(["web rustfmt and clippy"], f, "the page's wasm")
            continue
        # An app is read by any binary that reads apps/, its laws by `app laws`.
        name = "apps" if f.startswith("apps/") else f
        named = {b for b, (_, body, _) in bins.items() if name in body}
        named |= {s for s, cmd in re.findall(every, check, re.M) if f in cmd or (name == "apps" and s == "app laws")}
        if not named and not any(fnmatch.fnmatch(f, g) for g in typed):
            print(f"note       {f}: nothing places it, so every step runs", file=sys.stderr)
            return
        reach(named, f, "names it")
    grew = True
    while grew:
        more = {p["name"] for p in meta["packages"] if p["name"] not in crates and any(d["name"] in crates and d.get("path") for d in p["dependencies"])}
        crates, grew = crates | more, bool(more)
    for c in sorted(crates):
        reach([b for b in bins if b[0] == c], home[c], "its crate, or one it depends on")
    for f in (f for f in sorted(files) if any(fnmatch.fnmatch(f, g) for g in typed)):
        reach(["types", "typed lint"], f, "the pages' types")
    if any(f.endswith(".rs") or f.endswith("Cargo.toml") for f in files):
        reach(["clippy"], next(f for f in sorted(files) if f.endswith((".rs", "Cargo.toml"))), "Rust")
    # The steps: one a binary in it was reached, or named; `tests` the rest.
    runs = {s: set(re.findall(r"--test (\w+)", cmd)) for s, cmd in steps.items()}
    run_ = {s for s in steps if s in reached or any(b in bins and b[1] in runs[s] for b in reached)}
    # `tests`: whole crates, or one crate's binaries alone, never a mix.
    plain = {b for b in reached if b in bins and bins[b][2]}
    whole = crates | {b[0] for b in plain} if crates or len({b[0] for b in plain}) > 1 else set()
    pick = [f"-p {c}" for c in sorted(whole)] or [f"-p {b[0]}" for b in sorted(plain)][:1]
    pick += [] if whole else [f"--test {b[1]}" for b in sorted(plain)]
    if pick:
        run_.add("tests")
    for s in sorted(steps):
        hit = next((why[b] for b in sorted(reached, key=str) if b == s or (b in bins and b[1] in runs[s])), None)
        print(f"{'reached' if s in run_ else 'untouched':<10} {s}{f' ({hit})' if hit and s in run_ else ''}", file=sys.stderr)
    listed = "\n".join(sorted(run_))
    print(f"export SVRNGS_STEPS='{listed}' SVRNGS_TESTS='{' '.join(pick)}'")


if __name__ == "__main__":
    # `--files <path>...` plans those files as changed, for a look at a plan.
    if sys.argv[1:2] == ["--files"]:
        main("", sys.argv[2:])
    else:
        main(sys.argv[1] if len(sys.argv) > 1 else "")
