# Jellyfin lent from the home page beside its helper — setup and predictions, 2026-10-15

Row v1ui-polish (d) (`ralph/next/v1ui/STATE.md`): "a take of v1ui-run's
driver … with Priya lending Jellyfin from her home page beside its helper,
prediction 11 passing — zero accounts, no sign-in shown to any of the five
— beside the planted take, which is the page with no helper". Written
before this run's driver and before any data; `README.md` here will say
what it gave against each line below. A program is not a stranger: **the
people bar never-ran**, the operator's to run with real people
(`demo/README.md`). One host, one take of each: not a rate (AGENTS.md 7).

## 1. Setup

v1ui-run's (`demo/runs/v1ui/2026-10-15/PREDICTIONS.md` §1), its driver
forked here (`run.py`, `people.mjs`, `judge.py`), the original kept byte for
byte beside its data. Two changes, and nothing else:

- **Priya lends the library from her home page**, always: "Look for
  programs running on this computer", then "Lend jellyfin to everyone in
  your groups", as v1ui-run's plant did. Before she presses Lend, the
  row's words are kept (`people.mjs`, `lend`, `before`); after, Priya's
  declaration of the lend is read from her data dir (`lends/jellyfin.json`).
  The middleware (`apps/jellyfin/mw`, debug, on 8097, given its key from
  Jellyfin's API Keys) is started before, as `apps/jellyfin/README.md` says.
  svrngs starts it never: the driver does, as a lender would.
- **The plant, `nohelper`**: the same, with no middleware started. Data in
  `data/plants/nohelper/`; the clean take's in `data/`.

`W=<scratch> FILMS=<dir> python3 demo/runs/v1ui/2026-10-15-helper/run.py`,
and with `PLANT=nohelper`. `judge.py` writes each take's `verdicts.json`.

## 2. Predictions

1–14 and 16 are v1ui-run's, word for word (its PREDICTIONS.md §2), with
the library lent from Priya's page in place of her terminal. 15 and 17
are this run's:

15. **The plant `nohelper`.** Each newcomer's library step stops at
    Jellyfin's sign-in, with nothing of theirs to type: 5, 8 and 11 fail,
    10 with them; Money's predictions (4, 13) hold.
17. **The page offers the helper.** In the clean take, the row Priya
    lends from says, before she presses Lend, "Its helper is running on
    this computer, so everyone is let in as themselves, with no account to
    make.", and not "make each person one"; her declaration names
    `"sign_in": 8097`. In the planted take the row says "It keeps its own
    accounts, so make each person one in it" and "Or start its helper
    first, and nobody needs an account: here is how", and the declaration
    names no sign-in.

The bar this row names is **11 passing in the clean take**: no password
box or Jellyfin sign-in shown to any of the five, and Jellyfin's accounts
the five the middleware made and Priya's.
