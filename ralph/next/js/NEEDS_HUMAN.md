# svrngs js: for the operator

## A refused Stop on the home page is never said beside it (js-types, 2026-11-02)

Found while typing `crates/svrngs-node/src/home/page/acts.js` (6f18511e).
Kept as it runs: saying it is a change a person would see, which this
queue does not make (fence 3).

**What happens.** On the home page's "What apps use", Stop sends
`{ type: 'stop', lend, app }`. ACTS runs the `stop` effect with
`failed: 'unstopped'` (home/page/acts.js:270). When the post fails,
`machine` makes the failed message as `{ type, n, why }` (ui/machine.js:191),
so it carries no `lend` and no `app`. The `unstopped` step files the why
under `` `${m.lend} ${m.app}` `` (home/page/acts.js:274), which is the key
`"undefined undefined"`. `Uses` reads `said[`${lend} ${app}`]` beside each
Stop (ui/uses.js), so the refusal is never drawn. It is traced to
`window.__lines`, but no person sees it.

**The phone does it right.** Its io answers its own refusal with
`{ type: 'unstopped', of: `${l.label} ${app}`, said, why }`
(web/page/uses.js:188), and its step files it under `of` (:97).

**The choice.** Do it the phone's way: the home page's `stop` io catches
its own failure and answers `unstopped` with `lend` and `app`. A refused
Stop is then said beside its button. A person would see that, so it is
yours to choose. The fix is a few lines, and its test is a browser test
that refuses a Stop.
