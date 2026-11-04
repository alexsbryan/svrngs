# Onboarding scene — predictions before data

Prepared against `abc171afbd74`, before any people-scene run. These are
predictions to compare with observations, not results or acceptance thresholds.
The contract is `docs/ONBOARDING.md` §6; the run-of-show is
`demo/onboarding-room.md`. The operator's people bar is still owed.

## Expected observations

- A host can create the address from the group page, copy it, reload and
  recover the same address and controls without using a terminal.
- Each attendee can type that address and recognize the host and group.
  The five-word address may cause transcription mistakes: record every
  correction and request for help rather than quietly fixing it for them.
- Each attendee gives their name once. Native installation does not first
  admit a browser identity. Record the host's membership before and after
  each join, rather than inferring membership from the welcome page alone.
- The fresh installation is likely to spend most of its time downloading
  and starting. Record those steps separately from reading the instructions;
  no completion-time threshold is predicted or approved here.
- The authenticated local page focuses the intended group and exposes its
  actual folder. Attendees can find and open it without spoken directions.
  A same-named preexisting local folder should not receive the contribution.
- Each attendee can create an ordinary text file beside `WELCOME.txt` and
  see the other contributions arrive. File creation is a likely friction
  point for someone unfamiliar with their editor; record that friction.
- After an interrupted join, retrying the same command retains the computer's
  identity and produces no duplicate membership. An existing stopped service
  resumes after refusal/interruption.
- Closing joining rejects a new arrival through both the short address and
  its previously copied ordinary invitation. Already joined computers can
  still exchange a new file. A stale short-address page must not imply that
  closed admission still works.

## What would contradict these predictions

A second name prompt, duplicate person, guessed/wrong folder, help required
to find it, lost identity on retry, admitted newcomer after confirmed close,
or existing members unable to exchange files after close is recorded as an
observed failure, with its step and evidence. Do not revise these predictions
after seeing data. Unexpected provider failures are recorded as such; they
must say joining is temporarily unavailable with retry guidance, not ended.

## Judgment boundary

There are no people observations yet. Local automated proofs from earlier
rows do not judge these predictions. One scene is not a success rate. The
operator must set the people bar before participants start; report assistance,
non-completion and missing evidence explicitly.
