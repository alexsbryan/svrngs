#!/usr/bin/env python3
"""svrngs stays standalone and lean (docs/DESIGN.md §4, §5). Exit 0 holds, 1 broken.

1. Cargo.lock names no commonwealth-ai or sovereign crate.
2. svrngs-core declares no I/O dependency, and is no_std: the compiler refuses
   files, sockets, clocks, threads and HashMap there; no core source outside
   its tests links std back in.
3. RUST_ARCH, where a program can decide it: `anyhow` only in the `svrngs`
   command and the app; no `Arc` of a `Mutex` or `RwLock`, nor `Rc` of a
   `RefCell`, its path spelled any way (#195), in a crate's product code
   (ownership is a tree); no `async` in svrngs-core (it is synchronous).
4. Lines of code — not blank, not a comment, not a test — stay under the
   caps `BUDGET` holds below, their one home (#197), scripts included. The
   budget counts the product: apps/, installed packages (any
   node_modules/) and demo/runs/ apart. A recorded run's driver is what ran,
   kept byte for byte so its data reads against it (ralph/REVIEW_FINDINGS.md
   #61), and no check runs one; code elsewhere under demo/ counts.
"""
import re
import subprocess
import sys
from pathlib import Path

TEST_FILE = re.compile(r"(^|/)tests?(_\w+)?\.rs$|_tests?\.rs$")
ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent
FOREIGN = re.compile(r'^name = "(commonwealth-[\w-]+|sovereign-[\w-]+|oicp-[\w-]+|kernel-types|oplog(-types)?|corpus-engine|host-kit|mesh-[\w-]+)"$', re.M)
IO_DEPS = {"tokio", "async-std", "smol", "iroh", "reqwest", "hyper", "axum", "tower",
           "rusqlite", "sqlx", "notify", "mio"}
# Each cap one past a count: svrngs-core's where events-economize's cut landed,
# the repository's where events-wait's held dial to a lender landed (2026-10-14),
# so no room is left that the operator did not give. v1ui-economize found nothing
# to cut and left both (demo/runs/economize/2026-10-14-v1ui/SURVEY.md). A raise is
# asked only after an economizing pass (AGENTS.md "Done"; DESIGN §5). After that
# pass the operator gave v1ui +650 over 16,315, core unchanged (2026-10-14), each
# row raising only as far as it needs. arch-helper, the one arch row that may
# raise, spends the 31 lines v1ui left (operator, 2026-10-15; ceiling 16,965):
# the repository's cap is one past where its (c)'s last, #169, the table's
# first column an enum, landed. arch-peer's cut of the loopback bridge lowered it
# to one past where its (d) landed (2026-10-15). arch-view, a commit signing no
# view, lowered both to one past where it landed. arch-command-relay, one maker
# each of a new group's relay and of a link, lowered the repository's to one past
# where its (c) landed; svrngs-core's it left at one past, unchanged. arch-command
# (a), one state for the node's routers, lowered the repository's to one past
# where it landed; then its join on the home page took what was left of the
# operator's "Page join; pay with cuts first" (2026-10-15, at most 16,190), to
# one past where the row closed after its economizing pass. arch-version, one
# version check in place of the fields' fallbacks, lowered the repository's to
# one past where it closed after its own; svrngs-core's it left, unchanged.
# groups-economize cut five items only tests reached and lowered the
# repository's to one past where its cuts landed; svrngs-core's it left at one
# past (demo/runs/economize/2026-10-16/SURVEY.md). The operator gave the groups
# rows +200 over that cap and svrngs-core +40 over its own (2026-10-14).
# groups-tabs (b), one tab of the group's page at a time reaching it (#176),
# raised the repository's to one past where it landed: 22 of the 200. Its
# (c), one connection kept per computer and protocol (#177, #178), cut 3 and
# spent 2, and lowered it to one past where the row closed. groups-leave (a),
# a person's own Remove their leave (svrngs-core's fold and `Leave`), raised
# svrngs-core's to one past where it landed, 34 of its 40, and the
# repository's with it. Its (b), `svrngs leave` and a left group held live no
# more, raised the repository's to one past where it landed. Its (c), the
# page's leave and a cut key refused a new link, raised both to one past
# where the row closes: svrngs-core 39 of its 40, the repository 164 of 200;
# one read of the kept fold where (b) made two cut 6, and lowered it to one
# past where the row closes. groups-leave-others, within the 41 left (the
# operator, 2026-10-16): its (b), a leave told to the leaver's other
# computers and a computer cut told by the first it asks, raised the
# repository's to one past where it landed; its (c) cut 1 and spent 0 of
# svrngs-core's, and its (d), a cut key's refusal said and traced, raised the
# repository's to one past where it landed. gv-leave, within +30 over groups'
# close, at most 16,358, svrngs-core unchanged (the operator, 2026-10-16): its
# (a), the leaver's old lend addresses saying she left (#182), raised the
# repository's to one past where it landed; its (b), the acceptor's trace as
# it tells a cut dialer its cut (#183), the same; its (c), the hand-ons told
# with a cut (#184), the same.
# northstar-economize cut upstream's local-only namespaces (#200) and
# lowered both to one past where the cut landed. The northstar fixes, within
# +100 over that and svrngs-core +15, at most 16,423 and 2,996 (the operator,
# 2026-10-16), each raise to one past where they land: #187, one hole a run;
# #188, the time following only signed lines at most five minutes on; #189,
# an id held once a line of it verifies; #191, a torn tail ended first;
# #190, the leaver's round telling the leave until each computer answers;
# #203, the door's clock before 1970 refused; #153, a path kept within its
# lend; #194, the vocab gate reading what it missed; #195, the shared-state
# regex reading every path; #190 again, the leave told from the round's read.
# canon-economize cut `read_origin`, a wrapper only tests asked, and lowered
# the repository's to one past where the cut landed; svrngs-core's was
# already there. The canon rows have +150 over that and svrngs-core +40, at
# most 16,564 and 3,032 (the operator, 2026-10-16), each raise to one past
# where it lands: canon-floor (b), the trust floor's gate and its step in
# check.sh, 57 lines; its (c), the gate reading the floor's behavioural
# gates, 2. canon-pin (b), a carried app served at the files it was brought
# with, svrngs-core 9 of its 40 and the repository 18; its (c), a new
# version asked of each person and run only after their yes, svrngs-core 16
# and the repository 66, both pages included. V1.1.1, the musl allocator
# fix (demo/runs/leak/2026-10-17/), cut 3 first, door.rs's two dialer
# guards made one, spent 1, and lowered it to one past where it landed. The
# screen's economizing pass cut 3, home.js reading a read's answer and a
# write's one way, and lowered it to one past where that landed. The screen,
# a group on a TV or a projector (home.js `screen`), raised it by 23 to one
# past where it landed, within the operator's "Yes, up to +30" (2026-10-18).
BUDGET = {"crates/svrngs-core": 3017, ".": 16576}
CODE = {".rs", ".js", ".mjs", ".ts", ".py", ".sh"}


