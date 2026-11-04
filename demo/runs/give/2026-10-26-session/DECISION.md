# give-session: how a lend's page reaches a lend it reads — the director's decision, 2026-10-26

Answers `NEEDS_HUMAN.md` here. The charter leaves this fork to the director: it
is how the row executes ("the smaller reversible step"). It needs no new trust
floor line, it changes no page's policy or listener, and it brings no app's
noun into the infra. Option C would have needed a trust floor line, and it
is not taken.

## Reproduced

`CARGO_INCREMENTAL=0 cargo test -p svrngs-node --test browser --
a_navigation_from_another_lend_s_page_is_judged_as_its_fetch
opening_an_app_from_the_home_page_is_the_yes_to_what_it_reads
the_home_page_says_what_acts_as_you_and_its_stop_holds` at fbba26ca gave four
FAILED:

- navigations (chromium and webkit), `navigations.rs:109`: `["GET /linked"]`,
  and the page's own `"fetched": "TypeError: Failed to fetch"`;
- opens (chromium), `opens.rs:74`: `{"status": "TypeError: Failed to fetch"}`;
- acts (chromium), `acts.rs:135`.

Movie Night reads through the same fetch: `apps/movie-night/app.js:27` takes
`base` from `/svrngs-reads`' `address`, and `:61` fetches `${lib.base}${LIST}`.

## Decided: A, a capability in the read address, with no session minted from it

It is the computer's form of what a phone already does. DESIGN §3, "Lent
apps", call 6: a phone's read is carried by a pass naming `{app, reads}`. On
a computer, the read's address now carries proof that this browser's page of
that app asked.

1. **The capability.** It is the token's HMAC over the pair (the reading
   page's lend, the read lend). It comes from `Sessions`, the one home for
   what this computer gives a browser. Its input is kept apart from a host
   cookie's: it has a separator no host's first name holds. It is kept
   nowhere, and it lasts as long as the token does.
2. **Where it is given.** `/svrngs-reads` is asked at the page's own origin,
   so it carries that origin's session. It answers each read lend's
   `address` (`surface.rs:377`) as `http://<read>.localhost:<port>/svrngs-as/<cap>/`.
   Apps that build on `address` change nothing.
3. **Where it is taken** (`origin.rs::lent`). A path under `/svrngs-as/<cap>/`
   is let in when `cap` is this computer's for (the app `app_of` names, this
   lend), and that app is another lend.
   - The prefix is stripped before anything else is decided. The app is
     still read by `app_of` alone (call 6's one decider), so the exact-Origin
     guard stays as the row's second layer. A capability shown with another
     page's `Origin` is refused.
   - The refusals trace at debug, as the rest of `lent` does.
4. **A navigation never becomes a session.** A top-level navigation under
   the prefix (`Sec-Fetch-Mode: navigate`, `Sec-Fetch-Dest: document`) is
   sent on, 303, to the home page's `/to?lend=<read>&at=<the rest>`. There
   the home page's own session decides. This covers Movie Night's links
   (`app.js:91`).
   - The worker proposed that the capability set the read lend's session.
     That is not taken. The capability is a bearer of "this app's page", and
     the app's own server could hold it. Minting the read lend's session from
     it would let that server act as the read lend's own page, which is more
     than the person's yes to the read.
5. **The three red tests.** Their drivers (`navigate.mjs`, `opens.mjs`,
   `acts.mjs`) take the read's address from `/svrngs-reads`, as the apps do.
   Every assertion is kept.
   - The bars guard the yes and the judging of a navigation, not how the
     address is spelled.
   - Add one assertion: the plain origin, fetched from the page with no
     capability, is refused. That is the request a container forges.

What a capability gives away is bounded. Whoever holds one is the reading
app's page toward that one read lend, and only that. The person's yes is
still checked on every request (`surface::reaches`), so a Stop holds. A
container gets no capability without a session or the token.

## The budget, said now

`python3 scripts/lean_check.py` at fbba26ca gave `. 18446 / 18276`.
give-session's range is +110 to +220 over give-token's 18,275
(`demo/runs/economize/2026-10-25-give/SURVEY.md:222`), so its top is 18,495.
The row has used +171, and reads were not in its estimate.

- Build the capability lean: aim for +25 to +40.
- If the close still lands past 18,495 after the row's own economizing pass,
  stop with `NEEDS_HUMAN.md`, as the operator's decision in the row says.
