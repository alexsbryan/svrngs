# Spot

Lend your group a model you pay a provider for, for the night, without
giving anyone your key. Your friends chat with it from their phone or
computer, as themselves; you see who used how many tokens, never what they
wrote; at the time you set, or when the tokens you allowed are spent, it
stops answering.

- **Shape:** lent. A program on your computer, holding the budget; svrngs
  holds your key.
- **Run it:** [below](#run-it), from svrngs's source
  ([run one](../../docs/guide/examples.md#3-run-one)).
- **How it's made:** [How it works](#how-it-works).
- **Change it:** `server.mjs` is the server, `page.js` the chat. Restart it
  after a change.

## Run it

1. In a terminal, from svrngs's source, with svrngs running. Leave it open:
   ```
   node apps/spot/server.mjs --port 8090 --provider https://api.deepinfra.com/v1/openai --model deepseek-flash=deepseek-ai/DeepSeek-V4.1-Flash-1017 --until 02:00 --budget 2000000
   ```
2. Lend it from your home page: under **Already running here**, press
   **Look for programs running on this computer**. Fill in **Its name:**
   `deepseek-flash` and **Which program:** `spot`, tick the group, and
   press **Lend it to** your group.
   Then open its line under **What you lend** and add the header
   `Authorization`, `Bearer <your key>`.

   Or in a second terminal: `svrngs share 8090 --name deepseek-flash
   --program spot --group <group>`, pasting `Bearer <your key>` where it
   asks for a password.

Your key isn't shown, and stays on your computer: svrngs keeps it with the
lend and adds it to each friend's request on its way in. Spot never has it
on disk.

- `--provider` is any provider that speaks OpenAI's API, its address
  ending where `/chat/completions` goes on: DeepInfra
  `https://api.deepinfra.com/v1/openai`, OpenRouter
  `https://openrouter.ai/api/v1`, Together `https://api.together.xyz/v1`,
  Groq `https://api.groq.com/openai/v1`, OpenAI `https://api.openai.com/v1`.
- `--model <name>=<id>` lends one model, as many as you like: the name your
  friends see, and the provider's own id for it, as its `/models` lists it.
- `--until HH:MM` (its next one) or a date and time; `--budget` in tokens,
  everyone together. Leave either out for none.

Everyone in the group then finds "deepseek-flash" among what you lend:
tapped on a phone, or clicked on their home page, it opens Spot's chat
page. Each answer prints a line here, who and how many tokens; Ctrl-C says
who used how much. `svrngs unshare deepseek-flash` stops lending it.

## How it works

- `server.mjs`: the chat page and two of OpenAI's paths, `GET /v1/models`
  and `POST /v1/chat/completions`, nothing else. A chat's `model` is turned
  from the name you lent into the provider's id, and the request goes on
  with three headers: the `authorization` svrngs's door added, which is
  your key (the door puts it in place of any a friend sends,
  `crates/svrngs-node/src/door.rs` `rewrite_headers`), `content-type` and
  `accept`. No `x-svrngs-*`, no cookie, nobody's name. The answer streams
  back as it comes; a stream is asked for its usage, and the tokens the
  provider reports are counted per person by `x-svrngs-key`, the door's key
  for them, so two friends of one name are two. It binds loopback alone;
  svrngs carries everyone else to it.
- `page.html`, `page.js`: the chat, for a phone first. It asks only its own
  origin: `spot.json` for what is lent, until when and what is left, and
  the chat, streamed. Each chat
  stays in that browser (localStorage). svrngs's look: the browser's own
  fonts and colours, named colours.
- `markdown.js`: the answers' Markdown, read into a tree of a closed set of
  tags (headings, lists, quotes, code, tables, links to `http`, `https` and
  `mailto` only) that `page.js` builds with the DOM's own calls. No HTML
  from an answer is ever parsed, and nothing a model writes becomes a
  script or a link to `javascript:`.

## What it does not do

- **It hides nothing from the provider.** Every message goes to the
  provider under your account, as if you had sent it. Lend only to people
  whose messages you are content to send as yours.
- **It stops answering; it does not stop being lent.** Past `--until`, or
  with the budget spent, each chat is refused and the page says why, until
  you `svrngs unshare` it or stop the server.
- **A budget can be passed by one chat.** It is checked as each chat
  begins, and a chat's tokens are known only once it ends, so a chat begun
  under it finishes: a 60-token budget let one 64-token chat through, then
  refused (`demo/runs/spot/2026-11-17/`).
- **It counts only what the provider reports.** A provider that sends no
  `usage` is counted as nothing, and says so in the line it prints.
- **It keeps no log.** What it prints is all it says; send it to a file if
  you want one.
- **How many at once is svrngs's**: `svrngs share ... --limit 3`.
- **It never sees your key until a friend's request carries it,** so it
  can't check the key or the model ids before lending: a wrong one comes
  back as the provider's own refusal, on your friend's page.

## Tests

`server.test.mjs` (the server against a stand-in provider and a stand-in
svrngs) and `markdown.test.mjs` (the Markdown's laws) run in
`scripts/check.sh`'s "app laws". That svrngs's door adds the key in place
of a friend's and never lets it back is svrngs's own test,
`a_header_the_door_adds_is_given_on_the_home_page`
(`crates/svrngs-node/tests/home.rs`).
