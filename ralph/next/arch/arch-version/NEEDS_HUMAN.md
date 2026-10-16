# arch-version: what is the oldest build a page still has to talk to?

The row stops here by its own words ("if any has, NEEDS_HUMAN.md listing
them, the oldest the operator's to name"), and so does CHARTER ("Leave for
the operator: the oldest build"). The evidence is in `SURVEY.md` beside this
file, committed as 6d3c7352.

## The builds that left this host

1. **2026-10-05.** An iPhone ran the page `c891fade9e7f` from
   www.svrnme.sh/g/. The computer it reached was on this host.
2. **2026-10-07.** In Demo B, Sam's computer ran at `4b2cdfa` on a machine
   outside Dee's network, built from source.
3. **2026-10-12 to today.** `https://group.svrnme.sh/` serves the page and
   `get/`'s four tarballs, built from about 27d5ff9a, to anyone holding a
   group's link. Today's `SHA256SUMS` matches what Demo C's dry run recorded.

## What each answer means for part (b)

- **The published build (27d5ff9a), the oldest a page must talk to.** It
  predates every page-side branch the row names: bash-first-minute (c),
  v1ui-door (c)/(d) and v1ui-shelf (a)/(c), all from 2026-10-12 08:20 or
  later. Nothing comes out. Part (b) can still add one version to the answer
  and say "runs an older svrngs" once. That costs lines, and there is no
  budget for them.
- **None older than v1ui-door; the published build and Sam's 4b2cdfa are
  let go.** Every branch in `web/app.js` and `question.js` goes, and so do
  the `signs_in` and `group` serde defaults. The `reads` and `asks`
  defaults (u5-2, sup-2) also go, since both predate the published build. A
  computer installed from group.svrnme.sh then reads to a page as "runs an
  older svrngs", once, and no field-by-field fallback is drawn. This is the
  saving the audit counted (~25–40 lines).
- **Let them go, and republish first.** Republishing with
  `scripts/deploy-page.sh` is outward-facing, so it is yours (CHARTER: "Leave
  for the operator: outward-facing steps: publishing"). It is the only way
  the published computers and the published page catch up. Without it, a
  phone opening any group.svrnme.sh link runs the 2026-10-12 page against
  this build's computers (`SURVEY.md`, "Two follow-ons").

## What I did not do

No product change, and no deploy. The site was read with `curl` and never
written. I downloaded the darwin tarball to /tmp and ran it once, to look
for a version string. It has none: `svrngs --version` is "unknown command",
so today a build cannot say which one it is. That is one reason part (b)
carries a version at all.

## What would falsify the survey

- A run kept with `deploy-page.sh`'s printed HEAD that differs from about
  27d5ff9a.
- A record of another off-host computer that `demo/` does not hold (for
  example, someone who installed from group.svrnme.sh). Either would change
  which build is oldest, not whether one left.
