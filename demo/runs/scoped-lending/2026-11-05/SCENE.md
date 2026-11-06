# Scoped lending: the real-node House/Trip scene

Verdict: pass, 65 of 65 checks, at 15acd0668ac0e098b79c61fb836e3d1ea79e68b7.
Separate `svrngs` processes through `svrngs relay --at 127.0.0.1:<port>
--open`, each asked through its own local API or `svrngs` commands.
Proves docs/LENDING_SCOPE.md "Proof before landing" 1, 3 and 4 for an HTTP
answer only: no WebSocket, raw TCP/UDP, old pass, OIDC or browser ran here.
`scene.sh` is the driver; `logs/` holds its events (`events.jsonl`), the
results (`facts.json`), each keeper's trimmed trace and each upstream's
request log.

## The binary

    git rev-parse HEAD                  15acd0668ac0e098b79c61fb836e3d1ea79e68b7
    CARGO_INCREMENTAL=0 lockf -k /tmp/svrngs-gate.lock cargo build --offline -p svrngs
    shasum -a 256 target/debug/svrngs   5e199f4db84a97931389d4e0c799c88448f920d3af58cb80a34d1e4ebc16bf63

The build was not a no-op. `target/debug/svrngs` dated from 21:56, before
15acd066 was committed at 22:22: the gate's `cargo test` steps build their own
copies and do not refresh it. The build above made it from 15acd066 with `-p
svrngs`'s features, so it is the same source as the gate's binaries, not their
bytes. So the scene ran twice:

| take | binary | sha256 | verdict |
|---|---|---|---|
| main (`logs/facts.json`) | `target/debug/svrngs`, 22:59 | `5e199f4d…bf63` | pass, 65/65 |
| supplementary (`logs/facts-gate-binary.json`) | `target/debug/deps/svrngs-26985dae066a687b`, 22:47 | `c11fabc7…846b` | pass, 65/65 |

The 22:47 binary was built while the gate ran (22:25 to 22:57) and is taken to
be its `--workspace` steps' by that timing alone. Its 22:28 sibling,
`svrngs-736aa3f23d3108d8`, was not run.

## The run

    W=<scratchpad>/scene-house-trip sh demo/runs/scoped-lending/2026-11-05/scene.sh       # exit 0
    W=<scratchpad>/scene-house-trip BIN=$PWD/target/debug/deps/svrngs-26985dae066a687b \
      OUT=<scratchpad>/scene-gate-binary sh demo/runs/scoped-lending/2026-11-05/scene.sh  # exit 0

Keepers' `RUST_LOG=door=debug,tiers=debug,join=debug`; the relay's
`relay=debug,info`. In the order typed:

    Relay   svrngs relay --at 127.0.0.1:51730 --open
    Lender  svrngs new House --as Lender --relay http://127.0.0.1:51730/     (stopped)
    Lender  svrngs new Trip --as Lender --relay http://127.0.0.1:51730/
    Theo    svrngs join <link> --as Theo
    Lender  svrngs link House                                                (Trip's `new` stopped first)
    Hana    svrngs join <link> --as Hana
    Lender  svrngs share <port> --name hearth --kind probe --group House
    Lender  svrngs share <port> --name porch  --kind probe --group House
    Lender  svrngs share <port> --name atlas  --kind probe --group Trip
    Lender  svrngs share <port> --name attic  --kind probe --group House
    Ren     svrngs new Book --as Ren --relay http://127.0.0.1:51730/
    Lender  svrngs join <link> --as Lender                                   (asked of svrngs running there)
    Lender  svrngs unshare hearth --group House                              (three times)
    Lender  svrngs                                                           (each restart)

`new` and `link` claim the data dir, so a keeper stops between them. Each
upstream is a Python server on loopback logging every request; `/stream`
answers a line each 0.2 s. Beyond the brief's three keepers Ren is a fourth,
someone in Book alone, since item 4 needs a member of the third group who is
not already denied. `porch` is Hana's other lend (`atlas` is Trip's, which
she cannot open), `attic` the legacy lend.

## The sandbox

Every svrngs process ran under `sandbox-exec` with this profile:

    (version 1)
    (allow default)
    (deny network-outbound (remote ip "*:*"))
    (allow network-outbound (remote tcp "localhost:*"))
    (deny file-write* (subpath "<real home>/.svrngs") (subpath "<real home>/svrngs"))

The brief did not ask for it, and the product's bytes are unchanged by it. It
is there because a plain run does not show what the brief asks: a first take
with UDP left open met at the relay and then moved to a direct path on this
host's LAN address (`addr=Some(192.168.1.10)`, `relayed=Some(false)`). Refusing
UDP leaves the relay as the only way between keepers, and refusing the internet
removes n0.

- n0 was out of reach, by the kernel: `curl: (7) Failed to connect to
  use1-1.relay.n0.iroh.link port 443 after 3 ms`.
- The door traced 110 member uses `relayed=Some(true)` and none
  `relayed=Some(false)`.
