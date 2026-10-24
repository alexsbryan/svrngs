# Authorship by signature: the design

capability-authorship (a), at 289b9531. A design for the operator's yes
(CHARTER.md: "a change to what an act is"). It changes no product file. Its
golden tests are 78231455 (`crates/svrngs-core/tests/golden.rs`). Every
claim cites a file and line at that commit.

The rule it builds is DESIGN §1: "An entry's author is its signer;
replicas only carry." The note behind it is svrn-docs
`rings/current/WHAT_CAN_ACT_AS_YOU.md`: §4, §7.2 and §7.4, with §10's bar
for authorship: "a forged, replayed, or carried-for-another entry changes
nothing. A phone's yes and a phone's write, carried by two different
computers, count as that phone's person. Old acts still verify."

Three forks are the operator's. Each has a recommendation:
- **F1**: how a lent app on a port writes as the person asking (§6).
- **F2**: what a phone's chained pass does with a computer on an older
  svrngs (§11).
- **F3**: the ceiling for capability-signed and capability-keys (§14).

They are asked in `NEEDS_HUMAN.md`, beside this file.

## 1. Today, in five facts

1. **Every line is signed by the computer that writes it.** `SignedOp`'s
   `actor` is its signer (`lib.rs:54-58`). Nothing else is ever a signer:
   a phone's key signs passes, never a line (`pass.rs:14-20`).
2. **A computer writes for someone else by naming them.** A computer
   writes another person's act by naming them in `on_behalf_of`
   (`lib.rs:612-628`), and the rail "never lets it stand in for
   authorship". Apps read it anyway: `by = on_behalf_of ?? person`
   (`apps/expenses/expenses.js:146`, `apps/grapevine/grapevine.js:206`,
   `apps/canon/house.js:109`). The core reads it as authorship in one
   place, a yes to an app's version: `whose` (`synced.rs:182-184`).
3. **Four writers name another person:**
   - the carried app's route, for whoever the door resolved
     (`carried.rs:354`);
   - a yes to a version (`carried.rs:197`);
   - the record API, for anyone its body names (`rail.rs:300-328`), which
     `Append` checks only for standing (`record.rs:36-40`);
   - a lent app's server, such as Events', for each person asking
     (`apps/events/store.mjs:57`, `server.mjs:122-129`).
4. **A phone writes nothing itself.** Its acts reach a computer as
   requests, and that computer signs them. A phone holds its person's key
   on the group page's origin, sealed (`web/store.js:1-33`). A lend's
   origin holds only a pass and the pass's holder, a fresh key for each
   open (`web/app.js:264`, `:272`).
5. **What the golden fold shows.** It freezes today's fold
   (`tests/fixtures/acts_2026-10-24.fold.json`). Three facts in it:
   - alex's computer's `Runs` for bo counts as bo's yes;
   - cy's `Runs` names bytes no `Synced` act placed (shortfall 8), and
     they run as cy (`"money as cy"`);
   - cy's correction voids alex's dinner. The core voids whatever a
     correction names, and the apps judge it by author.

## 2. Who signs what, after

| What is written | Signed by | Carried by |
|---|---|---|
| A computer's own act: a command, a local program, a carried app opened on this computer | this computer's key, as today | any computer, unchanged |
| A phone's act in an app it opened | the pass's holder key, with the pass on the line (§3) | any computer, unchanged |
| A phone's decision: a yes to a version | the phone's own key, on the group page (§3) | any computer, unchanged |
| A lent app's act as the person asking | that person's own device, on the way back (§6, F1) | the lender's computer, unchanged |
| An act naming someone else in `on_behalf_of` | refused when written; one already held is judged as §5 says | — |

"Carried unchanged" means the bytes the device signed. A computer that
takes a line it did not sign adds nothing to it. It stores the line as
the sync round already stores a peer's (`svrngs-runtime/src/lib.rs:278`,
`ingest_all`). Then it gossips the line.

