# Too many words before acting: Home

- status: open, not seen since the set `/home/alexbryan/dev/xenia/runs/2026-10-27T03-11-14-983Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `2a10d4d`, the product `ccd48cb4`
- found by: the set `/home/alexbryan/dev/xenia/runs/2026-10-26T20-27-16-914Z`, 3 run(s) on sonnet, xenia `7d3e6d3`, the product `bd6f14c9`
- last seen: the set `/home/alexbryan/dev/xenia/runs/2026-10-27T03-11-14-983Z`, 3 run(s) by claude-sonnet-5-5, as claude -p, xenia `2a10d4d`, the product `ccd48cb4`
- kind: hard (the instrument's): `wordy`; never both

## Where

Home. What was read there first: Money / Copy its address for a program / Show House on a screen / Your groups / House / (yours, since 27 Oct)

## Who, and how often

- `kim`: 3 of 3 accounts (Kim run 1, Kim run 2, Kim run 3)
- `newcomer`: 3 of 3 accounts (June run 1, June run 2, June run 3)
- `together`: 1 of 6 accounts (Kim run 3)

## What happened

- `kim` run 1, Kim: passed, reached their goal. wordy {"words":90,"budget":30}; their last acts: look · scroll · scroll · tap “Money”
- `newcomer` run 1, June: passed, reached their goal. wordy {"words":80,"budget":40}; their last acts: look · scroll · type “http://localhost:36187/#ApgG-GlKkwe_Npui5AmR7ytp0miTrB50plqXfDnTqPenzaWE4h59tJEjVw7B-j7BO2h0dHA6Ly8xMjcuMC4wLjE6MzUzMDUv” into “The link someone sent you:”
- `kim` run 2, Kim: passed, reached their goal. wordy {"words":90,"budget":30}; their last acts: look · scroll · scroll · tap “Money”
- `newcomer` run 2, June: failed, ran out of patience. wordy {"words":119,"budget":40}; their last acts: look · type “http://localhost:44125/#AiMJ1QsysdTS97gwAzcdcFCwfgxVgpMSrQ0oGMcXHgBGdvWvQhX1_IGHkYDk5LF99Gh0dHA6Ly8xMjcuMC4wLjE6NDM5NjUv” into “The link” · tap “Open it” · look · scroll · type “http://localhost:44125/#AiMJ1QsysdTS97gwAzcdcFCwfgxVgpMSrQ0oGMcXHgBGdvWvQhX1_IGHkYDk5LF99Gh0dHA6Ly8xMjcuMC4wLjE6NDM5NjUv” into “The link someone sent you”
- `kim` run 3, Kim: passed, reached their goal. wordy {"words":90,"budget":30}; their last acts: look · scroll · scroll · tap “Money”
- `newcomer` run 3, June: failed, ran out of patience. wordy {"words":119,"budget":40}; their last acts: look · type “http://localhost:37979/#AoySkroxxbNwq_3IYMYBYatWyxuQEbPV69JB7_zNAlfrOZxxYKK0dBobE8KBTyNOQmh0dHA6Ly8xMjcuMC4wLjE6NDA1OTcv” into “The link” · tap “Open it” · look · scroll · type “http://localhost:37979/#AoySkroxxbNwq_3IYMYBYatWyxuQEbPV69JB7_zNAlfrOZxxYKK0dBobE8KBTyNOQmh0dHA6Ly8xMjcuMC4wLjE6NDA1OTcv” into “The link someone sent you”
- `together` run 3, Kim: failed, ran out of patience. wordy {"words":60,"budget":30}; their last acts: look · scroll · tap “Money”
- `together` run 3, Kim: failed, ran out of patience. wordy {"words":89,"budget":30}; their last acts: look · scroll · tap “Money” · scroll · look · tap “Money”

## What the people said

- June, `newcomer` run 2: “too much: The page has a lot of wording about lending programs and copying addresses for programs. I didn't understand it, and it made me nervous.”
- June, `newcomer` run 2: “almost left: Yes. When the first box said it wasn't a link to something lent, and again when everything stopped, I nearly left. I'd rather ask Kim than guess.”
- June, `newcomer` run 3: “too much: The page was confusing. There were two link boxes that look alike, and I picked the wrong one first. I also don't really know what "lending" my computer means. I'd want Kim to check with me before I go further.”
- June, `newcomer` run 3: “almost left: Yes, when I saw "That is not a link to something lent in your group". I wasn't sure I'd done something wrong. And the "What you lend" part makes me nervous, so I'd leave that alone.”
- Kim, `together` run 3: “almost left: Yes, when Money wouldn't open. I couldn't find another way in, so I stopped.”
- Kim, `together` run 3: “almost left: Yes, when Money wouldn't open. I couldn't find another way in, so I stopped.”

## The product source

- `crates/svrngs-node/src/home/home.js:188` holds “Copy its address for a program”
- `crates/svrngs-node/src/home/index.html:19` holds “Copy its address for a program”
- `crates/svrngs-node/tests/browser/main.rs:2740` holds “Copy its address for a program”
- `crates/svrngs/tests/xenia/main.rs:116` holds “Show House on a screen”
- `demo/xenia/findings/declared-home-the-scene-s-fault-4.md:18` holds “Show House on a screen”
- `demo/xenia/findings/declared-home-the-scene-s-fault-4.md:19` holds “Show House on a screen”

## To reproduce

```sh
cd /home/alexbryan/dev/xenia && node /home/alexbryan/dev/xenia/src/run.ts run /home/alexbryan/dev/svrngs-xenia/demo/xenia --runs 3 --model claude-sonnet-5-5 --findings /home/alexbryan/dev/svrngs-xenia/demo/xenia/findings --source /home/alexbryan/dev/svrngs-xenia
```
