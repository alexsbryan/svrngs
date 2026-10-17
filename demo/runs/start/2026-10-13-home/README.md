# The home page, walked again — 2026-10-13

Row start-home (`ralph/next/start/STATE.md`), its check: "start-measure
(c)'s walk again on the changed page, each act said done on the page or in
the terminal, with screenshots before and after". start-measure walked the
home page as someone a bit afraid technically and found it lists and leads
nowhere: of eleven acts five on the page, five only in a terminal and one
nowhere, no line to type named for any, and one silent computer holding
the whole page for 3.6 s (`demo/runs/start/2026-10-13/README.md` §3 (c),
§4). Its ranked list gave start-home rows 4-10 and 12 (§5).

This file's first commit holds the setup and the predictions, before any
data. The walk runs twice: **before**, on the page at this file's commit,
start-home's base; **after**, on the page start-home leaves. One host, one
run of each: not a rate (AGENTS.md 7). Nobody but the driver reads the
page: a word judged "to look up" is this run's reading, not a person's
(could-not-judge as a people bar).

## 1. Setup

`W=<scratch> python3 demo/runs/start/2026-10-13-home/run.py <before|after>`,
on svrngs's debug build at the commit walked, data in `data/<before|after>/`.

- **llama-server** as start-measure started it: Homebrew's, `-hf
  Qwen/Qwen2.5-0.5B-Instruct-GGUF:Q4_K_M --host 127.0.0.1 --port 18089 -c
  4096 --parallel 2`. Its answers are not judged.
- **Sam** `svrngs new Lab --as Sam`, and lends llama-server as start-llama
  lends it, nothing kept back typed: `svrngs share 18089 --kind
  llama-server --limit 4`; and a small page, `share <port> --name night
  --kind night`.
- **Bo**, the newcomer, put on as `web/install.sh` puts a computer on:
  `svrngs join <link> --as Bo --no-run`, then the bare `svrngs`.
- **Cy** `svrngs join <link> --as Cy`, and lends a page that asks: `share
  <port> --name taste --kind taste --ask "the films you have marked"`.
- **Dee** the bare `svrngs` over an empty data dir: in no group yet.
- **Eve** joins by the link Bo's page made, if it made one: `svrngs join
  <that link> --as Eve --no-run`.
- **The browsers**: Playwright's Chromium and WebKit (`web/node_modules`),
  `walk.mjs` opening a state's page, screenshotting it, listing its
  headings, words, buttons, links and boxes, and timing the first group's
  people apart from the whole load. **Each act is tried by name**: the
  button, else the link, whose name holds the words given, clicked; when
  none is on the page, that is said. Chromium is granted the clipboard, and
  read after a click.
- **Opening a folder**: a program named `open` and `xdg-open` first on
  each svrngs's `PATH` writes what it was asked to open to `opened.txt`,
  and opens nothing, so no window opens on this host.
- **The terminal**: what the page lacks, typed on Bo's computer while its
  svrngs runs, as it runs after `install.sh`: `svrngs link Lab`, `svrngs
  link Lab --device`, `svrngs new Second --as Bo`.

## 2. Predictions

### Before (this commit)

1. **Each act tried on the page** but opening a lend finds no control:
   "invite someone", "phone or another computer", "folder" and "Copy" are
   each said not on the page, in both browsers. Eve never joins: never-ran.
2. **"Sam’s llama"** is the link "Sam’s llama-server", and opens
   llama-server's chat page in the home page's own tab.
3. **At the terminal, while Bo's svrngs runs**, `link`, `link --device`
   and `new` are each refused, exit 1: "svrngs is already running on this
   computer (… is held); stop it, then run this again"
   (`crates/svrngs/src/cli.rs:545-548`, `claim`): on a computer svrngs
   runs at login, as `install.sh` keeps it, the terminal cannot invite
   anyone, add a phone or make a group.
4. **Asleep**: with Cy's computer stopped, Bo's page shows the group's
   people only once every computer has answered or not, about 3.5 s, as
   start-measure measured it.
5. **No group yet**: Dee's page says only that this computer lends
   nothing, and "What apps use" with nothing under it.
6. **After the restart** the page is found again at its address
   (start-address).
7. **No word of the fence's** in any state's words.

### After (start-home's last commit)

8. **At the top**, what the page is for and what can be done on it, in
   DESIGN §1's words; with no group yet, how to join one and to make one.
9. **Invite someone** and **add a phone or another computer** are done on
   the page: each button says what it makes before it makes it, and the
   page then shows the link, as `svrngs link` and `svrngs link --device`
   print it, and says for how long it works. Eve joins by the one the page
   made.
10. **The group's folder** is opened from the page: `opened.txt` names
    Bo's folder for Lab, and the page says it was opened.
11. **For a program**, each thing shared shows the address to give it,
    with a Copy button, said once in plain words; Chromium's clipboard
    holds the address after the click.
12. **Each thing shared** opens by a link saying what it is and whose,
    beside the home page, in a tab of its own.
13. **What stays in the terminal** is named on the page with the one line
    to type, and leaving a group is said plainly as not possible yet.
14. **Asleep**: Bo's page shows Lab's people within 1 s, while what Cy
    shares is still being asked.
15. **None of the fence's words**, and fewer to look up: "What apps use"
    is gone when nothing uses anything, the box says what its number is.

## 3. What the run gave

Written from the data, after each run: `data/before/` at f12597f, the
page c6c2212 left (`events.jsonl` "start", head); `data/after/` at
4bffef6, the page bece124 left. Each state's screenshots are in
`data/<before|after>/shots/<state>-<chromium|webkit>.png`, and after a
click `…-after.png`.

### Before

1. **Passed.** In both browsers, "invite someone", "phone or another
   computer", "folder" and "Copy": "no button or link named … on the
   page" (`summary.json`, `walk.<state>.<browser>.after.why`). Eve never
   joined: never-ran (`eve_join` null).
2. **Passed.** The link was "Sam’s llama-server", and it opened
   llama-server's chat page in the home page's own tab (`walk.lends-bo`,
   `after.at` a lend's origin, no tab).
3. **Passed.** `link`, `link --device` and `new` on Bo's computer, its
   svrngs running: each exit 1, "svrngs is already running on this
   computer (/tmp/sh-w/bo/.svrngs/lock is held); stop it, then run this
   again" (`terminal`).
4. **Passed.** Cy's computer stopped, Bo's page showed Lab's people at
   3,334 ms in Chromium and 3,419 ms in WebKit (`walk.asleep-bo`,
   `opened.people_ms`).
5. **Passed.** Dee's page: "What this computer lends", "This computer
   lends nothing.", "What apps use" with nothing under it, and its
   address (`walk.no-group-dee.chromium.text`).
6. **Passed.** After the restart the old address answered 200 in both,
   the address the same (`restart`: 63217 before and after).
7. **Passed.** No fence word in any state (`fence` empty throughout).

### After

8. **Passed.** Every state but the ask opens under "Your groups" with
   what the page is for: see your groups and who is in each, open what is
   lent, open each group's folder, make a link to invite someone or add a
   phone or another computer, choose how many people at once
   (`walk.<state>.headings`, `text`). Dee's, in no group: "This computer
   is in no group yet", with `svrngs new` and `svrngs join` to type.
9. **Passed, by the client's link alone.** "Make a link to invite someone
   to Lab" and "Make a link to add your phone or another computer of
   yours", each pressed in both browsers, showed "Send this link to the
   person you are inviting: svrngs://join/…" and "Open this link on your
   other computer or phone; it joins as you, so send it to nobody else:
   svrngs://join/…", each "It works for a day, while this computer is
   on." Lab keeps no page, so as `svrngs link` it printed no https link.
   Eve joined by the link WebKit's press made: exit 0, "You're in Lab, as
   Eve." (`eve_join`).
10. **Passed.** "Open this folder", pressed in each browser:
    `opened.txt` holds `open /tmp/sh-w-after/bo/svrngs/Lab` twice, and
    the page said "Opened, in a window of its own."
11. **Passed.** "Copy its address for a program", pressed: "Copied:
    http://ctn9x…localhost:61188/" in both; Chromium's clipboard held that
    address. The intro says once what it is for.
12. **Passed.** "Open Sam’s llama-server", target `_blank`: llama-server's
    page ("llama-ui") opened in a tab of its own, the home page kept
    (`walk.lends-bo.<browser>.tab`).
13. **Passed.** "Done in a terminal" names `svrngs share`, `svrngs
    remove` and `svrngs new`, each with its line, and "Leaving a group is
    not possible yet: nothing in svrngs does it." The terminal still
    refused all three of (3) while svrngs ran (`terminal`, exit 1 each):
    `link` and `link --device` are done on the page now; `new` is not
    (below).
14. **Passed.** Cy's computer stopped, Lab's people showed at 170 ms in
    Chromium and 175 ms in WebKit, then "Cy's computer did not answer, so
    what they lend is not listed." (`walk.asleep-bo`).
15. **Passed.** No fence word in any state. "What apps use" shows only
    after Bo's yes (`walk.after-yes-bo.headings`), and nowhere else.
    Each number box is labelled "How many people at once:"; before, it
    had only an `aria-label` and no words on the page (`boxes`).

### What the after walk found

- **The link made on the page ran out of its column** and could only be
  copied by selecting 130 characters by hand
  (`shots/invite-bo-chromium-after.png`). Fixed in ebfa0c6: the link wraps
  and has its own "Copy the link".
- **"Stop it with Ctrl-C" is not true for a computer set up by the one
  line.** `install.sh` keeps svrngs running from login, where no Ctrl-C
  reaches it, and `svrngs new` is refused while it runs. The page now says
  that (bd74631). Making a group on the page did not fit the budget:
  `NEEDS_HUMAN.md`, about 40 lines.
- **A link to a group that keeps no page says "They join with svrngs join
  and this link."** Someone without svrngs has no way in from that link
  alone. A group made with `--page` gives the https link, which leads to
  the one-line install. Not changed here: which groups keep a page is
  `svrngs new`'s, and the page says what the command prints.
- **Every load after Eve joined waited about 3.6 s**, though people showed
  within 0.6 s. Eve joined `--no-run` and never ran, so each list of what
  is lent waited the 3 s a computer is given to answer (`node.rs`
  `ANSWER_WITHIN`), then said "Eve's computer did not answer". That is the
  walk's own doing, and the page said it.
- **After the restart** WebKit showed Lab's people at 1,132 ms. That is
  one load on a node just started: one run, not a rate.

## 4. On the page, and in the terminal

| Act | Before | After |
|---|---|---|
| see groups, people, what is lent | page | page |
| open a thing lent | page, in its own tab | page, a tab of its own, its link saying what and whose |
| give a program a friend's model | not shown | page: address and Copy |
| say yes or stop to an app | page | page |
| change how many at once, stop lending | page | page |
| invite someone | terminal, refused while svrngs runs | page |
| add a phone or another computer | terminal, refused while svrngs runs | page |
| open the group's folder | a path to read | page |
| lend something (`svrngs share <port>`) | terminal, not named | terminal, its line on the page: the port and the kind are the lender's to type, and the program is theirs to start |
| remove someone (`svrngs remove`) | terminal, not named | terminal, its line on the page: who may remove whom is the core's, and nothing about it fit the budget |
| make another group (`svrngs new`) | terminal, refused while svrngs runs | terminal, the refusal said on the page; `NEEDS_HUMAN.md` |
| leave a group | nowhere | nowhere, said as not possible yet (CHARTER.md) |

One host, one run of each, and nobody but the driver reading the page.
Whether a person a bit afraid technically finds their way is
could-not-judge here (AGENTS.md 7).
