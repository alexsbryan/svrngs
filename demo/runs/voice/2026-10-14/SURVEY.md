# v1ui-voice (a) — every place a person reads svrngs's words, held to the house voice

Survey at `5bb4b6a`, 2026-10-14. Nothing to product here. Each place the row
names is read against svrn-docs `VOICE.md`: its task register (second person,
active, present, "You lend nothing yet" and not "This computer lends
nothing") and its checklist. The numbers below are the checklist's: 1 lead
with what it does, 3 second person, active, present, 5 name the trade-off,
6 cut internal vocabulary, 7 the privacy line plainly, 9 prose over
scaffolding, 10 explain, don't sell. Fence 3 is held beside them: no word a
reader must look up (key, member, record, act, hash, origin, port, relay),
and nothing begun on a page that needs the terminal to finish. Both come
from this queue's charter, fences 3 and 9.

What the door, lend, lend-ask, shelf and findings rows wrote is already in
the voice: fence 9 held them as they landed. That covers the door's cells,
the index's rows, "Already running here", "Bring things to the table" and
"… runs an older svrngs". Those are read here too, and pass unless a line
below says otherwise.

How a line was judged: `this computer` as the subject of something the
person does or has (they lend, they are in a group, their groups) fails 3.
`your computer` or `this computer` as the subject of what the machine does
(answers, is on, cannot read) passes, since the machine is the actor there.

## The home page (`crates/svrngs-node/src/home/`, and `home.rs`'s refusals the page shows)

| where | the words now | fails |
|---|---|---|
| index.html:15 | "This page is yours, on this computer alone. On it you see your groups and who is in each, open what …, and choose how many people at once may use what this computer lends them." | 1: one 90-word list of everything on the page, where one plain sentence should say what the page is for. 9: a page's contents written out as a sentence. 7: "yours, on this computer alone" never says who cannot see it. |
| index.html:16 | "A link works for a day, while this computer is on." | passes. "this computer" is the machine being on. |
| index.html:17 | "A program on this computer, such as one you chat with a friend's model in, reaches what is lent to you at the address beside it: its Copy button copies it …" | 1: the program's job is buried in a clause ("one you chat with a friend's model in"). The button is named "its Copy button", which is not what the page labels it. |
| index.html:26 | "Leaving a group is not possible yet: nothing in svrngs does it." | 3: passive ("is not possible"), with svrngs as the actor in place of you. |
| home.js:59 | "This computer could not read who is in {group}." | passes 3 (the machine reads). But it says nothing of now. Kept. |
| home.js:67 | "Where {group}'s folder is cannot be read on this computer." | 3: passive. |
| home.js:71 | "{group}'s folder is on this computer at {dir}" | passes. |
| home.js:107 | "{p}'s computer has not heard of this computer yet, …" | 3: the reader's computer is "this computer", not "yours". |
| home.js:109 | "Nobody lends anything here yet." | passes. |
| home.js:113 | "What this group lends cannot be read on this computer: …" | 3: passive. |
| home.js:165 | heading "What this computer lends" | 3: you lend, not the computer (fence 9's own example). |
| home.js:171 | "What this computer lends cannot be read on it: …" | 3: passive, and "this computer" lends. |
| home.js:173 | "This computer lends nothing." | 3: fence 9's own counter-example, word for word. |
| home.js:208 | "This computer is in no group yet. Make one below, or join one by the line the page of the link you were sent gives." | 3: you are in no group, not the computer. 1: "the line the page of the link you were sent gives" is three nested clauses. |
| home.js:237 | "Your groups cannot be read on this computer: …" | 3: passive. |
| question.js:31 | "What apps use of {lend} cannot be asked now." | 3: passive, with no actor. It is the lender's computer that did not say. |
| home.rs:233 | "this computer lends nothing by that name" | 3: you lend. |
| home.rs:239 | "that cannot be changed on this computer" | passes. A refusal the machine makes. |
| home.rs:312 | "This computer could not read what the page sent: …" | passes. |
| home.rs:379 | "this computer is not in that group" | 3: you are not in it. |
| home.rs:385 | "no link can be made on this computer: …" | 3: passive. |
| home.rs:436 | "not this group's folder" | 1: a fragment that does not say what was refused. |

## The group's page (`web/index.html`, `web/app.js`)

| where | the words now | fails |
|---|---|---|
| index.html:43,45 | "Open the link to join: … What the group shares stays on its people's own computers." | passes. 7 is plain. |
| app.js:195 | "If that's you, add this phone from your computer: `svrngs link {group} --device` there makes a link for it. Else join under another name." | Fence 3: a phone's page sends its person to a terminal, although their home page has made that link with one button since start-home ("Make a link to add your phone or another computer of yours", home.js:79). |
| app.js:352 | "Your group's computer said no." | Fence 2, "a refusal names its person": it names no one, though the page knows whose computer it reached (`via`, or the link's issuer). 1: "said no" to what is not said. |
| app.js:407 | "{p}'s computer has not heard of your group's computer yet, …" | 6/1: "your group's computer" is no computer a person can point to. It is the one the phone reached. |
| app.js:285 | "Your answer could not be sent." | passes. |
| app.js:500 | "This browser could not open the page." | passes. |

## The command (`crates/svrngs/src/cli.rs`, `share.rs`, `relay.rs`)

| where | the words now | fails |
|---|---|---|
| cli.rs:118 (USAGE) | "Run for every group this computer is in. Leave it running." | 3: you are in the groups. |
| cli.rs:137, 149, 162 (USAGE) | `--relay` (102 columns), `share`'s synopsis (163), `--limit` (119) | 9: three lines run past 100 columns in an 80-column usage. A layout fault, not a voice one, so it is a follow-up and not rewritten here. |
| cli.rs:559 | "this computer is in no group called {name}" | 3: you are in no group. |
| cli.rs:538 | "this computer cannot be reached from here yet (RUST_LOG=debug says why)" | passes. The machine is the actor. |
| cli.rs:1149 | "This computer is in no group yet: make one with `svrngs new <name>`." | 3: as home.js:208. And the home page makes one with no command (start-group), which this does not say. |
| cli.rs:1203,1205 | "Lent {name} (port {port}) to everyone in your groups, …" | 3: no subject, past tense. It is what you lend now: "You lend …". |
| cli.rs:1269 | "Stopped lending {name}." | 3: as above, "You no longer lend {name}." |
| cli.rs:1818 | "svrngs: this computer has not reached {relay hosts} in {n} s, where its group reaches it; it goes on, and keeps trying." | 6 and fence 2: it prints a relay's host to someone who runs no relay (DESIGN §1: "never … relay"; "a refusal names its person, never a relay's host"). 5: it does not say what waiting costs (nobody outside this computer reaches it until then). |
| share.rs:28 | "`{port}` is not a port on this computer: a number from 1 to 65535" | passes. `share` takes a port by name; its reader typed one. |
| relay.rs:194 | "Relaying at {at} for {whom}. Leave this running." | passes. Its reader runs a relay (DESIGN §1). |

## `web/install.sh`

| where | the words now | fails |
|---|---|---|
| :78 | "Checked: {tarball} matches SHA256SUMS." | 6: SHA256SUMS is a file's name a newcomer must look up. 1: it does not say what was checked: that what came is what the page lists. |
| :110 | "{bin} is not on your PATH, so to run svrngs yourself type {bin}/svrngs; nothing else needs it." | 6: PATH. 1: it leads with the fault, not with what to type. |
| :156 | "{unit} runs now, and whenever you log in ({kept})." | passes. |
| :170 | "Open it in your browser: {opener} did not." | passes. |
| refusals (:51, :58, :66-:76, :88, :106, :119, :164) | e.g. "could not get …, so nothing was installed." | pass: each says what was refused and what that leaves. |

## The door's refusals (`crates/svrngs-node/src/door.rs`, `origin.rs`)

A person reads these in a tab that opened something lent, or a program
reads them. Their reader is the asker, so "this computer" there is
someone else's.

| where | the words now | fails |
|---|---|---|
| door.rs:468 | "what a computer lends is asked only from another computer of its group" | 3: passive. |
| door.rs:476, :1035 | "this computer lends nothing to you" | 3: "this computer" is the lender's, not the reader's. |
| door.rs:499 | "what this computer lends cannot be read on it" | 3: passive. |
| door.rs:843 | "a lend is reached only through its group" | 6: "a lend" as a noun is svrngs's own word. 3: passive. |
| door.rs:998 | "a lend is reached only from another computer of its group" | 6, 3: as :843. |
| door.rs:1021 | "a path that can step outside a lend is refused" | 6, 3: passive, "a lend". |
| door.rs:1093 | "who is using this lend cannot be counted on the computer that lends it" | 6: "this lend". |
| door.rs:1310 | "who is asking cannot be told to this lend" | 6, 3. |
| door.rs:1372 | "the request is not one a lend can be asked" | 6: "request", "a lend". |
| door.rs:269, :901, :1027, :1081, :1333, :1385 | e.g. "{name} is not open to you here; open it again from your group's page" | pass. |
| origin.rs:149 | "a page reaches another lend only by fetch, which says where it is from" | Its reader is an app's developer (it refuses a page's navigation), who reads DESIGN §1's developer words. Passes for them. |
| origin.rs:222 | "this computer could not say which app asks" | passes. |
| origin.rs:235 | "this is not an address a lend can be asked" | 6: "a lend". |
| origin.rs:139, :247, :260 | e.g. "the computer that lends this did not answer; try again later" | pass. |

## The documents (`demo/start.md`, `demo/events.md`, README.md "Run it")

| where | the words now | fails |
|---|---|---|
| start.md:3-9 | the blockquote, "Written 2026-10-13 for anyone getting started (the operator: …) … the seat's proposal …" | 6: "the seat", "the operator", "DESIGN §8", which the newcomer reads first. 1: the doc's first words are its provenance, not what it does. |
| start.md:69-70 | quotes "This computer is in no group yet. Make one below, …" | follows home.js:208. |
| start.md:87-113 (step 5) | "**Done in a terminal** says what is not done on the page yet. Lending is one of them. In a terminal, type …" | Not true at HEAD: since v1ui-lend (b) the home page lends what runs here ("Already running here", home.js:139-157), and "Done in a terminal" lists only removing someone (index.html:24). Fence 3: nothing begun on a page needs the terminal. The run's own words stay the run's. The step says the page's way first and the run's line second. |
| start.md:106 | quotes "Lent llama-server (port 18089) …" | follows cli.rs:1205. |
| start.md:112-113 | "it lists it under **What this computer lends**" | follows home.js:165. |
| start.md:168-170 | "… is not on your PATH, …" | follows install.sh:110. |
| start.md:186-188 | "Sam types a number beside llama-server under **What this computer lends**" | follows home.js:165. |
| start.md:226-228 | the bars, "Gate (the seat's proposal, the operator's to set)" | 6: as :3-9. These are for whoever runs the demo, not a newcomer, and the section's title already says so. Passes. |
| events.md:3-17 | the blockquote, as start.md:3-9 | 6, 1: as start.md:3-9. |
| events.md:40-63 | "What this is": four bold-led bullets, then a bold-led paragraph | 9: scaffolding. Each bullet is a sentence that reads as prose. |
| events.md:89 | quotes "Lent events (port 65216) …" | follows cli.rs:1205. |
| README.md:17-26 ("Run it") | "Start with `demo/start.md`: a friend lends their model and the group uses it, every line to type and every button to press, … as its run did them on one computer (`demo/runs/…`). … E0 — the first experiment, a Discord bot … — ran before the fork and is at tag `e0-bot`." | 1: passes, it says what start.md does. 6/8: the run's path and E0's history are depth, not "run it". E0 is the project's past, not a step a newcomer takes. |

## Not in the row's list, found on the way (follow-ups, not rewritten here)

- `crates/svrngs-node/src/share.rs:97-101`, `room.rs:97`, `folder.rs:81-93`,
  `invite.rs:52` and `redeem.rs:314-324` hold refusals the command prints.
  Some say "stand in the group" and "the record", which are 6. The row
  names `cli.rs`, `share.rs` and `relay.rs` of the command, so these wait
  for a row of their own.
- cli.rs:137, 149, 162: three usage lines past 100 columns (above).
