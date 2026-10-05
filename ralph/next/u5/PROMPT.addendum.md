<!-- section: u5 after=intro -->
This queue is U5 (`docs/DESIGN.md` §8): Films, the one experience. Everyone's
lent library on one shelf, each title marked "from Sam's"; search; play here,
on a phone through U4's page or on a computer, seeking freely; the lender's
password never leaves their computer; three people on Sam's library at once
is the normal evening. Films is a reference app in `apps/films/`, for the next
developer to copy: it uses only DESIGN §3's surface, and what that surface
still lacks, the infra rows add first. It is the last unit before Demo A
(`demo/a.md`, now played in Films).

The plan for every row is DESIGN §3, §8 "Many people, one library", §8 "U5's
inventory" (written by u5-0 from `ralph/next/u5/UPSTREAM.md`, the seat's
survey) and §10. Upstream is commonwealth-ai at `e1a43bcd2`; read a file there
with `git -C ~/dev/commonwealth-ai show e1a43bcd2:<path>`. Jellyfin is
GPL-2.0-only: Films talks to it over its API, and copies none of its code.

**The seat's calls** (2026-10-05, from the survey; u5-0 writes them into DESIGN):
1. *One address for every lend, on a computer as on a phone.* U4's page
   serves `<scope>/l/<lender>/<lend>/<path>`; a computer's node serves the
   same scheme on its loopback, this computer's own lends included. So an
   app's page addresses any lend by a relative path and the same page works
   on both.
2. *The page does the work, under the viewer's key.* Films' page reads each
   lender's catalogue and streams from each lender's door from the viewer's
   own device, so each door counts and shares by the person watching. Films'
   server serves the page, answers the group's lends, and writes progress on
   behalf of the person the door verified. No server proxies a lend.
3. *The infra learns no Jellyfin.* Making the lend's credential, the refused
   paths, the merge, the playback profile and the device ids live in
   `apps/films/`; `scripts/vocab_check.py` keeps them out of the infra.
4. *Films is plain to copy:* HTML, CSS and JavaScript the browser runs as
   written, and Node's own `http` for its server — no framework, no build step,
   no dependency it does not need.
5. *Measured before built:* whether a device id and play-session id per person
   and per play keep one viewer's seek from restarting another's transcode
   through one lend, and whether any byte a transcode sends carries the lend's
   token, are measured (u5-3) before Films relies on either.

Tests that need Jellyfin run a scratch Jellyfin on this host as u3-8's dry run
did (`demo/runs/a/dry-2026-10-04/dry.sh`: the macOS app's server with its own
data dirs, short H.264 files made with ffmpeg); scripts/check.sh stays free of
Jellyfin, so Films' tests there run against a small stand-in origin that
answers the few API routes Films calls, and the Jellyfin runs are their own
script beside it. Anything a person reads uses only DESIGN §1's words; Films'
own words (film, library) are Films', inside `apps/films/`.
