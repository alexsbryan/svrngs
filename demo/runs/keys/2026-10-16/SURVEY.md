# A key per group on a computer — the survey (groups-keys (a))

The row: `ralph/next/groups/STATE.md`, groups-keys. The operator's words,
2026-10-14: "Final for now; per-group keys as a row". DESIGN §6, "A key per
device per group, bound to the person", decides the rule. Its own last
sentence says a computer departs from it: "A computer's one key for every
group it holds (`crates/svrngs/src/identity.rs`) still departs from 'per
group', a separate question."

This survey reads what that one key ties together, lays out the shapes a key
per group could take, and gives each one's cost against the budget the queue
has left. It changes nothing in the product.

## 1. What the one key ties together, at HEAD 75a4463f

- **One file, one key.** `crates/svrngs/src/identity.rs:97`
  (`load_or_generate_node_key`) reads `<data dir>/identity` or mints it.
  The command passes it once to `Node::start` (`crates/svrngs/src/cli.rs:948`).
  The relay host does the same (`crates/svrngs/src/relay.rs:136`).
- **The key signs.** `Node::start_every` hands the key to
  `RingRail::new(&data_dir, Arc::new(key))` (`crates/svrngs-node/src/node.rs:265`).
  The runtime keeps one `signer` for every namespace
  (`crates/svrngs-runtime/src/lib.rs:56-59`), and `commit` signs every act of
  every group with it.
- **The same key dials and is dialled.** `node.rs:264` binds the one iroh
  endpoint with `SecretKey::from_bytes(&key.to_bytes())`. The module docs say
  why: "One key signs and dials" (`node.rs:11-15`; DESIGN §8, "Four facts").
  An iroh endpoint has exactly one secret key
  (`iroh-1.0.2/src/endpoint.rs:524`, `Builder::secret_key`; `:1172`,
  `Endpoint::secret_key`). So one endpoint is one key.
- **Dials read the register's key as the endpoint's id.** Every dial of a
  member goes through `node::dialled` (`node.rs:688-712`). It turns each
  standing actor into `Contact::named(actor_key(actor), relay)`. So a key in
  the register *is* the address dialled. Dials come from the round, the
  lends' list, the live lane (`rail.rs:568`), a blob's holders
  (`blob.rs:117`) and a folder's fetch.
- **Accepts read the dialer's endpoint id as the register's key.** The
  acceptor admits a dialer standing in any group held. It hands the dialer's
  verified key to the handlers as a `Dialer` extension (`iroh.rs` module
  docs). The handlers then look that key up in the register: the sync route,
  the door (`door.rs:505`, `standing_anywhere`; `door.rs:1063`,
  `PersonKey::resolved`), and redemption. Redemption requires the dialer to
  *be* the key it admits (`redeem.rs:22`, "checks that the dialer is the new
  key"; `NotTheDialer`, `redeem.rs:101`).
- **"Who I am" is one string, used in 17 product files.** `Host::actor` and
  `rail.signer()` stand for "this computer" in each group. The count is 55
  lines over 17 files:

  ```
  grep -rhE "host\.actor|\.signer\(\)|endpoint\.id\(\)|host\.endpoint|self\.endpoint|host\.transport|self\.transport" \
    crates/svrngs-node/src crates/svrngs-runtime/src crates/svrngs/src --exclude='*tests*' | wc -l
  55
  ```

  The files are join.rs 5, node.rs 8, blob.rs 5, folder.rs 4, iroh.rs 4,
  removal.rs 4, surface.rs 4, invite.rs 3, group.rs 3, and 2 each in home.rs,
  synced.rs, redeem.rs, rail.rs, carried.rs, origin.rs and runtime/lib.rs,
  with 1 in ring_sync.rs.
- **The register: one key, one person.** `Member` (`crates/svrngs-runtime/src/register.rs:86-95`)
  holds a key's person, its device, `person_key` (the person's first key,
  the same for every device of theirs), when they joined, and its relay. The
  map is keyed by that key. A person is their first key in the group, so a
  new key in a group is a new person unless a device link binds it
  (DESIGN §6).
- **Why a leaver cannot come back from the same computer.** Their cut covers
  the one key. svrngs-core's `Redeem` refuses a cut key
  (`crates/svrngs-core/src/register.rs:73-78`, `RedeemRefusal::Cut`). Before
  that, the joiner's node refuses on its own copy
  (`crates/svrngs-node/src/join.rs:69-74`, `NotJoined::Left`). The pages say
  so: `home/home.js:92` and `:231` ("this computer can’t rejoin yet"), and
  `cli.rs:201` and `:1319`.
