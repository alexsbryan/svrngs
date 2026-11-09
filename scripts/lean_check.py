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
BUDGET = {
    "crates/svrngs-core": 3511,

    ".": 20721,

    "js": 8396,

    "swift": 1,
}
CODE = {".rs", ".ts", ".py", ".sh"}
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
    for rung, cap in BUDGET.items():
        n = code_lines(ROOT, RUNG_KINDS[rung]) if rung in RUNG_KINDS else code_lines(ROOT / rung)
        print(f"{rung:<20} {n:>6} / {cap} lines")
        if n >= cap:
            broken.append(f"{rung} has {n} lines of code, budget {cap}: raised only by the operator, with the reason in the commit")
    for b in broken: print(f"BROKEN  {b}")
    return 1 if broken else 0


if __name__ == "__main__":
    sys.exit(main())
