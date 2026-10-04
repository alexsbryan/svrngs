# svrngs

A group's shared state — money, plans, files, a library, its own AI — kept on the
devices of the people in it, entered from the chat they already use. The
design it is built to is `docs/DESIGN.md`.

## Run it

You need macOS or Linux, Rust (https://rustup.rs), Node.js 20 or later
(https://nodejs.org), a Discord account, and a Discord server you can add a bot
to, with one text channel.

**1. Install.** From this folder:

    cargo install --path crates/svrngs

Keep the folder: the Discord bridge runs from `bridge/` in it.

**2. Run `svrngs start`** and answer what it asks. It installs the bridge's
packages first (about a minute), then:

- **Make the bot.** At https://discord.com/developers/applications press *New
  Application* and name it after your group. Press *Bot*, then *Reset Token*,
  copy the token and paste it into the terminal.
- **Turn on two switches.** On the same *Bot* page, under *Privileged Gateway
  Intents*, turn on *Server Members Intent* and *Message Content Intent*, press
  *Save Changes*, then press Enter in the terminal. `svrngs start` says which are
  still off.
- **Invite it.** Open the link it prints, pick your server and press
  *Authorize*. It waits until the bot has joined.
- **Pick the channel** by its number, name the group, and say what the bot
  should call you.

It then installs a background service that runs whenever you are logged in, and
warns if the computer sleeps, since the bot cannot answer while it does.

**3. Try it** in the channel:

- `/paid` — fill in *amount* (60) and *what* (pizza). The others tap *Confirm*.
- `/owe` — who owes whom.
- `/settle` — fill in *amount* and *to* when you pay someone back.

**Where things are.** The group's whole record is `~/.svrngs/groups/<id>/`:
`genesis.json` and `journal.jsonl`, one signed act per line. Copy that folder and
you have the group. The service log is `~/.svrngs/logs/serve.log`; the bot's
tokens are in `~/.svrngs/secrets/`.

**Is it running?**

- macOS: `launchctl list | grep org.svrngs.serve`
- Linux: `systemctl --user status svrngs`
- Or run it in the foreground instead: `svrngs serve`

**Stop it.**

- macOS: `launchctl unload -w ~/Library/LaunchAgents/org.svrngs.serve.plist`
- Linux: `systemctl --user disable --now svrngs`

To start over, stop the service and move `~/.svrngs` aside rather than deleting
it: it is the group's record. Running `svrngs start` again picks up where it left
off and reuses the bot you already made.

**Demos.** For D1, set up a fresh server this way first, then follow
`demo/d1-split-the-pizza.md`; D2 is `demo/d2-your-own-group.md`. Record every run
as `demo/README.md` says.
