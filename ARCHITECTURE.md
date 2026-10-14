# Architecture

The map of svrngs as it is built today. What it is built toward is
`docs/DESIGN.md`. This file says what exists, and it changes in the same commit
as what it describes.

## Bird's eye

svrngs owns the group's code, forked from commonwealth-ai's ring slice (DESIGN
§4). The record and the journal on disk have landed; sync, the routes and the
member door come next, forked the same way (DESIGN §8, U1-U3); of sync, the
dialling and serving halves over iroh have landed, and the round and its
route, and the node that runs them. The `svrngs` command runs a node: it
makes a group, mints its link and runs for every group the computer holds,
serving the record to the computer's apps over loopback (U3, `rail.rs`);
joining by that link is next (U2).

## Code map

**`crates/svrngs-core`**: the group's record — signed acts (`Record`,
`Correct`, `Seal`, `Admit`, `Remove`, `HandOn`, `Invite`, `Synced`, `Relay`), the lines a computer holds as
a set (`ActSet`, `act_set.rs`), admission of that set into one total order
(`admit.rs`; only `admit` makes an `Admission`), whose state leaves the core
only as a projection (`project.rs`: the sealed `Projection`, folded for
`Membership` and `Invites` and handed out by `Admission::project`) or as
`Wire`, the acts serialized for an app to fold, membership (`membership.rs`), the per-actor sync digest
(`sync.rs`, `view.rs`), the write side (`command.rs`: `Command`, `commit`, the
`Store` port, `Authored` and `Recorded`, which only `commit` makes), effects
(`process.rs`: the sealed `ProcessManager`, the `Due` only `Admission::react`
makes and only when every act it names as a cause stands, and `Compact`, a
seal's retirement), the register's commands (`register.rs`: `MintInvite`, `Redeem`), the
record's (`record.rs`: `Append`, `Seal`); upstream's guest attestation
(`attest.rs`) and roster warrant (`introduce.rs`, `Roster.vouches`, with
`short_stamp`) were deleted in core-0, nothing of svrngs using them. `no_std`, so the
compiler refuses filesystem, clock, network, threads and `HashMap`; time and
keys arrive as arguments. It is commonwealth-ai's
`commonwealth-rail-core` at `e1a43bcd2`, with `oplog-types` (`op.rs`) and
`ContentHash` (`hash.rs`) inlined, and `hash.rs`'s wire test asserting the
JSON form directly rather than through kernel-types' `wire` module, which the
fork does not carry. Every source file carries its `ported from` line (the
JSON fixtures came over unchanged);
`scripts/ports.py` lists each beside its original. It keeps upstream's edition,
rustfmt policy and clippy allowances so a diff against upstream shows only
what svrngs changed. Then it changed (U2, DESIGN §6 "Ids cover everything
signed"): an act's id is the whole BLAKE3 hash, 64 hex, of its namespace,
actor, seq, previous act, timestamp and body (`sig.rs`'s `act_id`), and the
signature is over the id, where upstream signed the fields and kept 64 bits
of a hash of the line, signature included; `OpId` holds the whole hash, so a
line whose id is not 64 hex does not parse. Every domain the core signs or
hashes under is written in `sig.rs`, and none begins another, which
`sig_tests.rs` reads from the sources (onair-sign). Each act names its author's act
at seq − 1 (`SignedOp::prev`), and admission names one that does not a
`WrongPrevious` gap and admits it not. Membership, the view and sync take the
id admission derived (`derive_id`), never the line's. A link is in the record
(DESIGN §6 "A link admits each person once"): an `Invite` names its key and
lapse, and `membership` counts an `Admit` redeeming one only under the rule
in `membership.rs`'s module docs, naming each refusal a `RefusedRedemption`.
An issuer's newer `Invite` closes their older one from its place in the
order, so what the older link admitted keeps standing. A redemption names
what holds the new key (`Device`, u4-1): a browser's ships at line version
4, and `Membership::device` answers each standing key, a computer's unless
the counting `Admit` that last bound it named a browser. A person is their
first key in the group (bash-devices): a seed row's first, or the key an
`Admit` binds, so two people of one name are two people; a device link — an
`Invite` naming that person (`device_of`, line version 8) — counts only from
a key standing as them, and a key it admits is theirs, under their name.
`Membership::person_of` answers each key's first key and name, `joined`
when a person joined — the time on the `Admit` that made their first key a
person, or that they started the group (bash-same-name) — and `one_named`
the one person a name and, of two people of one name, that time name;
`Removal` and `HandOnTo` take it, refusing an ask that names no one person
with when each person of the name joined (`Which`), never guessing. Forks void nothing
(U7): both branches count, the fork named a `SequenceFork` beside a complete
answer. Removal has one decider (U7, `membership.rs`'s module docs): the
group's keeper — the seed's key, moved only by its own `HandOn` — cuts by
its `Remove`; anyone else's is an ask that cuts nobody. The cut names the
removed key's acts by id (`Remove::through`) and covers them and their
ancestors by `prev`; any other act of the key is `Removed`, and a cut naming
an act a copy lacks is `UnheldCut`. A `Correct` voids nothing that moves
membership, so nothing undoes a cut; voids fold after the walk. Each
key's relay is its own newest `Relay` act (relay-in-record): the fold
answers it for a standing key (`Membership::relay`, read by `relay::parse`),
and `RelayAt` writes it only when the record says otherwise.

**`crates/svrngs-runtime`**: the journal, its write index, the open-journal map
(`RingRail`) and the register's standing (`register.rs`), split out of
svrngs-node. The journal is crate-private: an adapter reaches a group only
through `RingRail`'s queries (`admission`, `standing`, `invite_for`,
`group_dir`), its `commit` and `seal`, `set_seed`, and the replication port
(`digest`, `ops_missing_from_within`, `ingest_all`, `compact`). What follows
about the journal describes this crate.

**`crates/svrngs-node`**: a group's journal on disk — one append-only JSONL
log per group under `<root>/rings/<ns>/` (`journal.rs`), the single-writer
door that signs and the peer-ingest path that does not; upstream's checkpoint
(`ring_checkpoint.rs`), which nothing of svrngs composed, went in lean-3.
The writer underneath is `oplog.rs`. It
is commonwealth-ai's `commonwealth-rail` and `oplog` at `e1a43bcd2`, forked the
same way as the core; its one edit is gating `oplog.rs`'s test-only serde
import. Upstream's shape stays: `RingRail` keeps its open journals in a map
behind a `Mutex` (`lib.rs`), which an actor per journal replaces when the node
becomes ours (DESIGN §5). Then it changed (U2): the write index holds the
ids each act's content derives, so a line claiming a held act's id is not
skipped as held; the door names, as the previous act, the line at its newest
seq whose signature verifies under its own key; compaction keeps a cited
act by its derived id; and the journal is the `Store` adapter `commit`
appends through: under the write lock it hands over the lines held and
appends only the `Authored` lines `commit` returns. Then (ceiling-2) the
write index keeps the digest's fold too (svrngs-core `DigestFold`, which
`digest` is over a whole journal), caught up from the log's tail with each
act's signature checked as the tail brings it in; and a peer whose digest is
ours is answered unread when it holds nothing the slow path would send
(`DigestFold::missing_nothing`), from the whole log otherwise. Then
(ceiling-3) the write index keeps each signature that passed as its line
entered, and the offer read whole judges by those (`Checked`), checking
again only a line it did not see enter. Then (ceiling-4) the write index
keeps the acts that move membership, with every act of a key a `Remove`
names (`standing_reads`; U7), in the log's order, and the standing
last folded from them with the seed it was folded from (`RingJournal::standing`):
the register answers from it until an act that moves membership arrives
or the seed changes, judged by `Ed25519Verifier`, which `standing` no
longer takes.

Beside the journals, `svrngs-node` holds the dialling half of sync over iroh
(DESIGN §8, U1's inventory), forked at `e1a43bcd2` the same way:
`peer_wire.rs` is `sovereign-peer-wire` whole (the sync bodies, the body limit
and the exchange budget); `alpn.rs` is `mesh-reach`'s `alpn.rs` with its one
`ALPN`, then the enum it promised (`Alpn`, U2), moved whole into svrngs-core
(u4-2) so a browser's page spells each protocol as the node does, and
re-exported as `svrngs_node::alpn`; `guest.rs` is `mesh-reach`'s guest dialer, the
relayed endpoint and the one loopback bridge (`HttpBridge`, `pump`); `iroh.rs`
is `commonwealth-transport`'s client side (`IrohTransport`,
`format_dial_string`, `preferred_bridge_port`). Its edits: no IP transport;
one ALPN, so the per-class ALPNs (`RPC_ALPN`, `CLIENT_ALPN`, `GUEST_ALPN`,
`OFFER_ALPN`, `MEDIA_ALPN`, `APP_ALPN`) and `alpn_for_class` go, and the guest
tunnel dialled `ALPN` until lean-3 deleted it (`guest/tunnel.rs`,
`parse_dial_string`), nothing of svrngs reaching it; no bench relay pin (`relay_pin_active`,
`build_relay_only_endpoint`); no `PeerTransport`, `PeerContact`,
`PeerEndpoint` or `TrafficClass`, so the transport is a concrete type whose
`base_url` takes a `Contact` (a key with optional dial hints) and its bridges
are keyed by peer alone; `NodePubkey` becomes iroh's `PublicKey`; the sync
bodies import the record from `svrngs-core`. Then it changed: a bare key, with
no address in its contact or seeded, is dialled when the endpoint has an
address lookup (n0's DNS under `RelayConfig::n0_services`), where upstream
refused it (DESIGN §8, "Four facts"); and `RelayConfig::loopback_only`
binds the endpoint to 127.0.0.1 alone, for nodes that share one computer in
a test (ralph/REVIEW_FINDINGS.md #33). `iroh.rs` also holds the
serving half, upstream's accept loop (`IrohAcceptor`) forked without its
`Forward` kinds and then changed: `IrohAcceptor::spawn` takes an admit decision
on each connection's ALPN and verified key (`Admit`), closing a refused
connection and naming, for an admitted one, the `axum::Router` that hyper's
HTTP/1 server serves on its every bi-stream, each request carrying the verified key as a
`Dialer` extension. There is no loopback listener on the serving side and no
request head parsed but by hyper. The bridge's own tests splice through a test
stand-in, since they test the bridge, not what the acceptor serves;
`tests/accept.rs` tests the acceptor.
iroh is pinned at `=1.0.2`, and its tree locked at upstream's versions, so the
fork runs on what upstream tested.

Ring sync came over the same way: `ring_sync.rs` and `ring_sync/journal.rs`
are `commonwealth-rails`' round, chunked exchange and responder (`answer`),
and the seam the round reads (`RingSyncHost`, `RingSyncJournal`, implemented
for `RingRail`); `ring_routes.rs` is its sync route, with `rail.rs`'s `err`.
Their edits cut the mesh ties: `RoundMembers` loses its walk of the mesh
(`RoundMembers::of`), and is built from named keys instead, which the node
reads from the register (DESIGN §8, U1); the mesh proof stamp the exchange
carried goes; `RingSyncHost::transport` is the concrete `IrohTransport`, and a
contact's addresses are its one `base_url`; `peer_client` is `pub`, since its one upstream caller,
`RailsDaemon`'s `impl RingSyncHost`, did not come over. The append
wrapper that nudges the round (`ring_sync::append`) came over in U3 with the
record API, which routes the append door through it. The live route
(`ring_live`) came over in U3, with `RingInbound`'s live buffer; it refused
an envelope for a group the rail does not hold, where upstream asked the
origin registry's live registrations, which did not come over. Then it
changed: it asks the sync route's question (`standing_asker`), so a dialer
not standing in the group its envelope names — one the acceptor admitted
for another group — is refused 403, as is a request off the stream. The
checkpoint route stays behind; the routes are an `axum::Router` where
upstream's was a `RouteBundle`. Then it
changed: the route reads the asker from the acceptor's `Dialer` extension,
where upstream read an `X-Mesh-Pubkey` header, and refuses (403) a request
with no `Dialer`, where upstream served it as a local process — the route is
served only on the iroh stream. Its tests are served there too, through
`iroh::tests::serve_on_stream`. Of the tests,
`ring_sync/tests.rs` came over but for the derived-roster prune test, which
installs upstream's `MembershipRosterSource` over a mesh, and
`ring_routes/tests.rs` kept only its sync test; `round_tests.rs` is U1's to
rewrite.

What runs is the node (`node.rs`), new code in place of upstream's
`RailsDaemon` and `RailsNode` wiring, which did not come over. `Node::start`
takes a data dir, a signing key, a `RelayConfig` and the loopback address
it serves the record to this computer's apps on (`api::bind`, refusing any
other, before the endpoint; `api::serve` once the endpoint's transport,
which the live lane dials through, exists; `Node::api_addr` answers the port bound); it binds the endpoint
with that key, so the key that signs a group's acts is the key iroh verifies
when the node dials (DESIGN §8, "Four facts"), and holds the `RingRail`
signed by it, an `IrohTransport` over the endpoint, which the round and the
live lane share, the accept loop serving
the sync route's router on the members' ALPN and the redemption route on
the second, and the round, which runs at start, every sixty seconds and at
once after an app's append (one `Notify`, held by the round and the record
API), and when the member door meets a pass whose signer stands in no
group held here, which waits up to 10 s for a round begun after its ask to
end before it answers (`ring_sync::AskRound`, mnp-1)
(`Node::sync_with` runs one exchange with a peer the register may not name
yet; a test runs a round now with `Node::round`, which with `transport` and
`api_routes` is test support since lean-3, `node/tests_support.rs` behind
svrngs-node's `test-support` feature).

Redemption is `redeem.rs` (DESIGN §6, U2), new code in the shape of
commonwealth-rails' mesh join. A link carries only the issuer's key and the
link's secret, so the joiner first asks: `AskRequest`, the link's key and
its own, signed by the link's key, answered (`Asked`) with the group whose
`Invite` of this node's names that key, its seed as the command keeps it
(read through the `SeedText` function `Node::start` is handed, since the
seed is the command's), the `Invite`'s id and, for a device link, its
person by name (`Asked::person`, bash-devices; `RingRail::invite_for`), whom
`svrngs join` and the page join as, asked no name. Then `RedeemRequest` — the group's id, the
`Invite`'s id, a person's name, the new key, its consent, what holds it
(`Device`) and the link's key, signed by the link's secret — posted on `Alpn::Redeem`, which the
accept loop serves to any dialer with these two routes. The two requests,
their signing and the check of the link's signature are svrngs-core's
(`joining.rs`, moved from here in u4-2 so a browser's page signs with the
command's body; `Asked` followed in u4-5, so the page reads the answer as the
command does), re-exported from `redeem`. Each route is an
adapter: `parse_redeem` and `parse_ask` judge what arrived (the dialer is the
new key and the link's key signed), so a refused request reads nothing; the
redemption is then svrngs-core's `Redeem` command, which `commit` decides
under the journal's write lock (the `Invite` is held, is this node's and names
the link's key; the key not standing already) and writes only when the
register, folded with the `Admit` in it, counts it. `own_invite` decides the
ask over each group's acts.

Every write is a command through `commit`: an app's act is svrngs-core's
`Append` (records and corrections only, and the person it speaks for must
stand, decided under the lock), a seal is `Seal` followed by compaction, and
`svrngs link` mints with `MintInvite`. Compaction is an effect: svrngs-core's
`Compact` decides from the seals in the record which lines retire, and the
journal deletes only from the `Due<Retire>` that `react` hands back, after
this computer's seal and a peer's alike. The journal's own `append` exists only
in tests, as a fixture that also goes through `commit`.
Every refusal answers `REFUSAL`, one sentence naming no group, but a name
that does not parse, answered with why, and a name a standing person goes by
(`RedeemRefusal::NameTaken`), answered `409` with theirs; a key that already
stands is answered `AlreadyIn`, nothing written.

Who is in a group is the register's (`register.rs`, DESIGN §6, U1):
`RingRail::standing` answers a group's standing keys from `membership` over
its seed — the group's `roster.json`, the founder's key alone — and the acts
admission authenticates; a group the node does not hold answers `NotHeld`,
and one with no seed `NoSeed`, which refuses everyone. Each standing key
comes with its person and what holds it (`Member`, u4-1). Its four callers: the
round dials every computer's key standing in a group the node holds — never
a browser's, by `Device::dialled` — by key, through
addresses a caller seeds on the transport or iroh's lookup finds, and offers
each group to its own; the accept loop admits a dialer standing in any group
the node holds, a browser's key as a computer's, and closes the rest; the sync route refuses (403) a dialer not
standing in the group its request names, and a group the node does not hold
in the same words; and `commit` signs only for a key standing in the seed
and the acts the journal holds, under its write lock.
`tests/group_not_mesh.rs` runs a founder, a member it admits, a stranger and
a member of another group in one process. `tests/scale.rs` is an instrument,
not a check: ignored, and run by hand in release, it stands up one group of
any size in one process and prints what an idle round, a write's round, an
asleep member, an append and an act on disk cost (its module docs say how to
read them), and each round's own account: members dialled, reached and
asleep, acts moved, processor and wall time, and each fold of the journal
kept or rebuilt, counted from the debug line each fold traces; that account's
hand count, a four-member round with one member killed mid-round, is not
ignored and runs with the tests. `Node::start_every` gives each of its nodes a round once a day, so
no node's own round runs inside one it times. The round reads the node's `Host` and the
`RingRail` directly: upstream's round read them through the `RingSyncHost`
and `RingSyncJournal` traits, and with one implementor of each here the
traits went (RUST_ARCH 7). Upstream's other answers to who is in a ring —
`RosterSource`, `RingRail::derive_roster` and `default_roster`, which let a
ring's roster be derived from the mesh, `RingRail::roster` and
`roster_origin`, which chose between them and the file, the core's
`RosterOrigin`, and `ring_sync::roster_names` — went once the register had
taken their callers, and a peer's seal is pruned against the group's seed
file.

The record over loopback (DESIGN §8, U3's inventory, the rail.rs row) is
`rail.rs` and `api.rs`, commonwealth-rails' `rail.rs` and `api.rs` at
`e1a43bcd2`: the append door (`append_act`) and the log (`log_answer`), with
`RailQuery`, `namespace_of` and `journal_of`, served as an `axum::Router`
(`api::router`) by `api::serve`, which refuses any address but loopback
(`check_loopback`). Then it changed: every request's caller is asked the
same (`from_this_computer`), so a caller elsewhere, or one whose address is
unknown, is refused 403 `not_loopback`, naming it. Its edits: the doors read the journals from a `RailApi`
where upstream's read `RailsDaemon`; the roster they read is the group's
seed file, since upstream's `RingRail::roster` went in U1, and so the
not-in-roster sentence is the rail's own, where upstream chose it by the
roster's origin; `err` is the one `ring_routes.rs` carries; `serve`
answers the address it bound with the task (`Serving`, which stops the
listener when dropped), where upstream's answered the task alone. The live
lane came over in U3: `LiveBuffer`, bounded per group, which the live route
fills and `GET /v1/rail/live` drains, and `POST /v1/rail/live`; `live.rs`
is the inference daemon's fan-out (`routes_rail_live.rs`, `push_ephemeral`
and `offer_peer`), its walk of the mesh's Online members cut, so its caller
names the peers (`LivePeer`), each dialled at its one `base_url`. Then it
changed: the push door, which buffered locally and answered `peers: []`,
offers the payload to every computer's key standing in the group but this
node's (`live_peers`, from the register; u4-1 skips a browser's), over the node's `IrohTransport`, held
by `RailApi`, and answers each peer and whether it took it. Then it
changed (onair-live): the live route keeps the key that dialled in and its
person's first key with each payload (`Live`), from the register row the
standing check hands back (`standing_asker`), and the drain answers them
with it. The sync
doors and `attest` stay behind, and with them the other routes beside them
in upstream's `api.rs`. Then it changed: the append door's
attestation branch went with `attest`, and `GuestAttestation` from the
core in core-0. An append's `on_behalf_of` is the
person an app names when it serves someone else — whom the member door
verified (DESIGN §8, "Localhost is the trust boundary") — signed by this
computer, and refused (`not_a_name`, `not_in_group`) unless it parses as a
person standing in the group; upstream dropped a bare one.
The register stands where the roster stood: `journal_of` asks
`RingRail::standing` before a journal is opened, refusing a group this
computer does not hold (`not_held`, 404) and one held with no seed
(`no_seed`, 409); the door's own fold refuses a computer that does not
stand (`not_in_roster`, in DESIGN §1's words); and the log answers the
people standing after its one read (`people_json`: each key, its person
and the person's first key), where upstream's answered the roster. `GET /v1/rail/people` is new:
the same people, through the same gate (`standing_of`), so an app needs no
second source for who is in its group or a key to seal to (DESIGN §3).

The member door (DESIGN §8, U3's inventory) starts from `declared.rs`,
commonwealth-media's `declared.rs` at `e1a43bcd2` (:51-77, 164-247): a
lender's credential for its own service, a header value in a 0600 file
under a 0700 dir, the file named by its header, never logged. Its house
store (`house_dir_under`, `read_house_in`, `write_viewer_in`) and rails'
data-dir helpers stay behind with their two tests, and with `dir_under` the
`PathBuf` import.
Then it changed: a hardening that fails fails the write, where upstream
ignored it and reported a secret readable by others as stored; the file is
created 0600; and that a credential is set is traced at debug, since the door
reads the store on every request.

The door is `door.rs`, new code holding upstream's rules (DESIGN §8, U3's
inventory, its identity and forward_stream rows), merged into the members'
protocol's router beside sync and live. `/lend/<lend>/<path>` reaches the
lend's port on 127.0.0.1 as `/<path>`, one hyper HTTP/1 connection per
request, both bodies streamed and neither buffered. It strips the hop-by-hop
headers both ways, every client header under `x-svrngs-` and every one the
lend's secret names (compared as `rewrite_head` compared,
`iroh_identity_forward.rs:297-328`), then sets `x-svrngs-person` — the
dialer's name in the first group held that they stand in — and
`x-svrngs-key`, that person's first key there (`Member::person_key`,
bash-devices), then the secret; every other header,
`Range`, `If-Range` and `Content-Range` among them, passes untouched. A
response header holding the secret is dropped. It refuses, by name, a
request off the stream, a dialer standing in no group held, a path with a
`..` segment or an encoded `.` or `/` (`route_by_prefix`'s guard), a lend
not declared, a path the lend refuses, and a newcomer past the lend's
limit; each answer it makes in a lend's place, the lend's origin's and the
room's too, carries `x-svrngs-refused` and the body's `kind`, one of
`door::LendRefusal`, and a service's own `x-svrngs-refused` is dropped (mnp-1).
Who is let in is `door/limit.rs` (u3-6), a pure decision with the
time an argument, in `fair_share_cap`'s shape counted in people: a person is
counted on a lend while a request of theirs is in flight or within
`limit::WINDOW` (30 s) of one, a viewer whose path ends at a private,
link-local or unique-local address is guessed home and not counted, and a
newcomer past the limit is refused 503 with the reason and `using`. A
person let in is told the lend's declared upload over the people counted,
in `x-svrngs-declared-share-bps` on the request and on its answer, none
when the lend declares no upload; each verdict is traced at info with the
path iroh reports (`iroh::Via::observe`: the selected path's address,
whether it is relayed, RTT and bytes). A lend is `lend.rs`: its
declaration, `<data dir>/lends/<lend>.json` (the port, an opaque kind, how
many people may use it at once, the refused paths and the declared upload,
`upload_bps`, if any; unknown fields refused, and a kind or limit absent
too), read per request, so a running
node sees a lend come and go; its secret is `declared.rs`'s store under
`<data dir>/secrets/lends/<lend>/`. `lend::declare` and `lend::withdraw`
are the one writer of both, which `svrngs share` calls. `GET /lends` on the
members' protocol answers a dialer standing in a group held here what this
computer lends (`lend::Offered`: name, kind, limit, upload) read from the
declarations at each ask, and how many people use each now: `door::Using`,
a task owning `limit::Counts` and the clock, told by messages, the answer's
body holding a use until it ends.

A lend may declare a sign-in instead of a secret (mwp-1; DESIGN §3, "The
room", call 6): `sign_in` in its declaration, the port of a program of the
lender's, set by `svrngs share --sign-in <port>`; `lend::declare` refuses a
secret beside it. `door/sign_in.rs` holds it. `SignIns`, one task the door
holds as it holds `Using`, keeps each person's answer by lend, port and key
in memory, and on a person's first request asks `POST /` at the sign-in
with the two identity headers alone, one ask at a time, within 30 s. The
answer is `200` and `{secret, headers, script}`, every field required and
none other (`Answer::parse`): a secret of 16 to 512 unreserved bytes; no
header under `x-svrngs-`, hop-by-hop or `host`; a script holding neither
the secret nor `</script`. Anything else refuses the request, `502` with
`kind: "sign-in"`, the reason at warn, and nothing is carried. The answer's
headers then stand where the one secret would (`rewrite_headers`); the
stand-in `svrngs-stand-in` in the query and in each header becomes the
asker's secret; `Accept-Encoding` goes. Outward, every secret held for the
lend becomes the stand-in in each header, and in a textual body
(`textual`: text, JSON, XML, a playlist, JavaScript, or no type) as it
streams (`Swapping`, holding back one byte fewer than the longest secret),
its length dropped; a textual answer compressed anyway is refused, `502`.
A `401` to a request signed with a kept answer forgets it. A person's
script (mwp-2) goes first in each `200` `text/html` answer to their `GET`:
`tag` wraps it in a `<script>` carrying the nonce of the answer's
`Content-Security-Policy` when it names one, saying at warn when the policy
runs no inline script; `Swapping` holds the page back until `first_in`
finds the place — after the first `<head…>`, else `<html…>`, else
`<!doctype…>`, else the start, as `web/lend.js` puts its shim — within its
first 64 KiB. Such a page goes `no-store`, without `ETag` or
`Last-Modified`, and a request accepting `text/html` from a person with a
script goes without `If-None-Match` and `If-Modified-Since`. A lend with no
sign-in is carried as before.

A lend may ask (sup-2; DESIGN §3, "What an app gets"): `asks` in its
declaration, its lender's words, set by `svrngs share --ask`, carried in
`lend::Offered.asks` on every list and never read. `door/choices.rs` holds
what its door keeps and decides: each person's `Choice { said: Yes | Stop,
at }` per app at `<data dir>/choices/<lend>.json` (`lend::choices_path`),
written through `Choices`, one task, read on each request for a lend that
asks; `judge`, the one check in `lend_for` before the path check, carries a
lend asking nobody and its own pages, and refuses `unasked` or `stopped`.
`POST /choices/<lend>` on the members' protocol takes the dialler's person's
`{app, said}`; with a pass, on either protocol, only `{said: "stop"}` (the
type `Stopping`). `GET /choices/<lend>` answers the dialler's own. Every
choice, sign-in answer and count of who uses a lend is keyed by
`door::PersonKey`, which only the door makes, from the register's person
key, so a person's devices are one person to each. `lend::withdraw` removes them with the lend.

The home page (`home.rs`, sup-2), `http://svrngs.localhost:<api port>/`, is
routed by its `Host` (`api::to_a_lend`) beside the lends' origins and
served only at its exact host and port: a read only from its own origin, a
typed address or a navigation, a write only from its own origin with its
exact `Origin` (`only_itself`), every answer `default-src 'self';
frame-ancestors 'none'`. It serves `home/index.html`, `home/home.js` and
`home/home.css`, the record API's `/v1/groups`, `/v1/lends` and
`/v1/rail/people` from the same routers, and `/choices`, kept by this
computer's door for its own lends and dialled to the lender's otherwise
(`origin::dialled`). A lend's page stops itself at its origin, `POST
/svrngs-reads/<label>/svrngs-stop`, carried the same way with the pass its
origin signs. `/svrngs-reads` gives each asked lend's `asks` and its `ask`
address there.
`Node::lends` asks every other computer's key standing in a group held, at once, and
answers each lend with whose it is and its address on this computer's
loopback, the dialler's bridge to its lender then `/lend/<name>/`; a
computer that does not answer is named, and nothing is kept between asks.
`GET /people/lends` (`node::PEOPLE_LENDS_ROUTE`, u4-5) answers a member the
same of the groups it stands in, this computer's own lends (`door::offered`)
among them and the asker's computer not asked, each lend with its lender's
key (`node::Lending`), so a phone dials each lender itself (DESIGN §3, call 4).
An app on this computer asks the same of one group on the record API,
`GET /v1/lends?namespace=` (`surface.rs`, u5-1, new code), through the one
body (`node::Asking::lent`), with no asker left out; beside it `GET
/v1/groups` answers each group held, its name from the seed the command
keeps (the `SeedText` the redeem routes read) and this computer's person
from the register. Removal as the command asks it (`removal.rs`, U7):
`POST /v1/removals` commits svrngs-core's `Removal` and `POST
/v1/handovers` its `HandOnTo`, each answering an `Outcome` the command
words; each written wakes the round (`AskRound::wake`), so every
computer of the group holds it now, and a cut is told to the removed
person's computers once, one sync request carrying the cut alone
(`ring_sync::post`). A dialer the
keeper cut is served, on the members' protocol, only svrngs's refusal
`removed` (`door::gone`, chosen in `node.rs`'s accept by
`removed_anywhere`), and a pass it signed is refused the same; `/v1/groups`
says `removed` for this computer, `null` when its register cannot be read,
and the home page and the group's page word it. All are merged
into `api::router` before its loopback check, and the node writes the address it serves on to `<data dir>/api`
(`api::API_FILE`) at each start. Each lend carries the upload its lender
declares (`lend::Offered`'s `upload_bps`, required on read, `null` when
none). At a lend's origin on this computer (`origin.rs`), `GET
/svrngs-reads` (`surface::READS_PATH`, mnp-2) answers that lend's pages
the lends of each kind its lender declared it reads, in the groups its
lender and this computer stand in, through the same body: each one's kind,
name, lender's person and origin here, and whose computer did not answer,
no key (`surface::reads`). A refused path covers what is under it
at a segment boundary, as upstream's `covers`, compared percent-decoded,
case-blind and with repeated slashes as one. `tests/door/` runs a lender
and a member through it, with upstream's `picky_origin` and Range origin
forked as fixtures. `tests/canaries.rs` stands up Sam's computer and Alex's
in one group, in one process with no relay, and runs each canary of DESIGN
§8 under Node against their loopback record APIs; `scripts/check.sh` runs it
as its own step, and a canary that cannot start fails.
The room door (`room.rs`, u6-1, new code in the shape of upstream's LAN
door and `advertised_base`, DESIGN §8, U6's inventory) serves the lends a
person names to this computer's network while the command runs:
`room::open` asks `Node::lends`, refuses a name nobody lends, and binds
each named lend alone at `<private address>:<room::port_of(label)>`, a
port the label decides so it is the same every run, its router the
origin's body (`origin::reach`, as this computer's person) behind
`from_this_network`, whose predicate is the door's `limit::guess`. The
address is `room::pick`'s, over the endpoint's listed addresses and the
outward route's (`room::route`, a UDP socket connected, nothing sent):
the route's when private and listed, else the one private one, else
none. Each served lend's address sits in `Host`'s `room`, a `watch` the
room alone writes, which `/v1/lends` answers as `room`; dropping the
`Room` aborts its listeners and clears it. `src/room_tests.rs` plants
the picker's interface lists and a caller off the network;
`tests/room.rs` runs Sam's canary through Alex's room door from this
host's private address.

**`crates/svrngs`**: the command (DESIGN §5), the one crate with a binary.
`identity.rs` is the device key, commonwealth-transport's `identity.rs`
:1-126 at `e1a43bcd2`, its join proof and dial-info signing left behind, its
`node_pubkey` answering hex until svrngs-core's `actor_of`, the one hex
spelling of a key, took its callers; then changed: a key file that cannot be read,
and a fresh key that cannot be written, refuse naming the file, where
upstream fell back to a key nobody keeps, and the file is `identity`, a word
a person may read. `link.rs`, now svrngs-core's, is mesh-join-vocab's https guest link
(`deep_link.rs:228-300`) and its `sovereign://join/` form (`:421-436`),
made one short base64url `Token` (u2-8): a `Format` byte, the issuer's key,
16 secret bytes the link's signing key is derived from, and the issuer's
relay only from a computer without address lookup. It is printed as the
client's link, `svrngs://join/<token>`, and as `<origin>#<token>` beside
it for a group that keeps a page origin; `Link::parse` reads either or the
bare token. u4-2 moved it whole, with its tests, into svrngs-core
(`svrngs_core::link`), so a browser's page parses a link with the command's
body: there the issuer's key is the ed25519 key iroh's `PublicKey` checks and
its relay the `Url` iroh's `RelayUrl` parses. What the core cannot do stays
here in `issuer.rs`: `mint` draws a fresh token's secret, and `key` and `addr`
name the issuer as iroh dials it. `group.rs` is new: a group's `Seed` — its
name, the founder's name and key, 16 random bytes — whose BLAKE3 hash is
the group's id and so its directory; `seed.json` holds it beside the
record, and `Seed::roster` writes the register's seed, `roster.json`, the
founder's key bound to the founder's name; `answered` believes a
seed an issuer answers only when it hashes to the group named. u4-5 moved
the `Seed`, its id, its text and `answered` into svrngs-core
(`svrngs_core::seed`), so a browser's page believes an issuer's answer with
the command's body; `group.rs` keeps drawing a seed's salt (`group::draw`)
and keeping it on disk, and re-exports the rest; start-group moved it
into the node (`crates/svrngs-node/src/group.rs`), with `cli::keep_group`
as `Host::keep_group`, so a running svrngs makes a group for the home page,
and the command re-exports it as `svrngs::group`. `cli.rs` is new, in the shape of
commonwealth-rails' `cli.rs` and `main.rs`: verbs parsed by hand (`svrngs`,
`new <name> [--as] [--origin]`, `join <link> [--as]`, `link <group>
[--origin]`, `share <port> [--name] [--kind] [--limit] [--refuse]...
[--upload]`,
`unshare <name>`, `room <name>...`); one svrngs per
data dir (`$SVRNGS_DIR`, else `~/.svrngs`), held by a lock on
`<dir>/lock`; `new` and `link` append an `Invite` through the group's door
that lapses in a day and print the link; `join` asks a link's issuer what
it is for and redeems it there (`svrngs_node::redeem`), writes the group's
seed only once let in, and the origin only of an https link, syncs once from the issuer (`Node::sync_with`) and prints "You're in
<group>"; each verb then runs the node until Ctrl-C, serving the record
on `127.0.0.1` at a free port (`cli::API`), which the bare command prints,
with what the people of its groups lend (`Node::lends`): "Sam's <name>"
and an address on this computer. `share.rs` is `svrn publish`'s
`resolve_target` and `svrn mesh media offer`'s `probe_origin` at
`e1a43bcd2`, then changed (u3-5): `share <port>` takes a port alone,
probes it, names its kind from `well_known.txt` (data, so the infra holds
no app's noun; each row may say how the kind signs a person in, `Signs`:
with accounts its lender makes, or from the door's `x-svrngs-user`, and
then no password is asked; or the paths every lend of it keeps back,
`kept_of`, added to `--refuse`'s; a row with `-` for its port is a kind
known by `--kind` alone, as llama-server is) or `--kind`, its name from
`--name` or the kind, its limit from `--limit` or none, its upload from `--upload` in Mbit/s or none (u3-6),
and decides the lend in `lend_of`, which touches
nothing; `cli` reads the password from a prompt, never argv, declares the
lend through `svrngs_node::lend::declare`, and runs the node unless the
data dir's lock is held, when it says svrngs is running already and
leaves. `unshare <name>` withdraws one. `room <name>...` picks this
computer's private address (`svrngs_node::room::pick`), refusing with
none (exit 3), opens the room door for the lends named, says who can
open them and where, and holds it until Ctrl-C. What a verb prints is a
closed `Said`, and refusals a closed `Refusal`, both in a person's words,
exit 1 for a refusal and 3 for a precondition absent. `tests/new.rs` runs
one node in process; `tests/join_by_link.rs`, U2's bar, runs Sam and six
others; `tests/share.rs` runs Sam lending to Alex in process, and the
program itself for `share` with svrngs running and not.

**`web/`**: the page a group's link opens on a phone, `<origin>#<token>`
(DESIGN §3, "Joining"; U4): the crate `svrngs-web`, wasm only, in a
workspace of its own with upstream's measured pins (iroh =1.0.2,
wasm-bindgen =0.2.128, wasm-bindgen-futures =0.4.78, js-sys =0.3.105) and
lock, and the static shell beside it — `index.html`, `app.js`,
`store.js`, `sw.js`, and for a lend's own origin `open.html`, `open.js`,
`lend.js`, `shim.js`.
Forked from commonwealth-ai's `sovereign/apps/ring-runtime` at `e1a43bcd2`
with `kernel-types` replaced by svrngs-core, then changed (u4-3, u4-4,
u4-5). `src/lib.rs` binds a browser endpoint with the person's key, 32
bytes, parses the token with svrngs-core's `Token::decode`, and dials by
key — through the relay the token carries, else by the resolver iroh's N0
preset adds in a browser — every layer traced to the page's `__trace` as it
happens: `connect` dials the link's issuer on the members' protocol,
`connect_to_join` the issuer on the redeem protocol, and `endpoint`'s
`dial` a lender by its key (DESIGN §3, call 4). The dialled computer's `fetch` is HTTP over
one bi-stream per request by hyper's HTTP/1 client (`src/http.rs`, u4-4), its
body a `ReadableStream` either way. `src/join.rs` (u4-5) signs and believes
with svrngs-core's bodies, as `svrngs join` does: `ask`, `believe` (the seed
hashing to the group, `seed::answered`) and `redemption`, naming a browser.
`store.js` (u4-5) is the page's store: IndexedDB `svrngs` v2, a
non-extractable AES-GCM key under `seal`, and under `groups` each group
joined — its name, the person's, the link's token, and the person's key
sealed with the group's id as additional data (call 1). `app.js` reads the
fragment as the token: a link joined by, or the page alone, is in; a new
link is asked of its issuer, and a group kept already is in; else it asks the
person's name once, redeems, keeps the group, and lists `GET /people/lends`
as the bare command prints it ("Sam's shelf"); it keeps the layers on the
console at debug and says only DESIGN §1's words. `sw.js` caches the shell
and serves no lend: this origin holds the key. A lend opens at an origin of
its own (u5-2; DESIGN §7): `<label>.<the page's host>`. The page signs a
pass (`pass`, svrngs-core's `Pass`) to a fresh key for that lend and the
lends it reads, and opens the lend's `open.html` with the pass, the key,
the lends, the relay (`relay_of`, never the token) and itself in the
fragment; `open.js` keeps them in that origin's store (`store.js`
`lendKeep`), registers `lend.js` over the lend's address and hands it the
pass in memory. `lend.js`, a module, is the runtime: it serves every
request at the lend's address, and at the address of a lend the pass also
names, by `fetch` over a dial to the lender's door, `/lend/<label>/<path>`,
on the pass holders' protocol (`holder_endpoint`, `dial_lend`), the pass
on every request; keeps each lend's cookies in a jar of its own (the
dial's `fetch` hands `Set-Cookie` over as `x-svrngs-set-cookies`, since a
page's `Response` drops it); reads a redirect to a path under the lend;
asks for another pass through the page when the door refuses one; and puts
`shim.js` in each lent page, which gives the page its lend's cookies,
carries `WebSocket` to the worker (`socket`, the door switching a raw
stream) and hands an app's own worker to it. At `/svrngs-reads` (mnp-2,
`told`) it tells its pages the lends they read from what the page handed —
each one's kind, name and lender, the people whose computer did not
answer, and an address under `/svrngs-reads/<label>/` of this origin to
ask it at, never the pass or its key; a navigation to such an address is
sent to the group's page at `#open=<label>&at=<path>`, which signs that
lend's pass and opens its `open.html?back=<path>` in its own place, so
Back skips it (`opening`). A lend that asks is told with its lender's
words, `asks`, and `ask`, `<this origin>/svrngs-ask?lend=<label>&app=<this
lend>`, a computer's shape (sup-3); navigated with `&back=<path>`, it
sends the page to the group's page at `#ask=<lend>&app=<label>&at=<path>`
(`asking`, through `onward`, as `opening` goes). There `app.js` asks "Let
Sam's night use Ana's taste?" around the lender's words; Yes posts
`{app, said: "yes"}` to the lender's door, `POST /choices/<label>`, on a
dial from `endpoint` as the person; yes or not, it opens the app again at
the path. Its "What apps use", when opened, asks each lender that asks
`GET /choices/<label>` and gives each yes a Stop. A page stops its own
app, `POST /svrngs-reads/<label>/svrngs-stop`, which the worker sends to
that lend's door with the pass, `{said: "stop"}` (`stopping`). The page
host is told to send `frame-ancestors 'none'` (`web/vercel.json`), which
a `<meta>` policy cannot, and the tests' server sends `vercel.json`'s
headers (`web/tests/page.mjs`). With no pass in memory and
none it can read, it sends a navigation to `open.html?for=<label>`, which
reads the pass and hands it over again. The shell's paths are relative, and
the CSP admits a relay on 127.0.0.1. `scripts/build-web.sh` (upstream's
`build-ring-runtime.sh`) builds it into `target/web/site` with Homebrew's
LLVM on a Mac, and `--lint` holds it to rustfmt and clippy for wasm32.
`crates/svrngs-node/tests/browser/` is its test: iroh's relay server and a
node on 127.0.0.1 (`harness.rs`), an `Admit` written directly and a joined
group held in the page's store, and `web/tests/drive.mjs` serving the built
page and driving Playwright 1.59.1's headless Chromium (`web/package.json`,
`web/tests/page.mjs`); without Playwright or its browsers it fails naming
the install command. `crates/svrngs/tests/join_page.rs`, with
`web/tests/join.mjs`, is joining's: a link `svrngs new --origin` printed,
opened, a name typed, Sam's journal read for the one `Admit`, every
storage the page writes scanned for the key's bytes, and the first lend
opened at its own origin, `svrngs new` deciding that itself for a page on
`localhost` (u5-2). The runtime's test, `web/tests/runtime.mjs` in the
browser binary, plays and seeks an H.264 film ffmpeg makes at its lend's
origin, stops the worker through Chromium's DevTools protocol, and reads a
second computer's lend, which the first reads, at its origin.
`web/tests/ask.mjs` is the phone's ask (sup-3): a lent page asking, the
group's page's Yes, "What apps use" and its Stop, the page's own stop,
the group's page framed by the lent page, and the page's pass and holder
key handed to the test, which posts a yes with them at the lender's door.

