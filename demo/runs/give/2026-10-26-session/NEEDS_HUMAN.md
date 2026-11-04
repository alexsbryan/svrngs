# give-session: how does a lend's page reach a lend it reads, now that a browser needs a session?

**The question.** A lend's page that reads another lend fetches that lend's
origin cross-origin. `fetch` sends no cookie cross-origin by default
(`credentials: 'same-origin'`), so the session that c219e643 requires at every
lend's origin never arrives, and every read is refused 401. Neither the row
nor GIVE.md §5 says how a read is let through. Which of the ways below should
the row take?

## What breaks, measured

`cargo test --workspace --test browser` at e549903e (this package's parent
commit): 69 passed and 7 failed. Four of the failures were test-carries and
now pass. The three left are all this one cause:

- `navigations::a_navigation_from_another_lend_s_page_is_judged_as_its_fetch`
  (chromium and webkit), `navigations.rs:109`: the page's fetch of the notes,
  which it reads, never reached them. Their service saw only `["GET /linked"]`,
  and no `GET /fetched`.
- `opens::opening_an_app_from_the_home_page_is_the_yes_to_what_it_reads`,
  `opens.rs:74`: `"after": {"status": "TypeError: Failed to fetch"}`. The 401
  carries no `Access-Control-Allow-Origin`, so the page cannot even read
  the refusal.
- `acts::the_home_page_says_what_acts_as_you_and_its_stop_holds`, `acts.rs:135`:
  `"opened": {"status": "TypeError: Failed to fetch"}`.

These are the same fetches the apps make:

- `apps/movie-night/app.js:61`: `ask(\`${lib.base}${LIST}\`)`, where `base` is
  `/svrngs-reads`'s `address` (`:27`);
- `apps/movie-night/picks/picks.js:71`, `:118`;
- `apps/taste/check/other/other.js:18`.

So Movie Night on a computer, and the movie-night scene with it, stops
reading anyone's library. That is fence 2's kept path "a lend opened from the
home page".

There is a second, smaller break. Movie Night links each film to the
lender's own page (`apps/movie-night/app.js:91`, `itemAt(lib.base, c)`).
Following that link is a top-level navigation to the read lend's origin. It
carries that origin's cookie only if this browser was opened there before,
through `/to`. Otherwise the person lands on "Open svrngs from your
computer".

## Why the obvious fixes don't work

- **A cookie for every host.** Even a `Domain=localhost` cookie, if browsers
  allowed one, is not sent by a fetch whose credentials are `same-origin`.
- **Trusting the `Origin` header.** The only signal on the request is
  `Origin: http://<page's lend>.localhost:<port>` with `Sec-Fetch-Site:
  same-site`. A program forges both, and a container on a Mac running podman
  is exactly that program (GIVE.md §5). Accepting them without a session lets
  the container act as any reading page toward what it reads: it could read a
  friend's films as the person.

## The ways I see

**A. A capability in the read address** (recommended).

- How it works:
  - `/svrngs-reads` is fetched by the page at its own origin, so it carries
    that page's session.
  - It answers each read lend's `address` with a path that carries a
    capability: the token's HMAC over (page's lend, read lend), so it lasts as
    the cookie does and is kept nowhere. An example:
    `http://<read>.localhost:<port>/svrngs-as/<cap>/`.
  - At the read lend's origin, `lent` takes a valid capability as that page's
    fetch, so `app` is the page's lend. It strips the prefix before the
    service is asked.
  - A top-level navigation carrying the capability would set that origin's
    session and redirect to the plain path. That covers Movie Night's links.
- Apps that build on `address` change nothing (movie-night, taste).
- A program cannot get a capability without a session or the token.
- Cost: one more way in at `origin.rs::lent`, about +40 to +70 library lines
  inside the row's range, and DESIGN §3 "Lent apps" call 6 said anew.

**B. Reads through the page's own origin.**

- How it works: `/svrngs-reads` answers `address` as a path under the page's
  own origin, such as `http://<page>.localhost/svrngs-lend/<read>/`, which the
  node carries to the read lend as the page's app. The fetch is same-origin,
  so the session rides with it.
- Simpler at the check, but the read lend's answers would be served under the
  reading page's origin.
- A navigation there would run the read lend's page at another app's origin.
  That is an origin mix the containment floor exists to prevent. I would not
  take it.

**C. Accept a page's cross-origin fetch on its `Origin`, with no session.**

- It keeps today's behaviour for reads. It reopens the hole for every read
  pair: a container acts as the reading page toward what it reads.
- It would need its own `Trust floor:` line, so it is not mine to take.

**D. Apps send `credentials: 'include'`.**

- The door would add `Access-Control-Allow-Credentials` and an exact
  `Access-Control-Allow-Origin` to the read lend's own answers.
- It changes every reading app, and it still fails until the browser was
  opened at the read lend.

## The state of the row at this commit

These are committed and green:

- `session.rs`'s two tests;
- `carried.rs` (13), `door/main.rs` (40), `home.rs` (8), `lend_origin.rs` (7),
  `surface.rs` (7) and `peer.rs` (4): `cargo test -p svrngs-node --test <each>`;
- 75 of the browser binary's 78 (2 ignored), the 3 above excepted.

Still to do, after the decision:

1. The three browser tests above, by the way chosen.
2. The svrngs crate's tests: home_group, address, new, share, join_page,
   scenes/people and install. The scenes include `browser/drive.rs` and want
   `drive::opens_at` for each person's svrngs.
3. `svrngs open` in `cli.rs` and USAGE.
4. install.sh's opener and its check in `install.rs`.
5. A new browser test: an open leads to the home page, then to a lend, then
   to a bookmark; a fresh context is told "Open svrngs".
6. The words in `home/index.html`.
7. DESIGN §3 and PLAN §1.
8. The container run under `demo/runs/give/`.
9. `scripts/check.sh`.
