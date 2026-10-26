# Minecraft Java — two homes, one server, through a port lent

Gate: **the operator's, set before the first run** (svrn-docs
`rings/current/GIVE.md` §11; `runs/give/2026-10-26-play-scene/NEEDS_HUMAN.md`).
Until the operator sets it, this run has no gate and is not run. The
candidate, by Factorio's (`factorio.md`): a 30-minute session between two
homes, once on a direct path and once with the relay forced, with no drop
on either, and the ping Minecraft reports on each path written down.

Minecraft: Java Edition speaks TCP on port 25565. svrngs carries each
connection on a QUIC stream of its own, on the one connection kept
between the two computers, so a loss on one player's stream never holds
back another's (GIVE.md §6). Over a relay, that connection's leg to the
relay is TCP, and a loss there stalls what follows it. Only play shows what
that does to the game, and this run is that play. The gate's "scenes"
step plays the same path with an echo program in place of the game
(`crates/svrngs/tests/scenes/ports.rs`). This run is the game itself.

**Before the run.** Two computers in two homes on two networks, each with
svrngs and Minecraft: Java Edition of the same version, and each person
their own account. **The host** runs the server. **The friend** joins it.
The two are in one group, joined by its link as any group is. No router
port is forwarded in either home. Both screens are recorded. Before
anyone starts, write down each home's upload and download, measured
once, and the host's public address (`curl -s https://ifconfig.me` on
the host's computer).

**The steps as the design has them now** (DESIGN §3, "A port, not a
page"). Nobody is told them.

1. The host runs Minecraft's own server, from minecraft.net, on its
   default port, 25565: `java -jar server.jar nogui` in its folder, with
   `eula=true` set in `eula.txt` once the host has read Mojang's terms.
2. The host lends that port, in a terminal on the host's computer:

   ```
   ~/.local/bin/svrngs share 25565 --tcp --kind minecraft --name minecraft
   ```

   It prints "You lend minecraft (TCP port 25565) to everyone in your
   groups, …".
3. The friend opens their home page. The host's row reads "minecraft"
   with **Play**. The friend presses it. The row says "Connect to
   127.0.0.1:<port>." and the address is copied. The port is the same
   every time, so the game's server list keeps working.
4. In Minecraft, the friend picks Multiplayer, then Add Server, pastes
   the address, and joins.

**What the host is told, and nothing else:** "Run a Minecraft server and
share it with your group."

**What the friend is told, and nothing else:** "Join the host's Minecraft
server from your home page, and play for half an hour."

**The two paths.** The run is played twice, 30 minutes each.

- **Direct.** Nothing is blocked. That the path is direct is read on the
  friend's computer, during play, by the host's address in UDP packets.
  svrngs's direct path is QUIC, so it is UDP, whatever the game speaks:

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
  Minecraft itself is untouched: it talks to `127.0.0.1`, which is not on
  `en0`.

**What is watched.** The clock starts when the friend presses Play.

- **Drops.** Each time the friend is dropped from the server, with
  Minecraft's words ("Timed out", "Connection lost", …), and the line in
  the host's `logs/latest.log`. They are counted per path.
- **Rubber-banding.** Each time the friend is pulled back to where they
  were a moment before, read from the recording, per path.
- **Ping.** The milliseconds Minecraft's Multiplayer list shows for the
  server, with the pointer on its bars, read before joining and again at
  15 and 25 minutes after rejoining from the list, per path.

The operator watches on a call with screens shared and says nothing. Any
help ends that attempt, and it goes in the notes.

**After.** One record per path in `runs/minecraft/<date>-<path>.json`,
README's fields with `"path": "direct"` or `"relay"`, `"minutes"`,
`"drops"`, `"rubber_bands"` and `"ping_ms"` (the three readings). The
svrngs commit is the one both computers ran. Each person's
`~/.svrngs/svrngs.log`, where the install line's svrngs writes
(`web/install.sh:83`, `:190`), is kept beside it. A path not played is a
record with `"never_ran"` and why, never a path left out.

**What this does not test.** The game's own LAN list (Open to LAN),
Bedrock, consoles, or more than two homes. Every player arrives at the
server from `127.0.0.1`, so the server's bans by address cannot tell
players apart (GIVE.md §6). Each waits for a group that asks.
