# Strangers through svrngs

xenia sends strangers through svrngs before people do: a model playing one
person, told only what that person was told, touching svrngs only through
a browser. This directory is svrngs's set of scenes for it. A rehearsal
points to where people will stumble; it is never a people result.

## Run the set

From xenia's checkout, with this repository's path in place of `<svrngs>`:

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path <svrngs>/Cargo.toml
xenia run <svrngs>/demo/xenia --runs 3 --model sonnet
```

The first line builds the worlds once, so no scene waits minutes on its
first build. `xenia` is xenia's `src/run.ts`. It runs xenia's plants
first, then each scene three times, and writes the accounts and a summary
under `runs/` where you run it, which is why you run it from xenia's
checkout.

The people are drafts until the operator writes them. xenia refuses a
person whose file does not open with its numbers, so the set won't run
while a `DRAFT` line tops any file in `people/`.

## The scenes

Each of the four below is one scene: a link to `world/scene.sh`, which
stands up the world the link's name names, prints the scene's line, and
keeps the world up until xenia closes its stdin.

- **`together`**: Kim, on her computer, puts the pizza into the house's
  money and tells Ana in the house chat. Ana, on her phone, opens the
  house's link and finds what she owes.
- **`kim`**: Kim alone, the pizza into the house's money.
- **`ana`**: Ana alone, the pizza already in, from the link to what she
  owes.
- **`newcomer`**: June, a bit afraid of computers, joins the house from
  her computer's home page by the link Kim sent, with no terminal.

Each world is a function in `crates/svrngs/tests/xenia/main.rs`, built on
the demo scenes' `crates/svrngs/tests/scenes/people.rs`: real svrngs
computers of this build, each with a home of its own, the house met at a
relay on this computer, and the group's page on localhost. Nothing leaves
this computer but what June's computer does: in no group, it meets
svrngs's shipped relays until it joins one.

What each person was told is in that file, beside the scene that tells
them. Who they are, and how fast they read, act and give up, is in
`people/`.

## What a scene counts

Each scene names the faults its screens must never show and the longest
its steps may take, with where each comes from: the evening of
2026-10-07 (`demo/runs/b/2026-10-07.json`), `demo/b.md`, and
`NORTH_STAR.md`. A fault is about svrngs, whoever the person was. What the
person says is theirs, and is kept apart.
