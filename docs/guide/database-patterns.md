# Shapes that merge well

Your group's computers keep copies of your app's database and merge them
when they meet ([how it works](database.md)). A few table shapes make
those merges come out the way people expect. Each one here answers a
thing that goes wrong otherwise.

## Counts are a row per person, then SUM

```sql
CREATE TABLE votes (
  person_key TEXT NOT NULL,
  film TEXT NOT NULL,
  yes INTEGER,
  PRIMARY KEY (person_key, film)
) STRICT;
```

Read the count with `SELECT film, SUM(yes) FROM votes GROUP BY film`, and
unvote by writing `yes: 0`. A shared counter raised by `n + 1` loses a vote
whenever two are made apart.

## A claim is a row per claimer

When Priya and Sam both say "I'll bring ice" while apart, both claims should
show. Make it Priya's own row and Sam's own row, and show both. A shared
`who` column keeps one of them without a word, and no computer can tell
who was first.

## Archive, don't delete

A delete beats an edit it never saw, even a later one, and a deleted row
keeps its key for good. So archive a card with a column, and put a
magnet back in the tray with a NULL position. For something that changes
all night, like "starting in five", update one row in place, with the
time it lasts `until`.

## Order by a key your page computes

To order a list, the page computes a text key between the two neighbors a
row goes between, then orders by that key and then by id. Appending is the
largest key plus one, as Magnets does in `board.js`.

## Values that change together go in one column

Columns merge one at a time. If a card's move sets `col` and `pos`, two
moves made apart can leave Ana's column with Ben's position. Set one
`place` column, like `'doing|' || key`.

## What a copy can't promise

Uniqueness besides the key, a row a reference points to, a rule across two
columns, or a limit like "ten seats" or "one borrower at a time" can't hold
across computers that are apart. Let the page show the break, and let
people fix it. A limit that must hold for everyone goes in the group's
record ([`svrngs/acts`](build.md#what-svrngsacts-gives-you)), whose entries
every computer orders alike.

## Large state

The subscription resends its whole answer on each change, so subscribe to
something small, a key and a version, and read only the rows that changed.
For a document, give each person's browser a part of its own,
`parts(person_key, slot, state BLOB)` with a random `slot` per browser,
written after a pause in typing, not on each key. Fold your own old slots
into the current one, then delete them. Who wrote a paragraph inside a part
is the page's word: only the part's owner is checked.

## Where the last move should win

The value changed most often wins a merge, so five taps on one phone made
apart beat one later move, which players read as the app undoing them.
Where that matters, write a row per move, `moves(person_key, at, ...)`, and
read the newest by `at`, then by key. `at` is the page's clock, a claim.
It costs a row per move, so your page deletes its own old ones.

## A thing between two people is two owned rows

A loan has two sides, and an owned row has one writer. Keep the lender's
`lent(person_key, tool, to_key, since)` and the borrower's
`have(person_key, owner_key, tool, back)`, and show when they disagree:
"Sam says he returned it."

## Undo in the page

There's no undo in the database. Keep what a change replaced, and write it
back by key. A document kept as one value per part keeps its own
snapshots: writing an old row back doesn't take a merged delete out of
everyone else's part.
