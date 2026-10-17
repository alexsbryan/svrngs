# What the budget could not carry — start-home, 2026-10-13

The row's words: "Whatever the budget cannot carry goes into NEEDS_HUMAN.md
with its estimate, and the parts that fit go on." The parts that fit went
on: start-measure's ranked rows 4-10 and 12, and two the walk after found
(README §3). This is what is left, for the operator. The budget's call is
the operator's (CHARTER.md, "Leave for the operator").

`scripts/lean_check.py` at start-home's close: 15,994 / 16,000;
svrngs-core 3,121 / 3,130. Six lines are left.

## Making another group on a computer where svrngs starts at login

**What the walk found.** On Bo's computer, with svrngs running as
`install.sh` leaves it, `svrngs new Second --as Bo` exits 1: "svrngs is
already running on this computer (…/lock is held); stop it, then run this
again" (data/after/summary.json, `terminal.new`; data/before likewise).
`install.sh` keeps svrngs running from login, a LaunchAgent on macOS and a
systemd user unit on Linux (web/install.sh:16-20, 82-86), so no Ctrl-C
reaches it, and the line to stop it is launchctl's or systemctl's, not one
for someone a bit afraid technically. The page now says so in its
"Done in a terminal" list ("Where svrngs starts by itself when you log in,
a group cannot be made yet on this computer while it runs, and this page
cannot stop it"), and the walk's `link` and `link --device`, refused the
same way, are done on the page now.

**What would close it.** `svrngs new` made by the running node, as
`svrngs link` now is (invite.rs `Host::link`, ab6ef1f), and a button on the
page: "Make a group".

**Estimate: about 40 lines past the 16,000 cap** (34 more than the six
left). A move first, behaviour kept and near neutral: svrngs's group.rs
`draw`, `Seed`'s writing and cli.rs `keep_group` into the node (+10 of
glue). Then the change: `POST /group {name, person}` with its refusals
(+15), the page's box and button (+5), and the founder's folder placed as
cli.rs `folder_at_join` places it (+10). Not estimated: which relay a
group made by a running node is met at, which cli.rs:705-735 decides at
start from the groups already kept.

**Asked of the operator.** Raise the cap by about 40 for it, or leave
making a group in the terminal until a later queue.
