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
# and the repository 66, both pages included. canon-spans' economizing pass,
# the membership fold's traces on one line each and the seed's bindings in
# one chain, cut 32 and lowered both to one past where the cut landed. Its
# (b), each person's time in the group folded by the membership and given
# a carried app, raised svrngs-core 24 and the repository 31, to one past
# where it landed: 23 and 7 left under the ceiling. canon-house's shelf, a
# shelf app bringing any files and `-` said as a space, raised the
# repository's by 4, to one past where it landed: svrngs-core's 23 and 3 left.
# canon-fix (d), the page asking the next computer to answer when one's
# answer fails (#205), raised it by those 3, to one past where it landed:
# the operator's ceiling, none left. The musl allocator fix
# (demo/runs/leak/2026-10-17/) cut 3 first, door.rs's two dialer guards made
# one, spent 1, and lowered it to one past where it landed. The screen's
# economizing pass cut 3, home.js reading a read's answer and a write's one
# way, and lowered it to one past where that landed. The screen, a group
# on a TV or a projector (home.js `screen`), raised it by 23 to one past
# where it landed, within the operator's "Yes, up to +30" (2026-10-18), and
# its founder drawn first by 1 more, 24 of the 30.
# guests-economize cut 5, redeem.rs `answer`'s two arms of one refusal, and
# lowered the repository's to one past where the cut landed; svrngs-core's
# was already there (demo/runs/economize/2026-10-19-guests/SURVEY.md). The
# guests rows have +150 over that and svrngs-core +60, at most 16,728 and
# 3,069 (the operator, 2026-10-18), each raise to one past where it lands.
# guests-visit (b), a visit folded by svrngs-core beside membership, raised
# svrngs-core 60 and the repository 60, to one past where it landed: none
# left under svrngs-core's ceiling, 90 under the repository's. guests-door
# (b), a visit's key let in at its host's own lends as their guest, raised
# the repository's 66, to one past where it landed: 24 left under its ceiling.
# guests-link (b), a visit made by a link in any chat and the guest's page,
# raised svrngs-core 7 and the repository 143, to one past where it landed,
# within the operator's "Cover both rows", at most 16,905 and 3,076
# (2026-10-20; demo/runs/guests/2026-10-19-link/NEEDS_HUMAN.md): none left
# under svrngs-core's ceiling, 58 under the repository's. guests-pages (a),
# who is visiting on the home page and its host's Stop, raised the
# repository's 34, to one past where it landed: 24 left under its ceiling.
# Its (b), who is visiting on the phone's page, raised it 6, to one past
# where it landed: 18 left.
# V1.1.3, within the operator's "Approve, up to +24 each" (2026-10-20): its
# away page, a lend's computer that did not answer drawn as its person's
# window, dark, on a computer and a phone, raised it by 20 to one past where it
# landed, after an economizing pass that cut V1.1.3's own lines and found no
# fold elsewhere.
# The phone's lend worker asking for gzip and unpacking it raised it by 3
# more, to one past where it landed.
# The phone's sockets coming back with their lender raised it by 1 more, to
# one past where it landed: the operator's +24, none left.
# V1.1.3's economizing pass, the page's possessive written in away.html, cut
# 4 and lowered it to one past where the cut landed.
# leave-economize cut `Membership::visiting`, which only a test called,
# and lowered both to one past where the cut landed
# (demo/runs/economize/2026-10-21-leave/SURVEY.md). The leave rows have at
# most +25 over that and none in svrngs-core (the operator, 2026-10-21,
# "Pass first, then raise"): at most 16,929, svrngs-core 3,073.
# leave-told (b), a cut told with what it covers (#211), raised the
# repository's 24, to one past where it landed: 1 left under the ceiling.
# oidc-economize found no cut and the operator set the oidc ceiling at +800
# repository and +80 svrngs-core over where it left them, at most 17,728 and
# 3,153 (CHARTER.md, 2026-10-21; demo/runs/economize/2026-10-21-oidc/
# SURVEY.md, 4239bc7c). oidc-token's token, its key and its JWKS raised both
# to one past where they landed: svrngs-core 3,152, the repository 17,110.
# oidc-provider's back channel, its code store, the declaration's client and
# the code's PKCE method raised the repository 466 to one past where they
# landed: 17,576 (svrngs-core within the 3,153 it left, its nonce bought by a
# move that kept behaviour: 0e6257cf). 151 remain under the operator's 17,728.
# oidc-authorize's front channel raised the repository 149 to one past where
# it landed: 17,725, after its first draft was cut by 31 counted lines (the
# ask's map, the PKCE test and the inlined own-address check). svrngs-core
# stays 3,152 under its 3,153. 2 remain under the operator's 17,728.
# oidc-hook-cut retired the sign-in hook the operator authorized (CHARTER.md,
# 2026-10-21): its file and wiring, the helper's steps and the page's
# signs_in, 523 counted lines, and lowered the repository's to one past where
# it landed: 17,203. svrngs-core is unchanged at one past its 3,152.
# oidc-turn-on's switch, its client and its one secret raised the repository
# 172 to one past where they landed: 17,375 (17,728 is the operator's
# ceiling). svrngs-core is unchanged at 3,152 under its 3,153.
# oidc-jellyfin's forwarded origin and its lend-origin redirect raised the
# repository 46 to one past where they landed: 17,421, inside 17,728. Its
# phone worker naming the browser's host (the run found it) raised it 2
# more, to one past where the row closes: 17,423. svrngs-core is unchanged
# at 3,152 under its 3,153.
# oidc-retire removed the program per app with the table's helper row and
# share::At::Helper (the row and enum were counted; apps/ is not): 3 counted
# lines, and lowered the repository's to one past where the cut landed:
# 17,419. svrngs-core is unchanged at 3,152 under its 3,153.
# The operator's three oidc fixes (2026-10-22): one typed person and code
# (token::Subject, S256Challenge) and the token read back as `Claims` cut
# 50, the redirect's one type (door::LendPath) spent 5, and the room's
# scheme 2. Both caps lowered to one past where they landed: 17,376, and
# svrngs-core's 3,150 over its 3,149.
# The room's economizing pass (2026-10-22) cut 49 by folds in cli.rs,
# home.rs, node.rs and group.rs, finding no dead item, and install.sh's
# PATH line spent 4: the repository's cap lowered to one past where they
# land, 17,331. svrngs-core is unchanged at 3,149 under its 3,150.
# The room, the running node's, on per lend on its network (the operator,
# 2026-10-22, after that pass): raised 231, to one past where it lands,
# 17,562. svrngs-core is unchanged at 3,149 under its 3,150. Within that
# landing (the operator, 2026-10-22: room.json unread said, never read as
# empty), a room that cannot be read said and serving nothing raised it 6,
# to one past where it lands: 17,568.
# The network and the room as domains of their own (the operator,
# 2026-10-22: "Approve all three, +67"): network.rs's move +5, the room's
# edges in room/ +24, the network's types and the room's pure turn +38.
# Raised 67, to one past where the series lands: 17,635. svrngs-core is
# unchanged at 3,149 under its 3,150.
# capability-economize cut 16 by four folds of a thing said twice (oidc's
# Key::kid only tests called, the door's Connection names, the page's
# IndexedDB opener and its computer's key) and lowered the repository's to
# one past where they land, 17,619; svrngs-core's it left at one past
# (demo/runs/economize/2026-10-23-capability/SURVEY.md).
# The operator's ceiling for capability-navigation through capability-what-acts
# (2026-10-23, "+265 / +3 now"): at most 17,884 and 3,153, each row raising only
# what it lands, to one past. capability-navigation, a frame's and a form's
# navigation judged as its page's fetch in origin.rs's app_of, +4: 17,623.
# capability-scenes' "scenes" step in check.sh (f15184ce) was counted, +5,
# and broke the budget at 17,627; it was folded to two lines, the
# conformance canary's emptying joined to could-not-judge's, and door.rs's
# two rebindings of a parameter and an answer made `mut` where bound, -6:
# 17,621, the cap lowered to one past. capability-scenes-found (a), a
# request that failed on a connection kept to a restarted computer sent
# again once in iroh.rs's fetch, +12: 17,634. capability-scenes-found (e),
# `new`'s keeping of a group and its first link minted under the Host's one
# links' sequencer (invite.rs's keep_linked), +27: 17,662; (b), a joiner's
# device link given the group page's address, +3: 17,665. capability-open,
# the yes to what an app reads kept in door/choices.rs and judged in
# surface.rs's reaches, the opens worded from what a lend declares, +54:
# 17,720, raised by the operator's delegate inside the ceiling, the
# permission classifier having refused it to the row's session.
# capability-what-acts, "What acts as you" on both pages, each yes with its
# Stop (question.js `acts`, home.rs's GET /reaching, app.js's `reads`),
# +70, at the top of its estimate (+30 to +70): 17,790, inside the ceiling.
# The operator's ceiling for capability-signed through capability-release
# (2026-10-24, "+535 / +180"): at most 18,325 and 3,330, each row raising
# only what it lands, to one past (demo/runs/authorship/2026-10-24/
# DESIGN.md §14). capability-signed, each device signing its own acts: a
# holder's pass on its line and Grant judging it, the keeper's cutover, the
# chained pass (H), /carry, the phone's worker and group page signing,
# +472 and +144, inside its estimate (+253 to +479, +96 to +168). Its
# economizing cut 101 and 75 first: RailGap and InviteFault's words by
# thiserror (-63 core), Device's serde by derive (-12), the runtime's
# standing_anywhere and removed_anywhere said as visit_anywhere (-23), the
# lend worker's wasm wait (-3). 18,263 and 3,295. capability-step-limit,
# each step of check.sh in a process group of its own, ended at a limit of
# 5 times its slowest passing run here, +6 after its economizing pass
# folded its own leftover-ending loop to one line (-2): 18,270.
# capability-keys, consent and authorship by first key: Runs and each
# act's writer keyed by it in svrngs-core, the carried route asking by it,
# and the pages telling two people of one name apart (question.js
# `called`), +14 and +3, under its estimate (+25 to +55, +5 to +10). Its
# economizing pass folded `written`'s changer to one line (-1); the rest
# it lands is struct fields rustfmt sets a line each and one JS line a
# site. 18,284 and 3,298. REVIEW-audit-capability-auto-1 gave /carry one
# const and its request one statement (-1): 18,283. keys-names, Money and
# Events by first key (apps/, uncounted), the node's answer naming whom an
# older act spoke for by key (rail.rs `ops_json`, not the core: the
# operator, 2026-10-25, "No extra core lines", then "Node"), and the
# node's lists by key beside the name, +55 and 0; its economizing pass
# first, chains parenthesized as the house writes them and bindings used
# once folded, cut 60 in svrngs-core and 31 in the node, so the caps come
# down to one past where it lands: 18,247 and 3,238. Its gate's fixes put
# the phone's four "whose computer did not say" sentences in one table, as
# the home page's (-5 after +1): 18,243. The lint's own line in check.sh
# (+1), ops_json's own-name guard folded into the name it reads (-2), and
# the home page drawn from a function its module does not await (-2):
# 18,240. give-economize's pass (demo/runs/economize/2026-10-25-give/
# SURVEY.md): the links' sequencer taken in one line (-6), an own pass's
# reads and the door's "unasked" each said in one statement (-4, -3):
# 18,227. give-token, the record asking every program for the token in
# the data dir's `token` file, under the operator's ceiling for give-token
# through give-play (2026-10-25, ralph/next/give/STATE.md give-economize,
# "+1,005 repository and +5 svrngs-core", at most 19,233): the token kept
# as the sign-in key is (`declared::write_secret`), its one check beside
# `from_this_computer`, `ask_running` and install.sh's curl showing it,
# +48 and 0, inside its estimate (+45 to +90, SURVEY.md §6), after its
# own cut of `NotLoopback`'s words to one line (-2): 18,275. give-session,
# under the same ceiling, its range +110 to +220 (SURVEY.md:222, top
# 18,495): a browser let in only by an open's session (session.rs, its
# checks in home.rs and origin.rs), a read's capability (+30), `svrngs
# open` (+26) and install.sh's opener (+6), after its own cut of `run`'s
# verbs printing once (-15, 2266a50a): +216 and 0, at 18,491.
BUDGET = {"crates/svrngs-core": 3239, ".": 18492}
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
