# events-doc-link — what svrngs lacked, for the operator

One thing the take found is svrngs's, not the app's. The app works around
nothing (fence 3), and this row changed no code. It would be a row of its
own, on the operator's budget (AGENTS.md "Done"). The lines are estimates,
not measured.

## A phone opening a link while the link's computer sleeps waits 30 s

**What the take showed.** With Sam's computer asleep, Cy's phone opening
the dinner's link from the chat was told after 30,497 ms (stopped) and
30,522 ms (frozen), at her group's page: "Your group’s computer did not
answer through usw1-1.relay.n0.iroh.link. Try again later." (`README.md`
here, P6 and P8; `data/shots/23-cy-L5.png`, `30-cy-L6.png`). On the
dinner's own address the same phone was told in 4,018 and 4,221 ms (P5,
P8).

**Why.** Before the page opens anything lent, it dials the computer that
made the link it joined by (`web/app.js:314`, `enter`, `connect(key,
group.token)`), and that dial waits as iroh does, 30 s
(`web/src/lib.rs:307-308`, `dial` passing no cap to `reach`). 5c097eb
left it uncapped: "the measurement did not cover it, and it broke a
working join" — the redeem dial a just-started computer answered in
3.03 s. The members' dial, `connect`, is a different dial from the
redeem one, `connect_to_join`; neither was measured.

**What svrngs could do.**
- Measure the members' dial from a phone's page to the link's computer,
  working, as events-asleep (a) measured the lend's, then cap `connect`
  by the same rule (`dial::DIAL_WITHIN` if it fits). About 2–5 lines in
  `web/src/lib.rs` and a test of 20–40 lines in the shape of
  `lend_origin.rs`'s cap test; the run itself is `demo/` and uncounted.
- Or: when the page was opened at a lend (`#open=`), dial the lend's
  computer first, which the link names, and the group's computer after.
  Not estimated; it changes what the page asks whom.
- The words on that page name a relay's host, which a person a bit afraid
  technically cannot act on; "Sam's computer did not answer" names the
  person, as the home page does (`home.js:95`). Whether the page holds
  that person's name before its first dial was not looked into.
- **The operator's call**: whether this is a row now, and which of the
  two.
