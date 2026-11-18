# Money

Who owes whom in a group, and for what. Nobody owes anything until they say
yes themselves.

- **Shape:** carried. A page in each person's browser; its entries are the
  group's.
- **Run it:** bring it from your home page (**Add an app**, Money). It's
  already in your group's folder after that, at `apps/Money/`.
- **How it's made:** `index.html` and `index.js` are the page. `expenses.js`
  folds the group's entries into balances, and holds the one validator.
- **Change it:** in a group of your own, edit `apps/Money/index.js`
  ([open one up](../../docs/guide/examples.md#2-open-one-up)).

Tests: `expenses.test.mjs` and `laws.test.mjs` (the sums and their laws),
`canary.mjs` (two computers' copies fold the same). `scripts/check.sh` runs
them. `nouns.txt` keeps Money's words out of svrngs's own code.
