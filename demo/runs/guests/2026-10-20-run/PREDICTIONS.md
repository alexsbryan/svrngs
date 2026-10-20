# A visit on one host — setup and predictions, 2026-10-20

Row guests-run (`ralph/next/guests/STATE.md`). Kim, Bo and Ana are a group
on three computers. Sam, Kim's friend, has only a browser. Kim sends Sam a
link, and Sam watches Kim's film as Kim's guest. The house sees the visit,
a third guest is past the house's bound, and three visits end: by Kim's
Stop, on time, and by Kim's leave. This is what guests-visit (ea4c6aba),
guests-door (c850c085), guests-link (7754eb7a) and guests-pages (bd358090,
e1a4694b) built, run as people meet it, with every computer and browser on
this host. It is written before the driver and before any data.
`README.md` here will say what the run gave against each line below.
Every person is a program, so no people bar is judged (AGENTS.md 7). One
host and one run are not a rate.

## 1. Setup

`W=<scratch> python3 demo/runs/guests/2026-10-20-run/run.py` writes its
data to `data/`. With `PLANT=<name>`, the same story runs against a build of
svrngs with one planted change, and the data goes to `data/plants/<name>/`.
With `TRIAL=1`, the story skips the wait for the visit until tonight to
end; its bar is then never-ran. `judge.py` reads a take's `summary.json`
and writes its `verdicts.json`, one verdict a bar (AGENTS.md 6: passed,
failed, could-not-judge, never-ran). `scripts/check.sh` runs none of this.

- **svrngs** at this commit: `cargo build -p svrngs`, each computer with
  a `HOME` and data dir of its own under `W`, run with
  `RUST_LOG=info,door=debug,redeem=debug,link=debug,removal=debug`. The
  page is built by `scripts/build-web.sh` and served on this host by
  `web/tests/serve.mjs`. svrngs uses n0's public relays, its default, and
  each browser reaches a computer through them, as a phone does.
- **The cast.** Oak is a group: Ana's, Kim's and Bo's computers. Ana makes
  Oak (`svrngs new Oak --as Ana --origin <page>`), so Ana keeps the order
  of removals and Kim is free to leave. Kim and Bo join by its page link,
  as someone sent it in a chat would (`svrngs join <page link> --as …`),
  so each keeps the page a guest's link opens on (`crates/svrngs/src/cli.rs:62`).
- **The lends.** Kim and Bo each run a stand-in for Jellyfin on this host
  (`jellyfin.mjs` here): a page with one `<video>` and a 60 s film, made by
  ffmpeg as the browser tests make theirs
  (`crates/svrngs-node/tests/browser/main.rs:638`). It keeps the
  `x-svrngs-person`, `x-svrngs-key` and `x-svrngs-user` of every request
  it gets. Each lends theirs with `svrngs share <its port> --kind
  jellyfin`, so the guest's page lists it as "Kim’s jellyfin".
- **The guests.** Each guest is a browser that keeps nothing of any group:
  Sam is WebKit as an iPhone, the likeliest place a link sent in a chat
  opens. Jo, Max and Lee each use Chromium. Each is a browser of its own,
  kept for the whole run.
- **Pages.** One program, `people.mjs`, keeps a browser for each person:
  Chromium for each computer's home page at `svrngs.localhost:<port>`, and
  each guest's as above. Every press is a click on what the page shows, by
  its words.
- **What's read.** Each page's words after each press. Each computer's
  `/v1/groups` (`visiting`, `removals`) and `/v1/rail/log`, for when a
  visit, a Stop or a leave reached it. The stand-ins' kept headers. Each
  computer's log, kept at `door`, `redeem` and `removal`.

## 2. The story, in order

Each step is timed from its press.

1. *Kim makes a link.* On Kim's home page, under Oak, Kim presses **Have a
   guest**, reads the length chosen to start and the lengths offered,
   keeps the first, and presses **Make a link for your guest**.
2. *Sam watches Kim's film.* Sam's browser opens the link, gives "Sam"
   when asked, presses "Kim’s jellyfin" in what the page lists, and plays
   the film. This is timed from opening the link to the film playing. It
   is the operator's 5 minutes as a program presses it, never a person's
   time.
3. *Bo's Jellyfin refuses Sam.* Sam's page lists no lend of Bo's. Then
   Sam's browser does what the page never offers: with the visit's own
   key, kept by the page (`web/store.js` `visitOf`), it signs a pass for
   Bo's jellyfin, as the page signs Kim's (`web/app.js` `open`), and opens
   it at its own address.
4. *Every home page shows Sam visiting.* Ana's, Bo's and Kim's home pages
   are read.
5. *A visit until tonight.* Kim presses **Have a guest** again and picks
   6 hours, the shortest the page offers (`crates/svrngs-node/src/home/home.js:91`),
   for a visit until tonight. Jo opens that link, gives "Jo", and plays
   Kim's film. With Sam and Jo, Oak has two guests and three members.
6. *A third guest is past the house's bound.* Ana presses **Have a guest**
   and keeps the 48 hours, and Max opens that link and gives "Max". Max
   would be the house's third guest with three members, so guests would no
   longer be fewer than members (GUESTS §3.7). Ana hosts no one else, so
   only that bound is passed, never the two per host.
