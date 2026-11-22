# svrngs-keeper

A card kept running on this computer. It is Give's local runtime as a
package of its own (svrn-docs `rings/current/HOSTING_AND_GIVE_BOUNDARY.md`).
Give's keeper asks it to run the group's app, and svrngs calls its program to
keep what someone tried out running here. It knows no group, and it links no
svrngs crate and no Give (`tests/surface.rs`).

## The files

- `src/card.rs`: a card, the app's own `compose.yaml` with an `x-svrngs`
  block, read once, with each refusal naming its field. A native card's
  `x-svrngs.native` names a program installed here (a command, a folder,
  the names of the settings it reads) and it has no services; Give refuses
  it (`tests/native.rs`).
- `src/runtime.rs`: start, health, snapshot, restore and stop, for one card
  under one name, on Podman's command line (`src/podman.rs`).
- `src/machine.rs`: Podman's machine on a Mac.
- `src/check.rs`: the author's check of a card (`give check`).
- `src/systemd.rs`: a user unit written, driven and read back, and linger
  asked for once; forked from commonwealth-ai host-kit's `service.rs`.
- `src/unit.rs`: a native card as its unit, `svrngs-keeper-<name>.service`,
  with the spike's settings: `Restart=on-failure`, `RestartSec=2`, at most
  4 starts in 60 s, `SuccessExitStatus=143`, `WantedBy=default.target`.
- `src/kept.rs` and `src/main.rs`: the `svrngs-keeper` command.

## The command

```
svrngs-keeper keep [--name <name>] <card>   # a native card, run here for good
svrngs-keeper list
svrngs-keeper stop <name>                   # until you start it, at boot too
svrngs-keeper start <name>
svrngs-keeper remove <name>                 # its card and its folder stay
```

A kept card lives in `~/.local/share/svrngs-keeper/<name>/compose.yaml`,
the one it replaced beside it as `previous.yaml`; the unit goes in
`~/.config/systemd/user/` (both follow `XDG_DATA_HOME`, `XDG_CONFIG_HOME`).
The same card kept again resets nothing. It supervises nothing: systemd
keeps the unit, and linger starts it at boot.

`tests/kept.rs` drives it through a stand-in `systemctl` and `loginctl`.
`tests/host.sh target/debug/svrngs-keeper` proves it on a Linux host with
linger on, under the real user manager: kill -9 brings it back, `stop`
stays stopped, `remove` leaves its data, and `keep` twice resets nothing.

Give's tests exercise its runtime through Give's names (`../give/tests`).
The queue that grows it, next into `keep <port>`, is
`ralph/next/keeper/STATE.md`.
