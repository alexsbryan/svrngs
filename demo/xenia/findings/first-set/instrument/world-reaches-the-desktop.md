# Instrument fault: a scene's world opened a window on the host

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: hard (the instrument's own): a fault in xenia or in a scene's world, not in svrngs.

## Where

Kim's home page, "Open this folder", in a scene's world on the Mac the set ran on.

## The xenia side

Not xenia's stage, which filters a browser's addresses only (invariant 13), but a scene's world: its svrngs computers run on the host, and svrngs opens the host's file browser when asked (`../folder-opens-a-window.md`). The world (`crates/svrngs/tests/xenia/main.rs`, through `people.rs` `svrngs`) gives each computer a `HOME` of its own and the host's `PATH`, so `open` is found.

## Who, and how often

Kim, 4 of 6 accounts (`kim` runs 1-2, `together` runs 2-3).

## What happened

The page said " Opened, in a window of its own."; a Finder window opened on the operator's Mac.

## What the people said

- Kim, `together` run 3: "It has an 'Open this folder' button, which I tapped and which said 'Opened'."

## The product source

`crates/svrngs-node/src/home.rs:634` (`OPENERS`), `:642`.

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- Should a scene's world keep `open` and `xdg-open` off its computers' `PATH`, so the act is refused as on a computer without a file browser, or is a window on the host acceptable in a scratch world?
