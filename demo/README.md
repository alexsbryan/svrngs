# Demos

Each demo in `docs/DESIGN.md` D7 has a run-of-show here, written before its
first run, and every run leaves one record in `runs/<demo>/<date>.json`, which
`scripts/e0_demo.py` reads. A run nobody recorded did not happen.

A record:

```json
{
  "demo": "d1",
  "date": "2026-10-20",
  "participants": 5,
  "succeeded_in_time": 4,
  "accounts_created": 0,
  "key_prompts": 0,
  "assisted": false,
  "balances_equal_fold": true,
  "recording": "path or link to the screen recording",
  "commit": "the svrngs commit the run used",
  "notes": "what went wrong for whoever did not finish"
}
```

`succeeded_in_time` counts people who did the demo's thing within its time limit
with no help. Any account created, any key prompt, or any help from the
operator makes the run's value zero, whatever the count.
