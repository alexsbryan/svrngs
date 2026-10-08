<!-- section: mw after=intro -->
This queue is the middleware spike (`docs/DESIGN.md` §8 "Then" :1093; §9
:1133; §3 "The room" call 6; §10 :1146-1149). Its trigger, Demo B
passing, fired on 2026-10-07 (`demo/runs/b/2026-10-07.json`, the
operator's call). The question it answers: can what u5-3 counted as four
rows of infra for Jellyfin's zero sign-in (§9 :1131; u5-3 finding 4,
`demo/runs/u5/2026-10-05-jellyfin/README.md`) be done instead by an
ordinary program on the lender's computer, in front of Jellyfin, with no
diff under `crates/` or `web/`?

What middleware is (operator, 2026-10-06; §9 :1133): a lent app that
forwards to another — it listens on a port, takes the door's verified
request and passes it to the service behind it, so `svrngs share <its
port>` lends the pair. Only the lender runs one, on their own computer.
svrngs ships no runtime, SDK or plugin format for it, and never starts or
supervises one. The door's guarantees stay outside it: identity set, the
limit and refused paths checked, an echoed secret stripped. Its contract
is HTTP, the door's `x-svrngs-*` headers and the loopback record API.
An app's nouns live in middleware, never the infra.

Fences the rows keep:
1. *No diff under `crates/` or `web/`.* The spike is killed, not bent, if
   it needs a svrngs change beyond headers the door already sets: a row
   that finds such a need writes it down with its evidence and goes to
   the verdict, it does not make the change. Each row's check runs
   `git diff <this queue's first commit>..HEAD -- crates web`.
2. *Jellyfin stays in `apps/jellyfin/`* (`scripts/vocab_check.py`), and
   `scripts/check.sh` stays free of it: the spike's runs need a Jellyfin,
   as u5-3's and u5-4's did (`apps/jellyfin/check/check.sh`).
3. *Measured before built, the rule before the data.* mw-0 commits the
   bars, the kill clause and a prediction per bar before anything runs,
   as u5-3 did; none is moved after its data.
4. *The open questions are recorded, not answered.* Demo A's unchanged
   list of users (`demo/a.md`) and the per-process tie (`mint_tie`, §9
   :1126) are the operator's; the rows record what bears on each, and
   mw-3 packages them.
5. *Forked, not rewritten* (AGENTS.md 11): upstream's `provision`
   (`mesh_media/viewer.rs`, under `cmnwlth/` at commonwealth-ai HEAD) is
   the shape of a user per person. Read upstream with
   `git -C ~/dev/commonwealth-ai show <commit>:<path>`. Jellyfin is
   GPL-2.0-only: talk to it over HTTP and its API, copy none of its code.

One host, one run of each: not a rate (AGENTS.md 7). A run that could not
finish is could-not-judge, never a pass (AGENTS.md 6).
