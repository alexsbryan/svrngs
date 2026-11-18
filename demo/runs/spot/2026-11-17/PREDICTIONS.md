# Predictions, written before the first server started

The run: one host, Ana and Bo each a program with a HOME and an
`SVRNGS_DIR` of their own, in one new group. A stand-in provider on
127.0.0.1 speaks OpenAI's chat API, answers only `Authorization: Bearer
<the key>`, streams its answers and reports usage. Ana starts
`apps/spot/server.mjs` against it, as `apps/spot/README.md` says, with a
budget of 60 tokens, and lends it with `svrngs share`, pasting `Bearer
<the key>` where it asks. Bo opens it from his own home page in Chromium,
at a phone's size, and chats. One run: not a rate.

1. Bo's home page lists Ana's lend; opening it shows Spot's page titled
   with Ana's name and the model ("ana’s deepseek-flash").
2. Bo's first message is answered on the page, as Markdown, streamed.
3. The stand-in received the model as the provider's id, `authorization:
   Bearer <the key>`, and no `x-svrngs-*` header and no cookie.
4. Ana's terminal names Bo and the tokens his chat used.
5. The key is in nothing Bo's browser holds: not the page's text, not its
   storage.
6. Once the 60 tokens are spent, Bo's next message is refused on the page,
   saying the tokens are spent, and the stand-in is not asked again.
