# Taste

Rate films you've seen, and Taste guesses what else in your friends'
libraries you'll like. Your ratings stay in your browser.

An example to copy and bend, not part of svrngs: svrngs knows nothing of
films or ratings. Taste is a folder with no server of its own. It uses what
any app in a group's folder can: reading a friend's lent Jellyfin
(`reads.txt`) and sending to a friend's lent llama-server (`sends.txt`),
each only on your yes.

## Start it

Copy this folder into your group's folder as `apps/Taste/`, and everyone in
the group has it. Then on your svrngs home page, open Taste. Under a
friend's llama-server, the line says *"Sends to Bo’s llama-server at
/v1/embeddings. Bo can see what it sends."* Tick it, and press **Open**.

Taste lists the films in your friends' libraries. Mark any you've seen
**Loved it**, **Liked it** or **Not for me**, and it lists the ones you'll
probably like.

Before you share a change with the group, `svrngs check <the folder>` says
what svrngs will refuse.

## What stays in your browser

- **Your ratings.** They're kept in this browser and sent nowhere.
- **Which films you rated.** To guess, Taste asks a model for a list of
  numbers that describes each film. It sends the whole list of films in
  your friends' libraries, in one order (by library, then by the library's
  own id), 64 at a time, never the films you rated. Your ratings turn those
  numbers into guesses in your browser. The friend whose model it is can
  tell that you asked, and when, but not what you rated.

## The model

Taste uses the first llama-server someone in your groups lends and lets
apps send to, started with `--embeddings`, for example:

    llama-server -m nomic-embed-text-v1.5.Q8_0.gguf --embeddings --port 8080
    svrngs share 8080 --program llama-server --group <group>

`svrngs share` asks whether apps may send to it; type `y`. Without that
yes, apps can only read it, and Taste says what's missing.

## How it's built

- `index.html` is the page and `index.js` all of its code: it reads each
  Jellyfin, sends the films to the model, and scores them. Your taste is
  the sum of your rated films' numbers, each weighted by your rating;
  an unrated film scores by how near it is (cosine).
- `reads.txt` and `sends.txt` name the programs Taste asks to read and send
  to. svrngs shows each on your home page and lets Taste reach only those
  you tick.
- `nouns.txt` keeps Taste's words out of svrngs's own code
  (`scripts/vocab_check.py`).

`crates/svrngs-node/tests/browser/sends.rs` opens Taste in Chromium and
WebKit against stand-in services. `crates/svrngs-node/tests/carried.rs`
`a_carried_taste_sends` runs it against a real llama-server, by hand.
