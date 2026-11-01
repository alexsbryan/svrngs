# Predictions, before the iPhone run

This file is committed before any run, so that a result can't bend what was
expected of it. Each chance is a guess, not a rate. This is one run, and one
run is not a rate (AGENTS.md 7).

**The run.** The operator follows `NEEDS_HUMAN.md`, using the commit it
names:

- **The page:** a v2 build (`scripts/build-web.sh --ui v2`), served from
  this Mac to the operator's iPhone over the tailnet
  (`tailscale serve`, so it is https).
- **The computer:** Alex's, `target/debug/svrngs new Elm --as Alex --ui
  v2`, built with test-support, with a data dir of its own under
  `~/svrngs-scratch/` and `RUST_LOG=folder=debug`.
- **The phone:** Sam's, joining Elm by the page and its link.

The bars come from the queue's demo (`ralph/next/page/STATE.md`, "The
demo"). Each one is the fault from the operator's iPhone, undone:

| Bar | What passes | Chance | Why |
|---|---|---|---|
| P0 the way in | The page loads on the iPhone from the tailnet's https address, and Sam joins Elm. | 65% | No page has been served from this Mac to a phone before. `tailscale status --json` shows `CertDomains: null`, so the tailnet's https may still need turning on. The iPhone was last seen on the tailnet 57 days ago. None of this is v2's, but any of it stops the run before the bars. |
| P1 the answer beside the button | Sam ticks Events and taps "Bring the ticked ones to Elm". On that button's own line, the page then says "You brought Events to Elm: everyone in it can open it in a few seconds." The words above the boxes stay as they were. | 85% | `ui/shelf.js` draws the answer through `Act`, `answer: 'beside'`. The browser tests hold it in WebKit as an iPhone (`tests/page_v2/index.rs:402-409`). The risk is that real Safari differs from Playwright's WebKit. |
| P2 a second tap refused | Sam taps the button twice, quickly. While it brings, the button and boxes are held and the page says "Bringing…". Alex's log then has exactly one `shelf: the apps asked brought` line for the bring, and the answer is said once. | 80% | `SHELF`'s `bring` refuses a second bring while one is under way (`ui/shelf.js`, `BUSY`). The risk is that the bring is answered before the second tap lands. Events is then offered no more, so there is still one line, but nothing was refused. This is judged by the log's count either way. |
| P3 Events in the phone's list, no reload | Within 20 s of the answer, Events is in the list of Elm's apps on the phone, with the page loaded once. | 70% | After the answer, the page reads the group at 0, 1, 2, 4 and 8 s (`ui/shelf.js` `listed`, `TRIES` 4), 15 s in all. The group lists Events once Alex's computer has placed its files in the folder (carried.rs). On a real phone through n0's relays, that may take longer than the reads do. A miss would then show up on the next read, when the page is shown again. |
| P4 the computer's index, no reload | Alex's home page is open in a tab on the Mac, behind another tab, while the phone brings. When the tab is shown again, Events is in Elm's index. Nothing is reloaded. | 75% | The home page reads again on `visibilitychange` (`home/v2/home.js:60`). A home page left in front the whole time shows nothing new until it is hidden and shown again, which is the design (PAGE_STATE §3): it is recorded, not judged. |

**Where it would be wrong.** If P0 fails, the bars wait on the way in, not
on v2. If P3 fails while P4 passes, Alex's computer was slow to place the
files, not the phone's page. The phone's next `visibilitychange` would
then show Events, and that is to be recorded. If P2's log shows two lines,
the button was not held. That is the fault this queue was opened for
(`question.js:203-208`), so it is a failure, whatever the page said.
