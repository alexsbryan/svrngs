# A newcomer's computer from the line — predictions, written before it ran

v1-onboarding (f) (`ralph/next/v1/STATE.md`), on this Mac (arm64), by
`run.sh` here: the page host served on localhost with
`scripts/build-release.sh`'s four tarballs under `get/`; Sam makes Trip
with a photo in its folder; the newcomer reads the line off the group's
page in Chromium on a computer and runs it with a HOME of its own.

1. `build-release.sh` builds all four tarballs, as it did at (a) (8m29s
   then; slower now if the host is loaded), and `shasum -c SHA256SUMS`
   says OK for each.
2. The line exits 0 inside 15 s: the tarball fetched over loopback (about
   10 MB), checked, installed in `~/.local/bin` (said not on PATH), the
   link joined through n0 (Sam's node and the newcomer's both on n0's
   lookup, as an installed svrngs is), the LaunchAgent bootstrapped, the
   home page's port answered, `open` given `http://svrngs.localhost:<port>/`.
3. The home page lists Trip and says "Trip’s folder is on this computer
   at <HOME>/svrngs/Trip" within 20 s of the paste.
4. Sam's beach.jpg is in that folder, byte for byte, within 60 s of the
   paste: the folder's first round after the agent's svrngs starts.
5. All of it well inside Demo C's five minutes on a computer, here; the
   download from a real host and a person reading the page are not in it.
6. The Linux tarballs build and are never run here: could-not-judge (no
   Linux host, no docker).