**The North Star's three tests.**
- What does a lazy person get in ten seconds? Nothing new to do. The one
  exception is a phone person whose yes went through another computer:
  their phone shows one line, once (§10).
- Could a developer read all of it, and did they have to? An app that
  writes as the person asking makes the same calls as today, with one
  rename (F1, §6). No SDK.
- Does this build a centre? No. Each device signs for itself, and no
  computer speaks for anyone else.

## 3. The phone signs its own acts

A phone keeps two kinds of key. Each signs its own chain of lines.

**Its own key, on the group page.** This is the key a device link admitted
as a `Device::Browser` (`lib.rs:486-493`). It signs **Decide** acts only. A
yes to a version (`RailAct::Runs`) is signed on the group page, which
holds the key (`web/store.js:145`). The page hands the signed line to the
computer it reaches. The yes is posted to `/runs` today
(`web/app.js:471-474`, `:483`). The chain's newest seq and id are kept in
the group's row of the `groups` store, written before the line is sent.
So a reload, or a second computer, never signs the same seq twice. If two
tabs do sign one seq, that is a fork. Forks void nothing
(`admit.rs:110-119`): both acts count, and the fork is named.

**A holder per lend origin, with its pass.** The lend's own pages write
through the worker. Today the worker forwards `POST /svrngs/acts` to a
computer (`web/lend.js:521-535`, `carried.rs:61`). After, the worker:
- shapes the act with the code the door uses today (`carried.rs:340-353`),
  moved into svrngs-core and built to wasm, so both run one copy of it;
- signs the act with the pass's holder;
- carries the signed line to the computer it dials.

Each of the holder's lines carries the pass that names it, in a new field
`grant` (§9). The fold judges every line on its own, so no line waits for
the chain's first line to arrive. The pass is verified once per fold,
however many lines carry it (§4).

Today the holder is fresh at each open (`app.js:264`). After, it is kept
per lend origin across passes: a renewed pass names the same holder,
which the origin's renewal request gives the group page. So a phone's
acts in one app are one chain, not one per open, and the sync digest
gains one entry per phone and app, never one per open. The chain's tip
is kept beside the pass (`web/store.js:174-181`), written before the line
is sent. A line no computer took yet waits in that store. It is sent
again to the next computer the worker reaches, the same bytes, so it is
never signed twice.

