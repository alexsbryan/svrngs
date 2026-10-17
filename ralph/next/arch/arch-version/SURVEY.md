# arch-version (a): which builds a page must talk to — the survey

A survey, nothing changed in the product. Read at 7bca97e6, 2026-10-15
(2026-10-16 UTC). The row asks for every svrngs build that has left this
host: the site's `get/`, `scripts/deploy-page.sh`'s and
`scripts/build-release.sh`'s outputs, and what `demo/` records of another
computer or a phone off this host.

**Verdict: builds have left this host.** Three are recorded. The row says
"if any has, NEEDS_HUMAN.md listing them, the oldest the operator's to name",
and so does CHARTER "Leave for the operator: the oldest build". Part (b)
waits on that answer (`NEEDS_HUMAN.md`, beside this file).

## The builds that left

| when | what | build | where it is recorded |
|---|---|---|---|
| 2026-10-05 | an iPhone (Firefox on cellular, then Safari) ran **the page** from `https://www.svrnme.sh/g/`; the computer it reached was this host's | page `c891fade9e7f` (e7ba903); the computer eb759a1, on this host | `demo/runs/u4/2026-10-05-phones/iphone-{firefox,safari}.json` (`commit`) |
| 2026-10-07 | Demo B's operator run: **Sam's computer ran on a machine outside Dee's network** | `4b2cdfa`, built from source (`cargo build --release -p svrngs`, `demo/runs/b/2026-10-06/NEEDS_HUMAN.md`) | `demo/runs/b/2026-10-07.json` (`commit`, `notes`) |
| 2026-10-12 → today | **the published site**, `https://group.svrnme.sh/`: the page, `install.sh`, and four tarballs under `get/` that anyone holding a group's link can install | about 27d5ff9a (below) | `demo/runs/c/dry-2026-10-12/README.md` ("The page and the binary are the published ones"); `data/summary.json` `published` |

### The published site is still the 2026-10-12 build

Fetched read-only today, the same fetch Demo C's dry run made:

```
$ curl -sS https://group.svrnme.sh/get/SHA256SUMS
a5ef35c7869f49cc13e7888fbdceea8d538aff837ce36e740b377169197164ac  svrngs-darwin-aarch64.tar.gz
ccc54809c41ef8f6d23b0cffb6cf2812a275930f946631faa81fed117503325b  svrngs-darwin-x86_64.tar.gz
e67a066b157f03368ffd64730f89fe91d59aacf8fdc9e9e1e01501453771608e  svrngs-linux-aarch64.tar.gz
9c2628ee843d1324c1eb48d0eb48de55d5bbcb9704b6825daee8213aa367bb54  svrngs-linux-x86_64.tar.gz
$ curl -sSI https://group.svrnme.sh/get/svrngs-darwin-aarch64.tar.gz | grep content-length
content-length: 12112144
```

These are the sums and the size `demo/runs/c/dry-2026-10-12/data/summary.json`
records (`published.sums`, `published.bytes`). The page it serves,
`app.js?v=b5fa725e638a`, is `web/app.js` as 11260e25 left it (with
`b5fa725e638a` read back as `__VERSION__`, its blob matches `11260e25:web/app.js`).
The next change to that file, c5335f34, is 2026-10-12 08:28. Demo C's
predictions (b1d8f407, 06:26) were written once it was published. So the
deploy ran between 2026-10-11 20:51 and 2026-10-12 06:26, and HEAD in that
window was 27d5ff9a. `deploy-page.sh` prints its HEAD, but no run kept that
output, so the exact commit is not recorded.

Two follow-ons are not this row's, but the operator should know them:

- A computer installed from that site keeps `https://group.svrnme.sh/` as
  its page (the `origin` file, `crates/svrngs-node/src/invite.rs:32-38`). A
  phone opening its link loads **the 2026-10-12 page**, whatever build its
  computers run. The page a phone runs is the published one, not HEAD's.
- Not counted: `relay.svrnme.sh` resolves (`dig +short`), but a relay is
  not a computer a page asks who is in or what is lent.

Nothing else in `demo/` ran off this host. Every other run says "this host",
and the "operator's run" sections in `demo/runs/{a,c,gf,mn,mw,sy,syp,relay-rate}`
are packages that were never run.

## What each compatibility branch guards, against those builds

The answer a page reads (`GET /people/lends`), and where the row would cut:

| branch at HEAD | guards a computer before | that change first lands | is it in the published build (27d5ff9a)? | in Demo B's (4b2cdfa)? |
|---|---|---|---|---|
| `web/app.js:354-355` `unheard = []`, `refused = []` | bash-first-minute (c) | f3d0c18e, 2026-10-12 | no | no |
| `web/app.js:360-363`, `:377-380` no `people`: "runs an older svrngs, so it cannot say who is on now" | v1ui-door (d) | 8bf344c3, 2026-10-14 | no | no |
| `web/app.js:383-389` no `shelf` | v1ui-shelf (c) | c97ea641, 2026-10-14 | no | no |
| `question.js:140-145` `signs_in` neither true nor false: "may ask you to sign in" | v1ui-door (c) | 5ad6d966, 2026-10-14 | no | no |
| `lend.rs:302-304` `reads` `#[serde(default)]` | u5-2 | d4c390b2, 2026-10-05 | yes | yes |
| `lend.rs:305-309` `asks` `#[serde(default)]` | sup-2 | 9a452e4b, 2026-10-09 | yes | no |
| `lend.rs:310-315` `signs_in` `#[serde(default)]` | v1ui-door (c) | 5ad6d966, 2026-10-14 | no | no |
| `lend.rs:316-320` `group` `#[serde(default)]` | v1ui-shelf (a) | c98635a9, 2026-10-14 | no | no |

(`git merge-base --is-ancestor <change> 27d5ff9a` and `… 4b2cdfa`, one per row.)

The audit's `live.rs:169` (at 5bf9941) is now `crates/svrngs-node/src/live.rs:108-110`
(c459041c, arch-peer (c)). That line is a peer-to-peer 404 on the live lane,
not a field in the answer a page reads.

So the page-side branches guard exactly the builds before 2026-10-14, and
the published build is one of them. If a page must still talk to the
published build, none of them can go, and part (b) moves no lines. If it
need not, all of them go except the `reads` and `asks` defaults, which guard
builds older still.
