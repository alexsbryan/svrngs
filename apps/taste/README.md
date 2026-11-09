# Taste

Rate films you've seen, and taste guesses what else in your friends'
libraries you'll like. Your ratings stay on your computer.

An example to copy and bend, not part of svrngs: nothing installs it, and
svrngs knows nothing of films or ratings. It uses what any app can (a lend,
`--reads`, `--refuse`, the door's word for who asks), and keeps its own rules
for what leaves your computer.

## Start it

Taste needs Node 20 or later. If you don't have it, get the installer from
[nodejs.org](https://nodejs.org) (the button marked LTS), open it, and click
through.

Get svrngs's source (on its repository page, **Code**, then **Download
ZIP**), open it, and put its `apps/taste` folder in your Downloads folder. Then open Terminal and paste:

    node ~/Downloads/taste/taste.mjs

If your Mac asks whether Terminal may use your Downloads folder, say OK.

It says where to open it: on your svrngs home page, press **Open Ana's
taste** under your group (your name in place of Ana's). Keep the window
open while you use it; closing it turns taste off.

The first time, the page asks you to rate five films you've seen: mark
each **Loved it**, **Liked it** or **Not for me**. Then it lists the films
you'll probably like first, and says which of your ratings each guess
comes from. To bring ratings from Letterboxd instead, export your data
there (Settings, then Data), open the file it gives you, and choose
`ratings.csv` on the page.

Each film has **Watch from Sam's library**. When you come back to taste
after watching, it asks **How was it?** once, and keeps your answer with
your other ratings. Which film you went to watch is kept in your browser
until you answer.

## What stays on your computer

- **Your ratings.** They're kept in `~/.svrngs/taste/diary.json`
  (`$SVRNGS_DIR/taste` if you set one) and go nowhere else. svrngs refuses
  `/diary` to everyone you share taste with.
- **Which films you rated.** To guess, taste asks a model for a list of
  numbers that describes each film. It sends a friend's model the whole
  list of films in your friends' libraries, in one order (by library, then
  by the library's own id), never the films you rated, and asks again only
  when the libraries change or taste starts again. Your ratings turn those numbers into guesses
  on your computer (`score.mjs`). A film you rated that no library here
  has is sent to no model at all, so it counts for nothing. The friend
  whose model it is can tell that you asked, and when; not what you rated.
- **Your guesses.** The page runs at your own address, served by taste on
  your computer, so the list and its reasons ("because you loved…") reach
  no page of anyone else's. Taste answers its own page and no other app's.

Only you can rate or see guesses: taste asks svrngs which person this
computer is, and refuses anyone else. Your friends in the group see taste
listed, and it tells them it's yours.

## The model

Taste uses the first `llama-server` someone in your groups shares that
answers, started with `--embeddings`, for example:

    llama-server -m nomic-embed-text-v1.5.Q8_0.gguf --embeddings --port 8080
    svrngs share 8080 --kind llama-server --group <group>

## How it's built

- `taste.mjs` serves the page, keeps the diary, and lends itself with
  `svrngs share <its place> --name taste --kind taste --reads jellyfin
  --refuse /diary --group <each of your groups>`.
- `page/` is the page. It reads each Jellyfin your groups share through
  the account its lender keeps for apps (`svrngs share --readers`,
  apps/jellyfin/README.md) and hands taste the whole list.
- `model.mjs` is the only code that sends anything to a model.
- `score.mjs` is the one scorer: your taste is the sum of your rated
  films' numbers, each weighted by your rating (5 stars pulls toward,
  1 pushes away); an unrated film scores by how near it is (cosine).
- `diary.mjs` keeps the ratings and reads a Letterboxd export.
- `svrngs.mjs` asks svrngs on this computer, with the `token` beside its
  `api` file, as apps/pantry does.

The taste scene,
`crates/svrngs/tests/scenes/taste.rs`, plays Ana's evening end to end.
