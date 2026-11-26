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
   One `unsafe` a home: every product crate root (crates/*/src/lib.rs and
   main.rs, web/src/lib.rs, Keeper's) denies `unsafe_code`, so the compiler
   refuses any other, and no source there exempts code from it but one
   function in each of `UNSAFE_HOMES`, libproc's read of sockets (DESIGN §5).
4. Lines of code — not blank, not a comment, not a test — stay under the
   caps `BUDGET` holds below, their one home (#197), scripts included. The
   budget counts the product: apps/, installed packages (any
   node_modules/), demo/runs/ and the pictures' generators
   (docs/diagrams/src/, docs/brand/src/) apart. A recorded run's driver is what ran,
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
# The files whose one function each may allow `unsafe_code` (uid.rs `kernel`).
UNSAFE_HOMES = {"crates/svrngs-node/src/uid.rs"}
# The pure availability decision and renderer, not the executor beside them.
PLAN_PURE = {"crates/svrngs-node/src/plan.rs", "crates/svrngs-node/src/plan/words.rs"}
PLAN_IO = re.compile(
    r"\b(?:async|await|SystemTime|Instant|SystemRandom|Command)\b"
    r"|\b(?:fs|io|net|env|process|thread|time|rand|tokio|reqwest|run)\s*(?:::|[,}])"
    r"|\b(?:include_bytes|include_str|env|option_env|print|println|eprint|eprintln)\s*!"
    r"|\bjob\s*::\s*(?!Keep\b)\w+"
)
# O3: app jobs are read here; only the executor may start the keeper. The
# command's own restart is outside the node and acts on svrngs itself.
NODE_COMMAND = {
    "plan/run.rs": 'Command::new(at.with_file_name(KEEPER))',
    "job.rs": 'Command::new("/usr/bin/systemctl").args(["--user","show","-p",asked,"--",unit])',
    "job/mac.rs": ('Command::new("/bin/launchctl").args(["print",&target])',
                   'Command::new("/bin/launchctl").args(["print-disabled",&domain])'),
    "home.rs": "Command::new(opener);",
    "network.rs": 'Command::new("/usr/sbin/scutil");',
    "update.rs": 'Command::new("tar")',
}
# N1: the keeper asks the OS to manage its own services, not individual processes.
PROCESS_SIGNAL = re.compile(r'\b(?:pidfd_send_signal|kill_process(?:_group)?)\b|\bkill\s*\(|\bCommand\s*::\s*new\s*\(\s*"(?:[^"\n]*/)?kill"')
# An exemption from the lint, however spelled: `allow`, `expect` or `warn`,
# alone or among other lints, inner or outer, under a `cfg_attr` too.
EXEMPT = re.compile(r"\b(allow|expect|warn)\s*\(([^()]*[,\s])?unsafe_code\b")
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
# verbs printing once (-15, 2266a50a): +216 and 0, at 18,491. give-udp,
# under the same ceiling, its range +135 to +250 and +2 to +5 core
# (SURVEY.md:223): `Alpn::Port`, a raw `Source`, `--udp`, and the UDP
# door (door/udp.rs), after its own pass over that file (383 counted at
# first, 199 at 36471346): +249 and +2, at 18,741 and 3,241. give-tcp,
# under the same ceiling, its range +60 to +130 and 0 core (SURVEY.md:224):
# `Raw::Tcp`, `--tcp`, the opening naming how a lend travels, and the TCP
# door (door/tcp.rs), sharing the UDP door's admission and opening
# (083cc6b6): +126 and 0, at 18,867. give-play, under the same ceiling,
# its range +165 to +310 and 0 core (SURVEY.md:225): Play's owner on the
# room's model (play.rs), sharing the room's store (64b96771) and port,
# `Offered.raw`, and the home page's row and line, after its own cut of
# 10: +226 and 0, at 19,094. give-cut, inside what that ceiling left, its
# range +10 to +40 and 0 core: the lender's sweep closing a raw lend's
# connection once its dialer no longer stands (door/udp.rs): +16 and 0, at
# 19,110.
# The page after joining (the operator, 2026-10-26: the shelf unticked and,
# just joined, folded; "What acts as you" folded; the group's things
# first), after its own economizing pass, the page's heading set by one
# assignment (-3): +1, at 19,108, so the cap lowered to one past it.
# give-install, after its own economizing pass over install.sh (-7,
# demo/runs/economize/2026-10-26-give-install/SURVEY.md, 60565068): Give's
# program as install.sh's --give, fetched, checked, placed and kept at
# login by the functions svrngs's own steps became, and built beside
# svrngs in build-release.sh (2d68d696): +13 net and 0, at 19,120
# (demo/runs/give/2026-10-26-install/NEEDS_HUMAN.md).
# The diagrams' generator (docs/diagrams/src/), the operator's raise
# (2026-10-27, "raise the budget for the generator"): its first set, kept so
# it can be drawn again from that commit (e5daca60), +406 and 0, at 19,523.
# Its economizing pass, the set cut to one point a picture and the kit to
# what that set draws: -93, at 19,430, so the cap lowered to one past it.
# give-redial, the operator's raise (2026-10-27, "30 lines approved", up to
# +30 and 0 core) on the director's A (demo/runs/redial/2026-10-27/
# DECISION.md): a request that fails on the connection kept to a computer
# closes every connection to it, the acceptor's included, before the dial
# afresh (iroh.rs `close_every`). Its economizing pass, the close folded
# into the count it traces (-2): +20 and 0, at 19,450.
# The svrngs mark's generator (docs/brand/src/), the operator's raise
# (2026-10-27, "Approve +53"): its geometry, three cuts, the lockup and the
# icon, and the diagrams' foot signed from it. Its economizing pass, one
# file wrapper for all five files in place of three, and the unused cutFor
# dropped (-21): +53 and 0, at 19,503. The rest is arcs the diagram kit has
# no helper for, and a mark the kit's figure frame must not wrap.
# The favicon, the operator's raise (2026-10-27, "Approve +6"): the mark's
# small cut as favicon.svg, white ink when the browser is dark, and the home
# page's route for it (home.rs `ICON`): +6 and 0, at 19,509.
# Swift on a rung of its own, the operator's (2026-10-30, ralph/next/ios/
# CHARTER.md): "Swift should count in its own rung. It will now be first
# party code that we maintain fully so we should account for it. It's not
# just demo code. But it's not the core core library which we're the utmost
# precious about". It counts .swift across the repository, and the other two
# caps count CODE, never Swift. The phone's app is the first; none is product
# yet, so the cap is one past 0, raised as the core's and the repository's are.
# The browser rows (svrn-docs rings/current/IN_THE_BROWSER.md, 2026-10-30),
# after their own economizing pass, the store's, the page's and the index's
# statements each said once (-56, demo/runs/economize/2026-10-30-browser/
# SURVEY.md): a lend's address opened cold through the group's page (+16),
# its alias (+15) and the address to keep (+4), at 19,488, so the cap
# lowered to one past it; svrngs-core's left at one past, unchanged. Then,
# from the operator's iPhone: a deep path opened cold by the site's
# 404.html (+2) and the join's Home Screen line in place of a phone's
# address to keep (+2), after `hex` said once and the cold guard folded
# (-4): at 19,488 again, the cap unchanged. The follow-ups after their own
# pass, uses, el and the worker's small functions each one expression (-28,
# demo/runs/economize/2026-10-30-browser/SURVEY.md): the alias's day in UTC
# (0), a computer's cold bookmark by the home page's /to (+8) and a person in
# two groups (+12), at 19,480, so the cap lowered to one past it.
# The pictures' generators uncounted, the operator's (2026-10-31, "Don't
# count diagram code"), when the guide's 15 pictures (docs/diagrams/guide/)
# came to +486 after their own economizing pass (-87, every SVG byte for
# byte): docs/diagrams/src/ and docs/brand/src/ draw pictures, not the
# product, as apps/ is not. The 10-27 raises for them (+406, +53) leave with
# them: -366 at c0c466fe, at 19,114, so the cap lowered to one past it.
# The operator's raise (2026-10-31, refuse rather than fall back): an app
# brought from the shelf lands in the group's own folder, found by the
# folder's name from the group's seed, and is refused where this computer
# keeps none (carried.rs `brought`, group.rs `folder_name`): +7, 0 core, at
# 19,121.
# The pages' v2 on a rung of its own while it is built beside v1, the
# operator's (2026-10-31, "Accept", ralph/next/page/STATE.md "Decided before
# launch" 1): v2's files, its vendored libraries and the parity report count
# on `v2`, capped at 1,150 (the queue's estimate, +815 to +1,112), and `.`
# skips them. page-switchover deletes the rung: `.` takes v2 back once v1 is
# deleted, after its own economizing pass, and asks a raise for what is left.
# The operator's raise (2026-11-01, "Raise the counts"): page-members to
# +180, page-join to +165 and page-acts to +190, each past its range after
# its own economizing pass (demo/runs/page/2026-10-31-members-range,
# 2026-11-01-join-range, 2026-11-01-acts-range). v2 671 landed and those
# three tops come to 1,206, so the cap is one past it.
# The operator's raise for the switchover (2026-11-01, "Yes to both", after
# page-switchover (a)'s pass, demo/runs/economize/2026-11-01-page-switchover/
# SURVEY.md): v2's imports in one line from ui/kit.js and one `post` (-71).
# `.` goes to one past the swap's estimate, 19,418 to 19,446: v2's 1,008 to
# 1,011 lines that stay count again and v1's 703 go. The swap lowers it to
# one past where it lands, and deletes `v2` with the rung.
# The switchover (2026-11-01, page-switchover (b)-(f), the operator's "Yes
# to both"): the rung and its paths are gone, so `.` counts the pages'
# flows, ui/ and the vendored Preact and htm again; v1's pages, parity.sh,
# serve.rs and the switch's hooks went. `.` landed at 19,421, inside the
# estimate, so its cap is one past it. Merged with main's relay `--at`
# (227193b1, +14), it lands at 19,435, under the operator's 19,447, so the
# cap is one past that.
# The operator's raise for the security work's six parts (2026-11-02, "As
# recommended": one raise to one past where it lands after the fixes): +843,
# all outside the core. Part 6, the per-origin browser key, is +675
# (session.rs +358, origin.rs +101, the three session*.js +188); confined
# folders +122; advisories.py +33; secrets +13. The pass cut 17 lines: origin.rs's
# inline tests out, one `host()`, a duplicated holder check, one signer
# derivation. session.rs's `Onwards` store (+111) is the next pass's first
# candidate. The seventh part waits on branch security-part7.
# The seventh part on design D, the operator's (2026-11-02, "as
# recommended"): +132 in the core (the pass's approver and its second
# signature check) and +456 in all (with the signer, web/sign.js, and its
# seal), at 3,373 and 20,734. The pass cut the wasm `approved()`, a second
# act-kind check in `approving()` and a line of `approval_message`.
# JavaScript on a rung of its own, the operator's (2026-11-02, ralph/next/js/
# CHARTER.md, "Answered" 2): "we'll just have a line count jubilee at the
# end, also I want to track js independently". `js` counts .js and .mjs with
# the same exclusions, and `.` no longer does: 2,233 of its 20,734 moved and
# this rung's own 4 came in, so `.` is one past 18,505. `js` stayed uncapped
# until the queue's jubilee below; its count was always printed.
# Relay fallbacks in tiers, the operator's raise (2026-11-02, "Approve the
# raise"; svrn-docs rings/current/GROUP_RELAYS.md §7): +104 in the core (the
# keeper's `Meets` act and its fold, `MeetsAt`, the two-tier `Meeting`, the
# two-relay link) and +355 in `.` net, at 3,477 and 18,860 (105df0bf..424d1dfc).
# The pass cut 17 lines first: a dial's addresses kept once (0f08ee6a), one
# way to make a contact (c0212347), one reader of a group's page (77768a50);
# it found nothing said twice in link.rs, relay.rs, membership.rs or
# register.rs. The rest is new: the tiers module, multi-relay dials and links.
# js-lint's type-aware lint (ralph/next/js/CHARTER.md "Answered" 4), the
# operator's raise (2026-11-03, "Approve +1"): +1 in `.`, the gate's `typed
# lint` step (check.sh, 94192670), at 18,861. The pass found nothing to cut:
# every check.sh line is a step or a verdict, and no script holds a function
# never called (ralph/next/js/NEEDS_HUMAN.md, "`.` is at its cap").
# The js queue's jubilee, preauthorized by CHARTER "Answered" 2: its final
# 7,338 lines set js's cap one past, 7,339. Removing uncapped mode cuts three
# Python lines, so . lands at 18,858 and its cap lowers to one past too.
# The +1 JavaScript plant and the restored count are recorded in
# demo/runs/economize/2026-11-03-js-read/CLOSE.md.
# The operator's raise for the home page's own svrngs (2026-11-03, "Let's
# do core + surface 1"): +388 in `.` and +145 in `js` over the js jubilee's
# one-past caps. The pass cut 44 first: the update's thirteen error variants
# folded to seven, the identical-binary re-compare dropped, the update's
# place one fixed directory, and the page this computer came from read by
# invite.rs `kept_page` (the command's HTTP client moved to the node's one
# home, net +3). What remains is each decision the gate names: the page's
# checksum, verified download, unpack, regular-file guard, swap and three
# verdicts, each traced; the home page's machine and view; and the one
# release hash install.sh keeps. No line is said twice in the touched files,
# and the rest cannot be cut without dropping one of those checks or a
# verdict.
# Join-while-running, the operator's ask (2026-11-03, "a pretty glaring
# feature gap: join another ring while svrngs runs"): a second group joined
# through the svrngs running here — the record's POST /v1/joins and its
# closed answer in svrngs-node join.rs, the command's ask and wording in
# svrngs cli.rs — at 19,053. The pass wrote both once and folded what a
# first draft said twice (the two success arms into one placement,
# NotJoined's mapping into Outcome::of, the paths aliased): about 30 of
# the draft's lines never landed. It looked for behaviour-keeping cuts
# beside them in the files the route joins — node.rs, api.rs, surface.rs,
# home.rs, group.rs, folder.rs, removal.rs — and found none: each holds
# one decider the design names, and nothing is said twice across them.
# Merged onto main's update raise (c85fcbeb), the tree lands at 19,441;
# the operator approved both caps 2026-11-03 ("Approve both"): `.` to
# 19,442 and `js` to 7,500, the js overage the page's stale-worker fix
# added (d6d2a120, 15 over its 7,484).
# Onboarding's upper estimate, the operator's approval (2026-11-03, "Budget
# approved"): +16 core, +391 `.` and +590 `js` over those caps, so 3,494,
# 19,833 and 8,090. Its current-base pass found no supported
# behavior-preserving product cut (the public/dead-code, test-cfg,
# line-window and page-structure instruments left no deletion to fund);
# semantic clone analysis could-not-judge because the tool was absent. Each
# landing still raises only to where it lands
# (ralph/next/onboarding/NEEDS_HUMAN.md).
# Grouped-home B, operator-approved measured remainder (2026-11-04): js
# 8,187 needs the exclusive cap 8,188, +98; core and non-JS caps unchanged.
# The pass established no preserving cut: it reuses existing components,
# machines, folder opener, native links and version data. What remains is
# the requested navigation/context/disclosure presentation; deleting it
# drops B behavior or the A/B comparison (ralph/next/onboarding/
# GROUPED_HOME_RECONCILE.md). This spends no lending approval.
# Separate rungs let the parallel backend/UI lanes adjust only their own cap.
# The relay-fallback queue's landing, the operator's decision (2026-11-05):
# the keeper meets at the advertised public fallback while it answers, the
# page takes every route its policy allows and races them; +80 over the
# cap, raised to one past where it lands: 19,913.
# The review repair, the operator's decision (2026-11-05, "raise to fit, add
# backoff"): an independent review found six defects the queue's audit passed
# (a fallback held on a bare TCP connect, one miss flipping the tier, links
# minted with no https route, a refused bind cached for good, visits racing
# one route, old passes throwing) and the re-entry backoff; `.` +33 and js +23
# over, each raised to one past where it lands: 19,946 and 8,212.
# Selected-ring lending's backend (onboarding-scoped-lending), under the
# operator's preapproval of its minimal necessary growth (2026-11-04,
# ralph/next/onboarding/CHARTER.md "Selected-ring lending"): `.` 20,331
# needs the exclusive cap 20,332, +386 over the 19,945 the relay-fallback
# landing left (the 2547d91c checkpoint +272, this row's finish +114); no
# core or js change. The fresh pass (docs/LENDING_SCOPE.md, resumed at
# b484b2b3) established no preserving cut; this row's one, `Lend::reaches`
# for three copies of its rule (95fa27b5, -12 lines), is counted.
# The rest is new policy: the audience, its witness, list and header
# scope, OIDC provenance, the edit lock, `share --group`, and cutting
# streams; within the +700 review bound.
# The seat's review of that row (2026-11-05), the same preapproval: the
# legacy stamp (7a8af0ac) and current membership — the lender's standing,
# a selection's, an OIDC code's (23caceac) — take `.` to 20,393, the
# exclusive cap 20,394, +62; core and js unchanged. +448 over the 19,945
# baseline in all, inside the +700 bound; no further preserving cut found.
# Guests by one decider (seat, 2026-11-05): the guest watch and a guest's
# code asked by guest_of (4fba7e59, bed790e6, 4ae0d111) take `.` to
# 20,430, the exclusive cap 20,431, +36; +485 over 19,945 in all.
# Selected-group audience UI, operator-preapproved (SCOPE_PARALLEL.md,
# "Checks"; the original combined guardrail is +235 JS): the two lending
# machines take `choose`/`audience` and carry `groups`, the rows draw the
# boxes and read paused []/legacy null, the audience threads from HOME's
# groups into found.js and lent-row.js, and web/page/dial.js `lends` names
# its svrngs version with `x-svrngs-version`, so the measured js is 8,300
# and its exclusive cap one past is 8,301, +113 over grouped-home B's 8,187.
# The economizing pass reused the existing machines, parts and group data,
# factored the boxes into the shared home/page/group-choice.js (found.js and
# lent-row.js each lost their local box markup and `tick`), and folded the
# discovery header into a one-line `SCOPED(core)` beside PEOPLE_LENDS; no
# preserving cut of the remaining audience presentation was found — deleting
# it drops the explicit-choice behavior the row requires. Each law in
# web/tests/pages/lend-groups.test.mjs failed on a named plant (no `choose`;
# `choose` replacing instead of merging; [] read as legacy; no maker in the
# label) and web/tests/pages/discovery-header.test.mjs on the header removed,
# before passing. Core and non-JS caps are unchanged; a served-module entry
# (`group-choice`) rides the existing home.rs table line, so the `.` rung
# does not move.
# Merged with the backend (merge-scope-ui): the UI lane's +113 under its
# preapproval above, on top of main's 8,211, which already carries
# relay-fallback's +24: 8,324 measured after the merge, the cap 8,325.
# The merge's integration fix in lent-row.js (absent groups) adds one: 8,326.
# The independent audit's findings (ralph/REVIEW_FINDINGS.md,
# "REVIEW-audit-onboarding-auto-1 (2026-11-05)"), fixed under the same
# lending preapproval: F4 the re-declaration's lock (+1), F5/F6 one
# `stands_here` and `door::reached` at redemption (+4), F12 a legacy
# `unshare --group` refused (+6), F8 `stamp`'s traces (+15) take `.` from
# The public resolver adapter, relay-meet's: +8 js, within the operator's
# "Also raise the js cap 50 lines" (2026-11-04), which main never took:
# 8,326 to 8,334. Nothing else moves.
# the audit's 20,428 to 20,454, cap 20,455 (lending +509 of its +700); F10
# a row naming every id (+4) takes js to 8,329, cap 8,330. Then the gate's
# app-nouns fix drops group-choice.js's cast (-4), so js lands at 8,325 and
# the cap comes down to where it lands, 8,326.
# The phone's readiness check and its home-network words (KEEP_GOING_AT_HOME
# 29, 21): +30 js, the operator's decision of 2026-11-06, after about 95 lines
# of duplicates were cut for the slice. 8,334 to 8,364.
# The operator's decision of 2026-11-06, "Raise to 20,476": `svrngs restart`,
# asking launchd or systemd so it works on any running build, and its refusal
# where neither keeps svrngs. 20,455 to 20,476. The operator's decision of
# 2026-11-06, "Raise to 20,498": `version::STEPS`, so a raise must say what it
# does to older computers (+16); `Refusal::Older`; the token line a program is
# told at a lend's origin, restored; check.sh's fast stage. 20,476 to 20,498.
# The operator's decision of 2026-11-07, "Raise to 3511/20651/8385": a carried
# app is one label in its group (`Label::carried`, `Label::lent`), the computer
# serving it chosen at dial time (origin.rs `serving`, lend.js `lenderOf`), a
# phone's line carried on to the next computer, `older` and the scope marker
# read from `STEPS` so wire version 6 cuts no computer at 5 off, and a page
# kept at an old label told where its app went. core 3,494 to 3,511; . 20,498
# to 20,651; js 8,364 to 8,385.
# The operator's decision of 2026-11-07, "8396 / 20835 with trace": the
# group's page asks again while no computer answers its read (enter.js
# `failing`, +8 js); the touched tier, check.sh's slow steps a change can
# reach (scripts/touched.py and its wiring, +159 .); and the page's opt-in
# trace of iroh's own events (web/src/lib.rs `trace_iroh`, +25 ., +3 js).
# . 20,651 to 20,835; js 8,385 to 8,396; core unchanged.
# The readers' secret (operator, 2026-11-07: "12 lines is fine. Let's clear
# the other blockers too"): the door's gate to another app's GET or HEAD
# (door.rs carry), its store inside the lend's secret (lend.rs
# `readers_dir`, `keep_readers`), `svrngs share --readers` and the table's
# `readers:` row (share.rs `readers_of`, `sign_in_readers`), +115. Its
# economizing pass (1bd95b84) cut 178 first, so no raise: . lowered from
# 20,835 to one past where both land, 20,772.
# Carried apps read lends (998a3047: reads.txt, /svrngs-reads and
# /svrngs-lend at the app's own origin, the operator's "Same-origin route",
# 2026-11-08), +72. Its economizing pass (cb414021) cut 117 first, so no
# raise: . lowered from 20,772 to one past where both land, 20,723.
# Its hardening after review (a read made anew, an answer made anew from data
# types, a yes kept with what it named, readers kept off the record), +70 after
# its own trims; an economizing pass of 72 first, so no raise: . to 20,721.
# A raise of 33, inside the operator's "We can just add 35 lines here"
# (2026-11-08, docs/LESSONS_FROM_BITTORRENT.md): the home page's waiting line
# +27, the Blobs dial's receive window +6. Its pass cut nothing already there;
# both were tightened as written. . to 20,754, one past where they land.
# (2026-11-09) An economizing pass cut RoundMembers and ring_sync/journal.rs
# (28), moved Node::start, which only tests reach, to tests_support (10) and
# cut format_dial_string (8, one trace its caller); . to 20,710.
# (2026-11-09) Raised 77 by the operator's "Ok agreed" to funding the send
# limit alone (demo/runs/swarm/2026-11-09/): folder.rs Sends, turns across
# fetchers. The pass above found the 44 it could; what remains is new state
# and a select loop no code there already held. . to 20,787.
# (2026-11-09) Economized ahead of capability connections steps 5 to 8, 33
# counted lines: forwarding, one-caller wrappers, a field only ever true and
# what the door and the record API each held a second handle of (the commit's
# body names each cut). Lowered to one past where it landed; . to 20,857.
# (2026-11-09) A derive for origin.rs Lenders' hand-written Default (tokio's
# watch::Sender has its own); . to 20,883, one past where it lands.
# (2026-11-09) origin.rs dialled, one caller, inlined into home.rs at_door (5) and
# parts.js Said, no product caller (3); . to 20,878, js to 8,472.
# (2026-11-09) door/limit.rs share_of, one caller, inlined (3); . to 20,875.
# (2026-11-09) Sessions::proof_of and update.rs write_release, one caller each,
# inlined (5); . to 20,873.
# (2026-11-09) update.rs's and http.rs's test modules moved to their own files
# (update/tests.rs, http/tests.rs), where the count does not read them (45); . to 20,828.
# (2026-11-09) oplog.rs read a log's lines one way in two places; api.rs's Bound and
# Serving each gave a field through a getter; the home page's `again` effect, two
# copies. . to 20,808, js to 8,494, one past where each lands.
# G1 (EXTENSION_POINT §7), the write limit, inside the operator's ceiling
# ("Ceiling (Recommended)", . 20,900, js 8,500, 2026-11-09), after the pass
# over origin, update, http, oplog, api and the home page's effects
# (2add1674, bbfeadbe; -65 of . of which 45 are tests moved where the count
# does not read them, -2 of js): . +63 to 20,871, js +5 to 8,499.
# The refill stops at empty, so a bulk import through the token cannot hold an
# app back (+4), inside the operator's ceiling (cap 20,900, 2026-11-09); . to 20,873.
# Programs as files: build.rs embeds one file per program and refuses a row of
# another program (+10), inside the operator's ceiling (cap 20,900, 2026-11-09),
# after searching the touched files; . to 20,883.
# (2026-11-09) AddressCopy's own copy machine, a second COPY beside ui/copy.js's
# (address-copying.js, 37 lines), became link-copy.js LinkCopy, which also says
# only its last answer, with the same words: js to 8,465, one past where it lands.
# (2026-11-09) The operator raised the ceiling to . 21,150 for queue rows 9, 10 and 12
# of EXTENSION_POINT §10, js unchanged at 8,500. readers: key, a pasted key for a
# program that takes one (+42 of ., +25 of js), after 2add1674, bbfeadbe and 59d67bd2;
# . to 20,925, js to 8,490.
# (2026-11-09) Each rung printed by a name a person reads (+3, this file is
# counted), inside the operator's ceiling of 21,150: . to 20,928.
# (2026-11-09) G2, an app's reads of a lend: 600 at once, then ten a second
# (+36, js +2), inside the operator's ceiling for rows 10 and 12: . to 20,964,
# js to 8,492.
# (2026-11-09) An app's routes parsed once, surface.rs `AppRoute` (+27, the
# enum and its parser in place of 12 lines of comparisons), inside the
# operator's ceiling for rows 10 and 12: . to 20,991.
# (2026-11-09) G3, `svrngs check <folder>` and the card's words from one
# function, folder/check.rs (+119, js -1), inside the operator's ceiling for
# rows 10 and 12: . to 21,110, js to 8,491.
# (2026-11-09) G5, an author's own change runs for them at once, carried.rs
# `runs_as` (+15), inside the operator's ceiling for rows 10 and 12: . to
# 21,125.
# (2026-11-09) The economizing pass before row 12 lowers it to where it
# landed: e05c81c2 to 56d62e46 cut 39 lines of code, b047c0fc moved
# `svrngs --help`'s 125 lines of words out of cli.rs into a file, as the
# guide is: . to 20,961.
# (2026-11-09) Row 12, a group's own programs/<word>.txt read through its
# record (+125), inside the operator's ceiling for rows 10 and 12: . to
# 21,086.
# (2026-11-09) The gaps after row 12 (d1c0c0e7 to 6695b96a): an economizing
# pass of -42 (d1c0c0e7, 2221a146, 70830209) paid for gaps 1, 2, 3, 4 and 7
# (+38), leaving 21,081. Gap 5, a pasted key tried once against the
# lender's own service (+34), inside the operator's ceiling for rows 10 and
# 12: . to 21,116.
# (2026-11-09) The operator raised the ceilings for gap 6 and row 13 to
# . 21,350 and js 8,560, core unchanged. Gap 6, the home page's lend keeps
# a group's own terms as shown (+73), after the pass f03cbf64 (js -7, which
# lowers js to where gap 6 lands, 8,487): . to 21,189, js to 8,488.
# (2026-11-09) Row 13, `sends:` and `sends.txt` (+166, js +29), after the
# pass b97e31ec to bfad7743 (. -59, js -16, to 21,129 and 8,471), inside
# the operator's ceilings for gap 6 and row 13: . to 21,296, js to 8,501.
# (2026-11-10) The operator's raise ("Raise by 6"): a phone's carried app
# once let read friends' lends has no entry approved (hand.js `writesBlocked`,
# its trace; sign.js the refusal, its words, `Told`, `AGAIN`), +6 js. The pass
# cut the draft from 14 to 6 lines; the signer's two lapsed-record sweeps
# differ, and nothing else in hand.js or sign.js is said twice: js to 8,507.
# (2026-11-11) Before Eudoxia native integration, share authoring/verification's
# signed-field projection in sig.rs: -4 core and non-JS, golden bytes unchanged.
# Bank that pass; Eudoxia integration replaces authority code and must shrink it.
# R0 shares eligible-group identity across four door paths: -1 non-JS, banked.
# (2026-11-12) Release 2 (svrn-docs rings/current/EUDOXIA_JUDGE.md §2a): the
# old fold and adoption deleted, every group decided by Eudoxia's
# interpreter; the caps ratchet down to where it landed (core 4,618 ->
# 3,773, . 22,666 -> 21,588, js 8,544 -> 8,527).
# (2026-11-12) R1 through rules (EUDOXIA_JUDGE §8 step 4): what a pass lets
# its holder do is decided by eudoxia-community's rules; authority.rs only
# reads passes and lines for them, and the fault enums moved with the rules
# (core 3,773 -> 3,558, . 21,588 -> 21,375).
# (2026-11-12) R3/R4 through rules where it shrinks svrngs (EUDOXIA_JUDGE §8
# step 5): the lend's credential, the person's yes or stop, a page's reach
# and a carried app's send, the stream's recheck and a reading app's class
# are eudoxia-community's `lend` and `app` rules; the ingresses whose reads
# would cost lines stay native (svrn-docs eudoxia-runs/2026-11-12/
# relocate.md). . 21,375 -> 21,323, core unchanged.
# (2026-11-12) Economized after R3/R4 so the authority work leaves svrngs
# under its 29,765 baseline, with room for guest slots (R5): items only tests
# call moved to test files (Pass::approve, authority::approving, Pass::carries,
# Seed::policy), one reader of a kept JSON file (declared.rs read_json), SendRow
# and RefusedPath through the newtype macros, the wasm Group's getters
# generated, and the home page's lend given its lender once (question.js
# lentBy). Core 3,558 -> 3,527, . 21,323 -> 21,228, js 8,527 -> 8,516; native
# 29,850 -> 29,744.
# (2026-11-12) Guest slots per computer, judged by Eudoxia's interpreter along
# the host's chain: the core's timestamp count of visits (`visiting`, its two
# constants) and `serve_visit`'s recount after the write went. Core 3,527 ->
# 3,496, . 21,228 -> 21,196; native 29,744 -> 29,712.
# (2026-11-12) A guest's consent to its host on each visit (+21), with an
# economizing pass that keeps behavior: SyncPath and Origin through
# text_newtype (synced.rs -7, link.rs -2), Payload's Serialize derived
# transparent (payload.rs -5), authority::check folded into its one caller
# (-3), the door's three choice writes through one `say` (choices.rs -3),
# rail.rs's log and people answering Json (-2), the runtime's journal() in
# RingRail's first impl (-2), principles_check.py reading ports.py's ROOT,
# UPSTREAM and git and trust_check.py lean_check's ROOT and TEST_FILE (-5),
# update.js's and copy.js's identical cases stacked (-3), the session's hex
# shared by its worker (-4). Core 3,496 -> 3,492, . 21,196 -> 21,187, js
# 8,516 -> 8,510; native 29,712 -> 29,697.
# (2026-11-13) Economized before the seams' harnesses (docs/CHECKS.md,
# "Seam migration ledger"): ring_sync.rs's `Reach`, which only re-read an
# exchange's stop, and `RingSyncHandle`, a newtype over the abort handle;
# guest.rs's relay choice made once for the endpoint and `build`;
# svrngs-runtime's invite reads through `admission`; group.rs `read_relay`
# and api.rs's Infallible match. . 21,185 -> 21,161; the cap is lowered
# where the harness this pays for lands.
# (2026-11-13) Harness 4, iroh.rs `Fetch` (+8), a member asked, the one
# trait at the boundary ring_sync::To and RailApi::transport cross; . to
# 21,169, the cap one past it, 18 under where the pass found it.
# (2026-11-13) Raised by the operator for the keeper-set guest setting:
# `RailAct::GuestSlots`, `Visit::under`, `SlotsAt`, `Membership::slots`, the
# adapter, `Host::slots`, `POST /slots`, the group's number on /v1/groups
# and the SLOTS row on the home page. Core 3,492 -> 3,526, . 21,169 ->
# 21,240, js 8,510 -> 8,552; native 29,679 -> 29,792. The economizing pass
# before it (81f5c577) cut 36; the rest found (about 23 lines, uncertain)
# would have left it about 22 over.
# (2026-11-13) Harness 3 (docs/CHECKS.md), a node's network half behind
# iroh.rs `Net`, so a node runs with no socket: +85, within the operator's
# "raise the non-JS cap by up to +90 for harness 3" (2026-11-13, through
# AskUserQuestion, after the economizing pass 9787472e, -24). . 21,240 ->
# 21,325.
# (2026-11-13) A node's folders end with it (node.rs `Stop`): their tasks
# under `Host::stopping`, cancelled at its drop, and the blob store shut,
# where a task that never ended held it past the node (+8); paid by
# cli.rs `serve`'s unreachable arm, its thirteen verbs one `_` (-14).
# . 21,325 -> 21,319.
# (2026-11-13) A group kept in one step (node.rs `Host::meeting`): a round's
# cut over no longer reads a group seeded and not yet said to meet, and
# writes `MeetsAt({})` over its founder's (flakes 2026-11-12, row 201), +3;
# paid by cli.rs `checked_page` taking the `--origin` that `new` and `link`
# each matched on (-5). . 21,319 -> 21,317.
# (2026-11-13) The touched tier runs no fragile test (scripts/touched.py
# `deterministic`, its plan's fold; docs/CHECKS.md, "The touched tier"),
# +35; paid by the economizing pass a808146d (svrngs-core, -25) and
# d2e6d8c7 (svrngs-node, -35). Core 3,526 -> 3,501; . 21,317 -> 21,292.
# (2026-11-14) The room's network beside a VPN (network.rs `lan`,
# `service_on`; room.rs `said_beside`): the interface the devices here are
# on, not the tunnel a VPN carries the traffic by, on Linux and macOS, and
# the VPN said where it may refuse them. +100, raised by the operator
# ("Raise the cap", 2026-11-14) after the change's own economizing pass,
# -32: netdev's `Interface` judged as it is, not copied into a struct of
# our own (-19), the owner reading the read's fields in place (-4), and
# scutil's answers read by one `value` (-9). The rest is the feature: the
# choice, macOS's service by interface, and the VPN's sentence.
# . 21,292 -> 21,360.
# gate-podman (2026-11-15): the room's serve returning early with no lend
# on, and `Here::unread` built in place (b9093fe9, -8), made room for the
# gate image's entrypoint, scripts/gate/delegate.sh (+7), with no raise;
# lowered to where both land. . 21,360 -> 21,359.
# (2026-11-16) The home page's group at scale: what a group shares drawn as
# grids of cells under captions that name nothing, where a table and its
# name-listing sentences were (index-rows.js gone, question.js `owner`, the
# `above` and `quiet` plumbing); each thing's address in its own cell's
# popover, where a fold's place listed every thing again (wifi-foot.js's
# place gone). No raise; the cuts lower the cap to where they land.
# js 8,552 -> 8,537.
# (2026-11-16) A person in a group's door can be pressed (the people the
# operator sat with each tried to): a popover of the browser's own says
# their state, when they joined, what they share here as jumps to those
# cells, and for whoever keeps the removals the whole line that removes
# them. +29, raised by the operator ("Raise to 8,566", 2026-11-16) after
# the economizing above and the accessibility pass's own: five single-use
# files folded into their one user (play-cell, lend-address, address-copy,
# signing-on, screen-link, -9, spent on every control's name and the
# popover's Close). The rest is the feature: about 3 more lines were found
# (people.js and removals.js into door.js), not 29. js 8,537 -> 8,566.
# It landed one under (b89c49e8), so the cap is where it landed. js 8,566 -> 8,565.
# (2026-11-16) No raise, by the operator ("Hold; economize first"): an
# economizing pass (493e12d0..1410e3e4, -57: ten single-use home files folded
# into their one user, the phone page's single-caller helpers inlined,
# `base64url` held once) made room for removing a person by two presses in
# their popover where a line was pasted in a terminal (95acdd8c, +29), each
# caption saying where its things run (e9931c6e, +7), "Add an app" saying
# how (22855229, 0) and a Stop named for its app (e315d5d8, +1). The cap
# comes down to where they land. js 8,565 -> 8,545.
# (2026-11-16) No raise: the invite flow's two economizing passes (35c53cc3,
# -12 js -23 other: the line's offer carries what was asked, publication says
# each thing once; 7dc3ffe4 and 551d6a1c, -49 js: the resolver's refusal codes
# a type, its EVAL's key count from its keys) made room for a phone joining
# from a short address (03e7ac52, 0), Windows offered no line (77d355a9), one
# Invite people press with a QR code (65074824, 57c170bb), a default resolver
# (51f5e669), and the resolver's new phrase and miss budgets (c327032f,
# 2241e350). The caps come down to where they land. js 8,545 -> 8,540;
# all code but JavaScript 21,359 -> 21,357.
# (2026-11-17) Raised by the operator ("Raise it and proceed"), asked after an
# economizing pass (837ff35f, -14 other -17 js: a lend's terms read once for
# every change on the home page, apps' account and key one box, the command's
# printed blocks whole, `lend_of`'s name and limit). The feature is headers the
# door adds to each request, given in a lend’s row on the home page (+38 other, +25 js): the
# change and its answer on /lent, one decider for it with `svrngs share`, the
# store's keeper, the name and value refused unless a header can carry them,
# and the door holding a file put there by hand to the same names. The rest
# was not found: what remained cost precision (cli.rs's one-sentence refusal
# variants merged, which tests match by name) or the closed set (`exit_code`'s
# exhaustive match). all code but JavaScript 21,357 -> 21,381; js 8,540 -> 8,535,
# where it lands.
# (2026-11-17) The economizing pass before the shared database (operator,
# "Now, own crate"; docs/PLAN.md §1, c0859400), over the code it extends:
# carried.rs, door.rs, surface.rs, ring_routes.rs and ring_sync.rs. /v1/groups
# fills each register field by name (57a31e07, -15), the door builds a pass
# holder's Asker once (bd06f75c, -10), and a round's namespace trace lists its
# skips as its offers (7c017670, -1). The cap comes down to where they land.
# All code but JavaScript 21,357 -> 21,331.
# (2026-11-17) The shared database's ceiling (operator, asked "Set one size
# ceiling for the whole database build, cut back to where it lands", "A) One
# ceiling"), for B1-B6 in svrn-docs rings/current/MAGNETS_BUILD.md: the core
# only a change set's signed message (+40), crates/svrngs-db and the node's
# routes (+1,700 in all), the phone's worker checking what it signs (+107).
# The pass above found 26 lines. The larger cuts in that code are ECONOMIZING_PLAN
# stages 1-4's, which follow and lower these caps. At the build's end each
# cap comes down to where it landed. The core 3,501 -> 3,539; all code but
# JavaScript 21,331 -> 23,031; js 8,540 -> 8,647.
# (2026-11-18) Raised by the operator for PR #1, svrngs as a package (svrn-docs
# rings/current/SVRNGS_PACKAGE.md: "Waits on the size-budget raise (+30 Rust,
# +2 JavaScript lines)"; "get [it] merged"). Its economizing pass (bb2e9334,
# -3 js: a socket the door didn't switch said closed in one place) holds the
# JavaScript, +3 with the update line's `packaged` case, under the cap. The
# rest is the feature (+32 other): `svrngs version`, who updates svrngs read
# from the package's `updated-by`, the can't-write guard before the download,
# their answers on the page and the command. Not found: merging the command's
# one-sentence refusal variants (tests match each by name) and leave/accept
# into one arm (8 lines formatted, not 6); `exit_code`'s match stays
# exhaustive (AGENTS 9). all code but JavaScript 21,381 -> 21,413.
# (2026-11-18) The two lines of work merged: the economizing pass's -26 and
# the headers and package raises' +56 leave all code but JavaScript at 21,387
# and JavaScript at 8,535. The shared database's ceiling keeps its +1,700 and
# +107 over them, and the core's 3,539. All code but JavaScript -> 23,087; js
# -> 8,642.
# (2026-11-18) Raised by the operator for PR #2, a lend whose program is not
# running said so before it is opened (door.rs `answering`, lend.rs
# `Offered::answering`). Its economizing pass (-3 js: lent-row.js's words
# say `down` where they are made; session.js and session-worker.js drop a
# page's sized headers in one statement) holds the JavaScript at the cap.
# The rest is the feature (+14 other): the dial, its wait, the field. Not
# found: folding the port into `offered`'s match, which rustfmt splits to
# +9; the push routes' twin handlers, already a line each.
# all code but JavaScript 21,413 -> 21,427.
# (2026-11-18) Merged again, with V1.11's release: PR #2's +14 leaves all
# code but JavaScript at 21,401. The shared database's ceiling keeps its
# +1,700 over it. All code but JavaScript -> 23,101; js and the core as
# above.
# (2026-11-18) Raised by the operator for PR #3 (827d99aa), `notRunning`
# for `down`: ralph/next/js/PROMPT.addendum.md fence 1 refuses a name
# shortened to fit, and lend-line.js's import is as the formatter lays it,
# one name a line. +9, the name kept whole; no pass cuts it. Its economizing
# came before (59851ecb, -1). JavaScript 8,535 -> 8,544.
# (2026-11-18) Merged with PR #3: the shared database's ceiling keeps its
# +107 of JavaScript over the 8,544 PR #3 leaves. JavaScript -> 8,651.
# (2026-11-19) Raised by the operator for webrtc-page (relay-commons): the
# window holds the direct path's RTCPeerConnection, since a worker cannot
# (shim.js +25), and lend.js starts the attempt (+31). +56 against 40 free;
# the pass's inlinings break the linter's complexity bound and
# lend-worker.test.mjs, so +16. JavaScript 8,651 -> 8,667.
# (2026-11-19) Raised again by the operator: self-host (b3ae73f2) took +7
# of that for the group's own relay on the home page, so webrtc-page's +56
# needs 8,674. JavaScript 8,667 -> 8,674.
# (2026-11-19) The relay queue (ralph/relay-commons) rebased onto V1.12.2: each
# fit alone, together all code but JavaScript 23,795 and JavaScript 8,687. The
# operator chose an economizing pass (2026-11-19: "Do an economize pass"), not
# a raise: these are the merge's counts as the pass's start, each of its cuts
# to land the counts back under 23,101 and 8,674, and the caps then lowered to
# where they land.
# (2026-11-19) The pass landed: a first round of cuts (49 commits) and the
# architectural one (C3, C1, C2, L1, N2, N3; 7cf71342..f24b2937) leave all
# code but JavaScript at 23,311 (8d61f819's -3 with them) and JavaScript at
# 8,666. The caps are lowered to there. All code but JavaScript stays 210
# over 23,101: a raise, the operator's, and granted (2026-11-19: "Approved").
# (2026-11-20) Raised by the operator for V1.13.1's fixes from the phone test
# (demo/runs/phone-home/2026-11-20): back to direct +23, the seek window +5,
# an idle folder at rest +5, after a pass that cut -12 (the direct attempt's
# steps in go), -2 (a transport config no dial reads), -1 (the trace
# collector) and -6 (the folder walk's second stat). All code but JavaScript
# 23,311 -> 23,323 (2026-11-20: "Approved").
# (2026-11-20) The share binding (a share bound to the socket or process the
# person approved) lands on V1.13.0: all code but JavaScript 24,275 (V1.13.1's
# 12, above, among them) and JavaScript 8,693 while it lands, 24,150 once its
# rules move to Eudoxia. A
# raise without a pass, the operator's one exception to cutting first: "Just
# raise caps -- I don't need breakages from line golfing … Cutting before
# raising is the norm. In this case you said cutting was leading to mangling
# and I have some urgency here." Its cuts follow as their own reviewed work,
# and lower the caps where they land. Its rules moved to Eudoxia (4e58132d),
# so all code but JavaScript is lowered to one past where that landed.
# Then 10 lines back for the approve route (surface.rs `process`, 6; the
# unread port, door.rs 3 and lend.rs 1): all code but JavaScript 24,160, the
# operator's word ("Raise it"), after a pass over the files it touches found
# no unused item and no repeat that wasn't layout or different words.
# Then 26 for an approval to cover only what was shown (share.rs 14, door.rs
# 9, uid.rs 3): 24,186, the operator's yes to raising by what the release's
# review fixes need, each reported. Its pass cut 10 lines in those files.
# Then 18 for `svrngs approve` to show what answers and ask before it
# approves (cli.rs): 24,204. Its pass cut 5 there.
# Then 12 for a Mac to refuse a UDP socket on every address that lets
# another bind beside it (uid.rs 10, share.rs 2): 24,216. Its pass cut 1.
# Then 13 for a Mac's TCP socket to be approved by its id in this boot
# (uid.rs): 24,229. Its pass cut 9 there.
# Then 45 for a server's socket for each of svrngs's flows (pasta's, for
# rootless Podman) to be carried in its approved cgroup: door.rs 27,
# door/bound.rs 13, uid.rs 4, udp.rs 1. 24,274; the operator: "Land it".
# Then 8 for a socket's peer to be read only where cgroups keep accounts
# apart and the socket is this account's (uid.rs): 24,282. Its pass cut 2.
# Then 1 for a Mac's port that only another account answers to be said as
# unread, not as nothing (door.rs): 24,283. Its pass cut 1.
# Then 12 for a closed flow to stay named while a socket is connected to it
# (door.rs, door/bound.rs): 24,295. Its pass cut 1.
# Then 15 for a flow checked while a server makes another flow's socket to
# be unjudged and stay open (door/udp.rs, uid.rs, door/bound.rs): 24,310;
# the operator: "Approved".
# Then 14 for a socket connected to nothing to be unjudged only while a new
# flow awaits it, and a flow opened while its port is unjudged to carry once
# it is (door.rs, door/udp.rs): 24,324. Its pass cut 4; the operator: "Yes".
# Then 62 for svrngs's command to show its token only to svrngs itself, as
# this account (uid.rs, api.rs, door/bound.rs, share.rs, cli.rs: 51), and
# for ::1 at its port to be svrngs's while it runs, held on Linux while off
# (api.rs: 15): 24,386. Its pass cut 26 (uid/Net `owner`'s `Side`, 7; one
# way to show a process, 19), after dbd9b24c and d30025b4 had cut 52 from
# the same files that day; the rest is the two readers themselves. The
# operator: "Raise by 62".
# Then 3 back, to 24,383: svrngs proves it holds the token before its
# command, its menu opener or Give sends it (api.rs 66, share.rs 5, cli.rs
# 5), in place of that command's process check (uid.rs 50, door/bound.rs
# 7 out), after a pass cut 22 from the files it touches (89501745: uid.rs
# `read`, `diag::socket`, `mac::Scan`, `linux::procs`, `waited`,
# `record::start`; cli.rs `parse`). Lowered to where it lands.
# Then 8 for an older svrngs, as an update leaves one running, to be
# restarted once by the command that finds it, as `svrngs restart` does
# (cli.rs `Here::ask`, 12), after a pass cut 4 there (6ce73f42: cli.rs
# `share_readers`): 24,391. The operator, 2026-11-21: "Restart
# automatically", approved as about 10 lines.
# Then 11 for that restart to come only where an older svrngs holds the data
# dir the login service runs, for the token's file to be said unreadable
# before anything is dialled, and for each answer to be bounded (cli.rs 25,
# api.rs 4, share.rs 2: 31), after a pass cut 20 from the same files
# (8ebd995c): 24,402. The operator, 2026-11-21: "Raise by 11".
# Then 117 for svrngs to take its token from no program, each asking with a
# code for its own connection, and for `svrngs ask` (api.rs 79, cli.rs 36,
# share.rs 6, node.rs and session.rs -4), after a pass cut what the files
# touched left unused (Sessions' token, `Bearer::file`, `Reached`): 24,519.
# The operator, 2026-11-21: "Raise up to 117", lowered to where it lands.
# Then 2 back, to 24,517: the token refused however a program sends it as a
# key (api.rs `Bearer::asked`, 3) and `svrngs ask` refusing a route http
# can't send (cli.rs `parse`, 2), paid for by what the record's refusal of
# another Bearer as no code left unused: `oidc::provider::bearer`, one
# caller's, read in userinfo (5), and `holds_the_token`'s Bearer arm (2).
# No raise was available; lowered to where it lands.
# (2026-11-22) The shared database's ceiling, pre-approved by the operator
# for its whole build, slices A to E (svrn-docs rings/current/MAGNETS_BUILD.md
# §3): +1,450 of all code but JavaScript and +80 of JavaScript, the core's
# +25 to +40 inside its own free lines ("A) +1,450 / +80 JS", over the
# estimate's middle). 2026-11-19's ceiling was raised whole and other work
# spent it, so this one is a pool, drawn by the seat. Each slice runs an
# economizing pass over the files it touches. Then the caps rise by what
# the slice needs, the caps come down to where it lands, and the pool's
# balance is written here. Only the database's own commits draw on it; other
# work asks for its own raise. Drawn so far: none. Left: +1,450 and +80.
# (2026-11-22) Slice A, Magnets on one laptop, draws +620 of all code but
# JavaScript. Its pass, over carried.rs, surface.rs, node.rs, lib.rs and
# door.rs, found nothing to cut. No item is unused, and each repeat costs as
# many lines to fold as it saves: the Retry-After refusal (carried.rs:635,
# door.rs:1847), the pass decode (door.rs:1360, :1434), "nothing is lent
# here" (surface.rs:502, :515, :725). All code but JavaScript 24,518 ->
# 25,138. Then +40 more, to the +660 its worker projects once cr-sqlite
# builds for Linux too (8ff2a8fd, 21985308): 25,178. A landed at 25,171,
# from 24,517: +654 of all code but JavaScript and none of JavaScript, and
# the cap comes down to it. Drawn so far: +654 (slice A). Left: +796 and +80.
# (2026-11-22) Slice B draws +23 for two fixes in svrngs-db: build.rs names
# cr-sqlite's archive by its object's hash, so a cache never links the engine
# before a fork bump (+3); a Db's connection closes as it drops, cr-sqlite's
# statements finalized first (+20). Their pass over svrngs-db's src/ and
# build.rs cut the w1 harness's flush of rusqlite's statement cache, which
# `Connection::close` already does, and found nothing else to cut. Drawn so
# far: +677 (A +654, B +23). Left: +773 and +80.
# (2026-11-22) Slice B draws +14 more: a new database's schema and its
# tables made CRRs in one transaction, so a refused schema keeps nothing and
# stays refused (lib.rs, `new_tables`). Its pass over svrngs-db found no
# unused item. Drawn so far: +691 (A +654, B +37). Left: +759 and +80.
# (2026-11-22) Slice B's sync and apply draw +485, to the top of their
# worker's estimate (+375 to +483 by file, the core's +30 to +38 inside its
# own free lines), before the build, so its commits land as it goes; the cap
# comes down to where it lands. Its pass over form.rs, ring_routes.rs,
# ring_sync.rs, the core's sig.rs, lib.rs, carried.rs and node.rs cut
# form.rs's one-line `refused` wrapper (-3, 58573586) and found no unused
# item else. From 25,205: the cap to 25,691. Drawn so far: +1,173 (A +654,
# B +519). Left: +277 and +80.
# (2026-11-22) Slice B's sync draws +50 more: the core and svrngs-db landed
# at +435 (5ed069a8, 7828cb73), the node's route and round at +94 more, so the
# half lands near +529 against the +483 estimated. rustfmt lays the core's
# signed message and svrngs-db's struct literals out a field a line, and
# waiting sets are held and judged again (about 30). The cuts left would
# undo a design choice: waiting sets refetched each round, or a page's reads
# kept on the runtime's threads. The cap to 25,741. Drawn so far: +1,223
# (A +654, B +569). Left: +227 and +80.
# (2026-11-22) Slice B's first half, sets, sync and apply, lands at 25,734
# (5ed069a8..200d0ad3), and the cap comes down to it. B so far: +563 from
# slice A's 25,171. Drawn so far: +1,217 (A +654, B +563). Left: +233 and
# +80.
# (2026-11-22) A pass that unwinds complexity, outside the database's files
# (d5a96cd8..f8135755: who serves a lend decided once a request, our relays'
# order in one place, legacy declarations found by Audience, the home page's
# member places named once, a group's section, Copy and UPDATE), lands at
# 25,717 and 8,670, and the caps come down to it: -17 and -23. Then the
# outside review's fixes to slice B's sync draw the pool's +233 whole
# (svrn-docs app-state-diligence/b-sync-fixes-2026-11-22/, estimated +195
# to +275); a build past it comes back to the operator. From 25,717: the cap
# to 25,951. Drawn so far: +1,450 (A +654, B +796). Left: none, and +80.
# (2026-11-22) The operator's second ceiling for the shared database:
# +700 of all code but JavaScript and +60 of JavaScript, for the review's
# fixes past the first pool and slices B's second half to E, drawn the
# same way (svrn-docs APP_STATE.md, Decisions). The review's fixes, at
# 25,945 after steps 1 to 6 (2b3bd031..145eda7e), draw +199 for the rest,
# estimated +125 to +175 by file (R8, R4, N2 and H1, H3, the bounds and
# the self-heal), to come down to where they land. The cap to 26,150.
# The second pool: +199 drawn, +501 and +60 left; the first's +80 of
# JavaScript also stands.
# (2026-11-22) The review's fixes land at 26,147 (2b3bd031..14ea09b3), +430
# over 25,717, the self-heal not built, and the cap comes down to it. The
# second pool: +197 drawn, +503 and +60 left.
# (2026-11-22) Slice B's second half (every computer whose person said yes
# keeps an app's database, prune, the push before sleep, WAL truncation and
# vacuum, the write refill, the lid) draws +250 before its build, so its
# commits land as it goes, to come down to where it lands. The cap to
# 26,398. The second pool: +447 drawn, +253 and +60 left.
# (2026-11-22) Slice B's second half lands at 26,322 (e5805a2d..2014ae99,
# rebased), +174 over 26,148, and the cap comes down to it. The second
# pool: +372 drawn, +328 and +60 left.
# (2026-11-22) One cap for the shared database's build, in place of a draw
# per slice: the second ceiling whole, 26,651 (+700 over 25,951), and
# JavaScript at 8,741 (+60, and the operator's +10 for a phone's signed
# write, "A: frame signs"). It comes down once, to where the build lands.
# (2026-11-22) The operator's raise for a phone's signed write, "Raise +170
# now": the cap to 26,821. Phones came to about +360 over 26,456, against
# +160 estimated: the phone's check of every change against its op before it
# signs (web/src/set.rs, 130), the home remembering each place it handed so a
# set carried after its prepare is never signed twice (+38), and the copied
# file's rebuild with twins (+40). The economizing first: the push before
# sleep (-35, d9118b3e), signed heads (-30, 86bbaff1) and sets kept waiting
# for a binding (-73, 1d411099). Phones' own cuts, in set.rs and the
# prepare, land in its commits. The rest is the phone's check, which is what
# keeps a computer between a phone and its group from signing for it. It
# comes down to where phones lands.
# (2026-11-22) The operator's B', "pass signs": a phone's set is signed by
# its open's approver and carries the pass that names it, since WebKit gives
# the framed signer a store of its own, without the person's key (svrn-docs
# HOLDER_APPROVAL section 1, measured). About +80 for the pass and stamp
# checked at apply and in the wasm, so the cap to 26,901. It comes down to
# where phones lands.
# (2026-11-22) The operator's "Raise +44": the cap to 26,945. W1's phones
# found the phone's signer refusing about one honest write in five (an
# update of a row not there, several ops on one row), and the fix judges a
# write by its net effect, +13. Slice D's answers to W7's trips took 31 of
# the room phones had counted on (1c481255..61c800eb). It comes down to where
# phones lands, and again by a fresh read of the phones code.
# (2026-11-22) The operator's raise for slice E, the release, "Raise +200":
# the cap to 27,145. Measured from the code there is (svrn-docs
# app-state-diligence/e-opening-2026-11-22/): the widest schema by a rebuild
# of the copy, a set whose schema a copy lacks dropped and sent again, the
# size cap with a share per device, and each bound's number in one home, about
# +110 to +250. It comes down to where E lands.
# (2026-11-22) The operator's yes to the signer picking a phone's places
# (svrn-docs app-state-diligence/invariants-2026-11-22/, change 5): one
# party decides a place, which retires the home handing places, the
# signer's twice-signed refusal, the rows a place handed and never signed
# leaves, and "write it again". JavaScript to 8,751, down to where it lands.
# (2026-11-23) From the operator's overnight pool (+100 code, +10
# JavaScript, drawn only after economizing): +43 for a value held to its
# column's type at parse, on every copy alike, so copies holding the same
# sets can't end apart by arrival order. Its pass cut 7 first (one refusal's
# per-type words, a wrapped signature); the rest is the column-type rule.
# The cap to 27,188. The pool: +43 drawn, +57 and +10 left.
# (2026-11-23) +15 more from the pool for each device's bytes and rows kept
# as running totals in the transaction that keeps or drops a set, so a write's
# size-share check reads one row however much the copy keeps (+24, after cuts
# in held, Room and the trigger loop prune shares). The cap to 27,203. The
# pool: +58 drawn, +42 and +10 left.
# (2026-11-23) +36 more from the pool: the security review's phone and parse
# fixes (a phone signs only what its ops make, a column change at an even
# causal length refused, a carry through the person's refill, +26) and a
# value held to its column's type (+53, after its pass cut 7) come to +79
# over 27,159. The cap to 27,239. The pool: +94 drawn, +6 and +10 left.
# (2026-11-23) The operator's raise, +43, past the pool: the review's schema
# fixes (one decider for what a version may keep +9, a schema.sql that only
# declares +10, a value bound read by SQLite and the parse alike +13, a
# restore that keeps a working connection +7, SQLite off the runtime's
# threads +4). Their pass cut first: check uses scratch, columns and texts
# read through sync::rows, one refusal for WITHOUT ROWID and GENERATED, the
# per-scratch bound and the DEFAULT probe dropped. The cap to 27,282.
# JavaScript lands at 8,747 with the phone's signer fixes in: its cap down
# to 8,748.
# (2026-11-23) The operator's raise for the delta review's fixes, the file
# column and a read without the JSON aggregates, landed at +104 over 27,281:
# a schema.sql that only declares +25, a key compared as SQLite orders it +3,
# every version through the one decider +4, a restore's swap whole at every
# step +9, a read's bounds put back on every path 0, a row that fits one
# answer, 32 columns and 128 KiB a value +16, a database opened outside the
# lock +5, its node callers off the runtime's threads +12, the file column
# +27 (of the operator's +28), the JSON aggregates refused +3. Cut first:
# INERT and its function list, the CHECK toggle in keep, one refusal for each
# kind, the node's callers from +25 to +12. The cap to 27,386, where it lands.
# Then the keeper's economizing pass before svrngs-renew, each cut lowering
# its caps to one past where it lands
# (demo/runs/economize/2026-11-keeper/SURVEY.md): door/bound.rs 6, 24,511;
# share.rs 16, 24,495; uid.rs 20, 24,475. JavaScript: lend-change.js 3, 8,690;
# lent-row.js 8, 8,682.
# Then svrngs-renew, within the operator's envelope for it (2026-11-22,
# "Those resources work for me": at most +180 and +30 JavaScript over the
# pass's counts, ralph/next/keeper/CHARTER.md "Budget"), each step raised
# to one past where it lands: bound.rs's wait moved into `mutated`, 24,483;
# the job named and the kept lend renewed (job.rs, bound.rs `renewed`,
# `Net::job`, `share::kept`, `Terms::keep`, `Hold::Changed`, `Answer::Job`),
# 24,655, the envelope's whole +180; JavaScript, the row's words, 8,688.
# Then avail-economize's pass before the available-by-default release, each
# cut lowering its caps to one past where it lands
# (demo/runs/economize/2026-11-22-available/SURVEY.md): door/bound.rs 11,
# 24,644; uid.rs 8, 24,636. JavaScript: lent-row.js 18, 8,650.
# Then avail-plan: the pure deployment fold, words and purity check land at
# 24,945 (+309 of the release's +435 envelope; CHARTER, "The release").
# Then avail-plan-economize folds the start verdicts together: 24,942.
# Sharing the return wording's template brings it to 24,940.
# Then avail-publish reserves revisions under the existing lend lock and
# makes the access write conditional: 25,064 (+124, within the +435 envelope).
# Then avail-terminal's replay and ordered owner results: 25,094 (+30,
# within the raised envelope in keeper CHARTER, "The release").
# Its terminal and owner adapters land at 25,499 (25,521 ceiling).
# O3 pins the node's process calls and the executor's keeper entry at 25,513.
# O4 keeps mechanism names out of the page's and planner's copy, at 25,521 and 8,651 JS.
# Then avail-unmagic: cut the live takeover and add N1, to 25,336.
# Then avail-run-linux: explicit Run and a fresh witness before saving a unit grant, to 25,424.
# Then avail-run-page's review and confirmation reuse the share executor, at 25,502 and 8,680 JS.
# Both doors then share the same service review, at 25,484 and 8,682 JS.
# avail-run-page-copy spends 10 non-JS of the release reserve on the shared
# Run confirmation, its two next steps and the keeper's verified port.
# avail-run-stdio spends 9 more on explicit stdio at that same keeper call.
# avail-run-mac adds the read-only sign-in observation for svrngs's own agent.
# avail-run-mac-renew spends 105 for a shared live grant witness and plist registration.
# avail-mac-plist-read spends 11 to parse the door's login plist, leaving 10 for shipping.
# avail-ship packs both binaries under one checksum and adds the uninstall notice.
# avail-page-prefill uses the existing Run route and form: +6 non-JS, +10 JS.
# (2026-11-23) The keeper branch, squashed onto main 05896fb0 (the
# operator: "Squash, with an archive tag"; keeper-archive-2026-11-23 holds its
# 195 commits from 789707dc). The caps sum main's with the keeper's approved
# growth over 789707dc's (24,518 and 8,694 exclusive): the keeper CHARTER's
# 25,727 and 8,741 ("The envelope now") are +1,210 and +48. So 27,386 + 1,210
# and 8,748 + 48: 28,596 and 8,796, to come down to where the squash lands.
# It lands at 28,590 and 8,795. Keeping the keeper's group-free Run and Stop
# gave up #7's two-line saving in running-here.js (its gate moved to MEMBERS),
# so two single-use bindings in `Running` were inlined, -2 JS.
# Merge the already approved +55 (f6581f34, 15f458cf): a lend withdrawn by
# any path is turned off for the room first, so one lent again under its name
# starts off the WiFi. Its pass cut one line from the lock read in `withdrawn`.
# The operator approved +26 for the delivery and security patch. The pass
# removed the unsigned loopback download path and its temporary server in
# deploy-relay.sh. The remaining lines verify the publisher manifest in
# install.sh, pair Keeper with the daemon build in plan/run.rs, and keep
# deferred database changes from being reported as synced in sync.rs.
# The operator approved +16 for scripts/dev.sh (2026-11-25), which runs this
# checkout's build as the login svrngs. Its pass (2a1d9f26) cut the two items
# only tests reached, cli.rs `share_lend` and set.rs `At::check`, -9, to
# 28,662 and the core's 3,526; dev.sh brings the whole to 28,678.
# The operator approved up to +97 JS (2026-11-25) for lending first, then
# the lend's offer to keep its program running (keep-running.js). Its pass
# cut the standalone Run form, its prefill and the found row's Run link
# (-30) and trimmed the offer (-8): 8,892, to come down with the Mac keep.
# The Mac keep (2026-11-25): +7 for one decider and the Mac approval, past -13 cut (the Run button's if, /run's starting): 28,672; the offer reads /lent alone: 8,880.
# The operator approved +4 (2026-11-25) for approve's `keep: true`, refused unless svrngs keeps what answers, so the page approves nothing the person didn't see but svrngs's own job. Its pass cut 1 (the route's `named`); nothing else in the touched files was needless. 28,676; the page's guess goes, JS 8,873.
# The found row's lend said in fewer words (2026-11-25): the review alone, its sends and kept-back paths as data for the page's Details, no terms-first step (home.rs `drawn` gone): 28,655; the drawn message one patch, the lend's row opened by its focus: JS 8,871.
# Its review shows its groups' own rows, by whom, in Details (2026-11-25): +6 paid by cutting needless lines in lend_one (the groups' names, the name-taken check) and LEND's two let-go patches made one: 28,649; JS 8,865.
# Keep it running reads nothing of how a program was started (2026-11-25): the /run?prefill route and the page's fill-in go: 28,641; JS 8,845.
# svrngs keep <port> -- <command> and the page's line for it (operator-approved raise, 2026-11-26, after an economizing pass): 28,665; JS 8,852.
# The CLI's main path (operator-approved raise, 2026-11-26, after an economizing pass): the
# proof names its build, another build is restarted onto this one, svrngs
# starts apart from the terminal, and svrngs stops itself on its own route,
# no signal. The pass cut four wait loops to one and the log path said three
# times to one (-4 net); the rest is behaviour a person reads or a check
# holds. 28,805.
BUDGET = {
    "crates/svrngs-core": 3527,

    ".": 28806,

    "js": 8853,

    "swift": 1,
}
CODE = {".rs", ".ts", ".py", ".sh"}
# Each rung as a person reads it; the keys above stay paths, as this file's
# history names them (the operator, 2026-11-09: "can we do better than
# whatever `.` means?").
LABEL = {"crates/svrngs-core": "the core (svrngs-core)", ".": "all code but JavaScript",
         "js": "JavaScript", "swift": "Swift"}
