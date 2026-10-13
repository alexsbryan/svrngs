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

Written from the data, after each run.
