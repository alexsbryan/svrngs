# Predictions, written before the run

PLAN §1, "Examples, a ladder", unit 4. One host, one run: not a rate.

**The run.** Ana and Bo are each a program here with a HOME and an
`SVRNGS_DIR` of their own. The binary is release v1.11.0's
`svrngs-linux-x86_64.tar.gz`, checked against its `SHA256SUMS`, never a
build. Ana makes Elm, and Bo joins it. Then Ana follows
`docs/guide/examples.md` alone, its words read on her home page in Chromium.

Two places the run can't follow the guide, said here first:
- install.sh isn't run. It would add a login service for this computer's
  real user, so the release's tarball is unpacked by hand, as install.sh
  unpacks it.
- Spot's provider is a stand-in on 127.0.0.1 answering one key, as in
  `demo/runs/spot/2026-11-17/`. No real provider's key is spent.

**Rung 2, open one up**
1. Under **Make a group**, Ana makes Sandbox. **Add an app** shows
   **See how they’re made**, and **Bring the ticked ones to Sandbox**
   brings Money.
2. **Folder location** names a folder holding `apps/Money/index.js`.
3. Ana changes `Add what you paid` to `Add what you paid for` and saves.
   Money, opened again, shows the new words.
4. `svrngs check <that folder>` exits 0.
5. Ana copies the folder into Elm's `apps/`. Bo's home page lists Money
   under Elm.

**Rung 3, run one**
6. `node --version` is v24 or later. `svrngs version` says
   `svrngs 1.11.0 (…)`, and the guide's three lines leave Ana in
   `svrngs-1.11.0/`, holding `apps/spot/server.mjs`.
7. Spot starts as its README's step 1 says (the provider aside).
   **Look for programs running on this computer** finds its port.
   **Its name:**, **Which program:**, Elm ticked and **Lend it to Elm**
   lend it. **Add this header** adds `Authorization`.
8. Bo opens Ana's deepseek-flash and is answered. The stand-in saw
   `Bearer <the key>`.

Where a line of the guide stalls is the finding, and the run records it as
it happened.
