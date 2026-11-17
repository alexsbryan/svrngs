# The invite flow, 2026-11-16

The whole gate on the invite flow's work, `03e7ac52..bcc9126f`, as asked by
the operator that day: a phone joining from a short address, Windows offered
no line, one **Invite people** press with a link, a short address and a QR
code, plainer words, the welcome's first hello a photo, a default resolver,
and the resolver's three words and four digits with miss budgets.

`check.txt` is `lockf -k /tmp/svrngs-gate.lock scripts/check.sh` at
`bcc9126f`: exit 0, 45 steps, 1,257 s.

The run before it, at `45fbaab7`, exited 1. Two Chromium tests timed out
waiting for the install line, because their driver opened the page as
Playwright's 'Desktop Chrome', which says Windows; `bcc9126f` says why and
what changed.

Not run here: a real phone, a real Windows computer, the people scene
(`docs/ONBOARDING.md` §6), and the resolver as deployed. The live
join.svrnme.sh still issues five words until it is redeployed, after a
native release that carries `c327032f` (`publication.rs` reads the new
phrase).
