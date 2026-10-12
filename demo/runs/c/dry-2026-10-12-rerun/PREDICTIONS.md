# Demo C's dry run again — predictions, before any data

Row bash-rerun (`ralph/next/bash/STATE.md`): Demo C's dry run
(`demo/runs/c/dry-2026-10-12/`) again, on what the bash queue landed —
bash-first-minute, bash-limit, bash-person and bash-devices. Its steps on
this one Mac, every person a program, as before. Its directory is
`dry-2026-10-12-rerun/`, not the row's `dry-<date>/`: the first dry run
holds that name, on the same date. It is not Demo C's run: nobody is a
person, so no people bar is judged (AGENTS.md 7), and no record goes in
`runs/c/<date>.json`. Written and committed before the driver ran.

## The setup, and what changed from the dry run's

As `dry-2026-10-12/PREDICTIONS.md`'s setup, but:

- **The page is served on this host**, never published (the queue's fence
  7): `scripts/build-web.sh`'s site with `scripts/build-release.sh`'s `get/`
  in it, built from this commit, served on localhost by
  `web/tests/serve.mjs`, as `demo/runs/onboarding/2026-10-11/` served it.
  Sam's link, every newcomer's line, `install.sh`, the tarball and Cy's
  phone all come from there.
- **Sam's folder from `new`** (finding 7, bash-first-minute (d)): no
  `share --folder`.
- **Ana lends with `svrngs share 4533`** (finding 2, bash-person (d)) and
  sets Navidrome as it prints, then starts Navidrome again with that.
  Navidrome first runs with no ExtAuth setting at all.
- **No `--limit`** on either lend, as `demo/c.md` names none (finding 3,
  bash-limit (a)).
- **Asleep's words** are read with the plain apostrophe the home page
  writes ("ana's computer did not answer"; the dry run's README, "When 'did
  not answer' was first said is not measured").
- **Jellyfin's and Navidrome's versions** kept apart in `summary.json`.
- **`name.py`'s second `ana`** joins at the line, a computer whose login is
  `ana`, after Ana's; the page it reads is the run's site, served again.
- **The media** — thirty JPEGs at 4032×3024, three albums of three tracks,
  `after.jpg` — made here by `media.sh`, since the dry run's were made
  outside its driver and not kept.

## Predictions

1. **The page and what it serves.** The link `new` prints is on the local
   page's address; fetched as a chat's preview fetches it, the page carries
   the invitation's title and description; in Chromium on a computer it
   shows the line within 10 s of opening. The darwin-aarch64 tarball
   matches the `SHA256SUMS` served beside it.
2. **Sam, by `new`; Ana, by the line.** `new` says where the group's folder
   is, `~/svrngs/lake-trip`, and it is there. Ana's line exits 0; Ana is in
   as `ana` with her folder there. `svrngs share 4533`, nothing else typed,
   lends `navidrome`, asks no password, and prints
   `ND_EXTAUTH_USERHEADER=X-Svrngs-User` and
   `ND_EXTAUTH_TRUSTEDSOURCES=127.0.0.1/32`. Each `share` says, in a
   person's words, that the lend lets in everyone in the group at once.
3. **Each newcomer, Alex, Bo and Dee, on a computer**, from opening the
   link to the last of a film of Sam's playing, an album of Ana's playing
   and a trip photo opened from their own disk: **within 5 min**, the
   machine's floor for Demo C's bar. Within it: the line shown within 10 s;
   the line exits 0 within 30 s; **the home page lists `lake-trip`, its
   folder, Sam's jellyfin and Ana's navidrome within 5 s of the line's
   exit** — bash-first-minute (b)'s bound (`HEARD_WITHIN`,
   `crates/svrngs/tests/join_by_link.rs:732`), here with Chromium's start
   inside it, against the dry run's 45–50 s; the film playing within 20 s
   of its play and the album within 20 s, **neither asking anyone to sign
   in**; a trip photo on the newcomer's disk, Ana's bytes, within 120 s of
   the paste.
4. **The names.** Each home page's people read `Sam, ana, alex, …`: these
   logins have no full name on this host (`id -F alex`: "no such user"),
   so `join` falls to `$USER`. Navidrome signs each newcomer in as **their
   own account, named by `x-svrngs-user`**: `<name>-<the first 16 hex of
   their key>`, the key `/v1/rail/people` gives as their `person_key` —
   never `ana`, never an admin.
5. **Cy, on a phone**: the group's page lists the group within **10 s** of
   the link opening; it asks Cy's name, which is typed (finding 5, the
   operator's); a film and an album play through the page, both within
   **2 min** of the link; Navidrome signs Cy in as `Cy-<16 hex>`.
6. **Removed** (`svrngs remove bo --group lake-trip` on Sam's computer):
   within **30 s**, Sam's jellyfin and Ana's navidrome each refuse Bo's
   computer with svrngs's refusal of a removed person; Bo's home page says
   "You are no longer in lake-trip. …"; a photo Sam puts in the folder
   `new` made reaches Alex's, Dee's and Ana's disks within 60 s and not
   Bo's in 60 s; Bo's own `svrngs remove Sam` counts nowhere; Bo's trip
   photos stay on Bo's disk.
7. **Asleep** (Ana's svrngs and Navidrome stopped, then Eve's line): all
   thirty photos on Eve's disk within **2 min** of Eve being in; Eve's
   home page, at its first look that lists the group, and Alex's say
   "ana's computer did not answer"; **no running computer is said not to
   answer, nor to have not heard, nor to have refused, on any home page
   look the driver keeps** (finding 6, bash-first-minute (c)); Sam's
   jellyfin plays for Eve and answers each other member's computer.
8. **No accounts.** Jellyfin's users after are `sam` and one account for
   each person who played (alex, bo, Cy, dee, eve); Navidrome's are `ana`
   and one for each who played an album (alex, bo, Cy, dee). **Each
   person's two accounts have one name**, the door's `x-svrngs-user`. No
   one was shown a sign-in, and nothing asked anyone for a key or a
   password.
9. **The limit.** With no limit on either lend, no newcomer is refused
   `full` by any door (the kept door logs). Whether a lend with no limit
   lets six in at once is **could-not-judge** here: on one host the door
   guesses each computer is in its lender's home and counts none
   (`demo/runs/devices/2026-10-12/README.md`, finding 1), and the
   newcomers go one at a time.
10. **A second `ana`** (`name.py`): a computer whose login is `ana` runs
    the page's line for a group where Ana already stands. The line exits
    non-zero; svrngs says, in a person's words, that someone in the group
    already goes by `ana`, and how to add this computer from one of theirs
    (`svrngs link <group> --device`) or to join under another name; the
    line says it did not join; no LaunchAgent is left. The group's people
    after are Sam and `ana`; Navidrome's users are `ana` alone, so no
    second account. A member opening Ana's Navidrome through the lend is
    signed in as their own `<name>-<16 hex>`, not as Ana's admin.
11. **Apart: never-ran.** Every computer is on this one host; the two
    homes are Demo C's last unit (DESIGN §8).
12. **Every people bar: never-ran.** No person read the page, found a
    terminal, or chose what to click; each number above is the machine's
    floor under each bar, not the bar.

One host, one run of each: not a rate (AGENTS.md 7).
