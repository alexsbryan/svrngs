# Predictions, written before the first server started

The run: one host, two people (Ana, Bo), each a program with a HOME and an
`SVRNGS_DIR` of their own, in one new group. Ana runs the real
`/usr/bin/llama-server` with `--api-key abc123` on 127.0.0.1:8301 and lends
it from her home page's routes, as the page sends them (`POST /running`,
then `POST /lent`). Bo calls it as a program, as `docs/guide/models.md`
says: the lend's address, his own svrngs token. One run: not a rate.

1. Lent with no header, Bo's chat is answered 401 by llama-server: his
   computer keeps his token, and nothing else reaches it.
2. Ana's `GET /lent` says `headers: {coder: []}`.
3. Ana adds `Authorization: Bearer abc123` on her row: 204; `GET /lent`
   says `["authorization"]` and holds no `abc123`.
4. Bo's next chat, nothing restarted, is answered 200 with the model's
   words.
5. Nothing Bo receives, headers or body, holds `abc123`.
6. A header named `x-svrngs-key` and a value with a line break are refused
   403, and nothing is kept.
7. Ana stops adding it: Bo's next chat is 401 again.
