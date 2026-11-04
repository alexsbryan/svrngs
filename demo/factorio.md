# Factorio — two homes, one factory, through a port lent

Gate: **the operator's, set before the first run** (svrn-docs
`rings/current/GIVE.md` §11;
`runs/give/2026-10-26-play-scene/NEEDS_HUMAN.md`). Until the operator sets
it, this run has no gate and is not run. The operator set none on
2026-10-26 ("we can do the two-home session another time"), so it waits in
`docs/PLAN.md` §2, never-ran until played. The candidate GIVE.md §11
gives: a 30-minute session between two homes, once on a direct path and
once with the relay forced, with zero desyncs on each, and the ping
Factorio reports on each path written down.

Factorio speaks UDP on port 34197. svrngs carries it as QUIC datagrams,
one flow per player, and splits a packet too large for one datagram,
dropping the whole packet if a piece is lost, as the internet would
(GIVE.md §6). Nothing about how the game plays over a relay can be
learned from that design. Only play can show it, and this run is that play.
The gate's "scenes" step plays the same path with an echo program in place
of the game (`crates/svrngs/tests/scenes/ports.rs`). This run is the game
itself.

**Before the run.** Two computers in two homes on two networks, each with
svrngs and Factorio of the same version, and each person their own copy of
the game. **The host** keeps the factory. **The friend** joins it. The two
are in one group, joined by its link as any group is. No router port is
forwarded in either home. Both screens are recorded. Before anyone starts,
write down each home's upload and download, measured once, and the host's
public address (`curl -s https://ifconfig.me` on the host's computer).

**The steps as the design has them now** (DESIGN §3, "A port, not a
page"). Nobody is told them.

1. The host starts Factorio and hosts a game: Multiplayer, then Host new
   game or Host saved game, on its default port, 34197.
2. The host lends that port, in a terminal on the host's computer:

   ```
   ~/.local/bin/svrngs share 34197 --udp --kind factorio --name factorio
   ```

   It prints "You lend factorio (UDP port 34197) to everyone in your
   groups, …".
3. The friend opens their home page. The host's row reads "factorio" with
   **Play**. The friend presses it. The row says "Connect to
   127.0.0.1:<port>." and the address is copied. The port is the same
   every time, so the game's saved address keeps working.
4. In Factorio, the friend picks Multiplayer, then Connect to address,
   pastes, and connects.

**What the host is told, and nothing else:** "Host a Factorio game and
share it with your group."

**What the friend is told, and nothing else:** "Join the host's Factorio
game from your home page, and play for half an hour."

**The two paths.** The run is played twice, 30 minutes each.

- **Direct.** Nothing is blocked. That the path is direct is read on the
  friend's computer, during play, by the host's address in UDP packets:

  ```
  sudo tcpdump -n -c 20 -i en0 udp and host <the host's public address>
  ```

  Twenty packets in a few seconds mean the bytes go direct. None means the
  run is not the direct one, and it is noted and played again.
- **Relay forced.** On the friend's Mac, before Play is pressed, block
  every UDP packet leaving its network but DNS, so svrngs can reach the
  host only through its relay, which it reaches over TCP:

  ```
  echo "block drop out quick on en0 proto udp from any to any port != 53" | sudo pfctl -ef -
  ```

  The same `tcpdump` then shows no packets. After the run, put the Mac
  back: `sudo pfctl -f /etc/pf.conf`, and `sudo pfctl -d` if pf was off
  before (`sudo pfctl -s info` says which, read before the block).
  Factorio itself is untouched: it talks to `127.0.0.1`, which is not on
  `en0`.

**What is watched.** The clock starts when the friend presses Play.

- **Desyncs.** Each one Factorio reports: its dialog on the friend's
  screen, and its line in the host's `factorio-current.log`. They are
  counted per path.
- **Ping.** The ping Factorio shows for the friend in its multiplayer
  Players window, read at 5, 15 and 25 minutes, per path.
- **Drops.** Each time the friend is dropped from the game, and what
  Factorio said.

The operator watches on a call with screens shared and says nothing. Any
help ends that attempt, and it goes in the notes.

**After.** One record per path in `runs/factorio/<date>-<path>.json`,
README's fields with `"path": "direct"` or `"relay"`, `"minutes"`,
`"desyncs"`, `"drops"` and `"ping_ms"` (the three readings). The svrngs
commit is the one both computers ran. Each person's `~/.svrngs/svrngs.log`, where
the install line's svrngs writes (`web/install.sh:83`, `:190`), is kept
beside it. A path not
played is a record with `"never_ran"` and why, never a path left out.

**What this does not test.** The game's own LAN list, consoles, or more
than two homes. Every player arrives at the game from `127.0.0.1`, so the
game's bans by address cannot tell players apart (GIVE.md §6). Each waits
for a group that asks.
