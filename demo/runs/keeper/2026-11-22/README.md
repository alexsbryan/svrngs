# keeper-demo, 2026-11-22

The demo `ralph/next/keeper/CHARTER.md` names: Sam runs llama-server in a
terminal, lends it, and promotes it with `svrngs-keeper keep 8101`. Then
three reboots with nobody logged in, and Alex's link opens each time with
Sam typing nothing. A stop stays stopped, and Stop lending stays stopped.
This file's first commit holds the setup and the predictions, before any
data. One VM, one run of each: not a rate (AGENTS.md 7).

## Setup

- **The VM**: the spike's Fedora 45 Cloud image
  (`Fedora-Cloud-Base-Generic-45-1.4.x86_64.qcow2`, SHA256 `50bdc5e7…`, as
  its CHECKSUM says), a fresh overlay, KVM, 4 CPUs, 4 GiB, no TPM. svrn-docs
  `rings/current/local-hosting-spike/vm/` is the VM this follows.
- **Two accounts, each with a node**: `sam` (uid 1000) and `alex` (uid
  1001), each running svrngs as the login service
  (`packaging/arch/svrngs.service`'s, `stage/svrngs.service`). Alex's account
  lingers (Alex's computer is on); Sam's lingers only if the keeper turns it
  on.
- **A local relay**: `svrngs relay --at 127.0.0.1:7900 --open`, a system
  unit. Sam makes the group with `svrngs new house --relay
  http://127.0.0.1:7900`; Alex joins by its link.
- **svrngs and svrngs-keeper**: debug builds at this file's commit.
  llama-server is Fedora's `llama-cpp` package; the model is
  Qwen2.5-0.5B-Instruct Q4_K_M, at `~/models/qwen2.5-0.5b.gguf`.
- **Sam's terminal** is an ssh login (`ssh -tt`), and every "Type y" in it
  is answered y. Sam types, in order: `systemctl --user enable --now
  svrngs.service` and `svrngs new house …` (the install and the group),
  `llama-server -m ~/models/qwen2.5-0.5b.gguf --port 8101 -c 4096`,
  `svrngs share 8101 --program llama-server --name coder --group house`,
  `svrngs-keeper keep 8101 --lend coder`. Then nothing, until `svrngs
  unshare coder` and `svrngs-keeper stop coder`.
- **Alex's request** (`stage/ask.py`, from Alex's account): the lend's
  address from Alex's svrngs, `GET /health` through it every 0.5 s until
  200, then one chat completion. After each boot it runs as a unit in
  Alex's manager, timed by the VM's uptime; after the kill -9 and the
  unshare the driver runs it.
- **The driver**: `run.py`, on the host, all its data in `run/`.

## Predictions

1. **Lent from the terminal**: Alex is answered 200 through the lend.
2. **Promoted**: `keep` asks, stops the terminal's llama-server, starts
   `svrngs-keeper-coder.service`, then runs `svrngs approve coder`, which
   names the job as that unit; both yes, it exits 0.
3. **Lent after the promotion**: Alex is answered 200, with nothing
   approved again.
4. **Boots 1–3**: each boot, Alex is answered 200 and the chat completion
   answers, with no session of Sam's in logind (linger's manager only) and
   no command typed as Sam. The door's line says the kept lend's unit was
   renewed.
5. **kill -9** of the kept llama-server: systemd starts it again
   (`NRestarts=1`), and Alex is answered 200 within about 10 s.
6. **Stop lending** (`svrngs unshare coder`): Alex's svrngs lists no coder,
   and coder still answers on 8101 for Sam.
7. **Boot 4, after the unshare**: the same, after the reboot.
8. **A stop stays stopped** (`svrngs-keeper stop coder`, then boot 5): the
   unit is inactive and nothing answers on 8101.

Each case is named in `run/summary.json` with one of the four verdicts:
passed, failed, could-not-judge, never-ran.

