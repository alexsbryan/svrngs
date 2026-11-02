# Vendored for the pages

Both pages draw with Preact and writes its views with htm, each
copied from npm and pinned here, with no build step (the operator,
2026-10-31: "Preact with htm, vendored with no build step"; "I'm aligned
with the 11.1.1 version"; `ralph/next/page/CHARTER.md`). Nothing is fetched
at run time. Only `crates/svrngs-node/src/home/ui/view.js` imports these
files (`web/tests/specifiers.test.mjs`).

| package | version | npm `dist.integrity` | sha256 of the tarball |
|---|---|---|---|
| `preact` | 11.1.1 | `sha512-vpFVfai437pbuLri7XVeslCPfrEZX6Z1S2gV+MOFrA/kji+NKUXvH7y01EnVjA1guLigfkVp77Sqipd4SIllCg==` | `4cc919be2c056d77f9e77f1089ea58f90d9967433dbb9fd972d9d962892a838d` |
| `htm` | 3.1.1 | `sha512-983Vyg8NwUE7JkZ6NmOqpCZ+sh1bKv2iYTlUkzlWmA5JD2acKoxd4KVxbMmxX/85mtfdnDmTFoNKcg5DGAvxNQ==` | `2425b9bee11409177bcabc7f32e319926fc6690c1701c0b257c88bdff2d5ba90` |

Each file, where it comes from in its tarball, and that file's sha256 as
the tarball has it:

| here | upstream | upstream sha256 |
|---|---|---|
| `preact.js` | preact `package/dist/preact.mjs` | `37918605414d7dfb7540a77fc853cdc2bd108308ca506301bdefed32418b00e5` |
| `hooks.js` | preact `package/hooks/dist/hooks.mjs` | `24231102dc3e509c9c657bca591524e348eb8a5272c9d40538faef9beeeb654c` |
| `htm.js` | htm `package/dist/htm.module.js` | `ab33dd3f38059b9be4d5f5350128eefb2356639c4e0bbe9d9e8b3ba75847e9e4` |
| `LICENSE.preact` | preact `package/LICENSE` (MIT) | `1fe6958409c8c257a70c587a18b6f7f412b179b456630790d30b2ec9a8e4b7d4` |
| `LICENSE.htm` | htm `package/LICENSE` (Apache-2.0) | `740725f7252e750af735d0028cc534970772f513331e9f68150fede8fb3ce00f` |

Preact 11 ships its modules as `.mjs`; they are kept here as `.js`, since
the tests' server answers `.mjs` as `application/octet-stream`
(`web/tests/page.mjs`), which a page under `nosniff` (`web/vercel.json`)
refuses as a module. htm's `LICENSE` is the same bytes as its repository's
at the tag `3.1.1` (github.com/developit/htm), which has no `NOTICE`.

Two edits, and only these:
- `hooks.js`'s one import, `from"preact"`, becomes
  `from"./preact.js?v=__VERSION__"`. The pages' policy allows no import map
  (`scripts/trust_floor.txt`), so a bare name cannot resolve; and every
  module is loaded at one URL, its version stamped in, since two URLs load
  Preact twice and its hooks then fail (`reading '__H'`).
- Each `//# sourceMappingURL=` line is removed: the maps are not vendored.

`web/tests/vendor.test.mjs` checks each file against its sha256 below, and
that no file keeps a map line and `hooks.js` imports exactly
`./preact.js?v=__VERSION__`:

| file | sha256 |
|---|---|
| `preact.js` | `997a7750f38b9e493f3ffe14708f94774c14b9d7f357eb5ac689bf3a47b0ea54` |
| `hooks.js` | `fae9f2f2249d540150cba566ce3cfb939c8368a857bb95a295c5c7eb4a36241b` |
| `htm.js` | `ab33dd3f38059b9be4d5f5350128eefb2356639c4e0bbe9d9e8b3ba75847e9e4` |
| `LICENSE.preact` | `1fe6958409c8c257a70c587a18b6f7f412b179b456630790d30b2ec9a8e4b7d4` |
| `LICENSE.htm` | `740725f7252e750af735d0028cc534970772f513331e9f68150fede8fb3ce00f` |

To make these bytes again, in an empty directory beside this one:

    npm pack preact@11.1.1 htm@3.1.1
    mkdir p h && tar xzf preact-11.1.1.tgz -C p && tar xzf htm-3.1.1.tgz -C h
    sed '/^\/\/# sourceMappingURL=/d' p/package/dist/preact.mjs >preact.js
    sed -e '/^\/\/# sourceMappingURL=/d' -e 's#from"preact"#from"./preact.js?v=__VERSION__"#' p/package/hooks/dist/hooks.mjs >hooks.js
    cp h/package/dist/htm.module.js htm.js
    cp p/package/LICENSE LICENSE.preact
    cp h/package/LICENSE LICENSE.htm
    shasum -a 256 *.tgz preact.js hooks.js htm.js LICENSE.*
