# V1.1.2, checked

V1.1.2 is V1.1.1 (`6db0cccf`) with a group on a screen and nothing else. It
was built from `aacb3fe5`, tagged `v1.1.2`, and published to
`group.svrnme.sh` as Vercel deployment `dpl_3gdXz6rRDQU15U5HqcQcHYQFEaLD`.

- `check.out`: `scripts/check.sh` at `aacb3fe5`, all 15 steps passed, exit
  0, in 17m53s.
- `SHA256SUMS` and `live-sums.txt`: the live site's list, the same bytes
  as the build's, and all four tarballs fetched from the live site matching
  it. `install.sh`, `index.html`, `app.js`, `question.js` and `look.css`
  were each the same as the build. The page's policy is
  `connect-src 'self' wss: https:`, as `scripts/trust_floor.txt` pins it,
  and the host sends `frame-ancestors 'none'`. `.env.local`,
  `.vercel/project.json` and `vercel.json` answered 404.
- **The Linux download.** Fetched on Fedora as `install.sh` fetches it, it
  matched the sums. Its binary, `850005d5…`, is the one `smoke/` ran,
  before it was published.
- **The Mac download** (darwin-aarch64, `22779bd6…`) runs `--help` and
  holds the screen's page.
- `smoke/`: `../../screen/2026-10-18/rehearse.py` on that Linux binary.
  It's the 25-computer burst, with Priya's screen and Aiko's open on the
  Mac, then Priya's computer frozen.

## The smoke, against V1.1.1's

V1.1.1's figures are from its own burst at one wall read a second
(`../../v1.1.1/2026-10-18/`) and V1.1's
(`../../twentyfive/2026-10-17/`).

| | V1.1.2 | V1.1 / V1.1.1 |
|---|---|---|
| joins in | 24 of 24, median 3.33 s, at most 3.53 | 24 of 24, median 3.3 s, at most 3.4 |
| lit on the wall, from the join's start | median 3.88 s, at most 4.83 | median 3.8 s, at most 4.2 |
| lend on the wall, from `share` returning | median 0.54 s, at most 1.5 | within 1.5 s |
| wall settled, after the last join started | 4.25 s | 3.3 s (V1.1, read back to back) |
| each window on Priya's screen | median 4.06 s, at most 4.54 | none |
| each lend on Priya's screen | median 0.74 s, at most 1.21 | none |
| Priya drawn "did not answer" on Aiko's screen, after she froze | 3.6 s | none |
| Bo's greeter through Aiko's computer, Priya frozen | 200, as Aiko | none |
| Priya's memory, peak | 146 MiB | 98.3 and 101.9 MiB |
| the other 23 computers' memory, peak | 103 MiB | none |

This smoke asks more than V1.1.1's did. Two screens and the wall each ask
every computer once a second, three asks a second against one. Priya's
computer asks twice a second, for her screen and the wall, and Aiko's
once. Priya's peak and Aiko's 113 to 118 MiB are from that, read by `ps`
every 10 s: one run, not a rate.
The 75 greetings of V1.1.1's smoke aren't in this one. One visit, made
after Priya froze, is.
