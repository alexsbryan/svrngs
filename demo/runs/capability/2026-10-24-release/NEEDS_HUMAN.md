# The capability release: prepared, for the operator to publish

capability-release (`ralph/next/capability/STATE.md`) prepares the release
and stops: publishing is the operator's (CHARTER, "Leave for the operator";
fence 9). Nothing below has been run.

## What it is

The next release, `RELEASES.md` "Next, not released yet": House rules
(the canon queue, already there) and this queue, what can act as you.
`README.md` beside this file maps each claim in the notes to its row's
commit, and names the gate this row's close passed.

## The one decision: publish it, and under what name

The notes call it "the next release". V1.2 is the name this file assumes,
since it brings two features past V1.1.3; the name is yours.

## The publish, as `scripts/deploy-page.sh` and the last release ran it

From the commit `README.md` names as the close's (`<commit>` below), in a
worktree of its own so the loops' tree and `target/` stay apart (memory
of V1.1.3: a shared `target/` broke the loops' builds):

```
cd ~/dev/svrngs
git worktree add --detach ../svrngs-v1.2 <commit>
cd ../svrngs-v1.2
npm ci --prefix web                 # a fresh worktree has no web/node_modules
scripts/check.sh                    # optional: the close already passed at <commit>
rm -rf target/debug                 # the release build needs the disk
git tag v1.2 <commit>
scripts/deploy-page.sh              # builds the page and the four tarballs, then vercel deploy --prod
curl -fsS https://group.svrnme.sh/get/SHA256SUMS   # read back: four tarballs, this build's sums
rm -rf .vercel .env.local           # what the Vercel CLI leaves in the upload dir, if present
cd ~/dev/svrngs && git worktree remove ../svrngs-v1.2
git push origin main v1.2
```

`deploy-page.sh` reads `SHIPPED` (`crates/svrngs/src/cli.rs:257`, `N0` at
this commit), so this build meets its groups at n0's relays and publishes
at once; the switch to svrngs's relay is not part of this release (PLAN §2).

Then `RELEASES.md`'s heading "Next, not released yet" becomes "V1.2,
<date>", with "V1.2 was built from commit `<commit>`." at its end, as
V1.1's is.

## What to know before you publish

- **Every computer should update.** A computer on V1.1.3 counts no line a
  phone signs on the new page (it names it `NewerVersionLine`; dcd8c4a3,
  3caa06f5), and a phone opening an app lent by one waits to send what it
  writes, saying so (`web/lend.js:488`; 0c9032ed). The notes say both.
- **Phone people may be asked once.** A phone's yes to an app's new version
  went through a computer before, and now counts only from the phone, so
  that person's version falls back until they tap "Open the new version"
  (dd932d60; `demo/runs/authorship/2026-10-24/DESIGN.md` §10). The notes
  say so.
- **Not built, from the note's §9:** the shelf's line "An app you bring can
  read and write its own part of the group's record, and can't load code
  from anywhere else. Bring what you trust. An app you open can share what
  it reads." The shelf says none of it today
  (`crates/svrngs-node/src/home/question.js:178`). The notes and DESIGN
  §3 say the trade, the page does not. No row held it; a row of its own if
  you want it before publishing.
- **Not run with people.** Every scene is a program on one host; nobody has
  used this release in a real group (AGENTS.md 7).
