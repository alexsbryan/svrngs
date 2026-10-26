# give-economize: the ceiling, §4, and the trust floor's two lines

The row ends here, as it says: an economizing pass, an estimate, and then
three asks of the operator (STATE.md, give-economize (c); CHARTER.md,
"Awaiting the operator" 1 to 3). give-token does not start until they are
answered. The evidence is `SURVEY.md` in this directory.

## What the pass did

- Cut 13 counted lines, each in its own commit with its behaviour kept
  and no test file changed:
  - the links' sequencer taken in one line, 3acd44aa, 6 lines;
  - an own pass's reads said in one line, 842f9da3, 4 lines;
  - the door's "unasked" read in one statement, 36f58a12, 3 lines.
- Lowered the repository's cap to one past where they land (85a1817b).
  `python3 scripts/lean_check.py`: **18,240 / 18,241 before, 18,227 /
  18,228 after**. svrngs-core is **3,238 / 3,239** throughout, already
  one past.
- Found nothing more to cut. The compiler's unreachable items, the clone
  report's clusters and the window pass's runs are each read and either
  kept or named (SURVEY.md §1 to §4). One finding is left open:
  `Capability::Read` is unwired, so the carried route's read and the
  log's append judge one scope with two deciders. Changing that is not a
  pass's job (SURVEY.md, "Open").

So the tree has 1 counted line under each cap, and give-token cannot land
without a raise.

## 1. One ceiling for give-token through give-play, core and repository

The estimate (SURVEY.md §6), in four ledgers:

| Row | Core | Library | `apps/` (not counted) | Surface |
|---|---:|---:|---:|---:|
| give-token | 0 | +45 to +90 | +15 to +45 | +1 |
| give-session | 0 | +110 to +220 | 0 | +1 |
| give-udp | +2 to +5 | +135 to +250 | 0 | +1 |
| give-tcp | 0 | +60 to +130 | 0 | 0 |
| give-play | 0 | +165 to +310 | 0 | 0 |
| give-play-scene | 0 | 0 | 0 | 0 |
| **Total** | **+2 to +5** | **+515 to +1,000** | **+15 to +45** | **3** |

GIVE.md §9 sizes the same rows at core +13 to +25 and library +650 to
+1,145. The core differs because the raw `Source` lives in svrngs-node
(`lend.rs:181`), not the core. give-tcp is smaller because give-udp lands
the admission, the limit and the dialling side first (SURVEY.md §6).

**The ask: +1,005 repository and +5 svrngs-core over the caps this pass
leaves, so at most 19,233 and 3,244.** Each row raises only what it
lands, to one past where it lands, and a row past its range stops for
the operator (CHARTER.md, "Leave for the operator"). That is the
estimate's top.

Two alternatives:

- **GIVE.md's top:** +1,170 and +25, so at most 19,398 and 3,264. This
  leaves room for the exploration's figures where they run higher than
  this estimate.
- **A smaller step:** the caller check alone, give-token and
  give-session, at +310 and 0, so at most 18,538 and 3,239. The door's
  ceiling would be asked again once the check has landed and been
  measured.

## 2. Does the caller check overturn WHAT_CAN_ACT_AS_YOU §4 for programs meant to be contained?

§4 says, of "Loopback is the auth": "**Your computer is you.** A program
you run can already read your files and keys, and svrngs cannot defend
you from it. Accept it, and say so." (svrn-docs
`rings/current/WHAT_CAN_ACT_AS_YOU.md:83-85`).

GIVE.md §5 measured, on this Mac (podman 6.0.1), that a container reaches
the Mac's `127.0.0.1` through `host.containers.internal`. A container is a
program meant **not** to be you. Without a check, any app run in podman
could write the record as you.

- **Yes:** the record API asks every program for the token in
  `<data dir>/token`, mode 0600. A program you run yourself reads it the
  way it could read your key, so §4 still holds for your own programs.
  The token closes the API only to a program that cannot read your files.
  give-token goes ahead as written.
- **No:** give-token and give-session do not go. "Loopback IS the auth"
  stays (`api.rs:12`), and Give's rows would need another way to contain
  apps. GIVE.md §5 measured the one it found, `--internal`, and it cuts
  the apps off the internet.

I recommend yes. It keeps §4's own claim, since your programs can still
read the token, and it is the record API's own seam (GIVE.md §5).

## 3. The trust floor's two new lines, word for word

Each is a `listen` entry (`scripts/trust_floor.txt:25-27`): the bind line
as the code reads it, then `|` and the reason. The row writes the code,
so the left of `|` is that line as it lands. The reason is what the
operator approves:

1. **The member's Play listener (give-play):**

   ```
   listen crates/svrngs-node/src/<Play's module>.rs: <its bind line> | Play: 127.0.0.1 alone, at the lend's own port (room.rs `port_of`), UDP or TCP as the lend is, only while its person has Play on for a lend of a group they stand in; nothing listens while it is off, and it closes when they stop standing
   ```

   Play binds UDP and TCP on two lines of code, so the floor holds each
   line with this same reason. It is still one widening.

2. **The lender's per-flow UDP socket (give-udp):**

   ```
   listen crates/svrngs-node/src/<the UDP door's module>.rs: <its bind line> | a UDP lend, on the computer that lends it: one socket per member's flow, on 127.0.0.1:0 and connected to the lent port alone, for a key standing in the group within the lend's limit, closed when the flow idles past its expiry
   ```

   The charter calls this "the keeper's" socket. In svrngs it belongs to
   whoever lends the port, and Give's keeper is that lender. The library
   learns no word of Give's (fence 3).

Neither line widens past `127.0.0.1`. give-tcp's lender side connects to
the lent port and binds nothing, so it needs no line. give-token's gate,
"a caller without the token is refused", narrows the floor, so it is not
asked here.