**`apps/expenses/`**: the Expenses fold (`expenses.js`) and its tests, from E0,
the bot at tag `e0-bot`. Its reducer (`init`, `reduce`) folds the acts
`actsOf` reads from the record's log, `by` the person an app wrote for, else
the person whose computer wrote it. `canary.mjs` is the Expenses canary of
DESIGN §8: apps on two computers write, a correction voids, and both copies
fold to the same balances, the ones the acts say, or it names both and what
each says. Its chat handlers went with the bot and are at that tag.

**`apps/web-conformance/`**: the conformance canary (DESIGN §8; u5-2), a
Node service with no dependency (`serve.mjs`), one route per thing a
self-hosted app leans on, and the checks its page runs (`checks.js`).
`crates/svrngs-node/tests/conformance.rs` lends two of it and opens both
from a computer and from a phone, each at an address of its own (DESIGN
§7), through `web/tests/conformance.mjs`, and keeps each route's verdict in
`target/conformance/`, which `scripts/check.sh` prints. Given the lender's
key, `serve.mjs` keeps accounts, a token each, and `answer.mjs` is the
lend's sign-in that answers each person's (mwp-4): a second app, after
Jellyfin's, on the door's sign-in (DESIGN §3, "The room", call 6).

**`apps/jellyfin/`, `apps/tap/`**: so far only each app's `nouns.txt` — the
words the infra may never use. `apps/jellyfin/` holds whatever knows
Jellyfin's API (DESIGN §3, U5's call 3); Tap is a canary (DESIGN §8).

**`demo/`**: how a demo is recorded. **`ralph/`**: the work loop. A queue is a
directory `ralph/next/<queue>/` holding `queue.toml` (models, timeouts, the
launch line), `STATE.md` (its rows) and `PROMPT.addendum.md`, which the driver
renders over `ralph/PROMPT.base.md`; E0's queue at tag `e0-bot`
(`ralph/next/e0/`) is a working example. The driver is commonwealth-ai's
`scripts/ralph.py`, run as `python3 ~/dev/commonwealth-ai/scripts/ralph.py run
--workdir ~/dev/svrngs --queue <queue>` — process tooling, not code svrngs
links; it marks a row done itself, when a session reports it. U1's queue is
`ralph/next/u1/`.

**`scripts/`**: `check.sh` is the definition of done; `lean_check.py` keeps the
repository standalone and within budget; `vocab_check.py` keeps every app's
nouns (`apps/<app>/nouns.txt`) out of infra code; `ports.py` lists the fork's pairs;
`principles_check.py` keeps `docs/principles/` pinned; `build-web.sh` builds
and lints the page (`web/`).
