# The grapevine canary's "names Sam", 2026-11-16

The whole gate at `46f002f3`, run for V1.10 on the Mac under the gate lock,
failed one step of 45, app laws, on one law there:
`apps/grapevine/canary.test.mjs` 'the canary runs §10's scene on
demo.json, each step timed', with "the
records after: the carries are Jo, Jo, Jo's, Waller's log names Sam"
(`gate-failure.txt`). Every step of the scene had held. The other session's
whole gate at the same code passed it (`../../test-audit/2026-11-16/gate.txt`),
and 20 runs of the test alone passed.

## Why

`canary.mjs` judged "Waller block names Sam nowhere" as
`JSON.stringify(waller).includes('Sam')`. Waller's log holds sealed boxes,
nonces and handles in base64, about 2,400 characters of it each run, and
base64's letters spell any three-letter name now and then.

Measured: 400 runs of `node apps/grapevine/canary.mjs`, 8 at a time, each
leaving its scratch `demo.json`; none failed. Across their Waller logs,
four other names stood in for Sam. "Zed" and "Bob" were each found in 2
runs, and "Max" and "Lou" in 1 each: 6 in 1,600 name-runs, about 1 run in
270 for any one name. Each was inside a sealed field:

```
Zed in op.payload.box "/ebgYZAACQvIZedcwSaGJXijIAD" len 128
Bob in op.payload.box "E4eL4zVCyTVhBobbbluFdIuY+6P" len 128
Max in op.payload.box "7CVcxc/rzvnPMaxzxE2cKMECeZc" len 64
Bob in op.payload.nonce "3BobzpMIP8gIiese" len 32
Zed in op.payload.nonce "QZedfqnLqNU0uPFI" len 32
Lou in op.payload.box "wbBbXG3qrgDZLouVqNMlyw2ikyh" len 68
```

So the gate's failure was ciphertext spelling "Sam", not a leak. Steps 3
and 7's `!has(kim, 'Sam')` and `!has(seen, 'Sam')` read Kim's view, which
carries base64 handles too, so they could fail the same way.

## The change

`named(v, who)` in `canary.mjs` counts a name only where no base64
character (`A-Z a-z 0-9 + /`) touches it, and the three Sam checks use it.
The full sealed words (`SEALED`, "Sam Reyes" among them) are still matched
as substrings, since nine characters or more don't turn up by chance.

## Plants (`plants.txt`)

Each plant adds one op to Waller's log, just before the final check, in a
scratch copy of `apps/grapevine`:

| Plant | Check | Exit |
|---|---|---|
| a box `q3abSamd+/x9` | the old `has` | 1, "Waller's log names Sam": the gate's failure, made on purpose |
| a box `q3abSamd+/x9` | `named` | 0 |
| `asker: 'Sam'` | `named` | 1, names Sam |
| `note: "Sam's saw"` | `named` | 1, names Sam |
| an op written as `person: 'Sam'` | `named` | 1, names Sam |
| none | `named` | 0 |

`node --test apps/grapevine/canary.test.mjs` passed 3 runs of 3 after.
