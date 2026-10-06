# Stock Jellyfin through a lend, measured — 2026-10-05

u5-3 (DESIGN §3, "Lent apps", calls 2, 4 and 5; §11 risk 7): what stock
jellyfin-web does through the door before anything is built on it, and
which way past its sign-in u5-4 builds. This section is committed before
any run; the data, and the decision it gives, come after it in later
commits.

## The rule, stated before the data

**Zero sign-in** — the lender's computer makes each person their own
Jellyfin user, through Jellyfin's API with a key the lender gives once,
and the door signs each person in as theirs — **is decided if and only if
both hold**:

1. *It fits in at most two rows*, u5-4 counted among them. A row is one
   piece of work with a check of its own landing in one place
   (`apps/jellyfin/`, the door, the page or the lend's worker, the
   command). Every piece the data shows a person needs to get from the
   list to a playing film with nothing typed is counted, and Demo A's
   "Jellyfin's list of users unchanged" clause, which users made during a
   run break (DESIGN §3, call 4), counts as one more unless it is the
   operator's alone.
2. *It adds no Jellyfin noun outside `apps/jellyfin/`*: every piece outside
   it is general — any app could use it — and names nothing of Jellyfin's
   API (`apps/jellyfin/nouns.txt`, and any of its routes, headers, storage
   keys or fields).

**Else sign in once**: the lend carries no secret, each person signs in
with an account the lender made, and zero sign-in goes to DESIGN §9 with
the trigger that brings it back. If sign in once also fails its bars
below, neither is decided and the question goes to the operator
(`ralph/next/u5/ctl/NEEDS_HUMAN.md`).

## What is measured, and the bar each is read against

Against a scratch Jellyfin 12.0.0 on this host (as U3's dry run,
`demo/runs/a/dry-2026-10-04/dry.sh`), lent with `svrngs share 8096` and
**no secret**, two accounts the lender made, from Alex's computer (its
browser at the lend's origin on Alex's loopback) and from the page (a
phone joined by the group's link), each in Playwright's Chromium and
WebKit:

- **(a) Sign in, play, seek; two people at once.** Two people, each in a
  browser of their own, each signs in with their own account and plays a
  different film. Plays: the `<video>` moves within 30 s of play. Seeks:
  one seeks five times, each playing again within 5 s (Demo A's bar);
  then the other seeks three times while the first plays. Left alone: the
  person not seeking has no `emptied` or `loadstart` on their `<video>`,
  never goes back more than 2 s, and never stands still 5 s, while the
  other seeks; and both are still signed in afterwards. One film plays as
  it is; the other is in a form no browser plays, so Jellyfin converts it,
  and the play it converts is the one watched while the other seeks.
- **(b) The live socket.** jellyfin-web's socket opens through the lend,
  and a message the lender's Jellyfin sends that person's session arrives
  on their page within 10 s.
- **(c) A secret in a byte received.** Every response header and body
  each browser receives is searched for the lender's password, the
  lender's own token, the other person's password and token, and every
  token the lend holds. With no secret, the lend holds none; so the same
  search is also run on a lend that carries a token, asking Jellyfin's
  playback info and its converted stream's playlists through the door,
  which is what zero sign-in would put through it (e0-bot's unmeasured
  claim, DESIGN §8, U5's inventory).
- **(d) What zero sign-in would take.** A user made per person through
  Jellyfin's API with the lender's key, signed in, and that person's token
  carried by the door on a lend opened with nothing typed: whether
  jellyfin-web reaches a playing film, and if not, what more it needs,
  each piece sized as a row under the rule above.

One host, one run of each: not a rate.
