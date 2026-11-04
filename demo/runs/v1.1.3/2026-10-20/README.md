# V1.1.3, checked

V1.1.3 is V1.1.2 (`aacb3fe5`) with four changes and nothing else:
- **#205:** a phone's page asks the next computer when the first one's
  answer fails.
- **(a), (b):** the away page, a lend's computer that did not answer drawn
  as its person's window, dark, on a computer and on a phone.
- **(c):** gzip asked for and unpacked by a phone's lend worker, so
  llama-server's chat page opens on a phone.
- **(d):** a phone's sockets come back with their lender.

It was built from `d1b4309e`, tagged `v1.1.3` (local, on no branch), and
published to `group.svrnme.sh` as Vercel deployment
`dpl_GvHAjrs1bqqYXaFBrunmwsLVgJE2`. On main the same changes are
`f65b5977`..`a620548e`, written against main's files. Each check was red
at its parent on both lines, as each commit body says.

- **`check.out`:** `scripts/check.sh` at `d1b4309e`, in the release
  tree's own target dir. All 15 steps passed: "check.sh exit 0".
- **`SHA256SUMS` and `live-sums.txt`:**
  - The live site's list is the build's, and all four tarballs fetched
    from it match it.
  - `install.sh`, `index.html`, `app.js`, `lend.js`, `away.html`,
    `look.css` and `question.js` are each the same as the build.
  - The page's policy is the one `scripts/trust_floor.txt` pins, and the
    host sends `frame-ancestors 'none'`.
  - `.env.local`, `.vercel/project.json` and `vercel.json` answered 404,
    and the upload directory is deleted.
- **The Linux download:** fetched on Fedora as `install.sh` fetches it,
  it matched the sums. Its binary, `0e40791f…`, is the one `smoke/` ran,
  before it was published.
- **The Mac download** (darwin-aarch64, `1844f7f0…`) runs `--help`, and
  holds the away page.

## The smoke

`smoke/smoke.py` runs from the Mac:
- **The computers:** three svrngs computers on Fedora (`computers.py`),
  each from the release's Linux binary. Ada starts House. Cy lends a
  shelf, a page that also holds a socket. Lee lends llama-server, an
  August build (1464c62) that answers `GET /` 415 without gzip, with
  Qwen2.5-0.5B-Instruct.
- **The phone:** an iPhone in WebKit, then in Chromium (`phone.mjs`), on
  the release's page served on the Mac. It joins House by its link over
  n0's relays.
- **The lid:** cy's computer is stopped with SIGSTOP and started again
  with SIGCONT, then stopped again. Then ada's, the link's, is stopped.
- **A computer's tab** at cy's shelf, on ada's computer through a tunnel,
  is read while cy's is stopped (`desktop.mjs`).

Every person is a program and there is one run of each engine, so no
people bar is judged here (AGENTS.md 7).

| | WebKit | Chromium |
|---|---|---|
| joined House by the link | 2,879 ms | 2,254 ms |
| lee's llama-server chat page | 200 "llama-ui", a box to type in, 3,253 ms | 200 "llama-ui", a box to type in, 2,677 ms |
| a socket at cy's shelf | open | open |
| cy's computer stopped: a socket | closed 1006 | closed 1006 |
| started again: a socket | open on the first try | open on the first try |
| stopped again: the shelf's address | 502 `away` in 4,012 ms: "☾ cy’s window is dark … so cy’s shelf can’t open right now … Goodnight from cy." | the same in 4,013 ms |
| ada's computer stopped too: the group's page | in through lee's computer, 4,065 ms; ada's and cy's "did not answer" | the same, 3,928 ms |

The first socket after the restart was asked as cy's computer came back,
and opened 144 ms (WebKit) and 146 ms (Chromium) after it was asked.

A computer's tab at cy's shelf, cy's stopped, on ada's computer: 502
`away` in 4,046 ms, "☾ cy’s window is dark", "cy’s computer didn’t
answer, so this can’t open right now. It opens again once that computer
answers.", "Try again", "Goodnight from cy." The shots are
`smoke/data/shots/computer-away.png`, `webkit-4-away.png` and
`chromium-4-away.png`.

## main's check, three times

`scripts/check.sh` ran three times at main's `a620548e`, in its own
target dir. None exited 0.

| run | output | failed | why |
|---|---|---|---|
| 1 | `main-check-1.out` | `tests` | a known flake |
| 2 | `main-check-2.out` | `ceiling canary` | load |
| 3 | `main-check-3.out` | both | both |

Every other step passed every time, `browser` included, where V1.1.3's
tests run.

- **The flake:** guests' film test,
  `guests::sam_opens_the_link_kim_sent_and_watches_kims_film_as_kims_guest::chromium`.
  Sam's first request at Kim's door was answered 403. It failed 2 times in
  5 alone at `e5cd75fc`, before V1.1.3, and 1 in 3 at `a620548e`.
- **The ceiling canary under load:** 0.24 s and 0.33 s, against 0.1 s. It
  ran while the release tree built and the smoke ran beside it. Alone at
  `a620548e` it gave 0.081 s, passed.

## What this does not show

- **The real evening:** venue WiFi, a phone on cellular, or twenty people.
- **The goodnight's look on a real phone:** the screenshots are
  Playwright's iPhone and desktop.
- **#205's own case,** a computer that answers the dial but not the
  request. Its test needs House rules, which V1.1.2 lacks. On the release
  tree, the failover the smoke shows is dials, through lee's computer.
