# Instrument: Money opened on the tap that spent Kim's patience, and she said it never opened

- status: open, for xenia
- found in: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T03-11-14-983Z` on the Fedora box, xenia `2a10d4d`, the product `ccd48cb4`; `together` run 3, Kim
- kind: hard (the instrument's own): the account says the opposite of what the surface did. Not a svrngs fault; svrngs is not changed for it.

## What she said

"It has a Money entry, but tapping it did nothing I could see." "Yes, when Money wouldn't open. I couldn't find another way in, so I stopped." (`run-3/together/Kim.md`; filed as `said-almost-left-money.md`.)

## What the trace says

`run-3/together/trace.json`, Kim's entries:

1. `tap "Money"` on Home, scrolled down, at `http://svrngs.localhost:36575/`: the only thing on the screen saying "Money" was the shelf's plain line "Money is on the table." (its `shown`); the Money link was scrolled out of view. The stage's `FIND` fell through to `body *` and tapped the plain text. Nothing changed, and the answer was the screen again, with no word that what she tapped was not a thing to tap. No `inert` fault, as `found` was not a tap.
2. `scroll up`, `look`: the link "Money" in view.
3. `tap "Money"`: the url became Money's, `http://x2llv2b0….localhost:36575/`, screen `Money`. The same act left her patience at -0.55 (`left`), so `end: spent`, and the answer was "You've given up." with no screen. She never read that Money opened, and told that it had not.

## For xenia

- A tap that changes the screen and spends the last of a person's patience is told only "You've given up", so the account reports the tap as failed. Whether the screen it reached should be said first, or the account's tells judged against the trace's url, is xenia's to decide (`src/core/world.ts` `step`, `spent`).
- A tap that lands on plain text, as here, is answered as any tap is. A person sees a link is out of view; the answer says nothing of it (`src/stage.ts` `FIND`'s fall-through to `body *`).
