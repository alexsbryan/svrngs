# notes-to-forms 0.3.1: three reasons claims didn't go through

Branch `fix-claims` (from `main` @ c6b4e86): `9908cad`, `14ce398`, `eac85ba`.
Each fix also has its own branch.
The instrumentation is on `instrument-failures` and is **not** part of this PR.

The trial of record is `2026-11-25T20-54-30-818Z-b2d84a6d`, run on
`fix-claims` after the clinic fixed its trial sandbox. Every trial from
before that fix is void; see "Earlier trials: void".

## What production showed

`/service/telemetry` for release 0.3.0 covers 30 notes: 22 `submitted`, 3 `rejected`
and 5 `error`. `/service/effects` shows 25 POSTs to `/claims`, 22 answered 202
and 3 answered 422. That leaves 8 notes that didn't go through, in three groups.
The span status, exception type and frames below passed as written. Messages
arrived only as `dropped`, and log bodies arrived only as shapes.

| # | Cause | Inputs (`input.ref`) | Production evidence |
|---|---|---|---|
| 1 | The client record has no plan (NULL `plan_id`) | `f6e9bf0aff7b`, `fd47233a0618` | `lookup` span error, `AttributeError` at `clients.py:10 plan_of` |
| 2 | Minutes written with a unit (`"<n> min"`) | `7f30b7118961`, `451520f309d7`, `3d0c98f71318` | `submit` span `http.response.status_code` 422, `outcome=rejected`, and three effects with status 422 (body len 202/202/203) |
| 3 | Service date written as `Mon d, YYYY - <remark>` | `ee11d561ad0e`, `194941a5fabf`, `26edf09a60c2` | `parse` span error, `ValueError` at `notes.py:23 parse` → `_strptime` |

### Making the why observable (instrumentation, `--for instrument`)

The production view gives the place but not the reason, so I shipped observation-only
candidates on `instrument-failures`. `observe.py` maps each value to a member of
a closed set declared in `recipe.json`, and only that member leaves the
service. All four instrumentation trials ended "it observed and changed no outcome", but those trials are void (see below).

The trial note says `"shown for review; this trial projected by the installed contract"`,
so the new attributes still arrived as shapes. They can be decoded because
the members of each declared set have different lengths. From round 2 on I
emitted yes/no flags under keys that name the question (keys pass as written;
`yes` is len 3 and `no` is len 2). Read from `/service/telemetry`, spans tagged
with each candidate version:

- **Cause 1** (@26ec1f2): `lookup.plan_id.kind` len 4 = `none`, and `lookup.plan_id.case` len 7 = `not_str`, for exactly the two inputs. The production log bodies agree: lengths 93 and 98 = `note ` + file name (37, 42) + ` failed: ` + 42, and 42 is the length of `'NoneType' object has no attribute 'upper'`.
- **Cause 2** (@26ec1f2, @2e80f9d): the rejected claims match the 22 accepted ones in every field form except minutes. `claim.minutes.form` is len 10 = `non_digits` against len 6 = `digits`. They show `minutes.is.digits_space_min=yes`, `has.letter=yes`, `has.space=yes`. In the rejection's own words `reject.says.minutes=yes` and every other field is `no`; its keys are `code` and `error`.
- **Cause 3** (@26ec1f2 → @ee1242b): `parse.date_form` len 11 = `has_letters`, and `parse.error_form` len 14 = `does_not_match`. The value starts with a month abbreviation and has one four-digit year, no ISO date, no weekday, two commas and a spaced dash, and is longer than 35 characters. It parsed under none of the 12 guessed formats. Split at `" - "`, `date.head.parses_as.abbr_month_d_comma_y=yes`. The tail has a comma but no digit and no month word.

## The change

1. `9908cad` **hold, don't crash, on a missing plan.** `clients.plan_of` raises `MissingPlan` when `plan_id` is NULL or blank. `server.one` records `outcome=held` and `held.reason=plan_missing` (declared in the contract), and logs the input ref instead of the client id. Acme can't know the plan, so these claims still need the clinic to complete the two client records.
2. `14ce398` **minutes.** `notes.minutes` keeps the digits of `<n> min|mins|minutes`. Every other form is passed on as written, as before.
3. `eac85ba` **service date.** `notes.service_date` tries ISO first, then the part before `" - "` as `%b %d, %Y` or `%B %d, %Y`. Every other form still raises `ValueError`.

## Tests (all fixtures synthetic)

`python3 -m unittest discover -s tests`: 8 tests pass on `fix-claims`.

- `tests/test_clients.py`: on `main`, 2 errors (AttributeError). Passes here.
- `tests/test_notes.py` `Minutes` and `fixtures/minutes-with-unit.txt`: on `main`, 1 failure and 1 error. Passes here.
- `tests/test_notes.py` `ServiceDate` and `fixtures/date-written-with-remark.txt`: on `main`, 3 errors. Passes here.

## Trial of record (quoted from `/trials`)

`2026-11-25T20-54-30-818Z-b2d84a6d`. Candidate `0.3.1-rc.1` from `fix-claims@eac85ba`, release 0.3.0, 30 inputs. `"verdict": "passed"`.

