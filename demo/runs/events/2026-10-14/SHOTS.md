# The shots take — setup and predictions, 2026-10-14

Row events-doc (`ralph/next/events/STATE.md`) writes `demo/events.md` with
"every step as events-run did it, its screenshots". The clean take saved no
pictures, and it never took the way a person reaches the event: the driver
opened each event's address itself (`README.md` here, finding 3). This take
is the same story once more, every page saved as a picture, with the way
in added. Written before its data. One host, one more take: not a rate
(AGENTS.md 7); the clean take's numbers stay the run's.

## Setup

`SHOTS=1 W=<scratch> python3 demo/runs/events/2026-10-14/run.py`, data in
`data/shots/`: PREDICTIONS.md §1 as written, and

- **every page** a person is shown saved as `data/shots/<n>-<who>-<step>.png`,
  the whole page, `<n>` its order in the take;
- **2w, before Bo's step 2**: Bo's home page,
  `http://svrngs.localhost:<his computer's port>/`, read; its link
  "Open Sam’s events" (`crates/svrngs-node/src/home/home.js:90`) followed
  in the same tab; on the app's own list, "Dinner" followed;
- **3w, before Cy's step 3**: Cy's phone opens the group's page with no
  link, its address alone; presses "Sam’s events"
  (`web/app.js:338`); on the app's own list, "Dinner" followed.

What the way in reads is kept in `summary.json` under `way_in`, apart from
`views`, so `judge.py` reads the same story it read of the clean take.

## Predictions

- **S1.** Bo's home page lists "Open Sam’s events" under The House within
  30 s of opening it, its address the one Bo's `/v1/lends` gives.
- **S2.** Following it opens the app's list, Dinner under The House;
  following Dinner lands on The House's address of the event, the page
  step 2 opens, within 5 s.
- **S3.** Cy's phone, at the group's page with no link, lists Sam’s events;
  pressing it opens the app's list within 30 s; following Dinner lands on
  The House's address of the event.
- **S4.** `judge.py` on this take's summary: every bar as the clean take's,
  10b included (the 30 s dial is svrngs's, `NEEDS_HUMAN.md` §1).
- **S5.** No picture shows a link's token: no step here shows a link.
