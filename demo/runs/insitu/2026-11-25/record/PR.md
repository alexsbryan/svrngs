# fix/cp1252-notes: read a note that isn't UTF-8 as Windows-1252

Branch `fix/cp1252-notes`, commit `e0a46f3` on top of `main` (0.2.0, `2a11392`).

## What failed

The clinic reported that some notes never became forms. The clinic's read-only view shows 3 of 20 inbox notes failing, all for the same reason:

- `/service/state`: `"counts": {"parsed": 17, "error": 3}`, `"inbox_seen": 20`, version `0.2.0`.
- `/service/events`: the 3 failures (`d186993bfa60`, `711b697432ee`, `646fc8e57bb4`) have `"outcome": "error"`, `"error_class": "UnicodeDecodeError"`, and all have the same shape: `"bom": null`, `"newline": "crlf"`, `"high_bytes": 5`, `"first_invalid_byte": "0x97"`, `"first_invalid_at_fraction": 0.07`. All 17 that parse have LF newlines and valid UTF-8 (no `first_invalid_byte`), with `"missing": []`.

Why: `notes.read()` decoded strict UTF-8 and nothing else. 0x97 is not valid as a UTF-8 start byte, but it is the em dash in Windows-1252. Together with the CRLF line endings, that points to notes saved by a Windows editor in cp1252. The encoding is my inference from the shape. The view never shows the bytes, and the trial below is what confirms it.

## The change

`notes.read()` tries UTF-8 first. If that fails, it decodes as strict `cp1252`.

- A note that is valid UTF-8 decodes exactly as before, so the 17 that already parse can't change.
- The fallback is strict. A byte that cp1252 leaves undefined (0x81, 0x8D, 0x8F, 0x90, 0x9D) still raises `UnicodeDecodeError` and is never guessed. Such a note stays visible as `error` in telemetry.

## Tests

- New synthetic fixture `fixtures/cp1252-crlf.txt`, written for this PR with no real person in it: CRLF newlines, em dashes, curly quotes and é, encoded cp1252. The existing `test_every_fixture_parses_whole` now covers it.
- `test_cp1252_note_keeps_its_values`: the cp1252 note's five fields come out with the exact expected values.
- `test_utf8_note_is_not_read_as_cp1252`: a UTF-8 em dash still decodes as UTF-8, not as mojibake.

`python3 -m unittest discover -s tests`:

- Before the fix (new tests, old `notes.py`): `Ran 4 tests`, `FAILED (errors=2)`. Both errors are `UnicodeDecodeError: 'utf-8' codec can't decode byte 0x97`.
- After the fix: `Ran 4 tests`, `OK`. CI (`ci.sh fix/cp1252-notes 0.2.1-rc1`) gave the same result: `Ran 4 tests ... OK`.

## In-situ trial (quoted from `/trials`)

```
"id": "2026-11-25T20-07-54-155Z-b58720b4",
"candidate": {"version": "0.2.1-rc1", "recipe": "b58720b499bd", "from": "fix/cp1252-notes@e0a46f3"},
"release": "0.2.0",
"inputs": 20,
"release_counts":   {"parsed": 17, "error": 3},
"candidate_counts": {"parsed": 20},
"fixed": ["d186993bfa60", "711b697432ee", "646fc8e57bb4"],
"regressed": [],
"still_failing": [],
"verdict": "passed"
```

## What the trial does and doesn't establish

**It establishes:**
- On the clinic's real 20 inbox inputs, run sandboxed side by side, the candidate parses all 20 with no missing fields. The release parses 17.
- The 3 inputs it fixes are exactly the 3 `UnicodeDecodeError` events above.
- None of the 17 that already parsed regressed.

**It doesn't establish:**
- **Field values.** The outcome "parsed" means all five labels were found. It doesn't mean each value is what the clinician meant. Neither of us can see the notes or forms, so no one has checked the values against the source. The fields are dates and codes, which are ASCII and the same in UTF-8 and cp1252, so the risk is low. Still, it hasn't been checked.
- **Future inputs.** It covers only these 20 inputs. A note in another encoding (UTF-16, or cp1252 using one of its five undefined bytes) would still fail. It would fail loudly, as `error`, not silently.
- **Installation.** The candidate is not installed at the clinic. The fix reaches the clinic only once this is merged and released, e.g. as 0.2.1.
- **One run.** This is one trial, not a rate.

## Noted, not changed here

- In `server.py` `watch()`, `ev["at"] = round(time.time(), 1)` overwrites the error-location `at` that `one()` sets. So live error events lose where they failed: every `at` in `/service/events` is a timestamp. That is a separate telemetry fix.
- `server.py` still says `VERSION = "0.2.0"`. The release needs a bump to 0.2.1.
- `__pycache__/*.pyc` is tracked in the repo and changes whenever the tests run. It's left out of this commit.
