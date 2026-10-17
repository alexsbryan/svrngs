# The primitive event on one host — setup and predictions, 2026-10-14

Row events-run (`ralph/next/events/STATE.md`): Sam hosts one dinner drawn
from two of Sam's groups, The House and dWeb, each on its own computers,
through the reference app (`apps/events/`, at a22edd0) that Sam's computer
lends. Written before any data; `README.md` here says what the run gave,
against each line below. Every person is a program: no people bar is
judged (AGENTS.md 7). One host, one run: not a rate.

## 1. Setup

`W=<scratch> python3 demo/runs/events/2026-10-14/run.py`, data in `data/`;
with `PLANT=<name>`, the same story against a planted copy of the app,
data in `data/plants/<name>/`. `judge.py` reads a run's `summary.json` and
writes its `verdicts.json`, one verdict a bar (AGENTS.md 6: passed, failed,
could-not-judge, never-ran). scripts/check.sh runs none of this.

- **svrngs** as the devices run built it (`demo/runs/devices/2026-10-12/`):
  `cargo build -p svrngs` at this commit, every computer a `HOME` and data
  dir of its own under `W`, each svrngs run by the driver, none under
  launchd. The page built by `scripts/build-web.sh` and served on this host
  by `web/tests/serve.mjs`, nothing published. n0's public relays, the
  default, as every run of several computers here has met.
- **The cast.**
  - The House: Sam's computer, Bo's computer, Cy's phone; later Fay's
    computer.
  - dWeb: Sam's computer, Di's computer, Eve's phone.
  - Sam makes The House (`svrngs new "The House" --as Sam --origin <page>`);
    Bo joins by its link (`svrngs join <link> --as Bo`), Cy's phone opens
    its page address. Sam's svrngs stops and makes dWeb the same way (`new`
    claims the data dir, `cli.rs` `claim`); Di and Eve join by its link.
- **The app.** `node apps/events/server.mjs` on Sam's computer, config
  `{"record": <Sam's API>, "who": "x-svrngs-key"}`, `EVENTS_DEBUG=1` to its
  trace; Sam lends it, `svrngs share <its port> --kind events --name
  events`, no password.
- **Computers** open the lend in Chromium at the origin their own svrngs
  lists for it (`/v1/lends`), Sam's own included. **Phones** are WebKit as
  an iPhone (`web/tests/page.mjs`), one browser each: Cy and Eve open their
  link, then the event at `<page>#open=<label>&at=/g/<group>/e/<event>`
  (`web/app.js:29`), the label the one in the lend's origin.
