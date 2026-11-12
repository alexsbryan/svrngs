# Movie Night

One search over every Jellyfin your groups lend: each film a card, with each
friend's copy under it, opening in that friend's Jellyfin.

An example to copy and bend, not part of svrngs: a static page with no server
and no credential. svrngs knows nothing of films; the page reads what svrngs
tells it at `/svrngs-reads`, and each library through the account its lender
keeps for apps that read it (`svrngs share --readers`,
[Jellyfin](../jellyfin/README.md)).

It comes with svrngs: bring it to a group from your home page, as you do
Money, and everyone in the group opens it from theirs. Anyone can still lend
their own copy from a port (`svrngs share <port> --program movie-night
--reads jellyfin --group <group>`, its files served by any plain file server).

`match.js` decides which films a search names and which copies are one film
(`match.test.mjs`); `crates/svrngs/tests/scenes/movie_night.rs` plays a person
new to the group finding a friend's film end to end.

The cold page keeps its controls disabled until its module has bound the search
form; it says **Opening…** meanwhile. `startup.test.mjs` holds the real module
back to check that an early tap cannot submit the unbound HTML form and lose
the query, then releases it and exercises the actual search.
