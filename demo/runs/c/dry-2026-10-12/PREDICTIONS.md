# Demo C's dry run — predictions, before any data

The operator, 2026-10-12, the page published by `scripts/deploy-page.sh`:
"We're published -- what's next? … can you do a run of the demo?" Demo C
(`demo/c.md`) is five people in their own homes; this is its steps on this
one Mac, every person a program, against the published page,
https://group.svrnme.sh/. It is not Demo C's run: nobody is a person, so
no people bar is judged (AGENTS.md 7), and no record goes in
`runs/c/<date>.json`. Written and committed before the driver ran.

## The setup

- **The page** is the published one: the group's link is on
  https://group.svrnme.sh/, each newcomer's line fetches `install.sh` and
  the darwin-aarch64 tarball from there, and the phone opens the page there.
- **Sam** and **Ana** have the published darwin-aarch64 tarball, checked
  against the published `SHA256SUMS`, unpacked into their own
  `~/.local/bin` (`demo/c.md`, "Before the evening"). Every person is a
  `HOME` of their own under one scratch directory, with `USER` their login
  (Sam aside, named by `--as Sam`): `ana`, `alex`, `bo`,
  `dee`, `eve` — a Mac's short login names, which `join` takes as the
  person's name when the line gives none (`crates/svrngs/src/cli.rs`,
  `user_name`; `runs/onboarding/2026-10-11/README.md`, finding 1).
- **Sam's library**: a scratch Jellyfin 12.0.0 (the macOS app's server, its
  own data dirs) with Demo A's three films, its middleware
  (`apps/jellyfin/mw`, debug) on 8097, lent by `svrngs share 8096
  --sign-in 8097`. Everyone plays Test Pattern One, the H.264 MP4 both
  engines play as it is, so nothing is converted.
- **Ana's music**: Homebrew's Navidrome 0.64.2 on 127.0.0.1:4533 with three
  albums of three tracks, made here, `ND_EXTAUTH_USERHEADER=X-Svrngs-Person`
  and `ND_EXTAUTH_TRUSTEDSOURCES=127.0.0.1/32`, Ana its admin; lent as
  `svrngs share 4533 --name music`. **The trip**: thirty JPEGs, about 150 MB,
  made here, put by Ana in `~/svrngs/lake-trip/trip/` once Ana is in.
- **The group** is `lake-trip`, named as a Discord channel would be.
- **One `install.sh` at a time.** The line keeps svrngs running under the
  one LaunchAgent label `sh.svrnme.svrngs`; on one host a second person's
  `launchctl bootstrap` of that label is refused while the first's holds
  it. So after each person's line and checks, the driver boots the label
  out and runs that person's svrngs itself, as the plist does (the same
  binary, `SVRNGS_DIR`, log), with their `HOME`. Each person's clock is
  done before that.
- **The browsers**: Playwright's Chromium on a computer's viewport for
  each newcomer; Playwright's WebKit as an iPhone 15 for Cy
  (`web/tests/page.mjs`). `open` is stubbed, as the onboarding run did, so
  the home page the line opens is loaded by Chromium.
- **Asleep** is `SIGSTOP` to Ana's svrngs and Ana's Navidrome: nothing of
  Ana's answers, and nothing is closed, as a lid shut is.

## Predictions

1. **The page and what it serves.** The link `new` prints is on
   `https://group.svrnme.sh/`; fetched as a chat's preview fetches it, the
   page carries the invitation's title and description; in Chromium on a
   computer it shows the line within 10 s of opening. The darwin-aarch64
   tarball matches the published `SHA256SUMS`.
2. **Ana, by the line.** Ana's line exits 0; Ana is in as `ana`, with
   the folder at `~/svrngs/lake-trip`. `svrngs share 4533 --name music`
   asks Ana a password (Enter alone for none) and says nothing of
   Navidrome's two settings: `crates/svrngs/src/well_known.txt` has no row
   for Navidrome, and `demo/c.md`'s "as `svrngs share` prints them" was
   never built.
3. **Each newcomer, Alex, Bo and Dee, on a computer**, from opening the
   link to the last of a film of Sam's playing, an album of Ana's playing
   and a photo from the trip opened from their own disk: **within 5 min**,
   the machine's floor for Demo C's bar. Within it: the line shown within
   10 s; the line exits 0 within 30 s; the home page lists `lake-trip`,
   its folder, Sam's jellyfin and Ana's music within 120 s of the paste
   (u7's joiner listed both lends at 52.6 s, `runs/u7/2026-10-11/take-2`);
   the film playing within 20 s of opening it and the album within 20 s,
   **neither asking anyone to sign in**; a trip photo on the newcomer's
   disk, the same bytes as Ana's, within 120 s of the paste.
4. **The names.** Each home page's people read `Sam, ana, alex, …` — the
   logins, as finding 1 of the onboarding run says. Navidrome makes its
   user from the header, named as the person is: `alex`, `bo`, `dee`.
5. **Cy, on a phone**: the group's page lists the group's people within
   **10 s** of the link opening; it asks Cy's name, which is typed (a
   finding against "nothing typed", for the operator); a film plays and an
   album plays through the page, both within **2 min** of the link.
6. **Removed** (`svrngs remove bo --group lake-trip` on Sam's computer):
   within **30 s**, Sam's jellyfin and Ana's music each refuse Bo's
   computer with svrngs's refusal of a removed person; Bo's home page says
   "You are no longer in lake-trip. …"; a photo Sam puts in the folder
   after reaches Alex's, Dee's and Ana's disks within 60 s and not Bo's in
   60 s; Bo's own `svrngs remove Sam` counts nowhere; Bo's trip photos stay
   on Bo's disk.
7. **Asleep** (Ana's svrngs and Navidrome stopped, then Eve's line): all
   thirty photos on Eve's disk within **2 min** of Eve being in, from
   whoever holds them; Eve's home page lists Ana's music as `ana`'s
   computer not answering; Sam's jellyfin plays for Eve, and answers each
   other member's computer.
8. **No accounts.** Jellyfin's users after are `sam` and one account the
   middleware named for each person who played (alex, bo, dee, Cy, eve);
   Navidrome's are `ana` and one Navidrome made from the header for each
   person who played an album (alex, bo, dee, Cy). No one was shown a
   sign-in, and nothing asked anyone for a key or a password but Ana's
   own `share` prompt (prediction 2).
9. **Apart: never-ran.** Every computer is on this one host; the two
   homes are Demo C's last unit (DESIGN §8).
10. **Every people bar: never-ran.** No person read the page, found a
    terminal, or chose what to click; each number above is the machine's
    floor under each bar, not the bar.

One host, one run of each: not a rate (AGENTS.md 7).
