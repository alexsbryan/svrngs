# Mesh house — a group on a Lightning Mesh, then the modem off

Ten people on a Lightning Mesh keep using one group while its modem is
cut. The Mac is the keeper and moves to the mesh relay; a Fedora at home is
the edge. The model runs on a mesh member, so the room's work is not
interrupted. Fedora's local llama keeps loading and inferring, but the group
cannot use it across the cut.

## Before

- Build this branch on the mesh laptops: `cargo build --release -p svrngs`.
  Use `target/release/svrngs`; an installed `~/.local/bin/svrngs` may be older.
- Relay host: an always-on mesh computer at a stable `10.42.x.y`.
  `svrngs relay --at 10.42.x.y:3340 --open`
- At home, the keeper makes the group and Fedora joins:
  `svrngs new house --as Sam`
  `svrngs join '<link>' --as Fedora`
  With no `--relay`, the group meets at the build's fallback (n0), which
  Fedora can reach from home.
- At the coworking space, stop svrngs on the Mac, then:
  `svrngs link house --relay http://10.42.x.y:3340/`
  Only the keeper may, and only the keeper's node stops: the ten and the
  relay keep running. This makes the new link; it carries the mesh relay.
  Start svrngs again, or leave the command running.
- Send that link to the ten; each joins:
  `svrngs join '<link>' --as <name>`
- On a mesh member:
  `llama-server -m <local.gguf> --host 127.0.0.1 --port 18089`
  `svrngs share 18089 --program llama-server --group <group>`
  Use a local file: `-hf` needs the internet at load time.

## The scene

1. Before the cut, open the model from another mesh laptop, start a long
   answer, and add a file to the group's folder.
2. Cut the modem. The answer finishes, the file reaches the others, a new
   file syncs, and the model still answers. The mesh relay is local.
3. Fedora still has its own internet, but it cannot reach a private mesh
   address; the mesh cannot reach n0. It is an island. Its local llama keeps
   loading and inferring; its lend shows as unanswered on the mesh side.
4. Restore the modem. Fedora syncs the entries it missed; the group is whole
   again. Anything written on both sides merges.

## Edges

- Join Fedora before the switch, or keep a relay it can reach in the set. A
  link carrying only the mesh relay cannot be opened from home.
- Keep the old relay until Fedora has synced. Two phases:
  `svrngs link house --relay <old> --relay <new>`, wait for a round, then
  `svrngs link house --relay <new>`.
- The relay host must stay on the mesh. If it is the Mac and the Mac sleeps,
  the room loses its meeting point.
- Phones are not in this demo: `--at` is plain HTTP, and an HTTPS page
  cannot use it.
- If any mesh router still has an uplink, Fedora may stay reachable through
  n0; then the cut is not complete.
- The relay change does not remove Fedora from the group. It changes where
  the group meets; Fedora's keys, membership, files and local copy stay.
