# Demos

Each demo in `docs/DESIGN.md` §8 gets a run-of-show here, `<demo>.md`, written
before its first run — Demo A's is `a.md` — and every run leaves one record in
`runs/<demo>/<date>.json`. A run nobody recorded did not happen. The script
that reads the records lands with the first demo.

A record:

```json
{
  "demo": "b",
  "date": "2026-10-20",
  "participants": 5,
  "succeeded_in_time": 4,
  "accounts_created": 0,
  "key_prompts": 0,
  "assisted": false,
  "recording": "path or link to the screen recording",
  "commit": "the svrngs commit the run used",
  "notes": "what went wrong for whoever did not finish"
}
```

`succeeded_in_time` counts people who did the demo's thing within its time limit
with no help. Any account created, any key prompt, or any help from the
operator makes the run's value zero, whatever the count.
