# demo/house-rules.md, run as written — 2026-10-17

Row canon-docs (c) (`ralph/next/canon/STATE.md`). `doc.py` reads every
code block of `demo/house-rules.md` and types it into the window its
paragraph names, then presses steps 5-10 on each person's page, each a
Chromium of their own (canon-run's `people.mjs`), then types "When you're
done". Every person is a program, on one host: it shows that the page's
lines and buttons do what it says, not how long a person takes
(AGENTS.md 7).

```
python3 demo/runs/canon/2026-10-17-doc/doc.py
```

- **The clean run** ran at c5d29109 with nothing dirty but this folder's
  `data/` (`data/transcript.txt`). Every block was typed and every step
  met its words. The windows closed with no process left, and
  `/tmp/house` was gone after the page's last line. It took 36.5 s from
  the first window.
- **The plant** is canon-run's `now` (`../2026-10-17-house/run.py`,
  `PLANTS`), in `apps/canon/house.js`: a past entry is counted only with
  the people in the house today. The page's `cargo build` built it in,
  and the tree was put back after (`git checkout apps/canon/house.js`).
  It ran red at step 10, as it should: once Bo left, Ana's page said "Your
  house has no rules yet." and listed the quiet hours as proposed again
  (`data/plant-now.out`, exit 1).

Two things the runs showed, neither a fault in House rules:

- **The order of "Everyone in your house has a say:"** was "you and Cy"
  in one run and "Cy and you" in the next. The page lists people in the
  order svrngs's `spans` gives them (`apps/canon/house.js:189`), which
  differs from one house to the next. The page names the two and not
  their order, and `doc.py` takes either.
- **`svrngs new` and `svrngs join` print no home page address.** Only the
  bare `svrngs` does (`crates/svrngs/src/cli.rs:1192`), and it can't run
  beside them. So step 4 prints it from `.svrngs/api`, which holds the
  same port (`crates/svrngs-node/src/home.rs:110-112`). A person who
  installs from the line in `start.md` never needs it: the install opens
  the home page.
