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
  turned on; forked from commonwealth-ai host-kit's `service.rs`.
- `src/unit.rs`: a native card as its unit, `svrngs-keeper-<name>.service`,
  at background priority (`Nice=10`) with the spike's settings:
  `Restart=on-failure`, `RestartSec=2`, at most 4 starts in 60 s,
  `SuccessExitStatus=143`, `WantedBy=default.target`.
- `src/promote.rs`: `keep <port>`, decided from what the host printed: the
  port's process and the parents that come with it (a wrapper script, up to
  the terminal's shell), what it declines and why, and the card.
  llama-server started as itself is composed from `cards/llama-server` and
  its `/props`; anything else is copied, with no environment.
- `src/host.rs`: reads Linux listeners and processes and signals only the
  recorded pid@start pairs through pidfds.
- `src/facts.rs` and `src/takeover.rs`: the review and its confirmation
  token, durable staging, verification and recovery.
- `src/kept.rs` and `src/main.rs`: the `svrngs-keeper` command.

## The command

```
svrngs-keeper keep [--name <name>] <card>   # a native card, run here for good
svrngs-keeper keep <port> [--lend <lend>]   # what runs on <port>, kept in its place
svrngs-keeper facts <port> [--name <name>]
svrngs-keeper keep <port> [--name <name>] --yes --token <token>
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

`keep <port>` shows what it will run and asks; `--lend <lend>` names it and
then runs `svrngs approve <lend>`, since only svrngs knows which lend a port
is. The JSON `facts` command returns `known`, `copyable`, `kept` or
`declined` in `kind`, with an offer and token for the first two. The token
binds the original, the draft and the manager settings to this boot. Pass
the same name and token to confirm without a prompt. `--yes` does not turn
on linger or publish a lend. A completed takeover returns `verified` (exit
0) or `rolled_back` (exit 1) as JSON; on refusal, review again.

The keeper locks both port and name, then saves the staged card and the
original command in owner-only `takeover.json` before replacing the running
copy. It enables the new unit only after verifying that it owns the port
and answers. Each call reconciles an interrupted move: leave the original
alone if it still holds the port, finish a verified unit, or restore it
in its own scope on the same boot. On a different boot it tries the staged
unit; if it cannot verify it, `list --json` reports `interrupted` with the
stage and the person can explicitly start it. An unrelated holder is never
stopped by recovery. A completed recovery clears the private journal.

`tests/promote.rs` holds the draft decisions; `tests/promote.sh
target/debug/svrngs-keeper` proves it on a Linux host (not in a toolbox,
whose programs the host's units cannot run): llama-server with a relative
and a `~` model path, a wrapper script, a renamed python, and a copy that
fails and is rolled back. On that same host, `python3 tests/takeover.py
<debug-keeper>` (from this directory) SIGKILLs the command at the durable
handover edges and checks recovery, stale confirmations, and explicit Start.

Give's tests exercise its runtime through Give's names (`../give/tests`).
The queue that grows it is `ralph/next/keeper/STATE.md`.