## What the run gave

`BASE=… MODEL=… BIN=target/debug W=<scratch> python3 run.py`, exit 0, the
fourth attempt, at HEAD `1a7de2a0` with `run.py` and `stage/ask.py` as
`e925c2ff` commits them; svrngs and svrngs-keeper built at `96bca94a`
(nothing under `crates/` or `apps/` changed since). Fedora 45, kernel
7.2.11, systemd 262, `llama-cpp-b9840-2.fc45`. The data: `run/events.jsonl`
(every step, the link redacted), `run/summary.json` (each case and every
command typed as Sam), `run/*-door.log` (Sam's svrngs's door lines at each
collection), `run/terminal-llama.log`. Boots finished at 6.8–7.0 s
(`systemd-analyze`).

| # | Case | Verdict | What it showed |
|---|---|---|---|
| 1 | Lent from the terminal | passed | `/health` 200 through the lend in 5.7 ms, the chat answered |
| 2 | Promoted | passed | `keep` exit 0, two yeses; it enabled linger, and `svrngs approve` named "the systemd unit svrngs-keeper-coder.service (coder, kept by svrngs-keeper)"; the terminal's llama-server ended |
| 3 | Lent after the promotion | passed | 200 in 6.2 ms, nothing approved again |
| 4 | Boot 1 | passed | answered at uptime 7.50 s, the chat at 8.89 s |
| 4 | Boot 2 | passed | answered at 7.40 s, the chat at 8.85 s |
| 4 | Boot 3 | passed | answered at 7.55 s, the chat at 9.00 s |
| 5 | kill -9 | passed | `NRestarts=1`; 502 "not answering now", then 503 "Loading model", then 200 at 2.82 s after the kill |
| 6 | Stop lending | passed | Alex's svrngs listed no coder for 30 s; 8101 answered Sam 200 |
| 7 | Boot 4, after the unshare | passed | no coder listed from 7.0 s through the driver's 300 s; 8101 200, the unit active |
| 8 | A stop stays stopped | passed | after boot 5, `ActiveState=inactive`, `UnitFileState=disabled`, nothing on 8101 |

- **No login, nothing typed.** At each of boots 1–4, logind's only
  sessions were Sam's and Alex's class `manager` (linger's), at Alex's
  first request. `typed_as_sam` in `run/summary.json` has nothing between
  `svrngs-keeper keep` and `svrngs unshare coder`.
- **The renewal is what opened it.** Each boot's door log (`run/boot-*-door.log`)
  holds the door's `Held(Another)` lines ("no connection's taker was
  read": the unit's llama-server not yet listening), then "written, unless
  it changed meanwhile … renewed=Ok(())" within 0.7 s, the request that
  was answered. `run/kill-door.log` holds the same after the kill -9, at
  14.5 s.
- **Where the nodes met: could-not-judge.** The group was made to meet at
  the local relay, but the VM also reached the internet, and the relay's
  log was not collected before the VM was removed.

## Found on the way

The attempts before this one stopped short of the boots; each is in
`attempts/` or its commit.

- **A program reaches a friend's lend only while Play is on.** Alex's
  request was answered 401 "Open svrngs from your computer" until Alex
  turned Play on for coder (`svrngs ask /v1/play`), once; Play stayed on
  through every boot (play.rs `PLAY_FILE`).
- **`svrngs share` asks two questions** for a llama-server: "Let apps
  send?" and the password in its Authorization header (Enter for none).
- **Fedora Cloud has no polkit,** so logind refused `enable-linger` for
  Sam ("Access denied"), and the keeper warned as it should. A Fedora
  desktop ships polkit, whose `set-self-linger` is `allow_any=yes`.
- **The keeper does not ask for linger.** `keep` turned it on without a
  question and said so; STATE's user flow has it asking once more. And
  before the yes, `keep` said "It starts when you log in", read before it
  turned linger on.
