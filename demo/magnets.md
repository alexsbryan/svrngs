# Magnets — a fridge for your group

Run-of-show and acceptance contract, written before any person plays, 17
November 2026. The product is Magnets V1 (svrn-docs
`rings/current/MAGNETS.md`): one sentence a group shares, a tray of word
magnets, and a tap that moves a word in or out. Its three proofs each get a
bar here. A people bar is a recorded run, and one run is not a rate.

Proof 1 runs now, against the stand-in for the database
(`apps/magnets/tests/standin.mjs`), because it is about the fridge, not the
store. Proof 2 waits for the database and phones' writes. Proof 3's count
runs now, and its fork waits for the database.

## Proof 1: five seconds to play

**Bar:** of 10 first-time players, each on a phone, at least 8 move a word
into the sentence within five seconds of being handed the open fridge, with
no instructions. The 10 come from at least two sessions, on different days,
so one room's mood is not the rate. Recruiting them is the operator's.

A first-time player has never seen Magnets and has never worked on svrngs.
Any comfort with phones will do. Help of any kind before their first move,
a word, a point or a demonstration, makes that player a miss.

**Before each session.** On the computer, start the stand-in on its WiFi
address, so a phone on the same WiFi reaches it:

```sh
node apps/magnets/tests/standin.mjs 8080 "$(ipconfig getifaddr en0)"
```

It prints the address to open. Plain http is enough: the face uses nothing
a browser keeps for secure pages. Its board lives in memory, so restart it
before each player: every player gets a fresh board, the sentence empty and
every word in the tray. The phone is the player's own if they're willing,
else one handed to them, joined to the same WiFi, with the fridge open and
drawn before it changes hands.

**What the player is told, and nothing else:** "Here, have a look."

**The measure.** A second phone films over the player's shoulder, the
screen and their hand in frame. The clock starts on the frame where the
player has the phone in hand with the board on it, and stops on the frame
where a word lands in the sentence. A player who has moved no word by five
seconds is a miss, whatever happens after. Record for each: the seconds,
the first word moved, whether they tried to drag a word before tapping one,
whether they took a word back out, and anything they said aloud, in their
words.

**Validate the instrument first,** in each session, before the first
player: the facilitator takes the phone and taps a word on a counted three,
and the film must read 3 s, give or take half a second; then takes it and
waits a counted seven before tapping, and the film must read a miss. Either
one read wrong, and the session's films aren't counted until the reading is
fixed.

**Then, alone with it.** After the five seconds, the player keeps the phone
for three more minutes, told nothing more. The facilitator steps away and
watches whether they try to send it to someone: a screenshot, the share
sheet, copying the address, or asking how to. At the end, one question:
"Is there anyone you'd want to change your sentence?" Their answer goes in
the record as they said it. This half has no bar. It is what MAGNETS.md
asks to see: whether a solo player sends the link because they want someone
else to change their sentence.

**Record** each session in `demo/runs/magnets/<date>/`: one line per player
(seconds, first word, drag tried, sent or tried to send, their answer), the
films or where they are kept, the stand-in's commit, and the instrument's
two validation reads. The bar is judged across sessions, never on one.

**If the bar fails,** the face changes now, while it is cheap (svrn-docs
`MAGNETS_BUILD.md`, U0), and the count of 10 starts again on the changed
face, since two faces' players are not one rate. A miss
that starts with a drag says the face needs drag as well as tap; a miss
that starts with reading says the first look holds too many words.

The cheap read before people is xenia's, a model playing a stranger
through the same page on a phone: `demo/runs/magnets/2026-11-17-xenia/`. It
points to where people will stumble, and it is never a people result. Its
first set moved the face before anyone played it: the tray keeps each
word's place, the pack fits one phone screen, and the line saying the
sentence is shared sits under it. It also left two things for this run to
watch: a cautious player who read that the sentence is shared and touched
nothing, and players who looked for a way to send it and found none.

## Proof 2: the laptop closes

Runs once the group's database syncs between computers and takes a phone's
own signed writes (`MAGNETS_BUILD.md`, units B2 to B4).

**Who and what:** the founder's laptop (A) and a second computer of the
group (C), both keeping Magnets, version accepted on each; a friend's phone
(F) in the group; and a third person (T) who opens Magnets through C, on C
or on a phone of their own.

1. On A, the founder brings Magnets from the shelf and makes a sentence: "I
   can explain the goose."
2. The founder sends the group's ordinary app link in the chat: "Improve
   this." On F, the friend opens it, takes **goose** out and puts
   **invoice** in. **Bar:** A's open page shows the friend's move within 5
   seconds, with no reload.
3. On C, open Magnets. **Bar:** it shows the friend's sentence. This is the
   second computer having synced the board.
4. Close A's lid.
5. T opens Magnets through C. **Bar:** T sees the friend's sentence, and
   T's move (put **again** at the end) is taken, with A asleep.
6. Open A's lid. **Bar:** within 60 seconds of A's svrngs answering again,
   A's page shows T's move; reloaded, it is still there. Sixty seconds is
   one sync round of a group's computers (`apps/expenses/canary.mjs`,
   `WAIT_MS`'s comment: the interval is 60 s).

**Checked apart from the screens:** no move adds a line to the group's
record (APP_STATE invariant 1); F's move is signed by F's own device, not
by the computer that served it (invariant 2; `MAGNETS_BUILD.md`, S2);
while A is asleep, nothing T sees says the fridge is fresh or empty.

**Record** in `demo/runs/magnets/<date>-laptop/`: each step's screen, the
seconds at steps 2 and 6, the record's line count before and after, each
change set's signer, and the svrngs commit. The scripted version of these
steps is the scene `crates/svrngs/tests/scenes/magnets.rs`
(`MAGNETS_BUILD.md`, B6); a person running it once with their own devices
is the people half.

## Proof 3: show the source

**The count.** `node --test apps/magnets/tests/lines.test.mjs` prints every
shipped file of `apps/magnets/` with its lines: its HTML, CSS, JavaScript,
SQL and word pack, `tests/` apart, since tests never ship. **Bar:** at most
1,000 in all. On 17 November it read 327: `app.js` 190, `board.js` 52,
`index.html` 29, `pack.json` 21, `schema.sql` 10, `style.css` 25. The same
law runs in the gate's app laws, with its plant.

**The fork.** Runs once the database exists.

1. Copy `apps/magnets/` to a folder of your own, and in `pack.json` change
   the prompt and at least five words. Keep the IDs you didn't change; give
   a new word a new ID.
2. Bring the copy to a second group, B, while Magnets stays in the first,
   A.
3. **Bar:** B's fridge opens fresh, the empty sentence under its own prompt
   and its own words in the tray, with no first-run step. A's sentence is
   unchanged.
4. Move a word in each group. **Bar:** neither group's move appears in the
   other's fridge, on any computer of either group; each group's board is
   a database of its own (APP_STATE §4, "Where it lives").

**Record** in `demo/runs/magnets/<date>-fork/`: the count's output, the
diff of the pack, both groups' boards before and after each move, and the
svrngs commit.