- **A phone already has a key per group.** `web/store.js` holds "One key per
  person per group". The page binds an endpoint per group key, and the page
  is never dialled (`node.rs:43-47`). On a phone, "one key signs and dials"
  already holds key by key, not device by device.

## 2. The budget left

`python3 scripts/lean_check.py` at HEAD:

```
crates/svrngs-core     2992 / 2993 lines
.                     16295 / 16296 lines
```

groups-economize left the caps at svrngs-core 2,953 and the repository
16,136 (`df996e3b`). The queue's ceilings are +40 and +200 over those
(CHARTER.md, "The budget"): **2,993 and 16,336**. groups-tabs and groups-leave
used most of it (`eb050a26`, `eb0248c4`, `f3a5a519`, `00864058`, `05d46b23`).
What is left for this row is **1 line in svrngs-core and 41 in the
repository**. For scale, groups-leave (a)–(c) cost svrngs-core 40 lines and
the repository 137.

## 3. The shapes

Each estimate counts product lines that `lean_check.py` would count (tests
apart). Each comes from the sites in §1 that the shape must touch, not from
a build.

### A. A signing key derived per group from the device's

The signing key is derived as `H(device seed ‖ group id)`. The endpoint stays
the device key.

- **It fails the bar on its own.** A rejoin is the same group id, so the
  derivation gives back the key that was cut. The new invitation is then
  refused as `RedeemRefusal::Cut`, exactly as at HEAD. To give a new key it
  needs an epoch kept per group (`H(seed ‖ group ‖ n)`). A kept epoch is a
  kept per-group fact, so it costs what a stored key costs.
- **It breaks "one key signs and dials".** The register would name a key
  that no endpoint is bound with. So every dial and every accept needs B's
  mapping (below), or C's endpoints.
- **Existing groups.** Their members stay who they are only if every group
  that exists is exempt and keeps the device key. That means a per-group
  record of which key the group uses, which is the epoch above.
- **Cost:** B's or C's, plus about 10 lines for the derivation. It is never
  the smallest.

### B. An `Admit` naming both a signing key and the dialled key

A fresh signing key per group (or per rejoin). The `Admit` (or the
`Redemption` inside it) also names the device key it is dialled at, which
co-signs, so nobody can name someone else's computer as theirs.

- **svrngs-core:** a field on `Redemption` with the device key's consent
  (`lib.rs:462-477`), at a new line version as `device` came in at version 4.
  Then `membership` keeping each key's dialled key, `standing_reads`, and
  `Redeem` checking the dialled key is not cut (`register.rs:73-78`). About
  **30–45 lines**. `Device`'s field cost 15–20 in the same files (u4-1), and
  this one also carries a signature and a check.
- **The node:** `dialled` reads the member's dialled key (`node.rs:705-709`,
  a few lines). But every accept-side lookup turns the dialer's endpoint id
  into the signing key standing in the group the request names. One endpoint
  now stands as different keys in different groups. Those lookups are the
  sync route, the door's `standing_anywhere` and `PersonKey::resolved`, the
  pass protocol, `removed_anywhere`, the folder's dialers (`folder.rs:636`)
  and the blob routes. Redemption's "the dialer is the new key" rule
  (`redeem.rs:22`, `:101`) becomes "the dialer is the named dialled key".
  The `Relay` act is signed by "the key it speaks for", so it would move to
  the dialled key or name it. join signs with the new key and dials with the
  device key. About **120–180 lines** in the node and the command.
- **Existing groups:** a line without the field reads as dialled at its own
  key, so every member stays who they are and no copy needs migrating.
- **What it changes:** the record's wire (a line version), and "one key
  signs and dials" as a rule of the record. The charter keeps a new *kind*
  of act for the operator. This adds no new kind, but it does change what
  an `Admit` says.
- **Does not fit:** svrngs-core needs 30–45 lines and has 1.

### C. An endpoint per key the computer holds

The device key keeps every group it signs today. A computer joining a group
where its device key was cut mints a fresh key for that group, keeps it in
the group's directory (0600, as `identity.rs:58-75` keeps the device key),
and binds a second iroh endpoint with it. "One key signs and dials" stays
true key by key, as on a phone.

- **svrngs-core:** **0 lines.** The register is already per key. The rejoin
  is a plain redemption of a new key under the person's name, and the name
  check passes because a person who left no longer stands
  (`register.rs:79-90`).
