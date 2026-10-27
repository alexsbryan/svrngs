# Diagrams

The three pictures in [`../OVERVIEW.md`](../OVERVIEW.md), drawn from data.
`src/kit.mjs` holds the shapes and `src/build.mjs` the three scenes. To change
one, edit its scene and draw them all again:

```
node docs/diagrams/src/build.mjs
```

It needs Node.js and nothing installed. Each SVG carries its own white
background, so it reads the same on a light or a dark page.

**The look is the pages' look** (`crates/svrngs-node/src/home/look.css`): the
system's own fonts, hard black rules, square corners, and color by the names
a browser knows. Nothing imitates a hand, paper or a stamp (DESIGN §1, rule
3). The method, scenes as data with one kit and one legend, is
commonwealth-ai's `docs/diagrams/`.

**One legend for every picture:** salmon is people, gold the record, plum
apps, sky blue the wire and what we host, light green let in, tomato refused.
Yellow marks the one line to keep.

**Two depths.** Plain words make each point. svrngs's own names are in
monospace, so a newcomer can skip them and an engineer can look them up.

**Few words.** Each picture has one point, and the reader has little
attention to give it. `v1/` is the fuller first set, about 2.5 times the
words. Its generator is this folder at `e5daca60`.
