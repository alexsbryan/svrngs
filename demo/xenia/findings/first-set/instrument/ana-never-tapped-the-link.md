# Instrument fault: Ana never tapped the link

- status: open
- found by: the set `~/dev/xenia/runs/2026-10-26T16-41-02-232Z`, 3 runs on sonnet, xenia `57d0da1`, svrngs-xenia `7341378c`
- kind: hard (the instrument's own): a fault in xenia or in a scene's world, not in svrngs.

## Where

Ana's phone, an empty screen holding her opening message from Sam with the link.

## The xenia side

The model, not the surface: no `tap` was ever sent. The stage would have opened the link (`src/stage.ts` `touch`, a tap matching a link heard). Whether the empty screen's answer ("The screen is empty.") leads a session to think it can't act is xenia's to judge.

## Who, and how often

Ana, 1 of 6 accounts (`together` run 3).

## What happened

"failed, stopped before their goal". Her acts: look · say in the house chat · wait 10 · look · tell.

## What the people said

- Ana in the house chat, `together` run 3: "I tried to open the link, but my screen is still blank. What should I do with it?"
- Ana, `together` run 3: "My screen is blank and I can't open the link."

## The product source

None.

## To reproduce

```sh
cargo test -p svrngs --test xenia --no-run --manifest-path ~/dev/svrngs-xenia/Cargo.toml && cd ~/dev/xenia && node src/run.ts run ~/dev/svrngs-xenia/demo/xenia --runs 3 --model sonnet
```

## Open questions for the operator

- For the xenia worker: does a person whose url is null need the empty screen's answer to say their messages' links can be tapped? (Invariant 7: in neutral words.)
