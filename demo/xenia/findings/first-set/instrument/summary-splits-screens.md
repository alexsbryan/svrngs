# Instrument fault: the set's summary splits one screen into many

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: hard (the instrument's own): a fault in xenia or in a scene's world, not in svrngs.

## Where

`summary.md`, "Hard faults, by screen": 17 lines, each "wordy, in 1 of 3 runs".

## The xenia side

`src/set.ts:32` (`summary`) keys a fault by `${h.screen} ${h.kind}`, its raw address. Each world has its own ports and each link its own token (`crates/svrngs/tests/xenia/main.rs` `House::up`), so no screen is the same address twice and nothing reads as more than 1 of 3. Grouped by hand, the wordy screens are 3 of 3 or 2 of 3 (`../wordy-*.md`).

## Who, and how often

Every scene, every run.

## What happened

e.g. `http://svrngs.localhost:63418/: wordy, in 1 of 3 runs`, `http://svrngs.localhost:63873/: wordy, in 1 of 3 runs`, `http://svrngs.localhost:64407/: wordy, in 1 of 3 runs`: Kim's home page, three times.

## What the people said

None: the summary.

## The product source

None.

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- Should xenia fold an address into a screen (say, its path with the origin's port and the fragment dropped, or a name the scene gives), or should a scene's world keep fixed addresses across runs?