**The computer's part.** A new route on the door takes a signed line.
Members' and holders' protocols both serve it. The route:
- refuses a line that does not verify, or whose signer neither stands nor
  carries a pass for a lend here (traced at debug, said as "this entry
  was not signed by anyone in House");
- stores the line with `ingest_all`;
- answers whether the fold counts it now, so the page can say "Saved" or
  why not.

The route never writes a line of its own. A correction of another app's
acts is still refused at the door, as `carried.rs:352` refuses it today,
and the fold refuses it too (§4).

## 4. `Grant`: what a signer may do in the log

svrngs-core gains the note's §10 target shape for the log side only.
Reach keeps `Pass::check` as its one decider (note §10, 2).

```rust
/// What a line's signer may do as a person: a standing key is that
/// person's own device; a holder is what a pass on the line grants.
pub struct Grant<'a> {
    pub by: &'a str,              // the person's first key
    pub holder: &'a str,          // the line's actor
    pub app: Option<&'a Label>,   // None for the person's own device
    pub until: Option<i64>,
    pub proof: Proof<'a>,         // Own (standing) | Pass(&'a Pass)
}
pub enum Capability<'a> { Read(&'a str), Append(&'a str), Decide }
pub enum Verdict { Passed, Refused(Why), CouldNotJudge(Unjudged) }
impl Grant<'_> { pub fn may(&self, cap: Capability, at: i64) -> Verdict; }
```

- **Append**, by signer:
  - A standing key appends any kind as its person. That covers this
    computer, its programs, and a phone's own key ("your computer is
    you", note §4).
  - A holder appends only `<kind>/…` of its pass's app, at a time before
    the pass's `until`. The kind comes from the label's carried name,
    `"{kind}-{group[..16]}"` (`lend.rs:229-230`). That naming moves into
    svrngs-core as `Label::carried`, one decider for the node and the
    fold.
  - A holder's correction counts only when its target is of that kind. So
    Money's page cannot void its person's Events acts.
  - When the pass's signer is cut, the cut covers that signer's holders
    too. A cut's `through` names the newest acts the remover held of the
    key and of each holder its passes named. `covered` follows those
    chains as it follows the key's own (`membership.rs:518-549`). Old
    cuts name no holder, since none existed.
- **Decide**, by the person's own devices alone. A `Runs`, and every act
  of the register (`Admit`, `Invite`, `Remove`, `HandOn`, `Visit`,
  `Relay`), counts only from a standing key. No pass grants Decide, so a
  holder's `Runs` counts nowhere.
- **Read**: the carried route's filter, `own` (`carried.rs:314-315`),
  becomes `may(Read(kind))`. One scope then decides what an app reads and
  writes. What reads stays as it is today. Sealing will narrow Read to the
  keys a line is sealed to, when its trigger comes.
- **Every refusal** is a `RailGap` naming its fact, traced at debug in
  `admit` as each gap is today (`admit.rs:527-541`).

`admit` resolves a holder through the pass on its line. A signer standing
nowhere, with no pass, is `UnknownSigner`, as today. `AdmittedOp.person`
is the grantor's person, and `actor` stays the line's signer. So an app's
fold reads a phone's act as that phone's person's, with no `via`.

## 5. `on_behalf_of`: new acts write none, old ones keep their meaning

**For new acts, it is gone.**
- `carried.rs:354` and `:197` write none.
- The record API (`rail.rs:300-328`) accepts a body naming this
  computer's own person, and drops the field. That keeps Grapevine's and
  Events' store, which name `me` (`apps/grapevine/store.mjs:57`). It
  refuses a body naming anyone else, `kind: "not_yours"`: "This computer
  writes only as Sam. Another person's entry is signed on their own phone
  or computer."
- `Append.on_behalf_of`, `AppendRefusal::NotInGroup`
  (`record.rs:11-46`) and `Command::on_behalf_of`
  (`command.rs:50-53`, `:175`, `:226`, `:263`) are cut.
- The field stays on `SignedOp` (`lib.rs:627-628`), so old lines keep
  their bytes.

**Old acts.** A line already held that names someone else is either an
honest phone's write, the Events scene's guests, or a forgery. Its bytes
cannot tell which. A malicious computer can still write lines at version
1 that look old. Timestamps can be backdated, so a date cannot separate
them. A hash can. So, as a cut does (DESIGN §6, "A removal names the
removed key's acts by hash"):

- **The keeper's cutover**, a new act: `RailAct::Signers { through }`.
  `through` names each actor's newest acts the keeper held. The keeper's
  computer writes one per group, on its first round at the new version.
  It is one decider, the office that already sequences cuts. A Decide
  act's walk is the one `HandOn` takes (`membership.rs:405`).
- **An act naming another person counts as today only when the cutover
  covers it**, by ancestry, the way `covered` works. Apps keep reading
  `by` and `via` for it, so "bo, through cy's computer" stays true of
  every old entry.
- **An act naming another person, written past the cutover, counts
  nowhere.** It is a new gap, `CarriedForAnother { id, actor, named }`, a
  Contradiction, so it does not flip completeness. This closes shortfall
  2 at the root, for a computer running svrngs's code or not.
- **An act naming its own signer's person counts as the signer's,
  always.** That is every line `carried.rs:354` wrote for a computer's
  own person. So an older computer's own writes keep counting after the
  cutover.
- **A `Runs` naming another person never counts, old or new.** Decide is
  the person's own devices alone. That is the one move in old folds, §10.
- **Until the keeper's computer runs the new svrngs**, the group folds
  every act naming another person as today. Every gap above is traced,
  and `admit` says at debug which acts it counted only on the keeper's
  coming cover. The window closes when the keeper updates.

**What it costs.** An older computer's acts for someone else, written
after the cutover, count on older copies and not on newer ones. An
older build names the cutover act `NewerVersionLine` (`admit.rs:89-92`).
So it already says its answer covers a subset. In practice such an act
comes only from a phone page left open across the release, on an older
computer.

## 6. F1: a lent app that writes as the person asking

Events lent from Sam's computer writes each guest's "coming" as that
guest, through Sam's record API (`apps/events/server.mjs:122-129`,
`store.mjs:57`). The Events scene plays it in every gate
(`crates/svrngs/tests/scenes/events.rs:1-14`, `:174-181`). That is
"carried for another", and §5 refuses it. Fence 2 says nothing a person
does today breaks. NORTH_STAR §2 says any app on a port. So it needs a
way to keep working.

**Recommended: the person's own device signs it, on the way back.** The
app's answer names the act in a header, `x-svrngs-append: <act>`, where
it now calls the record API. The person's own side reads that header on
the response path: this computer's door for a lend opened at
`<label>.localhost`, or the phone's worker. It does three things:
- checks the act against the lend's declared `--kind` and the group the
  person stands in, as `Grant` does for a holder (§4);
- signs it as the person, or with the phone's holder;
- carries it to the lender's computer before the page sees the answer.
  So the app's next read holds it.

The capability is "writes its own part of your group's record as you".
Its consent is opening the lend, whose button says so. It has the shape
capability-open gave the open (`home/question.js:21-28`): "Open Sam's
events. It writes what you say there into The House's record, as you."

A lend with no `--kind` writes as nobody but its lender. The app's
change is one call:

```js
res.setHeader('x-svrngs-append', JSON.stringify(act))
```

That replaces `store.append(group, act, name)`. `apps/` is not counted.

- What it keeps: Events' scene; any port app writing as whoever asks;
  server-side apps with no page script.
- What it narrows: today the server writes any kind as any member. After,
  it writes only its declared kind, and only for the person whose own
  device signed it.
- What it costs: about +25 to +45 lines, the door's response path and
  the worker's.

**The other option (F1-b): only code the person chose writes as them.**
The note's capability table puts Append behind pinned code
(WHAT_CAN_ACT_AS_YOU §2). Under that rule, a lent app on a port writes
only as its lender. Events' scene moves to the carried Events, which is
on the shelf already (`shelf.txt:13`). A guest on Sam's lent Events is
told: "Sam's Events writes only as Sam. Open Events from your group's
page to say you're coming."
- It costs no lines.
- It narrows what a port app can do, which `NORTH_STAR.md` §2 and the
  operator's 2026-10-20 "any application that can run on a port" weigh
  against.

## 7. The case the scenes found: another computer's copy

A member's home page lists every computer's copy of a carried app, sorted
by person and then by name (`node.rs:910-931`). It opens the first copy,
`once` (`home/question.js:43`). So Alex's home page can open Sam's
computer's copy of Money, and every act Alex writes there is signed by
Sam's computer with `on_behalf_of: "alex"`. That is shortfall 2's shape,
reached from svrngs's own page (c7452b2c).

Two changes close it:
1. **The home page opens this computer's own copy.** Each member's
   computer carries every app of its groups (`carried.rs:4-10`). So
   `once` keeps the copy whose key is this computer's, and `lent` lists
   this computer's own copy first. That is +2 to +6 lines. A phone keeps
   whichever copy its page lists first, since the phone signs its own
   acts and any computer carries them.
2. **Structurally, another computer cannot write as Alex at all.** If
   Alex's browser reaches Sam's copy anyway, a bookmark for example,
   Sam's door has no way to sign as Alex. It refuses the write, in words
   Alex can act on: "Open Money from your home page to write in it." It
   still answers reads.

## 8. Keys everywhere (capability-keys)

- `Runs.said` is keyed by kind and the person's first key
  (`synced.rs:150-155`), not by name.
- `Runs::of(kind, first)` (`synced.rs:163-167`) and `changed`
  (`synced.rs:156-157`) carry the first key, with the name for display.
- `whose` (`synced.rs:182-184`) becomes the signer's first key, through
  `Membership::person_of` (`membership.rs:307-311`).
- `AdmittedOp` gains the author's first key, skipped when absent. Apps
  then fold by key, and pages say "Ana (joined 3 Oct)" where two people
  share a name, as `remove` does (`Membership::one_named`,
  `membership.rs:352`).

## 9. Line versions, and what an older build does

- **`AUTHORSHIP_LINE_VERSION = 12`.** One version for three things:
  - a holder's line, which carries `grant`;
  - the keeper's `Signers`;
  - a `Remove` whose `through` names a holder's acts.

  `line_version` (`lib.rs:713-738`) gains three arms. `UNDERSTANDS`
  (`lib.rs:706`) moves to 12.
- **`grant: Option<Pass>`** is a new `SignedOp` field. It is declared
  last and skipped when absent, as `on_behalf_of` and `view` are
  (`lib.rs:623-643`). It is inside the id through `body_json`
  (`admit.rs:803-835`). So every line without one keeps its bytes, its id
  and its signature. The golden test's first case holds that.
- **A phone's own `Runs`** is an ordinary `Runs` at version 10, signed by
  a standing key. An older build counts it, as the phone's person's,
  since its signer is that person's key.
- **What an older build does with a holder's line:** it names it
  `NewerVersionLine`. It does not count the line, and it says its answer
  covers a subset. It never misreads the line as another person's.

## 10. The one question asked again

Under §4, a `Runs` signed by a key not its person's counts nowhere. On
today's record, that is every phone person's yes: a phone's yes always
went through some computer (`carried.rs:197`, signed by whichever
computer `reachGroup` dialled, `web/app.js:471-474`).

So at the release, each such person's version falls back. It goes to
their last yes signed by their own device, else to the app as it was
brought (`Runs::of`). Their group page then shows the line it already
shows for a changed app (`home/question.js:173-177`): "1 app changed. See
what's new", with "Open the new version". One tap signs the yes with the
phone's own key. It is asked once, and it blocks nothing.

The fallback runs the older version until they tap, the safe direction,
and the page says so. A computer person's yes was always written by
their own computer (`carried.rs:178-186`, `asker` = this host), so
nobody on a computer is asked.

## 11. F2: the chained pass, and an older computer

capability-pass-key's (H) moves here (the operator, 2026-10-23: "K now,
H with signed"; `demo/runs/capability/2026-10-23-pass-key/NEEDS_HUMAN.md`).
- The holder becomes a WebCrypto Ed25519 key made with
  `extractable: false`, imported by `open.html` from the handoff and
  never kept as bytes. The probe showed it signs in both engines
  (`probe.mjs`, `data/probe.txt`).
- It signs the phone's acts, since a line's signature may be async.
- Each worker run makes an endpoint key in memory, for iroh's synchronous
  handshake (`web/src/lib.rs:330`). The holder names it in a short
  chained pass.
- The door checks the chain beside `Pass::check` (`pass.rs:201-214`): the
  outer pass against whoever dials, and the inner pass with the outer's
  signer as its holder. The outer pass may name no more lends than the
  inner one. That is svrngs-core +15 to +25, and the door +10 to +20.

**What a door on an older svrngs does with a chained pass.** The page
knows each computer's version: `older`, `web/src/lib.rs:361`, already
says "runs an older svrngs" at `web/app.js:448` and `:516`.
- **Recommended: today's raw-bytes pass for an open that reaches an older
  door.** If any lend of an open is at an older door, the page signs that
  open's pass to a fresh holder as today, with its key's bytes. Then:
  - a phone still opens a friend's Films on a computer not yet updated
    (fence 2);
  - the hole stays open only for lends at older doors, and only until
    they update;
  - that open's writes wait in the worker's store until a computer on
    the new svrngs takes them (§3), since an older door has no route for
    a signed line;
  - the page says: "Sam's computer runs an older svrngs: what you write
    here reaches House when a computer on the new svrngs is on."
- **The other option: refuse.** The open is refused with the words above.
  It closes the copyable key everywhere at once. It breaks opening any
  lend whose computer has not updated, until it does (fence 2,
  "Nothing a person does today breaks").

**What H does not close** is as the pass-key package says. A chosen
app's page script can still use the key object while it is open, as §5
of the note already trusts.

## 12. What moves in the golden fold, and when

`crates/svrngs-core/tests/golden.rs` is red, by design, at exactly these
moves. Each is named in the commit that makes it, which updates
`acts_2026-10-24.fold.json`.

| Fixture line | Today | After | Row |
|---|---|---|---|
| 15: alex's computer's `Runs` for bo | bo's yes | counts nowhere, a `CarriedForAnother` gap; bo's own yes (16) still says the same version | capability-signed |
| 17: cy's `Runs` naming bytes no folder placed | runs as cy | counts nowhere (shortfall 8); Money runs as cy as brought | capability-signed |
| `said[].whose`, `changed.money.who` | by name | by first key, the name beside it | capability-keys |

These stay exactly as frozen, with no cutover in the fixture's group:
- every byte, id, signature and line version (the first test);
- line 10, alex's computer's record for bo;
- the corrections, the membership, the visit and its Stop, the cut, the
  hand-on, the relays and the layout.

A fixture with the keeper's `Signers` lands with capability-signed, beside
this one, never in place of it.

## 13. Each site it changes, with its lines

**capability-signed**

svrngs-core:
- `lib.rs`:
  - `RailAct::Signers` (after `:435`);
  - `SignedOp.grant` (after `:643`);
  - `AUTHORSHIP_LINE_VERSION`, `line_version` and `UNDERSTANDS`
    (`:696-738`).
- `record.rs:11-46`: `on_behalf_of` and `NotInGroup` cut.
- `command.rs:50-53`, `:175`, `:226-263`: `on_behalf_of` cut.
- A new `grant.rs`: `Grant`, `Capability`, `Verdict`, `may`.
- `admit.rs`:
  - `:452-560`: a holder resolved by its line's pass, each pass verified
    once;
  - `:83-160`: `CarriedForAnother`, its `Display` and its class;
  - `:341-348`: kept for old lines.
- `membership.rs`:
  - `:405-510`: the walk counts a holder's act under its grantor's
    standing; `Signers` is read as the keeper's;
  - `:518-549`: `covered` follows a cut key's holders, and a cutover's
    cover.
- `synced.rs:182-184`: `whose` becomes the signer's person; a `Runs`
  counts only from a standing key, and only when every file it names was
  placed at that path by a `Synced` act.
- `pass.rs:201-214`: the chained check beside `check`, and
  `Label::carried`.
- `sig.rs:225-245` and `admit.rs:803-835`: `grant` in `body_json`.
- The act's shape from `carried.rs:340-353`, as one function.

svrngs-node:
- `rail.rs:207-240`, `:300-328`: `stamp` becomes "own person or
  refused".
- `carried.rs`:
  - `:180-198`: `says`, with no `on_behalf_of`;
  - `:340-354`: a write from another person's computer refused in
    words; a holder's write takes a signed line.
- A new carry route on the door's members' and holders' protocols, over
  `ingest_all` (`svrngs-runtime/src/lib.rs:278`).
- `door.rs:1048-1110`: `holder_of` takes the chained pass.
- The response path's `x-svrngs-append` (F1).
- `node.rs:910-931`: this computer's copy first.
- The keeper's `Signers`, on its first round per group.

web, and the wasm side:
- `web/src/lib.rs:329-395`: the act's id and message for a line,
  `grant` included; the chained pass.
- `web/lend.js:521-535`: the worker signs and carries `POST
  /svrngs/acts`, the tip written first, unsent lines kept.
- `web/app.js:264-285`, `:471-483`: the holder kept per origin and
  renewed in place; the yes signed with the phone's key.
- `web/store.js:145-181`: the two tips; the holder as a WebCrypto key.
- `web/open.js:44`: the key imported, never kept as bytes.
- `home/question.js:43`: `once` prefers this computer's copy.

apps/, not counted:
- `apps/events/server.mjs:122-129` and `store.mjs:57`: the header (F1).
- `apps/expenses/canary.mjs:45-52` and `apps/events/canary.mjs:45-51`:
  each person written on their own device.
- Money's, Events', Grapevine's and House rules' folds read a holder's
  act as its person's, and `by`/`via` for old acts as today.

**capability-keys**
- `synced.rs:150-184`, `membership.rs:307-352` and `admit.rs:313-349`.
- Both pages' names: `home/question.js`, `web/app.js`.

**capability-release**
- DESIGN §1 and §3: authorship, the carried route's `on_behalf_of`
  paragraph (`docs/DESIGN.md:186-195`), and the version yes
  (`:211-223`).
- `RELEASES.md`: the one question a phone may ask again.

## 14. Lines (F3)

Measured: `python3 scripts/lean_check.py` at 289b9531 gives 17,789 of
17,790 for the repository and 3,149 of 3,150 for svrngs-core. The
estimates are from the sites above, against the economizing pass's
estimate (`demo/runs/economize/2026-10-23-capability/SURVEY.md` §6:
signed +110 to +200 / +55 to +110; keys +25 to +55 / +5 to +10).

| Part | Repository | svrngs-core |
|---|---:|---:|
| `Grant` and `may` | +30 to +45 | +30 to +45 |
| a holder resolved by its pass in `admit`; the walk; a cut covering holders | +23 to +40 | +23 to +40 |
| the keeper's cutover: act, version, cover, `CarriedForAnother` | +32 to +48 | +32 to +48 |
| `Runs` by own key and placed files only; `grant` on the line | +11 to +20 | +11 to +20 |
| `on_behalf_of` cut from `Append`, `Command`, `sign_lines` | −15 to −10 | −15 to −10 |
| the chained pass (H): core and door | +25 to +45 | +15 to +25 |
| the carry route; `stamp`; the carried route's refusal; own copy first; the keeper writing its cutover | +37 to +86 | 0 |
| F1, the response path signed by the person's own side | +25 to +45 | 0 |
| wasm: a line's id and message; the chained pass | +10 to +20 | 0 |
| the worker and group page signing, tips, unsent lines; H's key in `web/` | +70 to +130 | 0 |
| the pages' words | +5 to +10 | 0 |
| **capability-signed** | **+253 to +479** | **+96 to +168** |
| **capability-keys** (SURVEY §6) | **+25 to +55** | **+5 to +10** |
| **Both** | **+278 to +534** | **+101 to +178** |

Why it is past the pass's estimate:
- H moved here after the pass (2026-10-23): +25 to +45 / +15 to +25.
- The keeper's cutover is what keeps old acts' meaning when no byte can
  tell an old act from a forged one (§5): +32 to +48 / +32 to +48.
- F1 keeps lent apps writing (+25 to +45).
- The phone's tips and unsent lines keep a person's write from being
  signed twice or lost (+70 to +130 in `web/`, where the note sized none).

If the operator chooses:
- F1-b saves +25 to +45;
- refusing older doors under F2 saves about +10.

**The ask:** the top of each range, +535 repository and +180 svrngs-core
over today's caps, at most 18,325 and 3,330, for capability-signed and
capability-keys. Each row raises only what it lands, to one past where
it lands. A row past its range stops for the operator.

## 15. What this does not close, and what waits

- **A chosen app's own code** can still use its holder while its page is
  open, and send out what it reads (note §5). That trust is the person's
  choice, and the sandboxed carried page waits for its trigger.
- **Before the keeper's computer updates**, a group still counts a
  computer's word for another person, as today (§5).
- **Verification's cost per fold** grows by one Ed25519 verify per
  distinct pass, not per line. capability-signed measures it before and
  after (its row's check), under `demo/runs/authorship/<date>/`.
- **Sealing** waits for its trigger. `Grant`'s `Read` is the place it will
  narrow.