def files(base):
    """What git calls source, tracked or new and not ignored, so a build's
    output never counts wherever it lands (#74: web/target/ was counted). A
    tree outside git, such as an exported revision, is walked whole, and says so."""
    r = subprocess.run(["git", "-C", str(ROOT), "ls-files", "-co", "--exclude-standard", "-z", "--",
                        base.relative_to(ROOT).as_posix()], capture_output=True)
    if r.returncode != 0:
        print(f"lean_check: {ROOT} is not a git tree, so every file counts, build output included",
              file=sys.stderr)
        return base.rglob("*")
    return (ROOT / p for p in r.stdout.decode().split("\0") if p)


def code_lines(base):
    n = 0
    for f in files(base):
        rel = f.relative_to(ROOT).as_posix()
        if (f.suffix not in CODE or not f.is_file() or "/tests/" in f"/{rel}"
                or "/node_modules/" in f"/{rel}"
                or rel.startswith(("apps/", "target/", "demo/runs/"))
                or re.search(r"(^|/)tests?(_\w+)?\.rs$|_tests?\.rs$|\.test\.m?js$", rel)):
            continue
        for line in f.read_text(errors="replace").splitlines():
            s = line.strip()
            if s and not s.startswith(("//", "#", "/*", "*")):
                n += 1
    return n


def main():
    broken = []
    lock = ROOT / "Cargo.lock"
    if lock.exists():
        for m in FOREIGN.finditer(lock.read_text()):
            broken.append(f"Cargo.lock names {m.group(1)}: svrngs links nothing from commonwealth-ai or sovereign")
    for crate in ("svrngs-core",):
        manifest = ROOT / "crates" / crate / "Cargo.toml"
        if manifest.exists():
            deps = re.findall(r'^([\w-]+)\s*=', manifest.read_text().split("[dependencies]", 1)[-1].split("\n[", 1)[0], re.M)
            for d in deps:
                if d.split("-")[0] in IO_DEPS or d in IO_DEPS:
                    broken.append(f"{crate} depends on {d}: the core takes no I/O dependency")
    for manifest in sorted((ROOT / "crates").glob("*/Cargo.toml")):
        crate = manifest.parent.name
        if crate != "svrngs" and re.search(r'^anyhow\s*=', manifest.read_text(), re.M):
            broken.append(f"{crate} depends on anyhow: crates carry typed errors; anyhow is the command's and the app's")
    for f in sorted((ROOT / "crates").glob("*/src/**/*.rs")):
        rel = f.relative_to(ROOT).as_posix()
        text = f.read_text(errors="replace")
        shared = re.search(r"Arc<\s*([\w:]+::)?(Mutex|RwLock)\s*<|Rc<\s*([\w:]+::)?RefCell\s*<", text)
        if shared and not TEST_FILE.search(rel):
            broken.append(f"{rel} shares mutable state: one owner and messages instead (RUST_ARCH §3)")
        if (rel.startswith("crates/svrngs-core/") and not TEST_FILE.search(rel)
                and re.search(r"^\s*extern\s+crate\s+std\b", text, re.M)):
            broken.append(f"{rel} links std: the core is no_std, so I/O, clocks and threads stay outside it")
        if rel.startswith("crates/svrngs-core/") and re.search(r"\basync\b|\.await\b", text):
            broken.append(f"{rel} is async: the core stays synchronous (RUST_ARCH §2, §4)")
    core_lib = ROOT / "crates" / "svrngs-core" / "src" / "lib.rs"
    if core_lib.exists() and "#![cfg_attr(not(test), no_std)]" not in core_lib.read_text():
        broken.append("crates/svrngs-core/src/lib.rs lost #![cfg_attr(not(test), no_std)]: the core is no_std")
    for path, cap in BUDGET.items():
        base = ROOT / path
        n = code_lines(base) if base.exists() else 0
        print(f"{path:<20} {n:>6} / {cap} lines")
        if n >= cap:
            broken.append(f"{path} has {n} lines of code, budget {cap}: raised only by the operator, with the reason in the commit")
    for b in broken:
        print(f"BROKEN  {b}")
    return 1 if broken else 0


if __name__ == "__main__":
    sys.exit(main())
