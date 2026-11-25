# notes-to-forms 0.3.1: three reasons claims didn't go through

Branch `fix-claims` (from `main` @ c6b4e86): `9908cad`, `14ce398`, `eac85ba`.
Each fix was also delivered and trialled alone, from its own branch.
The instrumentation is on `instrument-failures` and is **not** part of this PR.

**Read the "Before you trust the trials" section first.** The gateway answered
claims that production never sent, so two of these "improved" verdicts were
not judged against recorded answers.

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
service. All four trials ended `"verdict": "passed", "note": "it observed and changed no outcome"`,
with moves `submitted→submitted 22, rejected→rejected 3, error→error 5`.

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

## Trials (quoted from `/trials`; 30 inputs each, release 0.3.0)

| Trial id | Candidate | moves | improved | regressed | unjudged | verdict |
|---|---|---|---|---|---|---|
| `2026-11-25T20-35-43-264Z-70e9380f` | instrument @26ec1f2 | submitted→submitted 22, rejected→rejected 3, error→error 5 | [] | [] | [] | passed, "it observed and changed no outcome" |
| `2026-11-25T20-38-39-663Z-81beee78` | instrument @2e80f9d | same | [] | [] | [] | passed, same note |
| `2026-11-25T20-40-50-850Z-51b9182f` | instrument @6210e13 | same | [] | [] | [] | passed, same note |
| `2026-11-25T20-41-38-735Z-4ef361d3` | instrument @ee1242b | same | [] | [] | [] | passed, same note |
| `2026-11-25T20-39-48-062Z-2f612659` | fix-missing-plan @1f34987 | submitted→submitted 22, rejected→rejected 3, error→error 3, **error→held 2** | f6e9bf0aff7b, fd47233a0618 | [] | [] | passed |
| `2026-11-25T20-40-23-571Z-ad775a52` | fix-minutes-unit @4f37dbe | submitted→submitted 22, **rejected→submitted 3**, error→error 5 | 7f30b7118961, 451520f309d7, 3d0c98f71318 | [] | [] | passed |
| `2026-11-25T20-43-13-870Z-b1d5fbf3` | fix-date-written @3c644b7 | submitted→submitted 22, rejected→rejected 3, **error→submitted 3**, error→error 2 | ee11d561ad0e, 194941a5fabf, 26edf09a60c2 | [] | [] | passed |

Every trial reports `effects_unrecorded: {release: 0, candidate: 0}` and
`telemetry: {release: [], candidate: []}`. Telemetry from the trial arms showed up in
`/service/telemetry` a few seconds later, tagged with the candidate version.
The combined `fix-claims` branch was **not** delivered (see below).

## Before you trust the trials

The trial contract says a call answers with what production recorded for the
exact request, a request production never sent gets 599 and is "unjudged", and
nothing reaches the clearinghouse. Here is what `/service/effects` showed:

- **Requests production never sent were answered 202.** In the date trial's candidate arm (20:43:16) there are 28 POSTs. Three of them carry request hashes that don't appear in production (`ebb2c64162f1`, `18c075707153`, `a9696db92cf2`), each answered 202 with a claim-receipt-shaped body (len 21). Those notes errored in production before any submit, so no recording exists for them. They should have been 599 and unjudged, but `unjudged` is `[]`. The minutes trial reports the same pattern (`rejected→submitted 3`, `unjudged: []`). Its effect rows had already scrolled out of the 100-row view when I looked.
- **Answers to a repeated request differ.** Take request `640146405105`. Its production answer has keyed hash `aa04deadcc`. In four trial arms it was answered `efe5662572`, `d6a4d47413`, `95d72b700c` and `2c4eee187b`. A replay from the recording would give the same answer each time. The 422 answers do repeat exactly (`36b033d3aa7f` → `8bb4ebedc9` in production and in every arm), so the hash key is stable and the 202 bodies really are new each time.
- **Every trial arm's calls are written into the production effects log** with no trial marker: two batches of about 25 per trial.

I can't tell from here whether the gateway passed trial calls through to the
live clearinghouse or made up the 202s. If it passed them through, each trial
arm filed duplicate claims: 7 trials × 2 arms × about 25. Either way, the
improvements for causes 2 and 3 were **not** judged against recorded answers.
For that reason I stopped delivering, and the combined branch is untrialled.

## What the trials do and don't establish

- **Do:** the instrumentation changed no outcome (4 trials). The plan fix turns the two crashes into `held` without touching anything else (`error→held 2`, nothing regressed). That move happens before any outside call, so the gateway issue doesn't affect it. On the clinic's real 30 inputs, each fix changes only the inputs it targets, and none of the 22 claims that went through moved.
- **Don't:** that the clearinghouse accepts the corrected claims. For the minutes and date fixes, "submitted" means only that the gateway answered 202 to a request it had no recording for.
- **Don't:** that the combined branch behaves as the sum of its parts. It wasn't trialled.
- **Don't:** a rate. This is one run of one week's 30 notes. Other ways of writing dates or minutes may exist and aren't covered. Every unhandled form still fails as before.
- **Not judged:** whether the remark after `" - "` in a date carries billing meaning, such as telehealth, which can change place of service or modifiers. I deliberately didn't probe the remark's words. The fix drops it from the date.

## What would settle it

1. The clinic checks the gateway and the clearinghouse for claims filed at the trial times (20:35–20:43 UTC on 2026-11-25), voids any duplicates, and fixes the gateway so that requests with no recording get 599.
2. Once the gateway is fixed, re-trial `fix-claims`. Causes 2 and 3 should then show as **unjudged**. The clearinghouse can only judge them on a real submit after merge and install, or the clinic can send one corrected claim by hand and see it accepted.
3. The clinic fills in the plan for the two held clients (`held.reason=plan_missing`) and resubmits.
4. Someone at the clinic reads one of the three date remarks and says whether it carries billing information.
5. The clinic approves the contract additions on `instrument-failures` (or a trimmed set), so later trials can show those attributes as written instead of as shapes.

## Follow-ups not in this PR

- `server.py` still logs `claim rejected for <client_id>: <body>`, and every span carries `note.file`. Both send identifiers to telemetry, where they arrive only as shapes. They should be dropped.
- The service keeps `seen` only in memory, so a restart re-submits every note in the inbox.
