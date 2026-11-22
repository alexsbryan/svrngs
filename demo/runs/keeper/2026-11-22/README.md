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