RUNG_KINDS = {"js": {".js", ".mjs"}, "swift": {".swift"}}


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


def code_lines(base, kinds=CODE):
    n = 0
    for f in files(base):
        rel = f.relative_to(ROOT).as_posix()
        if (f.suffix not in kinds or not f.is_file() or "/tests/" in f"/{rel}"
                or "/node_modules/" in f"/{rel}"
                # Preact's and htm's type declarations, vendored for `types`
                # and never served (ralph/next/js/CHARTER.md, "Answered" 3).
                or (rel.startswith("web/vendor/") and rel.endswith(".d.ts"))
                or rel.startswith(("apps/", "target/", "demo/runs/", "docs/diagrams/src/", "docs/brand/src/"))
                or re.search(r"(^|/)tests?(_\w+)?\.rs$|_tests?\.rs$|\.test\.m?js$", rel)):
            continue
        for line in f.read_text(errors="replace").splitlines():
            s = line.strip()
            if s and not s.startswith(("//", "#", "/*", "*")):
                n += 1
    return n


def process_signals(paths):
    for f in paths:
        rel = f.relative_to(ROOT).as_posix()
        if TEST_FILE.search(rel) or "/tests/" in rel:
            continue
        text = f.read_text(errors="replace")
        for site in PROCESS_SIGNAL.finditer(text):
            line = text.count("\n", 0, site.start()) + 1
            yield f"{rel}:{line}: direct process signaling ({site[0]}), use only keeper-owned services (KEEP_IT_RUNNING_RELEASE N1)"


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
        if rel in PLAN_PURE and (site := PLAN_IO.search(text)):
            broken.append(f"{rel} uses {site[0]}: the share planner is pure; put effects in plan/run.rs (KEEP_IT_RUNNING_RELEASE P4)")
        if not TEST_FILE.search(rel) and '/tests/' not in rel:
            if '"svrngs-keeper"' in text and rel != "crates/svrngs-node/src/plan/run.rs":
                broken.append(f"{rel}: only plan/run.rs may run the keeper (KEEP_IT_RUNNING_RELEASE O3)")
            if (node := rel.removeprefix("crates/svrngs-node/src/")) != rel:
                code = re.sub(r"\s+", "", text)
                allowed = NODE_COMMAND.get(node, ())
                for call in (allowed,) if isinstance(allowed, str) else allowed:
                    code = code.replace(call, "", 1)
                if "Command::new(" in code:
                    broken.append(f"{rel}: an unlisted process call; the node may only read app jobs (KEEP_IT_RUNNING_RELEASE O3)")
    broken.extend(process_signals(sorted([*(ROOT / "crates").glob("*/src/**/*.rs"),
                                         *(ROOT / "apps/svrngs-keeper/src").rglob("*.rs")])))
    core_lib = ROOT / "crates" / "svrngs-core" / "src" / "lib.rs"
    if core_lib.exists() and "#![cfg_attr(not(test), no_std)]" not in core_lib.read_text():
        broken.append("crates/svrngs-core/src/lib.rs lost #![cfg_attr(not(test), no_std)]: the core is no_std")
    roots = [*(ROOT / "crates").glob("*/src/lib.rs"), *(ROOT / "crates").glob("*/src/main.rs"),
             ROOT / "web" / "src" / "lib.rs", ROOT / "apps/svrngs-keeper/src/lib.rs", ROOT / "apps/svrngs-keeper/src/main.rs"]
    for root in sorted(roots):
        if root.exists() and "#![deny(unsafe_code)]" not in root.read_text():
            broken.append(f"{root.relative_to(ROOT).as_posix()} lost #![deny(unsafe_code)]: every product crate root denies it (DESIGN §5)")
    for f in [*files(ROOT / "crates"), *files(ROOT / "web" / "src"), *files(ROOT / "apps" / "svrngs-keeper" / "src")]:
        rel = f.relative_to(ROOT).as_posix()
        if f.suffix != ".rs" or not f.is_file():
            continue
        found = len(EXEMPT.findall(f.read_text(errors="replace")))
        if found > (rel in UNSAFE_HOMES):
            broken.append(f"{rel} exempts code from `unsafe_code`: svrngs's `unsafe` is one function in each of {sorted(UNSAFE_HOMES)} (DESIGN §5); write this safe")
    print("Lines of code, tests and apps/ not counted, and the most each may have:")
    for rung, cap in BUDGET.items():
        n = code_lines(ROOT, RUNG_KINDS[rung]) if rung in RUNG_KINDS else code_lines(ROOT / rung)
        print(f"  {LABEL[rung]:<24} {n:>7,}   most {cap - 1:>7,}")
        if n >= cap:
            broken.append(f"{LABEL[rung]} has {n:,} lines of code, over the {cap - 1:,} it may have: raised only by the operator, with the reason in the commit")
    for b in broken: print(f"BROKEN  {b}")
    return 1 if broken else 0


if __name__ == "__main__":
    sys.exit(main())
