# On air, bytes by hash for an app: predictions

onair-blob's check: "the harness's 4.1 rows green". The harness
`apps/on-air/harness/run.py`, its scoped bar's letter rewritten in
429525c and otherwise as onair-red wrote it, runs once on this host at
the commit after this one's, with `/v1/blob` (db6ba51), `/v1/sign` and
the live sender:

    W=/tmp/onair-blob-w python3 apps/on-air/harness/run.py demo/runs/onair/2026-10-13-blob

Written before the harness ran with `/v1/blob`. One host, one run: not a
rate (AGENTS.md 7).

| bar | predicted | why |
|---|---|---|
| 4.1 | **passed** | every put 200, answering the bytes' BLAKE3, which the station checks; Bo's listener gets each piece by its hash, his computer asking Ana's and Cy's at once what they hold, then fetching from a holder, verified: 10 of 10, each within the 4,000 ms bound the harness set before it ran (`BOUND_MS`), from its line's arrival |
| 4.1, carried | **passed** | Ana's computer stopped, Cy's get of Y asks Ana's (no answer, within 3 s) and Bo's (holding it) and fetches it from Bo, the trace's `blob: held from a computer of the group` line naming Y's hash and Bo's key; the plant Z, held by Ana's computer alone: Bo's says `never_held`, Ana's does not answer, so Cy is told 502 `unanswered`, "no holder answered", each computer named — never an empty piece |
| 4.1, lapses | **passed** | 65 s after the last put, Bo and Cy each say 410 `lapsed` of piece 0 and of Y, each having held them; of a hash nobody put, each says 502 `unanswered` (Ana's computer stopped, the other saying `never_held`), told apart from lapsed |
| 4.1, scoped | **passed** | Dee, removed, is refused Y by her own computer, which no longer stands in the group: 403 "this computer does not stand in", traced `blob: refused` in Dee's log. Should Dee's record not yet hold her removal, her computer asks Bo's and Cy's, each closing her ask as removed (`door::gone`), and she is told 502 `unanswered` with each refusal named: refused either way, said in Dee's trace |
| 4.2 | passed | as in `../2026-10-13-sign/` |
| 4.3 | passed | as in `../2026-10-13-sign/` |
| off air | passed | as in `../2026-10-13-sign/` |

Where 4.1 could part from this: Bo and Cy fetch the same piece at about
the same time; a computer says it holds a hash from the moment its own
fetch begins, so Bo may ask Cy first and be refused by a store not yet
whole, then take it from Ana — slower, and said in his trace, but within
the bound unless the host is loaded. A miss past the bound is said with
its milliseconds, not hidden.

Anything else that fails is the harness's, and is said as that.