- **The story, in order.**
  1. Sam posts "Dinner", "Friday 7", "the house", to The House and dWeb, in
     one form.
  2. Bo says he's coming; then, changing his mind, types "Maya" and saves.
  3. Cy (phone) says she's coming. Di says he's coming. Eve (phone) says
     she's coming.
  4. Di's computer goes to sleep (its svrngs stopped).
  5. Eve takes her name off.
  6. Cy looks for a "Maya came" button on her page and presses it if there
     is one (the guest plant's door in).
  7. Bo says he came, and that Maya came; Cy says she came.
  8. Di's computer wakes. Di says he came.
  9. Bo makes a link to The House (`svrngs link "The House"`, which runs
     on); Fay's computer joins by it, opens the event, says she's coming.
  10. Each person's view of the event, read: Sam's, then Bo's, Cy's, Di's,
      Eve's and Fay's. Cy opens dWeb's address of the event, and Di The
      House's.
  11. Each computer's copy of each of its groups' records, read and folded
      by `events.js`: The House on Sam's, Bo's and Fay's; dWeb on Sam's and
      Di's.
  12. Sam's computer goes to sleep (its svrngs stopped; the app stays up).
      Bo's browser and Cy's phone open the event; the app is asked on
      loopback as the door would ask it for Sam. Sam's computer wakes; Bo
      opens the event again.
- **Every press** the driver makes is written down (who, which group, what
  it said); the record is then read against it.
- **Plants**, each the whole story against a copy of the app with one
  change, the change checked to have applied:
  - `cross`: every page is the host's view of every group this computer
    holds (`server.mjs`, `if (e.host === me)` made always, all groups,
    `me` the host) — an answer crossing groups through the app.
  - `guest`: every guest's "came" button shown to everyone in the group,
    and its act written as the member who named the guest — a guest's
    `came` said by someone other than their member.
  - `no`: "Take my name off" writes `{"kind": "not-coming"}` past the
    app's validator — a "no" recorded.

## 2. Predictions

1. **Posted once, to both**: Sam's one press writes one `event` in each
   group's record under one id; Sam's page lands on The House's event
   within 5 s of the press.
2. **Answered from a computer and a phone, in each group**: each answer a
   press of its own person, within 5 s on a computer; a phone's event page
   open within 30 s of `#open=`, each of its presses answered within 15 s.
   Each act lands in its own group's record, written by Sam's computer for
   its person (`on_behalf_of`): Bo's and Cy's in The House, Di's and Eve's
   in dWeb.
3. **A guest is a name, and only their member says they came**: The
   House's record holds Bo's `coming` with `guests: ["Maya"]` and Bo's
   `came` with `guest: "Maya"`; no "Maya came" is offered on Cy's page;
   Sam's view says "bo + Maya" and "Maya, bo's guest". Plant `guest`: red,
   by the record read against the presses (a `came` for Maya by Bo that Bo
   never pressed).
4. **Changing your mind is a correction**: Bo's first `coming` corrected
   into the one with Maya; Eve's `coming` voided by her own correction.
   Eve appears nowhere on any page but as invited; no page says she is
   not coming.
5. **Sam's view**: two sections, The House and dWeb, each with its people
   but Sam as Invited, its Coming and its Came: The House invited Bo, Cy
   and Fay; coming "bo + Maya", Cy, Fay; came Bo, Cy, "Maya, bo's guest";
   dWeb invited Di and Eve; coming Di; came Di. (Names as each person gave
   them: Bo, Cy, Di, Eve, Fay.)
6. **Each other person's view is their own group's alone**: one section,
   their own group's; Cy refused dWeb's address of the event, and Di The
   House's, 403, "You're not in …, so this event isn't yours to see."
   Plant `cross`: red, by Bo's page holding dWeb's section.
7. **Every group's copies fold the same**: The House's on Sam's, Bo's and
   Fay's computers, dWeb's on Sam's and Di's, each the same answers within
   30 s of the last act; Di's copy, asleep through Eve's correction and
   the came's, holds them within 60 s of waking. Phones hold no copy.
8. **Nothing records "no"**: every payload in both records is `event`,
   `coming` or `came` with only the fields `events.js` reads; no page read
   says not coming, declined or no-show. Plant `no`: red, by the
   `not-coming` payload in dWeb's record.
9. **Every word in the record is its own person's**: each act in each
   record matches a press by the person it names, in that group.
10. **A group whose computers are asleep**: with Sam's computer asleep,
    Bo's browser is answered by svrngs, not the app, `502`,
    `x-svrngs-refused: away`, in JSON ("the computer that lends this did
    not answer; try again later", `origin.rs` `away`), within 30 s: said as
    that, in braces, not a page (as start-run found of the door's words).
    Cy's phone says "The computer lending this did not answer. Try again
    later." (`web/lend.js`). The app, asked on loopback for Sam, answers
    `502` "This group can't be read on this computer right now." Never an
    event page with no one coming. Sam's computer awake again, Bo's page
    holds the same answers as before it slept.
11. **What "invited" means**: Fay, joined after the post, sees the event
    on her page, can say she's coming, and is in Sam's Invited for The
    House: the app as built takes "invited" as whoever is in the group now.
    The run keeps each person's `joined` beside the event's posting time,
    so both readings are in the data. Decided after the run, in README.md.
12. **Words**: no page read says key, member, record, act, namespace or
    origin.
13. **A phone on cellular, a second host, real people**: could-not-judge
    here; the operator's (CHARTER.md). Every people bar: never-ran.