7. *Kim's Stop ends Sam's visit.* Kim's home page: **Stop Sam’s visit**.
   Each computer is watched until it lists Sam no longer. Every home page
   is read, and Sam's film page is opened again.
8. *Tonight's visit ends then.* The driver waits until Jo's visit ends
   (the visit's `until`, as Kim's computer answered it), then watches each
   computer until it lists Jo no longer. Every home page is read, and Jo's
   film page is opened again. Under `TRIAL`, this step is not run.
9. *Kim's leave ends a third.* Kim makes a third link at 48 hours. Lee
   opens it, gives "Lee", and plays Kim's film. Then Kim presses **Leave
   Oak** on Kim's home page. Ana's and Bo's computers are watched until
   each holds Kim out and lists Lee no longer. Their home pages are read,
   and Lee's film page is opened again.

## 3. Plants

Each plant runs the whole story under `TRIAL` against a build with one
change. The run checks that each change applies once, and puts the tree
back after the build.
- `host`: the door carries a guest as their host. A visit's key is let in
  as Kim (door.rs `guest_of`, the `Asker`'s person the host, no host
  said).
- `bo`: a visit opens every lend in the group. The door's `own` and
  `named` are always true (door.rs `guest_of`).
- `count`: the fold counts no guests. svrngs-core's two bounds are dropped
  from the fold (`crates/svrngs-core/src/membership.rs:633`).
- `lapsed`: a visit that has ended still opens. The door reads every
  visit as live, never stopped and never its host's cut (door.rs
  `guest_of`).

## 4. Predictions

1. **Kim makes a link at the 48 hours it starts at.** The page asks "For
   how long, from when they open the link" with 48, 24 and 6 hours, 48
   chosen. Pressed, it shows a link on the page's address, `#visit=`
   then its token, within 5 s.
2. **Sam watches Kim's film within the operator's 5 minutes.** Sam's page
   asks a name once and then says "You’re Kim’s guest until …", 48 hours
   from when Sam opened the link (Kim's record: the visit's `until` within
   48 hours of its act, and at least 47 h 59 m past it). It lists "Kim’s
   jellyfin" alone. The film plays within **5 minutes** of opening the
   link. I expect under 60 s, through n0's relays on WebKit. Kim's
   stand-in is told "Sam (Kim's guest)" on every request, with Sam's visit
   key, never Kim's computer's, and an account name ending `.guest`.
   Plant `host`: red, told "Kim".
3. **Bo's Jellyfin refuses Sam, said which fact.** Sam's page lists
   nothing of Bo's. The pass Sam's browser signs for Bo's jellyfin is
   refused at Bo's door, and Sam's browser shows "jellyfin is not Kim's to
   open to you" (door.rs `guest_of`). Bo's stand-in gets no request from
   Sam. Plant `bo`: red, Bo's film opened to Sam.
4. **Every home page shows Sam visiting.** Within 30 s of Sam getting in,
   Ana's, Bo's and Kim's home pages each list "Sam, Kim’s guest, until
   <that day and hour>" under "Visiting now". Kim's page also offers
   **Stop Sam’s visit**, and the others offer no Stop. Each computer's
   `visiting` gives the same `until`.
5. **Kim's visit until tonight is made like the first.** The page offers 6
   hours. Jo is in as "Kim’s guest until …", 6 hours from opening, and
   plays Kim's film. Every home page lists Jo within 30 s.
6. **A third guest is not a guest on any copy.** Max's page says "This
   link lets nobody in. Ask for a new one." and lists nothing. Max's visit
   is in Ana's record and reaches Kim's and Bo's within 30 s. Ana's log
   says the visit was written and not counted (`redeem`, `counted=false`).
   No computer's `visiting` lists Max, and no home page shows Max, then or
   at any later read. Plant `count`: red, Max a guest on every copy.
7. **Kim's Stop ends Sam's visit on every copy.** Within 30 s of the
   press, no computer lists Sam, and no home page shows Sam. Jo is still
   visiting, and Max is still not. Sam's film page, opened again, is
   refused: "Kim ended your visit". Plant `lapsed`: red, Sam's film opens
   again.
8. **Tonight's visit ends then, on every copy.** Within 60 s after Jo's
   `until`, no computer lists Jo, and no home page shows Jo. Jo's film
   page, opened again, says "your visit with Kim is over: its time is up".
   Under `TRIAL` and in every plant: never-ran.
9. **Kim's leave ends Lee's visit on every copy.** Lee is in and plays
   Kim's film. Within 30 s of Kim's **Leave Oak**, Ana's and Bo's
   computers each hold Kim out and list Lee no longer. Their home pages
   show nobody visiting. Lee's film page, opened again, says "Kim is no
   longer a member, so your visit ended". Plant `lapsed`: red, Lee's film
   opens again.
10. **Words.** No page read says record, key, fold, standing, signer or
    actor (fence 8; DESIGN §1's words).
11. **Every page drawn,** every step finished, and no page error in any
    browser.
12. **A visit with real people**, the operator's measure: in the
    operator's own group over two weeks, a host has someone over twice,
    and a guest is watching within 5 minutes of opening the link with
    nobody helping. Its kill bar: a host hands a friend their own login
    instead, or nobody makes a second visit (`ralph/next/guests/STATE.md`,
    "The measure"). Never-ran, the operator's. Every people bar:
    never-ran.
