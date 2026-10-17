# events-run — what svrngs lacked, for the operator

Two things the run found missing are svrngs's, not the app's. The app works
around neither (fence 3), and svrngs's code is unchanged by this queue.
Each would be a row of its own, on the operator's budget (AGENTS.md
"Done"). The lines are estimates, not measured.

## 1. Told the host's computer is asleep only after 30 s

**What the run showed.** With Sam's computer asleep, Bo's browser waited
30,019 ms and Cy's phone 30,024 ms before svrngs said "the computer that
lends this did not answer" (`README.md` here, 10b; `data/summary.json`,
`asleep`). The wait is the iroh bridge's dial timing out: Bo's request at
08:36:44.54, "transport: iroh bridge: dial failed … error=timed out" at
08:37:14.55 (`data/bo.log`). A person a bit afraid technically looks at a
blank tab for half a minute.

**What svrngs could do.**
- Bound the dial to a lend's computer by a constant of its own, as the
  door bounds its service's connection (`door.rs:270`, `CONNECT_WITHIN`,
  10 s), traced at debug. About 10–15 lines in `origin.rs` and a test.
- The cost: a slow relay path, a phone on cellular say, read as away when
  it would have answered.
- **The operator's call**: how long a person should wait before being told.
  This host cannot measure cellular (CHARTER.md).

## 2. No address for an event that anyone in the group can open

**What the run showed.** The driver built each phone's address of the
event itself, `<page>#open=<label>&at=/g/<group>/e/<event>`
(`run.py` `phone_at`), from the label in a computer's `/v1/lends` origin.
The app cannot build it:
- it is told who asks, and nothing of the label it is lent under or the
  group's page address (the door's headers, `door.rs:114-180`);
- a computer's own address for the lend, `http://<label>.localhost:<port>/`,
  is that computer's port, which no one else's computer shares.

So "Sam posted the dinner, here is the link" has no link. People reach the
event through the group's page and the app's own list; this run did not
take that path (`README.md`, finding 3).

**What svrngs could do.**
- Tell a lend its label and the group's page address in two headers under
  the door's prefix, so an app can write the phone's address. About 15–25
  lines in `door.rs` and `origin.rs`.
- Whether a computer opens the same `#open=` address on its home page is
  not tried here. If it does not, add that: not estimated.
- Together, a guess of 20–40 lines plus the home page's part.
- **The operator's call**: whether an app may hand out such links at all,
  or only svrngs's page may.
