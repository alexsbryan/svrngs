# The mark

![svrngs, sovereign rings](lockup.svg)

Three people, each a dot with a ring through the one place they meet. Three
equal rings through one point make a fourth of the same size, through all
three people (Johnson's theorem, 1916). That fourth ring is the group's, in
gold: the record they keep. Inside it nothing is anyone's, so each person's
ring keeps only its outer arc and stops short of the gold where it meets
another's. The large cut adds each person's far reach, a ring through the
other two.

`src/mark.mjs` holds its geometry and `src/build.mjs` writes the files. To
change it, edit the geometry and draw them again:

```
node docs/brand/src/build.mjs
```

The overview's diagrams sign their foot with it, from the same file
(`docs/diagrams/src/kit.mjs`).

## Rules

- **Its colors are the diagrams' legend:** salmon is a person, gold the
  record. Ink is `currentColor`, so the mark goes white on a dark page.
- **One cut per size.** Each is heavier the smaller it's shown:

  | File | Shown at | What it drops |
  |---|---|---|
  | [`mark-large.svg`](mark-large.svg) | 64 px and up, and beside the name | nothing |
  | [`mark-medium.svg`](mark-medium.svg) | 32 to 63 px | the far rings, which fill in |
  | [`mark-small.svg`](mark-small.svg) | under 32 px | thin lines, which vanish |

- **On the salmon band, people are white**, or they disappear into it.
- **The favicon is the small cut** ([`favicon.svg`](favicon.svg)), its ink
  white when the browser is dark, or it vanishes into a dark tab bar. The
  home page serves it (`home.rs`), and `scripts/build-web.sh` puts it beside
  the group's page; each page's policy allows an image from its own origin.
- **The app icon is white on black** ([`icon.svg`](icon.svg)), where the
  gold reads best. Gold on salmon goes muddy.
  An iPhone's Home Screen takes it as a PNG, never an SVG:
  [`apple-touch-icon.png`](apple-touch-icon.png), `icon.svg` drawn at
  180 px with no transparency (iOS fills transparency black). The group's
  page links it, and `scripts/build-web.sh` puts it beside the page, so a
  group's page added to the Home Screen shows the mark.
- **The name goes with what it's short for:** "svrngs" alone reads as
  "savings" ([`lockup.svg`](lockup.svg)).

## What it must not become

A ring of rings has a lot of history behind it. These came up while it was
drawn, and each one ruled a shape out:

- **A dot inside a tight ring reads as an eye**, and two side by side read
  as a face staring back. Rings around a dot read as a target.
- **A bullseye** is Target's logo, the RAF roundel, and crosshairs on a
  person.
- **Three rings opening outward around a center** is the biohazard sign.
  Three lobes around a disc is the radiation sign.
- **Arms turning inside rings** is the structure of the Black Sun, a
  neo-Nazi symbol. Four points on a circle is a step from the sun cross.
- **A six-pointed star in yellow** reads as the badge Jews were forced to
  wear.
- **Small heads touching a gold disc** read as sperm at an egg.

The eye and the face came from a rehearsal with strangers played by a model
(xenia, 2026-10-27), not from people. Show the mark
to people before trusting it at 16 px, where its shape is closest to a
trefoil.
