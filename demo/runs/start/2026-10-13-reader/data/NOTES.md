# Notes — following demo/start.md as a newcomer

Started 2026-10-13 19:07 PDT. Read start.md and its six pictures (sam-page,
sam-home-made, sam-home-link, bo-line, bo-home-chromium, bo-chat-chromium).
Group's page: http://localhost:63831/ (in place of https://group.svrnme.sh/).
"Before the page is published" skipped, as told.

Plan from the document, section "Trying both friends on one computer":
window 1 (Sam): HOME=/tmp/try/sam, steps 2–3, then bootout + run Sam's svrngs
in the foreground; steps 4–5 (5 in window 2 with HOME=/tmp/try/sam); window 3
(Bo): HOME=/tmp/try/bo, steps 6–9; clean up.

Each "terminal window" here is a separate shell (Bash tool calls do not keep
env vars, so each command line starts with its own `export HOME=…`).
Each browser person = its own Chromium persistent profile folder under
/tmp/start-reader/me/browser-sam, /tmp/start-reader/me/browser-bo.

## Step 1 — llama-server (19:08:08)
Typed the document's line exactly, in the background, output to
`llama-server.log`, with my own HOME (step 1 comes before the one-computer
section's `export HOME`). The model was already in this computer's cache, so
`/health` answered 200 within ~1 s (log: "listening on http://127.0.0.1:18089").
I did not open its chat page at http://127.0.0.1:18089/ as Sam; health was enough.

## Step 2 — the group's page, no link (19:08:24, page read in 2.4 s incl. browser start)
Opened http://localhost:63831/ in Sam's Chromium. It said, as the document
and its picture say: red "Open this page from your group’s link.", then "To
make your group, paste this line in a terminal on your computer: …", the line
`curl -fsSL http://localhost:63831/install.sh | sh -s -- 'http://localhost:63831/'`,
**Copy**, and the name box / Join / Yes / Not now (left alone, as told).
Browser console also logged an *error* "Said: Open this page from your group’s link."
Pressed **Copy**; the clipboard held exactly that line.

## One-computer section, window 1 + step 3 (19:08:32, 1.47 s)
Window 1: `export HOME=/tmp/try/sam; mkdir -p $HOME; cd $HOME`, then
`export PATH=/tmp/start-reader/stub:$PATH`, then pasted the copied line.
It said:
```
Getting svrngs for this computer from http://localhost:63831/get/svrngs-darwin-aarch64.tar.gz
Checked: svrngs-darwin-aarch64.tar.gz matches SHA256SUMS.
Installed svrngs in /tmp/try/sam/.local/bin.
/tmp/try/sam/.local/bin is not on your PATH, so to run svrngs yourself type /tmp/try/sam/.local/bin/svrngs; nothing else needs it.
svrngs runs now, and whenever you log in (/tmp/try/sam/Library/LaunchAgents/sh.svrnme.svrngs.plist).
Your home page, the same address each time svrngs starts: http://svrngs.localhost:64333/
```
Differs from the document only by the first line and the "not on your PATH"
line (the document lists that one under "When something does not work", so
expected). `open` (stub) was asked for http://svrngs.localhost:64333/; I
opened it in Sam's Chromium: "This computer is in no group yet. Make one
below, or join one by the line the page of the link you were sent gives." —
as the document says.

Then, still in window 1: `launchctl bootout gui/$(id -u)/sh.svrnme.svrngs`
(exit 0) and `SVRNGS_DIR=$HOME/.svrngs $HOME/.local/bin/svrngs`, run in the
background with output to `sam-svrngs.log`. It said "This computer is in no
group yet: make one with `svrngs new <name>`. Apps on this computer reach your
groups at http://127.0.0.1:64333. Your home page, …: http://svrngs.localhost:64333/
… Leave this running; Ctrl-C stops it." Same port 64333, as the document promises.

- Small doubt (not a stall): the terminal line "Apps on this computer reach
  your groups at http://127.0.0.1:64333" sounds like an address for a program,
  but the document's troubleshooting says giving a program the home page's
  address gets `{"error":"this page answers only itself"}`. A newcomer could
  paste that one into a program. I did not.

## Step 4 — make the group, then a link (19:09:01)
On Sam's home page typed `Lab` in "A name for the group", `Sam` in "Your
name in it", pressed **Make a group**: "You made Lab. Make a link below to
invite someone to it." 0.1 s after the press; "Lab’s folder is on this
computer at /tmp/try/sam/svrngs/Lab" — in Sam's home folder, as the section
intends. Pressed **Make a link to invite someone to Lab**: "Send this link to
the person you are inviting: http://localhost:63831/#AtaAp54K… It opens in a
browser, … It works for a day, while this computer is on." with **Copy the
link** (0.06 s). Pressed it; clipboard held the link. "Sent" it to Bo by
saving it in `link-for-bo.txt`. Matches both pictures.

## Step 5 — lend the model (19:09:09, 0.016 s)
Window 2: `export HOME=/tmp/try/sam; cd $HOME; export PATH=…stub…`, typed
`~/.local/bin/svrngs share 18089 --kind llama-server` and pressed Enter at the
password question (fed as a newline on stdin). It said the document's lines,
naming more parts kept back: "/slots, /v1/stream, /v1/streams,
/v1/chat/completions/control, /lora-adapters, /metrics, /tools, /cors-proxy,
/models", and also "To let fewer in at once, change it on your home page, or
`svrngs share 18089 --limit <number>`."
- **STALL (small, unsure):** in my run the password question was printed
  *first*, before "Lent llama-server …", not last as the document shows. My
  "terminal" was not a real tty (stdout piped), so this may only be output
  buffering; I could not tell which order a person in Terminal sees. If it is
  the real order, the person sees a password question before anything says
  what is being lent.
- Note: Sam's home page, "Done in a terminal", still says `svrngs share <the
  number …>` with no `--kind`; the document is what says to add it.

Sam's home page reloaded (19:09:18): under **What this computer lends**:
"llama-server: everyone in your groups at once, 0 using it now. How many
people at once: [ ] Let in this many  Stop lending". Under Lab, Sam's own page
also shows "Open Sam’s llama-server  Copy its address for a program".
The "Send this link …" block is gone after the reload (the link was copied
already, so fine).
Sam's part: from the first page open (19:08:24) to the lend listed (19:09:18)
≈ 1 min, all of it my own pace.

## Step 6 — Bo opens Sam's link (19:09:45, page read in 2.2 s incl. browser start)
Opened the link in Bo's own Chromium profile. Heading "Join Lab" (the picture
shows "Opening your link…", the state before it resolved), then "On this
computer, paste this line in a terminal: it gets svrngs, joins you, and opens
your home page, with the group’s folder on your disk.", the line
`curl -fsSL http://localhost:63831/install.sh | sh -s -- 'http://localhost:63831/#AtaAp54K…'`,
**Copy**, and "What’s your name? Everyone in Lab sees it." / Join / Yes / Not
now (left alone, as told). Here the red "Open this page from your group’s
link." did *not* show. Pressed **Copy**; clipboard held the line.
- Observation: the page's console shows it reaches an outside relay
  (`https://usw1-1.relay.n0.iroh.link/`) to ask Sam's computer what the link
  is for. The document never says anything leaves the computer; I did nothing
  to cause it beyond opening the link.

## Window 3 + step 7 — Bo pastes the line (19:09:53, 1.54 s)
Window 3: `export HOME=/tmp/try/bo; mkdir -p $HOME; cd $HOME`, `export
PATH=/tmp/start-reader/stub:$PATH`, pasted the line. It said:
```
… Installed svrngs in /tmp/try/bo/.local/bin.
/tmp/try/bo/.local/bin is not on your PATH, …
You're in Lab, as Alex Home.
Lab's folder is /tmp/try/bo/svrngs/Lab: what anyone in Lab puts there reaches this computer, and what you put there reaches theirs.
svrngs runs now, and whenever you log in (/tmp/try/bo/Library/LaunchAgents/sh.svrnme.svrngs.plist).
Your home page, the same address each time svrngs starts: http://svrngs.localhost:53527/
```
`open` (stub) was asked for http://svrngs.localhost:53527/; opened it in Bo's
Chromium (19:10:02): Lab, "People: Alex Home, Sam", Lab's folder
/tmp/try/bo/svrngs/Lab, and "Open Sam’s llama-server  Copy its address for a
program" — as the picture shows.
- **STALL (naming):** the document says it says *"You're in Lab, as Bo."*;
  mine said **"You're in Lab, as Alex Home."** The document explains svrngs
  takes the name the computer knows the person by, "nothing to type", so on
  one computer both friends' computers have the same owner and Bo is named
  after this computer's account, not Bo. Sam's page then lists "People: Alex
  Home, Sam". Nothing in the document or page says how to set or change that
  name (the page's name box is the one we were told to leave alone). I went
  on with it: it does not block anything, but a reader comparing to the
  document will think something went wrong.

## Step 8 — Bo asks Sam's model in its chat page (19:10:18)
On Bo's home page pressed **Open Sam’s llama-server**: a new tab opened
(0.24 s) at `http://44amsjtm3tljr5mc3gq8938kp0fdrs9003xv60gwdrp3h8m71bgtl0v0j15kh4.localhost:53527/`;
llama-server's chat page, "Hello there / Type a message or upload files to get
started", model Qwen/Qwen2.5 0.5B Instruct; message box ready 1.8 s after the
press. Typed `Name three rivers in Spain.` in "Type a message…" and pressed
Enter. First words 0.08 s after Enter; the answer finished well within 0.5 s:

> Three rivers in Spain are the Guadalquivir, the Serrano, and the Llanos.

(Guadalquivir is real; the rest is the small model, as the document warns.)
The page showed nothing wrong to the person. Behind it, as the document's
troubleshooting predicts: 403 on `GET /tools` ("llama-server is lent without
/tools", console error from the page's ToolsStore, twice — on open and on
send) and 403 on `POST /v1/streams/lookup`. Matches the document.
Bo, from opening the link (19:09:45) to the answer (19:10:23): ≈ 40 s at my pace.

## Step 9 — a program on Bo's computer (19:10:35)
On Bo's home page pressed **Copy its address for a program**: the page said
"Copied: http://44amsjtm3…kh4.localhost:53527/" and the clipboard held that
address. Put it in the document's Python program verbatim as `base_url`,
`api_key="none"` (saved as `bo-program/ask.py`), ran it with
`/tmp/sm-py/bin/python -I ask.py` (HOME=/tmp/try/bo). Answered, streamed,
in 2.3 s total including Python start:

> Spain is a country in Europe, and among its major rivers are the:
> 1. **Rio Segre** … 2. **Rio Segura** … 3. **Rio De Los Reyes** …

(Again the small model's facts.) No key asked for but the program's own field.
- Small doubt (not a stall): the copied address is the bare root (`…:53527/`),
  with no `/v1`. Many programs expect a base URL ending in `/v1`; it worked
  for Python's `openai` as-is, so I did not add anything.

Sam's home page afterwards (19:10:52): "People: Alex Home, Sam";
"llama-server: everyone in your groups at once, 0 using it now."

## Clean up (19:10:58 – 19:11:05)
As the section's step 4 says:
- `launchctl bootout gui/$(id -u)/sh.svrnme.svrngs` (Bo's) — exit 0.
- "Control-C in window 1 (Sam's)": Sam's svrngs ran as a background process
  here, so I sent it SIGINT (`kill -INT`, the same signal as Control-C); it
  exited.
- Looked at /tmp/try (only bo/ and sam/, each .local .svrngs Library opened
  svrngs), then `rm -rf /tmp/try`.
- Checked: no `sh.svrnme.svrngs` service left, no svrngs process left.
- **Not in the document:** I also stopped llama-server (it was started by me
  in step 1, in the background). The document's clean-up does not mention it,
  reasonably, since Sam runs it "as they always do".
- Left in place: the two Chromium profiles and my scripts/screenshots under
  /tmp/start-reader/me. Not touched: my real `~/.svrngs` (it existed before I
  began; nothing here used it).

## Summary of stalls
1. Step 5: the password question came *before* "Lent llama-server …" in my
   (non-tty) run, not last as the document shows — unsure whether a real
   Terminal shows the same.
2. Step 7: "You're in Lab, as **Alex Home**", not "as Bo"; on one computer
   Bo gets this account's name, and nothing says how to name Bo.
No step blocked. Everything else read as the document and its pictures say.
Total: 19:08:08 (llama-server) → 19:10:45 (program answered), ≈ 2.6 min.

REACHED: yes
