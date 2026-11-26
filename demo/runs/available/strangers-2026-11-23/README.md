# Which share comes back? — 2026-11-23

**Task: failed in both arms.** Neither simulated stranger sent Cy an answer
before the scene ended. Hana eventually produced the expected distinction on
the built page and its inverse on the plant, but neither answer was sent.
Whether strangers can reliably tell the two kinds apart in use is
**could-not-judge** from this rehearsal. The real-people bar is **never-ran**.

[Run-of-show and prediction](../../../available-strangers.md), frozen at
`b9915ef5`. The answer in either arm is **notes** comes back after sign-in;
**coder** does not without action. [The fixture](scene.mjs) uses the built
Home HTML and modules over `svrn-docs`'s Maple St house world
(`rings/current/play-lend-give/world/`, `d79f8b3d`). It stubs `/lent` and
`/run` and refuses writes. Only the two status strings are exchanged in
the plant; the app names and controls stay in place. The copy comes from
`crates/svrngs-node/src/plan/words.rs:18,52`. This exercises the two labels
in Home, not the live API or the restart itself.

## Browser control: red, then green

Inside `svrngs-gate`, with `XENIA=/xenia` (`d7600a54`) and
`PLAY_LEND_GIVE_WORLD=/play-lend-give/world`, from `/work`:

```text
$ AVAIL_ARM=plant node demo/runs/available/strangers-2026-11-23/run.mjs check
plant: coder: Run by svrngs. Run it with svrngs; notes 8099 · Shared while you run it.Stop notes
AssertionError [ERR_ASSERTION]: The input did not match the regular expression /^Shared while you run it\./. Input:

'Run by svrngs. Run it with svrngs'
```

Exit 1 (`avail-rehearse-2.out:535–554`, also recorded in `b9915ef5`).

```text
$ AVAIL_ARM=built node demo/runs/available/strangers-2026-11-23/run.mjs check
built: coder: Shared while you run it. Run it with svrngs; notes 8099 · Run by svrngs.Stop notes
```

Exit 0: the launcher uses `bash -ec` and went on to the built play.
Both checks opened coder's collapsed share details; the notes status was
already visible under **Already running here**.

## Play: planted wording first

The same container ran `AVAIL_ARM=plant node
demo/runs/available/strangers-2026-11-23/run.mjs play --variant max`, then the
built browser check above, then the play command with `AVAIL_ARM=built`.
Both used fresh Hana and Ivo sessions. Xenia's driver sends
`reasoning_effort: 'max'` to DeepSeek through DeepInfra (`run.mjs:46–60`).
The launcher exited 0 after 11m29s; that means it collected the runs, not
that the task passed. From `avail-rehearse-2.await.log`, with the built
browser check quoted above:

```text
plant: deepseek-ai/DeepSeek-V4.1-Flash-1017 at deepinfra, reasoning_effort=max
plant/hana: attempted change: /run/action
Hana: failed (ran out of patience), 0 chat reply(s)
plant/ivo: attempted change: /run/action
Ivo: failed (ran out of patience), 0 chat reply(s)
built: deepseek-ai/DeepSeek-V4.1-Flash-1017 at deepinfra, reasoning_effort=max
built/hana: attempted change: /run/action
Hana: failed (ran out of patience), 1 chat reply(s)
the house chat: notes — it's run by svrngs, so it'll be up after I sign back in. coder isn't running and I'd have to start it myself, so it won't be open tomorrow unless I do that.
Ivo: failed (ran out of patience), 0 chat reply(s)
```

| Arm / stranger | Status phrases read in the trace | Attempted `/run/action` | Task verdict |
|---|---|---|---|
| Plant / [Hana](plant/hana/scene/Hana.md) | notes: “Shared while you run it.” The other phrase was shown but not counted as read. | Yes, refused | **failed**, no in-scene reply (78.6 s on Xenia's clock) |
| Plant / [Ivo](plant/ivo/scene/Ivo.md) | notes: “Shared while you run it.”; coder: “Run by svrngs.” | Yes, refused | **failed**, no reply (68.0 s) |
| Built / [Hana](built/hana/scene/Hana.md) | notes: “Run by svrngs.”; coder: “Shared while you run it.” | Yes, refused | **failed**, no in-scene reply (84.0 s) |
| Built / [Ivo](built/ivo/scene/Ivo.md) | notes: “Run by svrngs.”; coder: “Shared while you run it.” | None logged | **failed**, no reply (82.1 s) |

Xenia's `said` list counts an attempted `say`, even after patience runs
out (`src/core/account.ts:58`); chat only sends while the participant is
still acting (`src/core/world.ts:144–148`, both at `d7600a54`). On the
built page, Hana's first `say` came after `end: "spent"`, and Xenia replied
“You've given up” instead of sending it ([trace](built/hana/scene/trace.json#L1650-L1682),
[transcript](built/hana/scene/Hana.jsonl#L32-L33)). In the plant Hana's only
attempt came after the closing “That's all. Thank you.” and isn't even in
the trace ([transcript](plant/hana/scene/Hana.jsonl#L41-L43)):

> coder — it's set to be shared persistently, so you can open it tomorrow
> without me doing anything. notes is only shared while I'm running it, so
> that one won't be there after a restart unless I start it.

Hana's two late answers follow the label swap, but the unchanged controls
also cue what runs, and all four sessions exhausted the same persona
patience limits before answering. No settings were changed. For the people
session, watch whether a newcomer finds coder's collapsed label and can
answer from the two rows without hunting for a startup setting or operating
the controls. The packaged restart proof must still check the old link and
data, separately from this wording rehearsal.
