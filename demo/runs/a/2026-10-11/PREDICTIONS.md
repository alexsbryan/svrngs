# Demo A's dry run with nothing typed — predictions, written before it ran

v1-onboarding (e) (`ralph/next/v1/STATE.md`): Demo A's steps through
Jellyfin's middleware (`apps/jellyfin/mw`), each of Alex, Bo and Cy on a
computer of their own, on this one host; Sam keeps a library, Family,
back with `--keep-back Family` (`e877024`). Instrument:
`apps/jellyfin/check/check.sh` `up`, `mw`, `trio` with `MW=1`, `down`.

1. `mw` started without `--keep-back` (the red run): mw.py's bar
   "Family kept back from each" fails, Alex's and Bo's libraries listing
   both Movies and Family. Every other mw.py bar passes.
2. `mw` with `KEEP_BACK=Family`: every bar passes; each lists Movies alone.
3. The trio, ten minutes, Alex in Chromium on One, Bo in WebKit on Two
   (converted), Cy in Chromium on Three, ten seeks each: nobody is asked
   to sign in; all three play through the window; of 30 seeks none
   misses 5 s (u5-9's computer seeks ran 375–592 ms; Bo's converted ones,
   on a phone, 430–4,921 ms); a miss, if one, comes from Bo's.
4. Jellyfin's users after: `sam` and three the middleware named, one
   per person, `<Name>-<16 hex>`; nobody made one. Each of the three
   opens Movies alone.
5. No byte any of the three received holds Sam's password, Sam's token
   or the key Sam gave the middleware (scan.py).
