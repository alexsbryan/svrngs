# n0 off, on one host — predictions, 2026-10-12

Row relay-n0-off (b) (`ralph/next/relay/STATE.md`): a group of three
computers and a phone's page with n0 out of the path, each computer
`RelayConfig { relay_urls: [one iroh-relay on this host], n0_services:
false }` (`crates/svrngs-node/src/guest.rs:41-94`), to see what breaks
before relay-in-record fixes it. Written and committed before the driver
ran. One host, one run: not a rate (AGENTS.md 7). A step that could not
finish is could-not-judge, never a pass (AGENTS.md 6).

## The setup

- **The relay**: iroh's relay on 127.0.0.1, plain HTTP, no QUIC, started as
  the browser tests' harness starts it — `crates/svrngs/tests/n0_off.rs`
  `the_relay`, a process of its own.
- **Each computer** is the `svrngs` command, `src/main.rs`'s body with that
  one relay and n0 off — `n0_off.rs` `the_command`, the row's "set as the
  tests' harness sets a node's relay": the command cannot yet be pointed at
  a relay (`crates/svrngs/src/main.rs:46`). A `HOME` of its own each, every
  verb typed as a person types it: Sam `new house --as Sam --origin <the
  page>`, Ana and Alex `join <link> --as <name>`, Ana `share <port> --kind
  film --name film`. Listening on every interface, as the command does.
- **The page** built from this commit (`scripts/build-web.sh`), served on
  localhost by `web/tests/serve.mjs`, nothing published.
- **The phone**: Cy, WebKit as an iPhone (`web/tests/page.mjs`), opening the
  https link Sam's `new` prints.
- **The film**: 30 s of H.264 made by ffmpeg, served with ranges by
  `film.mjs` beside a page that plays it, lent by Ana.
- **What counts as n0's**: the hosts iroh names for n0
  (`iroh-1.0.2/src/defaults.rs:27-33`, `:94-96`: the four
  `*.relay.n0.iroh.link` relays and the two staging ones) and its lookup
  (`dns.iroh.link`, `staging-dns.iroh.link`; iroh-dns-1.0.2
  `src/dns.rs:45-47`), resolved by `dig` at the start of the
  run; every remote address each process of the run opened is sampled
  every 2 s with `lsof -i -n -P` per pid — the relay, the three computers,
  the page's server, the film's, and the phone's browser and its children.
  `lsof` shows TCP and connected UDP; an unconnected UDP send and a name
  asked of the system's resolver it cannot show, so iroh's own traces at
  debug are read beside it.

## What iroh does with a bare key, read before the run

A computer dials a member by its key alone (`Contact::by_key`,
`crates/svrngs-node/src/iroh.rs:96`; DESIGN §8, "A bare key is
dialled"). With n0 off the endpoint is built from `presets::Minimal` and has
no address lookup (`guest.rs:133`). iroh answers a dial from the paths
it holds for that key (`iroh-1.0.2/src/socket/remote_map/remote_state/path_state.rs:161-167`)
and otherwise asks its lookup (`remote_state.rs:850-881`); with none, the
dial ends "No addressing information available"
(`iroh-1.0.2/src/endpoint.rs:909-910`). It holds a key's paths while a
connection to it lives and for 60 s after (`remote_state.rs:67-73`). So a
computer reaches a member only while some connection between the two —
from either side — lives or ended under a minute ago.

## Predictions

1. **Joining by link holds.** Sam's `new` waits for the relay and its link
   carries it: Sam's trace says "no lookup, so the link carries the relay"
   with `http://127.0.0.1:<port>/` (`crates/svrngs/src/cli.rs:1699-1735`).
   Ana's and Alex's `join` each print that they are in within 10 s, and
   their folders are at `~/svrngs/house`.
2. **The round between Ana and Alex fails, both ways, every round.**
   Neither issued the link and neither has opened a connection to the
   other, so each dial between them ends "No addressing information
   available" in the dialler's trace (`transport`, "iroh bridge: dial
   failed"). While Sam answers, each still holds the other's acts, through
   Sam. With Sam stopped (`SIGSTOP`), a photo Ana puts in the group's folder
   is **not** on Alex's disk 150 s later.
3. **Sam reaches Ana and Alex, and they Sam**, while their rounds keep a
   connection within a minute of the last: each joiner dialled Sam through
   the link's relay to join, and Sam is dialled back on the paths that left.
   Not judged past that: a member that stayed silent past 60 s.
4. **The lends.** Sam's home page lists Ana's `film` within 60 s of her
   `share`. **Alex's does not**: it says Ana's computer did not answer, its
   trace naming the dial to Ana that had no address.
5. **A folder's file from a member that is not its writer.** A photo Ana
   puts in the folder reaches Sam's disk within 30 s and Alex's within 90 s,
   Alex's from Sam: Alex's trace "folder: fetched, verified" names Sam's key
   as its holder, and a dial to Ana's key fails with no address.
6. **The phone.** Cy opens the https link in WebKit: the group's page asks
   Cy's name, is in within 30 s of the name, lists Ana's `film` within 30 s
   of being in, and the film plays (`currentTime` past 1 s) within 30 s of
   its button: the page dials every key through the relay its link carries
   (`web/src/lib.rs:181-182`), and Ana's computer is on that relay.
7. **A door's `AskRound` for a newcomer.** Cy's first request at Ana's door
   comes before Ana's round has Cy's admission, so Ana's door asks the
   ring (`ring_sync::AskRound`): Sam answers it — Ana holds Sam's path from
   her join and her rounds — and its dial to Alex ends with no address,
   traced. The film plays (6).
8. **No n0.** No sample from any process of the run holds an address of
   n0's, and no line of iroh's traces names an `iroh.link` host or a pkarr
   publish.