- The only relay ever given to the Lender's endpoint is the local one. The
  `new`, `new` and `link` starts each leave n0's (logs/Lender.log); the bare
  restarts trace no move at all:

      tiers: this computer meets its groups at another tier now from=Fallback to=Own
      tiers: a relay given to this computer's endpoint relay=http://127.0.0.1:51730/ tier=Own
      tiers: a relay taken from this computer's endpoint relay=https://use1-1.relay.n0.iroh.link/ tier=Own

So the numbers below are through the relay, with no n0 and no internet. What
this does not show is the same scene with n0 reachable, where the tiers would
hold the fallback while it answers (`crates/svrngs-node/src/tiers.rs`).

## Results

1. **Pass.** Hana lists attic, hearth and porch in House. Through hearth:
   200, `answer from hearth at /`, and hearth's log gained that one request.
2. **Pass.** Theo lists atlas alone in Trip. Asking House's lends: 404
   `not_held`. The address copied from Hana's list, on `/`, on `/stream` and
   with forged `X-Svrngs-User`/`-Group`/`-Person`: each 403, `x-svrngs-refused:
   closed`, body `{"error":"the computer you reached lends nothing to
   you","kind":"closed"}`. hearth's request count stayed at 1. The door:

       door: refused — the dialer stands in no group this lend is lent to actor="86334453…" lend=hearth

3. **Pass.** Theo through atlas: 200, `answer from atlas at /`, on the same
   Lender, half a second after the refusals.
4. **Pass.** Lender joined Book (`You're in Book, as Lender.`). hearth.json
   sha256 before and after:
   `96b6f9de117f4e11f949105c82b8e041a04d684716aa24529b9c2fd089dd98ad`, and the
   same for each declaration. Ren lists none of the Lender's lends in Book, with
   nobody unanswered, and the copied address is refused with no upstream
   request.
5. **Pass.** See below.
6. **Pass.** Lender restarted twice (pids changed). hearth.json the same
   `96b6f9de…` bytes, its `groups` still House's id. Hana got hearth, with the
   request in its log. Theo lists atlas and not hearth; refused, traced, no
   upstream request.
7. **Pass.** `groups` was taken out of attic's real declaration with svrngs
   stopped, attic having answered Hana a moment before, so her copied address
   is real. Bytes around one start:

       before  73e4bb74…fd6f  {"port":51729,"kind":"probe","refused":[]} (indented, a trailing newline)
       after   3ed84ca9…e4c5  {"port":51729,"kind":"probe","groups":null,"refused":[]} (indented, none)

   Exact bytes are in `events.jsonl`, `legacy-before` and `legacy-after`. Hana's
   House list is hearth and porch, Theo's Trip list atlas, Ren's Book list empty.
   Hana's copied address is refused with no attic request. `svrngs unshare attic
   --group House` exits 1: `attic is lent to no group yet, so no group was taken
   from it: choose who can open it with svrngs share <port> --group <group>`.

## Withdrawal

Hana streaming `/stream?lines=300` through hearth, `svrngs unshare hearth
--group House` run mid-stream, three times, with hearth lent again between.
Seconds from the command's return to the stream's end, the socket's own EOF:

| run | return to end, s | supplementary take | `unshare` took, ms | lines at unshare / total |
|---|---|---|---|---|
| 1 | 0.304 | 0.294 | 23 | 9 / 10 |
| 2 | 0.928 | 0.927 | 24 | 10 / 15 |
| 3 | 0.540 | 0.566 | 24 | 12 / 15 |

All within the documented 2 s, and within the sweep's 1 s. The spread is phase:
the door's cuts fall on whole seconds from the stream's start (3.00 s in runs 2
and 3), so the delay is where the command lands in that second. Each stream
ended without its closing chunk, hearth's upstream saw `BrokenPipeError`, and
the door said `door: an answer cut — its asker no longer stands in a group the
lend reaches lend="hearth"`. A fresh request right after: 403, `closed`.

Unrelated traffic across each cut, same take:

| run | porch stream | porch `GET /`, every 0.25 s | House sync, ms: Lender to Hana before, Hana to Lender during, Lender to Hana after |
|---|---|---|---|
| 1 | 40 of 40 lines, clean end | 29 of 29 200, worst 24 ms | 74, 72, 74 |
| 2 | 40 of 40 lines, clean end | 27 of 27 200, worst 12 ms | 78, 72, 75 |
| 3 | 40 of 40 lines, clean end | 26 of 26 200, worst 11 ms | 81, 75, 73 |

Sync is an act appended on one computer through `/v1/rail/append` and timed
until the other's `/v1/rail/log` holds it.

## The rules

- Every process's environment was built from nothing. `ps -wwE` on Relay,
  Lender, Hana and Theo showed only their own HOME under the scratch dir.
- The real `~/svrngs` held 466 entries before and after, none new. That many
  were already there, from before this run.
- Everything started was stopped by its recorded pid; `left_running` is empty
  in both takes. The live keeper, pid 826, ran throughout.
- No token or link is in these files: the driver redacts them as it writes, and
  a scan afterwards found only keys, group ids and lend labels (64 and 62
  characters) and HEAD. A token is 43.

## Not shown

WebSocket and raw TCP/UDP withdrawal, old passes, OIDC, the browser, a person
reaching a lend by a browser session rather than the local API's token, and any
of LENDING_SCOPE's items 2, 5, 6's old-binary half, 7 and 8.