- **moves:** `submitted→submitted 22`, `error→held 2`, `rejected→unjudged 3`, `error→unjudged 3`
- **improved:** `f6e9bf0aff7b`, `fd47233a0618` (error → held)
- **regressed:** `[]`
- **unjudged (6):** `7f30b7118961`, `451520f309d7`, `3d0c98f71318` (rejected → unjudged) and `ee11d561ad0e`, `194941a5fabf`, `26edf09a60c2` (error → unjudged)
- **effects_unrecorded:** `{release: 0, candidate: 6}`
- **partly:** "6 input(s) now send what was never sent, so the replay can't say how the outside answers them"

Each arm's telemetry, as it appears in the trial record:

- **Release arm** (120 records, all 0.3.0): it reproduces production exactly. 22 submitted. 3 rejected, each with a `submit` span returning 422. 3 errored with `ValueError` in `parse`. 2 errored with `AttributeError` in `lookup`. 8 logs.
- **Candidate arm** (120 records, all 0.3.1-rc.1): 22 submitted, the same inputs as the release arm. The 3 written dates now parse, and the 3 minutes claims now go through `lookup`. All 6 reach `submit` and get `http.response.status_code` 599, so they are unjudged. The 2 no-plan notes end at `lookup` with `clients.MissingPlan` and `outcome=held`. There are 2 WARN logs (len 49, the held line), and no ERROR log.

Isolation held this time. `/service/effects` still has its 25 production rows, the last at 20:53:54, before the trial. `/service/telemetry` holds only 0.3.0 production records.

I didn't re-deliver the single-cause branches. The three fixes touch disjoint inputs, and the combined trial attributes each move to its input, so separate trials would show nothing new. They would only add more runs.

## Earlier trials: void

Seven earlier trials ran before the clinic fixed its sandbox: four instrumentation trials (`instrument-failures` @26ec1f2, @2e80f9d, @6210e13, @ee1242b), and one each for `fix-missing-plan` @1f34987, `fix-minutes-unit` @4f37dbe and `fix-date-written` @3c644b7.

The clinic's machine couldn't create a network namespace. systemd logged "PrivateNetwork=yes is configured, but ... proceeding without" and ran each trial arm with the network. Each arm's collector and gateway lost their ports to production's, so every replayed claim went through production's gateway to the real clearinghouse. The clearinghouse's own log counts **353 duplicate claims**, 50 per trial.

That accounts for three things I saw at the time:
- the 202 answers to requests production never sent;
- 202 answers that differed on every repeat;
- trial telemetry and effects showing up in the production views.

Those trials' "improved" verdicts for minutes and dates were real clearinghouse answers, given to duplicate claims. They weren't replayed answers. Those records are gone from `/trials`, and none of them is evidence for this PR.

The site has since been fixed:
- Before any trial, the site checks `ip -o link` under the sandbox's exact properties and refuses the trial unless only loopback is present. Each arm checks again.
- Trial arms now use their own collector and gateway ports, so they can't reach production's.
- A planted machine that can't isolate had its trial refused, and the clearinghouse count didn't move.

The cause analysis above rests on telemetry from those voided runs: the projected attributes of the instrumentation arms. Those observations are still valid as observations of the clinic's inputs, which are unchanged after the restage (same 30 notes and client records). But the arms that produced them also filed duplicate claims. That instrumentation hasn't been re-run under isolation.

## What the trial does and doesn't establish

- **Does:** the plan fix turns two crashes into `held`, before any outside call (judged, improved).
- **Does:** none of the 22 claims that went through moved, and nothing regressed.
- **Does:** the date and minutes fixes do what they were written to do on the clinic's real inputs. All 6 notes now build a claim and send it.
- **Doesn't:** that the clearinghouse accepts those 6 corrected claims. They are **unjudged**: the replay has no recording for requests production never sent.
- **Doesn't:** a rate. This is one run of one week's 30 notes. Other ways of writing dates or minutes may exist and aren't covered. Every unhandled form still fails as before.
- **Not judged:** whether the remark after `" - "` in a date carries billing meaning, such as telehealth, which can change place of service or modifiers. I deliberately didn't probe the remark's words. The fix drops it from the date.
- **Held is not "went through":** the 2 held claims need the clinic to add a plan to two client records.

## What would settle it

1. The clinic voids the 353 duplicate claims filed during the void trials (its clearinghouse log has them).
2. After merge and install, watch production's `submit` spans for the 6 inputs, or the clinic sends one corrected claim by hand. Either shows 202 or 422 for the corrected form. A judgement can come only from the real clearinghouse.
3. The clinic fills in the plan for the two held clients (`held.reason=plan_missing`) and resubmits.
4. Someone at the clinic reads one of the three date remarks and says whether it carries billing information.
5. The clinic approves the contract additions on `instrument-failures` (or a trimmed set), so later trials can show those attributes as written instead of as shapes.

## Follow-ups not in this PR

- `server.py` still logs `claim rejected for <client_id>: <body>`, and every span carries `note.file`. Both send identifiers to telemetry, where they arrive only as shapes. They should be dropped.
- The service keeps `seen` only in memory, so a restart re-submits every note in the inbox.
- A held note's `lookup` span is marked error, because `MissingPlan` passes through the span. Its outcome is `held`. Recording that span as ok would be clearer.