- **The runtime:** `RingRail` signs per namespace, with the device key as
  the fallback. That means a map read at open and saved at join. About
  **20–30 lines** (`lib.rs:56-59`, `:76-83`, `:99-101`, and `commit`).
- **The node and the command:**
  - The `Host` holds an endpoint and transport per key, and the acceptor
    loop runs on each of them.
  - Each dial picks the endpoint of the key standing in the dialled group:
    the round, the lends' list, the live lane, a blob, a folder and a
    removal's tell.
  - Every "this computer" check of the 55 lines in §1 asks per group:
    blob.rs, folder.rs, removal.rs, carried.rs, group.rs, rail.rs,
    surface.rs and home.rs.
  - The relay is said per key (`Host::say_relay`), a link names the
    group's own endpoint (`invite.rs:126`), and the door's `key`
    (`node.rs:305`) is set per group.
  - A restart binds every kept key.
  - join binds the fresh endpoint before it asks the issuer, since the
    issuer lets in only the key that dials it (`redeem.rs:22`).
  - The pages and the command lose "can’t rejoin yet" (`home.js:92`,
    `:231`; `cli.rs:201`, `:1319`), and `NotJoined::Left` goes
    (`join.rs:69-74`).

  About **200–300 lines**.
- **Existing groups:** unchanged. Only a group this computer was cut from
  ever gets a second key, and nothing a computer holds is migrated.
- **What it costs at run time:** one more relay connection and accept loop
  per rejoined group (iroh-relay keeps one connection per key,
  iroh-relay-1.0.2 `server/clients.rs:80-92`, groups-tabs). Several
  endpoints in one process are what svrngs's tests already run
  (`crates/svrngs-node/src/iroh/tests.rs:53`, `:82`, `:162`).
- **Does not fit:** svrngs-core fits (0 of 1). The repository needs 220–330
  lines and has 41.

### D. The cut key admitted again (found in the reading)

The fold lets a cut key stand again from a later counting `Admit`, and the
issuer's `RedeemRefusal::Cut` goes.

- It fails the bar. The person is not "someone new to the record": their
  person stays their first key, now cut. This is the row's own plant ("the
  device key reused, as at HEAD"). It also reverses DESIGN §6's "a cut key
  never stands again".
- svrngs-core about 10–20 lines, in `membership.rs`'s walk. That needs the
  operator twice: past the ceiling, and a change to what a cut means.
- **Not safe:** a keeper's removal would undo itself at the next link that
  person is sent.

### E. A second svrngs on the same computer (found in the reading)

A rejoin runs under a second data dir, with its own `identity` and its own
process.

- Almost no code. But it means a second home page and a second
  `svrngs.localhost`, and a person who must know which one holds which
  group. That breaks "the home page for the afraid" (one page, no terminal
  to finish what it starts). `cli.rs`'s claim also holds one node per data
  dir, not per computer.
- **Not safe for a person.** It is a workaround, not a shape.

## 4. Which is smallest and safe

| Shape | svrngs-core | repository | existing groups | meets the bar |
|---|---|---|---|---|
| A, derived | B's or C's | B's or C's +10 | only with a kept epoch | only with a kept epoch |
| B, `Admit` names both | 30–45 | 150–225 | unchanged | yes |
| C, an endpoint per key | 0 | 220–330 | unchanged | yes |
| D, the cut key again | 10–20 | 20–40 | unchanged | no (not someone new) |
| E, a second svrngs | 0 | ~10 | unchanged | no (two home pages) |

**C is the smallest safe shape.** It touches no line of svrngs-core and no
byte of the record. It keeps "one key signs and dials" true key by key, as a
phone does. And it leaves every existing group's members who they are,
since only a group this computer was cut from ever sees a second key.

**C does not fit what the queue has left.** It needs 220–330 repository
lines against 41 left under the +200 ceiling. B needs svrngs-core lines the
ceiling does not have (30–45 against 1). By the row ("where it does not fit
… NEEDS_HUMAN.md with the survey's estimates, and the row ends there") and
the charter ("The per-group keys row stops and asks if its survey says it
needs more"), (b) is not built. The question goes to the operator in
`NEEDS_HUMAN.md` beside this file.

What would falsify this: a build of C under 41 repository lines, or a shape
not listed here that meets the bar inside 1 svrngs-core line and 41
repository lines. The estimates are from the sites in §1, not from a spike;
a spike of C is the first step of (b) if the operator raises the ceiling.
