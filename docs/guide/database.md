# Keep your app's data in the group's database

Put a `schema.sql` beside your app's `index.html`, and your group's
computers become its database. Each computer whose person said yes to your
app keeps a copy. A write on one computer reaches the others within
seconds when they can reach each other, or when they next meet. Your page
reads with SQL and writes by naming rows, at its own address, with plain
`fetch`. There's no server to run and no login.

It's for data people change together, like a board, votes or a shopping
list. For a record of who said what, use the group's entries
([`svrngs/acts`](build.md#what-svrngsacts-gives-you)). For a program that
runs on one computer, lend it instead: [Build an
app](build.md#or-lend-a-server-on-a-port) draws the two shapes side by side.

The database comes with the release after 1.13.3. To tell whether your
svrngs has it, run `svrngs check` on a folder with a `schema.sql`: one that
has it names the tables after "Nothing in … that apps can't load."

On this page: [start from Magnets](#start-from-magnets) ·
[write schema.sql](#write-schemasql) · [read](#read) ·
[write](#write) · [whose row it is](#whose-row-it-is) ·
[change your schema](#change-your-schema) ·
[send the link](#send-the-link) · [test your app](#test-your-app) ·
[what everyone sees](#what-everyone-sees).
The shapes that merge well are on [their own page](database-patterns.md).

## Start from Magnets

Magnets is a fridge of word magnets your group shares, and it's on the
shelf. Make a group of your own with `svrngs new <name>`, bring Magnets
from its card on your home page, and open its folder,
`~/svrngs/<name>/apps/Magnets/`. Its whole database is one table in
`schema.sql`, and `board.js` reads and writes it. Change a word in
`pack.json`, save, and reload. Copy the folder into another group's
`apps/`, and it starts there with a fridge of its own: each group's
database is its own, and neither sees the other's moves.

## Write schema.sql

```sql
CREATE TABLE films (
  id TEXT NOT NULL PRIMARY KEY,
  title TEXT
);
CREATE TABLE votes (
  person_key TEXT NOT NULL,
  film TEXT NOT NULL,
  yes INTEGER,
  PRIMARY KEY (person_key, film)
) STRICT;
```

A schema only grows: a later version may add to it, never change what's
there ([see how](#change-your-schema)). Check it before anyone opens the
app:

```
$ svrngs check ~/svrngs/Test/apps/Movie-Votes
Nothing in /Users/you/svrngs/Test/apps/Movie-Votes that apps can't load. Its database's tables: films, any member's to change. votes, each row its person's own.
```

Copies on computers that were apart have to agree once they meet, so a
table has limits a database on one computer doesn't. `svrngs check` names
each one it finds, with the fix, and exits non-zero:

- Every table has a primary key, and no part of it can be NULL.
  `INTEGER PRIMARY KEY` needs `NOT NULL` too.
- No `UNIQUE` besides the key, and no `REFERENCES`: two computers apart
  can't keep a value unique between them, and a row can arrive before the
  row it points to.
- A `NOT NULL` column needs a `DEFAULT`.
- No `AUTOINCREMENT`: two computers would give two rows the same number.
  Key a row by what makes it the same thing, in lower case, like a film's
  `lower(title) || ' (' || year || ')'`, so two people adding it make one
  row. Otherwise use `crypto.randomUUID()` from your page.
- No `COLLATE NOCASE` on a key. Keep the text in the case you compare it in.
- Only tables and indexes: no triggers, views or virtual tables. Do that
  work in your page.

A `CHECK` holds for each computer's own writes, but not for what a merge
makes. With `CHECK (lo <= hi)`, if Ana sets `lo` to 8 and Ben, apart, sets
`hi` to 5, both copies end at 8 and 5, and the next write that leaves the
row like that is refused. Write columns that have to agree in one write,
and have your page show a row that broke the rule so people can fix it.

## Read

```js
const SQL = 'svrngs/sql';
const read = (sql, params = []) => fetch(SQL, {
  method: 'POST',
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify({ sql, params }),
}).then((r) => r.json());

const { rows, more, syncing } = await read(
  'SELECT film, SUM(yes) AS n FROM votes GROUP BY film ORDER BY n DESC');
```

A read is one statement that only reads your app's own tables. It gives at
most 1,000 rows, with `more: true` when there were more, and stops after
250 ms. Name the columns you want, rather than `SELECT *`.

`syncing: true` means this computer's copy hasn't synced with another
computer of the group yet. Say "still syncing" then, never an empty board.

To keep a view current, subscribe. Each event is a read's answer, now and
again each time the rows change, `syncing: true` beside its rows while this
copy hasn't synced yet:

```js
const live = new EventSource(`${SQL}?q=${encodeURIComponent('SELECT film, SUM(yes) AS n FROM votes GROUP BY film')}`);
live.onmessage = (e) => {
  const { rows, syncing } = JSON.parse(e.data);
  if (syncing) say('Still syncing with your group…');
  else draw(rows);
};
live.onerror = () => say('Your group’s computers can’t be reached right now.');
```

A subscription takes its query alone, `q`, with no `params`. To read rows by
a value, such as your own votes, read with `POST` again as each change
arrives. An `EventSource` can't read a refusal's words; to show them, read
the stream with `fetch`, as Magnets does in `index.js`. Every change sends
the whole answer again, so subscribe to a small query.

## Write

A write names its rows by key, as a list, kept whole or not at all:

```js
await fetch(SQL, {
  method: 'POST',
  headers: { 'content-type': 'application/json' },
  body: JSON.stringify({ write: [
    { insert: 'votes', row: { person_key: you_key, film: 'f12', yes: 1 } },
    { update: 'films', key: { id: 'f12' }, set: { title: 'Paddington 2' } },
    { delete: 'films', key: { id: 'f9' } },
  ] }),
});
```

It answers `{count}`, the rows it changed. A value is null, a number, a
string, or `{base64: "..."}` for a BLOB. An `insert` names every key
column. On a key that has a row it sets only the columns it names; on a
new row the rest are NULL, so a `DEFAULT` never applies and a `NOT NULL`
column has to be named.

SQL that writes is refused, with words that say this form. A write like
`n = n + 1` or `WHERE claimed IS NULL` would hold only on the computer that
ran it: two made apart count as one, or both claim. Read first, then write
the rows you mean by key.

**Which value wins.** Each column merges on its own. When two computers
change one apart, the value changed more times wins, then the larger
value: a number beats text, text beats a BLOB, and anything beats NULL. It
isn't the later one. Five taps on one phone made apart beat one later
move on another. Where the last move should win, give each move a row of
its own ([see how](database-patterns.md#where-the-last-move-should-win)).
A delete beats an edit it never saw, so archive with a column rather than
delete a row people still edit.

A person's writes are limited, about 30 a minute for each app, with an
hour's worth at once. Past that, a write is answered 429, `refused:
"limit"`, with a `Retry-After` in seconds. A list of 300 rows is one write.
One write is at most 65 KiB of JSON, several hundred short rows; past
that it's answered 413, `refused: "too-large"`, nothing of it kept. Send
a longer list in parts.

Today a write is taken from your own computer, and a phone reads. An app
that reads your friends' lends (`reads.txt`) doesn't write the group's
database.

## Whose row it is

A table whose key begins with `person_key` keeps each row its person's.
Only that person's devices change it, on every computer. Write the
`you_key` that `GET svrngs/acts` gives your page:

```js
const { you_key, spans } = await (await fetch('svrngs/acts')).json();
const names = new Map((spans ?? []).map((s) => [s.person_key, s.person]));
```

Name a row's person from `spans`, which pairs each name with its
`person_key`; two people can share a name, and `joined` tells them apart.
When a row names a key your page doesn't know, someone may have joined
since it read `svrngs/acts`, so read it again. `spans` is null for a moment
while svrngs settles who's in; read it again then too. A write to someone
else's row is refused, 403 `not-yours`, with words that say what to
write instead. Once the group's keeper removes a person, any member can delete
their rows. An owned table is `STRICT`, with `person_key TEXT NOT NULL`
first in its key, and `svrngs check` refuses one that isn't.

Every other table is shared: any member's page may change any row of it.
Votes, claims, free nights and each person's part of a document go in
owned tables.

## Change your schema

A schema only grows. A new version of `schema.sql` may add tables, indexes,
and columns that can be NULL, with no `DEFAULT`. It never drops or
renames a column, changes a type or a key, or renames the app's folder,
since the database is named by the folder. Each computer keeps what it
holds, and a page on the earlier version still writes its rows.

Before you copy a new version over the one your group uses, name both:

```
$ svrngs check ~/svrngs/Test/apps/Movie-Votes ~/svrngs/House/apps/Movie-Votes
```

It refuses a version that doesn't only add to the one it follows.

Today a database keeps the schema it was first made from: a version that
adds tables or columns is checked, but svrngs doesn't take the additions
into a database that already exists yet. Try a schema that changes in a
group of your own, under a new folder name, whose database starts fresh.

## Send the link

A page on a computer is served at its own local address, which a friend
can't open. `GET svrngs/acts` gives your page `link`, the address a member
opens your app by from the group's page, such as
`https://group.svrnme.sh/#open=<your app's label>`. Put it behind a
**Copy** button, so people can send it in the chat. It's null while the
group keeps no page for links.

## Test your app

A group of your own is a group of one, so it can't show what happens
between two people. Make a second test computer on the same machine, with
a home and a data folder of its own:

```
$ HOME=~/test/ana SVRNGS_DIR=~/test/ana/.svrngs svrngs new Test --as Ana
$ HOME=~/test/bo SVRNGS_DIR=~/test/bo/.svrngs svrngs join '<the link Ana's home page gives>' --as Bo
```

Each runs on as svrngs does, with a home page of its own. Put your app in
Ana's group folder, `~/test/ana/svrngs/Test/apps/` or wherever **Folder
location** on her home page says, open it on both, and act as each. To see
owned rows hold, have Ana write her row, then have Bo's page change or
delete it: each is refused, 403 `not-yours`. An update of a key with no
row changes nothing and answers `{count: 0}`, so try it on a row that's
there. A test group has no group page for links, so its `link` is null.

## What everyone sees

Everything in the database is the group's: every member's computer keeps
every row. An invitation is a row everyone reads, so a private ask goes in
the chat. A list of your own stays in the browser's storage for now, and
your page should say so.

A time your page writes comes from that device's clock, so treat it as a
claim, not a fact. A date people choose is data.

There's no undo in the database. To let people take a change back, keep
what they replaced and write it back by key.
