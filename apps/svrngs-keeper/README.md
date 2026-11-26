# svrngs-keeper

A card kept running on this computer. It is Give's local runtime as a
package of its own (svrn-docs `rings/current/HOSTING_AND_GIVE_BOUNDARY.md`).
Give's keeper asks it to run the group's app. It also runs a native card
someone gives it. It knows no group, and it links no
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
  turned on; forked from commonwealth-ai host-kit's `service.rs`.
- `src/unit.rs`: a native card as its unit, `svrngs-keeper-<name>.service`,
  at background priority (`Nice=10`) with the spike's settings:
  `Restart=on-failure`, `RestartSec=2`, at most 4 starts in 60 s,
  `SuccessExitStatus=143`, `WantedBy=default.target`.
- `src/host.rs` and `src/promote.rs`: read which program holds a port,
  with `ss` on Linux and `lsof`, `ps` and `netstat` on a Mac
  (`src/host/mac.rs`). Keeper never reads how a program was started: you
  give `run` its command and folder.
- `src/given.rs`: what `run` is given, checked before anything is written
  (`tests/given.rs`). The page's line is split as a shell would split it,
  `~` read as home, and a setting (`FOO=1 prog`), `$`, a backtick or an
  operator refused, pointing to the terminal. The folder must be whole,
  there and a folder; the program is found as a shell finds it and kept by
  that path. Neither may be in a folder a group syncs (`--synced`, which
  svrngs passes), and each, through every link on the way to the program,
  must be owned by you or root and writable by nobody else, as OpenSSH's
  StrictModes asks of a key: root's sticky folders such as `/tmp` aside,
  and your group's write bit only where the group is yours alone.
- `src/native_run.rs`: `run`. The words after `--` are a terminal's: it
  shows them, with their folder, and keeps nothing until you press Enter
  there, nor with no terminal to answer. It keeps only what it verified:
  started under the service manager, then seen as that job answering on
  the port within 30 seconds. Anything else is stopped and its unit or
  agent and card removed, and why is said, with its exit status and what
  it last printed. On Linux linger is asked (or `--at-boot` taken, the
  page's yes) before the start and turned on only once it is verified.
- `src/kept.rs` and `src/main.rs`: the `svrngs-keeper` command.

## The command

```
svrngs-keeper keep [--name <name>] <card>   # a native card, run here for good
svrngs-keeper list [--json]
svrngs-keeper stop <name>                   # until you start it, at boot too
svrngs-keeper start <name>                  # clears a failed unit's start limit
svrngs-keeper restart <name>                # only if running
svrngs-keeper remove <name>                 # its card and its folder stay
```

A kept card lives in `~/.local/share/svrngs-keeper/<name>/compose.yaml`,
the one it replaced beside it as `previous.yaml`; the unit goes in
`~/.config/systemd/user/` (both follow `XDG_DATA_HOME`, `XDG_CONFIG_HOME`).
The command can hold a key, so every file that can hold it (the unit or
plist, the card, `previous.yaml`, `unit.hash`, and on a Mac the program's
`program.log`) is made 0600, and Keeper's folders 0700; an older Keeper's
open ones are closed at the next keep.
The same card kept again resets nothing. It supervises nothing: systemd
keeps the unit, and linger starts it at boot. While the account doesn't
linger, `keep` asks first whether it may run with nobody logged in; on no,
or with no terminal to ask in, the unit starts at login.

`tests/kept.rs` drives it through a stand-in `systemctl` and `loginctl`.
`tests/host.sh target/debug/svrngs-keeper` proves it on a Linux host with
linger on, under the real user manager: kill -9 brings it back, `stop`
stays stopped, `remove` leaves its data, and `keep` twice resets nothing.

`tests/promote.rs` checks the listener parser.

Give's tests exercise its runtime through Give's names (`../give/tests`).
The queue that grows it is `ralph/next/keeper/STATE.md`.
