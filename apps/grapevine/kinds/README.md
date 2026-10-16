# Write a kind of ask

A kind is what people ask for: a tool, a room, a question. You write it in
one file here, add it to the server's list of kinds, and grapevine carries
it between groups.

**What a kind cannot touch.** You don't write routing, scope, handles,
sealing, carrying or names. The app does all of that, the same for every
kind: who sees an ask, which groups it may go to, how replies find their
way back, what is sealed so only the two ends can read it, and when names
are shown. `kindsOf` in `grapevine.js` refuses a kind that sets anything
beyond the words below, and says which.

## The file

```js
// kinds/tool.js
export default {
  kind: 'tool',                                    // the word on the wire; unique
  ask: { fields: { what: 'text', when: 'text' } }, // what the asker fills in
  show: (f) => `${f.what}, ${f.when}`,             // how the ask reads on the page
  reply: { fields: { when: 'text', where: 'text' } }, // what someone offering fills in
  listing: { fields: { what: 'text' } },           // optional: what people list as theirs
  notice(ask, mine) {                              // optional: "you might have this"
    return mine.records.some((r) => r.kind === 'tool' && r.listing?.what === ask.fields.what);
  },
  hops: 1,                                         // optional: how many groups it may cross by default, 0 to 3
  name: 'withheld',                                // optional: the asker's name waits for both to say yes
};
```

Then add its file's name to the list in `kinds.mjs`, beside this folder.
That one line is all you change outside this folder: the server and the
canary both read that list.

**Fields** are one of four types: `'text'`, `'date'` (as `2026-10-17`),
`'number'` (a whole number) or `{choice: ['small', 'large']}`. A kind has 1
to 6 fields in each of `ask`, `reply` and `listing`, each named in short
lower case. The page draws every form from them, so you write no HTML. A
text holds up to 1,000 characters, and the form refuses a longer one with
the limit said. That keeps every sealed reply under the most a group's
record takes in one act.

**`show`** gets the ask's fields and returns the line people read. Keep it
short: it's what someone two groups away sees, with "carried by Jo" beside
it.

**`notice`** gets the ask (`{kind, fields}`) and `mine`, whose `records` are
what this person wrote in the groups they're in: their listings, and other
apps' records. Return `true` to tell them they might help. It runs on their
own computer, never sees a group they're not in, and nothing happens
because of it until they tap.

**Listings** are plain records in the person's own group, written by the
page from your `listing` fields. They are never asks, so they are never
carried.

## What the asker chose to share

An ask's fields are readable by everyone in every group it reaches. Ask
only for what someone needs to know to say "I can help". Everything after
that goes in sealed replies.
