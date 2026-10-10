# syp-1: sy-1's S1-S4 rerun with streaming dropped — 2026-10-10

`python3 demo/runs/syp/2026-10-10/no-stream/run.py` at `12d0ba2`, the
commit that dropped group folders' endpoint and one-way share
(`ralph/next/syp/STATE.md`, syp-1-no-stream). It runs
`demo/runs/sy/2026-10-09/run.py`'s setup and its `s1`-`s4` as they ran,
each plant first, without M1 and M2, its data in `run/`; its one change
is the readiness probe, `GET /v1/groups` for the dropped `GET /v1/folders`
(the script's docstring). Load averages 54.30 52.03 39.88 at its start
(`run/events.jsonl:1`). One host, one run of each: not a rate (AGENTS.md 7).

| Bar | Plant, its guard out | Bar, guard in | sy-1's (`demo/runs/sy/2026-10-09/README.md` §7) |
|---|---|---|---|
| S1 both ways | `deaf`: Ana's add reached no one in 10 s, `add_s` null, failed | **passed**: add 1.13 s, edit 1.67 s, delete 1.19 s | 1.06, 1.63, 1.36 s |
| S2 its author asleep | `own-only`: Cy held nothing in 10 s, from no one, failed | **passed**: Cy held it 4.27 s after her command, from Ben | 4.47 s, from Ben |
| S3 conflicts kept | `later-wins`: all three held Ana's edit alone (`e59a14e6…`), Ben's in none, failed | **passed**: 17.42 s after Ben's start, `s3.txt` Ana's `36f7c2fd…`, `s3 (conflict Ben).txt` Ben's `eb066e59…`, in all three | 17.0 s |
| S4 members only | `open-gate`: Dee's member connection open after 5 s, the bytes given whole (`payload` 65,536), 2 hits in her data dir, failed | **passed**: Dee refused on both protocols ("not admitted"), 0 hits in her data dir, 0 of the planted file anywhere | the same |

Every number is in `run/summary.json` by its bar's key. The synced folder
does with streaming dropped what it did before it, on this host.
