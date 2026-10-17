#!/usr/bin/env python3
# svrngs never widens what runs as you, or what can reach you, silently (svrn-docs
# rings/current/WHAT_RUNS_AS_YOU.md §1, §4). Exit 0 holds, 1 broken.
#
# scripts/trust_floor.txt is the one home for what a page or a socket may do;
# its head says how it reads. This gate fails on:
# 1. 'unsafe-eval' in any policy, served or pinned;
# 2. a script-src or worker-src, or the directive standing in for it, holding
#    'unsafe-inline' or anything unquoted: *, data:, blob:, a scheme or a host;
# 3. 'wasm-unsafe-eval' in a carried page's policy (ralph/next/canon/CHARTER.md);
# 4. a served policy differing from its pinned form, token by token; a pinned
#    widening with no reason. A new page is pinned by whoever adds it: this
#    gate finds a new listener by its code, not a new page;
# 5. a line that binds a socket and that the floor does not name, or one it
#    names that is gone;
# 6. the floor changed in a commit whose body has no `Trust floor:` line, or
#    changed and not committed, so no reason can be read.
import re, subprocess, sys
from collections import Counter
from pathlib import Path

ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent
FLOOR = "scripts/trust_floor.txt"
TEST_FILE = re.compile(r"(^|/)tests?(_\w+)?\.rs$|_tests?\.rs$")
BINDS = re.compile(r"TcpListener::bind|UdpSocket::bind|\.bind\(|\.bind_addr\(|Server::spawn\(|UNSPECIFIED|0\.0\.0\.0|\[::\]")


def git(*args):
    return subprocess.run(["git", "-C", str(ROOT), *args], capture_output=True, text=True)


def read(file):
    return (ROOT / file).read_text() if (ROOT / file).is_file() else ""


# The one string literal after `anchor` in `file`, Rust's line continuations
# joined, as directives: each name to its tokens, sorted. None when the anchor
# is not there exactly once.
def served(file, anchor):
    text = read(file)
    literal = text.count(anchor) == 1 and re.match(r'\s*"((?:[^"\\]|\\.)*)"', text.split(anchor, 1)[1], re.S)
    policy = literal and re.sub(r"\\\n\s*", "", literal.group(1))
    return policy and {d.split()[0]: sorted(d.split()[1:]) for d in re.split(r"[;,]", policy) if d.strip()}


# Rules 1-3, over one policy's directives: 'unsafe-eval' anywhere, and in a
# carried page 'wasm-unsafe-eval'. A worker's scripts fall back to child-src,
# then to the scripts', which fall back to default-src.
def widenings(name, d):
    out = [f"{name}: {k} holds {t}" for k, v in d.items() for t in v
           if t == "'unsafe-eval'" or (t == "'wasm-unsafe-eval'" and name.startswith("carried"))]
    script = d.get("script-src", d.get("default-src"))
    for k, v in (("script-src", script), ("worker-src", d.get("worker-src", d.get("child-src", script)))):
        out += [f"{name}: its {k} holds {t}" for t in v or [] if t == "'unsafe-inline'" or not t.startswith("'")]
    return out


def main():
    broken, pages, listens = [], {}, Counter()
    for n, line in enumerate(read(FLOOR).splitlines(), 1):
        body, _, why = (s.strip() for s in line.partition(" | "))
        if line.startswith("  ") and body:
            pages[page][2][body.split()[0]] = sorted(body.split()[1:])
            broken += [f"{FLOOR}:{n}: {body} widens with no reason given"] if not why and set(body.split()[1:]) - {"'self'", "'none'"} else []
        elif body.startswith("page "):
            _, page, file, anchor = body.split(" ", 3)
            pages[page] = (file, anchor, {})
        elif body.startswith("listen ") and why:
            listens.update([body[7:]])
        elif body and not body.startswith("#"):
            broken.append(f"{FLOOR}:{n}: not a page, a directive or a listener with its reason")
    for name, (file, anchor, pinned) in pages.items():
        got = served(file, anchor)
        broken += widenings(name, pinned) + widenings(f"{name} as served", got or {})
        broken += [f"{name}: {file} holds no one policy after `{anchor}`, so it could not be judged"] if not got else [
            f"{name}: {file} serves `{k} {' '.join(got.get(k, ['(absent)']))}`, the floor `{' '.join(pinned.get(k, ['(absent)']))}`"
            for k in sorted(set(got) | set(pinned)) if got.get(k) != pinned.get(k)]
    tracked = git("ls-files", "-co", "--exclude-standard").stdout.split()
    sites = Counter(f"{f}: {s.strip()}" for f in tracked if re.match(r"(crates/[\w-]+|web)/src/.*\.rs$", f)
                    and not TEST_FILE.search(f) for s in read(f).splitlines() if BINDS.search(s) and not s.strip().startswith("//"))
    broken += [f"{s}: binds a socket the floor does not name" for s in sites - listens]
    broken += [f"{FLOOR} names `{s}`, which no code holds now" for s in listens - sites]
    log = git("log", "--no-merges", "--format=%h%x00%B%x01", "--", FLOOR)
    if log.returncode != 0 or git("status", "--porcelain", "--", FLOOR).stdout:
        broken.append(f"{FLOOR} is not committed here, so whether its change gives a reason could not be judged")
    broken += [f"{c} changed {FLOOR} and its body gives no `Trust floor:` reason" for c, _, b in
               (e.strip().partition("\x00") for e in log.stdout.split("\x01") if e.strip()) if not re.search(r"^Trust floor: \S", b, re.M)]
    print(f"trust floor: {len(pages)} policies, {sum(listens.values())} listeners" + "".join(f"\nBROKEN  {b}" for b in broken))
    return 1 if broken else 0


sys.exit(main())
