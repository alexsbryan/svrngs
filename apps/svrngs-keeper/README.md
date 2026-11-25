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
- `src/host.rs`: reads Linux listeners with `ss`, and how one of your own
  programs was started from `/proc/<pid>/cmdline` and `cwd`; on a Mac
  (`src/host/mac.rs`), `lsof`, `ps` and the kernel's `KERN_PROCARGS2`, in
  Keeper's one `unsafe` function.
- `src/promote.rs` and `src/facts.rs`: read those for the person to review
  as the command and folder Keeper would run. They read no environment,
  adopt no listener, and read nothing of another account's program.
- `src/kept.rs` and `src/main.rs`: the `svrngs-keeper` command.

## The command

```
svrngs-keeper keep [--name <name>] <card>   # a native card, run here for good
svrngs-keeper facts <port> [--name <name>]
svrngs-keeper list [--json]
svrngs-keeper stop <name>                   # until you start it, at boot too
svrngs-keeper start <name>                  # clears a failed unit's start limit
svrngs-keeper restart <name>                # only if running
svrngs-keeper remove <name>                 # its card and its folder stay
```

A kept card lives in `~/.local/share/svrngs-keeper/<name>/compose.yaml`,
the one it replaced beside it as `previous.yaml`; the unit goes in
`~/.config/systemd/user/` (both follow `XDG_DATA_HOME`, `XDG_CONFIG_HOME`).
The same card kept again resets nothing. It supervises nothing: systemd
keeps the unit, and linger starts it at boot. While the account doesn't
linger, `keep` asks first whether it may run with nobody logged in; on no,
or with no terminal to ask in, the unit starts at login.

`tests/kept.rs` drives it through a stand-in `systemctl` and `loginctl`.
`tests/host.sh target/debug/svrngs-keeper` proves it on a Linux host with
linger on, under the real user manager: kill -9 brings it back, `stop`
stays stopped, `remove` leaves its data, and `keep` twice resets nothing.

`facts` is read-only. When one program of your own listens on the port,
it returns `kind: prefill` with the port, a name (`--name`, else the
program's), the folder it runs in and the command it was started with,
each argument quoted as you'd type it; else `kind: unavailable` with why.
Its arguments can hold a secret, such as `--api-key`; they go only to your
own home page and the settings Keeper saves on this computer, and its
debug trace names only the program and how many arguments it has. Stop
the copy you started yourself before running it. `tests/promote.rs`
checks the listener parser, the argument readers and a real listener's
command and folder read back.

Give's tests exercise its runtime through Give's names (`../give/tests`).
The queue that grows it is `ralph/next/keeper/STATE.md`.
