# relay-park (c)'s run — predictions, before any data

Written at 4fee205, before `run.py` has run. What is predicted, and what
would falsify each:

1. **n0 answers.** Some of n0's four relays, as the binary lists them,
   answer `/ping` (`scripts/relay-answers.sh`). If none does, the run is
   could-not-judge, never a pass, and nothing below is judged.
2. **The link carries one of n0's relays.** `svrngs new house --as Sam
   --origin <page>`, from the binary as shipped (`SHIPPED = MeetAt::N0`),
   prints a link on the page's origin within 15 s. Its token carries a
   relay, and that relay is one of the four n0 relays the binary lists:
   the nearest from this host, which I expect to be `usw1-1`. Falsified
   by a bare token, or by a relay that is not n0's.
3. **The page joins through it, in both engines.** In Chromium and in
   WebKit as an iPhone, the page asks for a name. Given Bo, it says
   "You’re in house, as Bo", and it says that again when opened alone.
   Each engine takes under 30 s from the link handed over to its last
   line. Falsified by any other status, or by `page_exit` other than 0.
   If WebKit refuses n0's relay, as it did before u4-9's dotless
   spelling, that is a failure with WebKit's words, not a skip.
4. **Sam's computer is reached through n0.** Among the svrngs process's
   sampled remote addresses, at least one belongs to an address that n0's
   relay names resolve to. It opens no other remote address, apart from
   the router's port mapper and the n0 DNS that its lookup publishes to.
   That host is not one of the relay names, so this run does not resolve
   it, and it would show as an address not in `n0_hits`. Falsified by
   `n0_hits` being empty. A sample taken each second can miss a short
   connection, so an address that is absent is not proof that none was
   opened.

One host, one run of each engine: not a rate (AGENTS.md 7).
