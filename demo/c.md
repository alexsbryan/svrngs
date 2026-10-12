# C — the group's whole life

> **Written 2026-10-10 for the operator** ("Yeah write the scene that we can
> fulfill and the Demo C run of show"; and on the chat, "Hopefully not more
> the 50 lines in our own source and a pretty commodity integration", then
> "Rewrite that way": the link is the chat's door, no bot). Every bar below is the seat's
> proposal and the operator's to set. DESIGN §8 names it and its units, §10
> the operator's words.

Gate: four of five newcomers, each at home, get from the link in the group's
chat to a film of Sam's playing, an album of Ana's playing and, on a
computer, a photo from the trip opened from their own disk — within five
minutes on a computer, getting svrngs from the link included — one line the
group's page gives, pasted in a terminal, the only thing typed (the operator,
2026-10-11: "Pasting one line in the terminal is fine") — and two on a
phone — unassisted; and on the phone the group opens within ten seconds of
the link (NORTH_STAR §1's test). Nobody made an account but by a program a
lender runs, and nothing asked anyone for a key.
Shown after U7, onboarding and the two-homes run (below): one evening of a
group of friends who already share a Discord channel — its link posted
there, joined from there, its library, its music and its photos used,
someone removed, a computer asleep — everyone in their own home.

**The chat's door is the link.** No bot, no commands, no chat's SDK: Sam
pastes the link in the channel as anyone pastes a link, and it works the
same in Discord, iMessage, WhatsApp, Slack or Signal. svrngs's whole part is
the link previewing as an invitation, a title and a description on the
group's page, five lines of HTML (`web/index.html`, "You’re invited to a
group", v1-onboarding (c)). E0's door at tag `e0-bot` — `discord.js` on Discord's
gateway, a bot token, slash commands, a guest book keyed by Discord user id,
517 code lines (`bridge/*.mjs` 349, `crates/svrngs/src/bridge.rs` 168, by
`git show e0-bot:<file>`, blank and comment lines out) — is the failure this
answers. What that gives up, deliberately: a group named for its channel by
itself (Sam names it), membership following the channel (removal is U7's,
in the command), and apps inside the chat (E0's "split the pizza").

## What it rests on, and what is built for it

Only what exists or one of the rows below. Each row is one merged row, its
parts in order and `scripts/check.sh` once at its close (operator,
2026-10-10).

| The scene needs | Today | Built by |
|---|---|---|
| the group's link in the channel | landed, v1-onboarding (c): `svrngs new <name> --origin <page>` prints it on the page's address, and pasted it previews as an invitation, a title and a description in `web/index.html` naming no group; the token stays in the link's `#` part, which no chat's preview fetches | — |
| a newcomer's computer in from the link by one line pasted in a terminal, nothing else typed, the group's folder on their disk | landed, v1-onboarding: on a computer the group's page gives `curl -fsSL <page>install.sh \| sh -s -- '<link>'`, which installs svrngs from the page's host, joins, keeps it running at login and opens the home page; the group's folder at `~/svrngs/<group>`. Once on one host, the paste to a member's photo on disk: 6,489 ms (`runs/onboarding/2026-10-11/`). The person is named by their computer's login (its finding 1) | the operator's `scripts/deploy-page.sh`, so group.svrnme.sh serves `install.sh` and the tarballs |
| Sam removing Bo, and only Sam | landed, U7 (u7-removal; DESIGN §6): `svrngs remove Bo --group <name>` on Sam's computer is the cut, anyone else's an ask; every lender refused the removed person 0.048 s after, on one host (`runs/u7/2026-10-11/`) | — |
| every person in their own home | every number so far on one host, every path direct (the branch `folders`' `demo/runs/perf/2026-10-10/README.md` §9 and `…/2026-10-10-make-it-fast/README.md`) | **two homes**: the library, the folder and a join measured between computers on two networks, through a relay |
| the trip's photos on every computer, kept while their author sleeps | landed on the branch `folders` (syp-1 to syp-4, perf-2; S2, a file whose author is asleep reaching a member who starts: 4.47 s at sy-1, 4.27 s at syp-1) | the operator's merge of `folders` |
| Sam's library, each person signed in with nothing typed | landed (`apps/jellyfin/`, `demo/runs/mw/2026-10-08/`, mwp) | — |
| Ana's music as each person, nothing typed | run through a lend in the dry run (`runs/c/dry-2026-10-12/`): Navidrome 0.64.2 signed each person in as their name from `X-Svrngs-Person`, nothing typed, on a computer and on a phone; but a second member joined as `ana` was signed in as Ana, its admin (its finding 1) | the operator's call on that finding, before Demo C |
| a phone in the group within ten seconds | Chromium through n0 joined in 5.4 s (`runs/u4/2026-10-05/README.md`, its finding 4) | — |
| a person's words for each refusal and each asleep computer | landed ("Cy's computer did not answer, so what they lend is not listed", `runs/supf/2026-10-10/`); the removed person's, landed with U7 ("You are no longer in <group>. …", on the home page and the group's page) | — |

**Left out, each said:** money — Actual asks every person the one password
its lender set (CENSUS.md:59), and no reference app for the money class is
built (E0's `apps/expenses/` is the canary's reducer); a phone in the folder
(the operator's, the branch `folders`' `ralph/next/syp/CHARTER.md`); the group reachable while
every computer sleeps (the phone replica, on the back burner, operator,
2026-10-10); the TV (Demo B's).

## Before the evening, each set up once and not timed

**The page**: group.svrnme.sh serving `install.sh` and the four tarballs
beside the page, by the operator's `scripts/deploy-page.sh` (DESIGN §7).

**Sam**, at home: svrngs on a computer, the tarball for it from the page's
`get/` unpacked into `~/.local/bin` (not timed; the line is for one who
joins: it keeps a svrngs running, and `svrngs new` refuses while one runs,
`crates/svrngs/src/cli.rs`, `Refusal::Busy`);
Jellyfin with at least three
films, lent as `svrngs share` tells Sam, with the program Sam runs in front
of it (`apps/jellyfin/`); Jellyfin's list of users exported.

**Ana**, at home: svrngs on a computer, likewise; Navidrome with at least
three albums, run with `ND_EXTAUTH_USERHEADER=X-Svrngs-Person` and
`ND_EXTAUTH_TRUSTEDSOURCES=127.0.0.1/32` — named here, since `svrngs share`
prints neither — and lent as `svrngs share 4533 --kind navidrome`, Enter
alone at its password (the dry run's finding 2); Navidrome's list of users exported; thirty
photos from a trip, about 150 MB, in a folder on Ana's computer.

**The five** — Alex, Bo, Dee and Eve on their own computers (macOS), Cy on
a phone only — each at home, in the Discord channel, never in a group of
Sam's, svrngs not on their computers. Every screen is recorded; every
svrngs log is kept.

## The steps, as the rows above will have them

Nobody is told them. Where a row has not named its command or page yet, the
step names the row; the run-of-show is rewritten in place when it lands,
before the first run.

1. **Sam makes the group and posts its link in the channel.** On Sam's
   computer Sam makes the group, named as the channel is, Sam its keeper:
   `svrngs new <name> --origin https://group.svrnme.sh/`, which prints the
   link on the page's address; Sam pastes that in the channel, where it
   shows as an invitation.
2. **Ana brings the trip.** Ana opens the link on Ana's computer; the
   group's page gives one line with the link in it and a button that
   copies it; Ana pastes it in a terminal, `curl -fsSL
   https://group.svrnme.sh/install.sh | sh -s -- '<the link>'`, and is in,
   the group's folder at `~/svrngs/<name>` on Ana's disk; Ana puts the
   thirty photos in it.
3. **The newcomers, one at a time, each timed.** Alex, Bo and Dee open the
   link on their computers and paste the line the page gives in a
   terminal, as Ana did; it installs svrngs in `~/.local/bin`, joins, keeps
   svrngs running at login and opens the person's home page (DESIGN §7);
   there they see the group, Sam's jellyfin and Ana's music, and the
   group's folder is at `~/svrngs/<name>` on their disk with the trip in
   it. Each plays a film from
   Sam's jellyfin, plays an album from Ana's music — Navidrome opening as
   them, nothing typed — and opens a photo from the folder. Cy does the
   same on a phone, the link opening the group's page in the browser,
   without the folder.
4. **Sam removes Bo**: `svrngs remove Bo --group <name>` on Sam's
   computer, which says "Bo is no longer in <name>. …" (DESIGN §10,
   2026-10-09: removing someone stays in the command until a row moves it).
5. **Ana's laptop is closed.** Then Eve, the last newcomer, opens the link
   on Eve's computer and does step 3's three things.

## What each person is told, and nothing else

**Sam:** "Make a group for this channel, share your Jellyfin with it, and
post its link in the channel." Later: "Remove Bo from the group."

**Ana:** "Join the group from the link, put your trip photos in its folder,
and share your music with it." Later: "Close your laptop."

**Each newcomer:** "Open the link in the chat. Play a film from Sam's
library, play an album from Ana's music, and open a photo from the trip."
(Cy, on the phone: the first two.) **Bo**, after step 4: "Open them again."

## What is watched

The newcomers go one at a time. A person's clock starts when they click the
link in the chat and stops when the last of their things is open, read from
the recordings; within five minutes on a computer, two on the phone, they
succeed. On the phone a second clock stops when the group's page lists the
group's people: within ten seconds. The operator watches and says nothing;
any help ends that person's attempt and goes in the notes, as does every
dialog the computer showed while getting svrngs.

Read from the logs, each passed, failed, could-not-judge or never-ran
(AGENTS.md 6):

- **Removed.** Within 30 s of Sam's removal, Sam's jellyfin and Ana's music
  each refuse Bo with svrngs's refusal, Bo's home page saying in a person's
  words that Bo is no longer in the group; a photo Sam puts in the folder
  after it reaches every member's disk but Bo's; Bo's own `Remove`, tried
  from Bo's computer, counts nowhere. What Bo already held stays on Bo's disk:
  removal sends nothing back.
- **Asleep.** With Ana's laptop closed, Eve's disk holds all thirty photos
  within two minutes of Eve being in, from whoever holds them; each home
  page lists Ana's music as Ana's computer not answering; Sam's jellyfin plays
  on for everyone.
- **Apart.** Each person's computer on its own network; each fetch's path
  as iroh selected it, direct or relayed, counted.
- **No accounts.** Jellyfin's and Navidrome's lists of users against the
  exports: unchanged, but one each made by a program its lender runs (as
  Demo A) or by Navidrome from the header, for a person who came.

## After

Write the record in `runs/c/<date>.json` before looking at anything else,
as `README.md` has it — `participants` 5, `succeeded_in_time` the newcomers
who met the gate, `accounts_created` from the comparisons, `key_prompts`
the times anything asked a person for what DESIGN §1 keeps from people,
`assisted` — then the notes: the four checks above, each with its verdict,
and where each person who did not finish got stuck. A failed run is kept,
not rerun until it passes.
