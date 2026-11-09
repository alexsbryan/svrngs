# Movie Night

One search over every Jellyfin your groups lend: each film a card, with each
friend's copy under it, opening in that friend's Jellyfin.

An example to copy and bend, not part of svrngs: a static page with no server
and no credential. svrngs knows nothing of films; the page reads what svrngs
tells it at `/svrngs-reads`, and each library through the account its lender
keeps for apps that read it (`svrngs share --readers`,
[Jellyfin](../jellyfin/README.md)).

Lend it from any computer with its files served on a port, for example:

    python3 -m http.server 8597 --directory apps/movie-night
    svrngs share 8597 --kind movie-night --reads jellyfin --group <group>

`match.js` decides which films a search names and which copies are one film
(`match.test.mjs`); `crates/svrngs/tests/scenes/movie_night.rs` plays a person
new to the group finding a friend's film end to end.
