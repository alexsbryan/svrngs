# Opening a folder opens a window on the host

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: soft (the person's act), with a side effect outside the scene: no fault the instrument counts. Its world half is `instrument/world-reaches-the-desktop.md`.

## Where

Kim's home page, House's section: the button "Open this folder", beside "House’s folder is on this computer at /Users/…/target/scenes/xenia-…/kim/svrngs/House".

## Who, and how often

Kim, 4 of 6 accounts tapped it (`kim` runs 1-2, `together` runs 2-3).

## What happened

The page said " Opened, in a window of its own." and svrngs ran the host's file browser on the folder, on the Mac the set ran on. `together` run 3: look · tap "Open this folder" · look · tell.

## What the people said

- Kim, `together` run 3: "It has an 'Open this folder' button, which I tapped and which said 'Opened'."
- June, `newcomer` run 2 (not tapped): "I didn't tap either of them because I don't know what they do."

## The product source

`crates/svrngs-node/src/home/home.js:106` (the button and " Opened, in a window of its own."); `crates/svrngs-node/src/home.rs:141` (`POST /folder`), `:634` (`OPENERS`, `open` and `xdg-open`), `:642` (run on the folder).

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- Is a window on the host what svrngs should do from a page a person opened? (Here it is the scene's own; in a real house it is the person's own computer, as designed.) Should the worlds stop it, or the stage refuse it (`instrument/world-reaches-the-desktop.md`)?
