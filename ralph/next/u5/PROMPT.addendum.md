<!-- section: u5 after=intro -->
This queue is U5 (`docs/DESIGN.md` §8): any app a member runs on a port,
served to the group, and stock Jellyfin working through it out of the box.
The operator, 2026-10-05: north star 2 holds for the apps people already
self-host — `svrngs share 8096` lends Jellyfin as it comes, and someone who
never used it opens it from the list, plays and seeks, on a computer or a
phone; Films is dropped (its `apps/films` rows are gone, and Demo A plays in
stock Jellyfin); group folders are an experiment after U5, settled by the
operator in the wild (call 7). The infra rows add what lent apps need first;
the catalogue run proves the claim on five apps beside Jellyfin.

The plan for every row is DESIGN §1, §3, §8 "U5's inventory" (written by u5-0
from `ralph/next/u5/UPSTREAM.md`, the seat's survey) and §10. Upstream is
commonwealth-ai at `e1a43bcd2`; read a file there with
`git -C ~/dev/commonwealth-ai show e1a43bcd2:<path>`. Jellyfin is
GPL-2.0-only: svrngs talks to it over HTTP and its API, and copies none of
its code.

**The calls** (2026-10-05; u5-0 writes them into DESIGN):
1. *Lends off the key's origin, on any host a group trusts.* A computer
   serves every lend at an origin of its own on loopback (`<id>.localhost`,
   or a port per lend). A phone has two modes: the floor, any static host
   serving two hostnames — the page's, holding the key, and one for every
   lend under `/l/<id>/`; and, when the page's host answers a wildcard (ours
   does), each lend at `<id>.<page host>`, its origin's root and storage its
   own, at the price that the host and DNS see which lend is opened. `svrngs
   new` says which the group got. (Operator, 2026-10-05: one shared origin let
   any lent page unseal the viewer's key — u4-6, DESIGN §11 risk 5; no step
   may be added for a person joining or a group starting; and a group that
   does not trust ours, or wants a stricter privacy policy, runs on any
   static host.)
2. *Jellyfin as it comes.* The lend serves stock jellyfin-web from the
   lender's Jellyfin; nothing in svrngs renders a library of its own. What
   U3's dry run found (`demo/runs/a/dry-2026-10-04/`: a sign-in asked of
   everyone; a member's sign-in logging out the lend's token) is what u5-3
   and u5-4 solve.
3. *The infra learns no Jellyfin.* Anything that knows Jellyfin's API lives
   in `apps/jellyfin/`; `scripts/vocab_check.py` keeps it out of the infra,
   and what the door gains for it is general (any app).
4. *Zero sign-in if it is small, else sign in once* (operator, 2026-10-05).
   Zero sign-in: the lender's computer gives each person their own Jellyfin
   user and the door signs them in. Sign in once: the lend carries no
   secret, and each person signs in with an account the lender made. u5-3
   measures both and decides by the rule it states before its data; the
   other goes to §9 with its trigger.
5. *Measured before built:* what stock jellyfin-web does through the door —
   its sign-in, its live socket, two people's plays through one lend, and
   whether any byte carries the lend's token — is measured (u5-3) before
   u5-4 builds on it.
6. *The person's key never enters a lend's origin.* The page signs a pass —
   a fresh key may act as this person toward these lends until this time —
   for the lend it opens and the lends its lender declared it reads; the
   lend's origin dials with that, and the runtime keeps each lend's cookies
   apart. On the floor lends share one origin, so a lent page can use another
   lend's pass while it lasts; with the wildcard, only its own. Never the key
   (u5-2).
7. *Group folders, an experiment after U5.* A member shares a folder at one
   level from a closed set; its file list in the record, its bytes content-
   addressed and fetched by verified range from whoever holds them
   (iroh-blobs 0.103, which takes iroh ^1.0.0); the meeting point runs one
   Jellyfin over everyone's shared films as `.strm` stubs. Opt-in beside
   lends, settled in the wild. Its first measurement:
   `~/dev/svrn-docs/rings/runs/strm-2026-10-05/` (browsers played the holder's
   MP4 directly; WebKit remuxes MKV through Jellyfin, the double hop). u5-0
   writes it into §9 with that trigger; no U5 row builds it.

Tests that need Jellyfin run a scratch Jellyfin on this host as u3-8's dry run
did (`demo/runs/a/dry-2026-10-04/dry.sh`: the macOS app's server with its own
data dirs, short H.264 files made with ffmpeg); scripts/check.sh stays free of
Jellyfin, so what runs there uses a small stand-in origin, and the Jellyfin
runs are their own script beside it. Anything a person reads uses only DESIGN
§1's words.

Containers run with podman, never docker or OrbStack (operator, 2026-10-05:
"Never docker. Only podman and open software."): `podman machine start`,
then `podman run`, `podman pod` or `podman kube play`; the settings allow
those and refuse docker. u5-catalogue-dry's row names "OrbStack's
`docker`": that premise is false, so correct the row as the rules say,
citing this paragraph. A tool the run needs that is not open software, or
that this host does not have, is the operator's to give.
