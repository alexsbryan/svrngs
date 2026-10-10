# Group folders' run (gf-1), kept as their record

`run.sh` and its helpers ran group folders' second spike at `575a911`
(`demo/runs/gf/2026-10-09/README.md` §7): a folder's films played in one
Jellyfin from `.strm` stubs at the meeting point's loopback endpoint.
Streaming was dropped by syp-1 (operator, 2026-10-10, "Drop streaming +
share"; `git log --grep syp-1-no-stream`): the endpoint, `SVRNGS_FOLDER_PORT`,
`GET /v1/folders` and the one-way `share --folder` these scripts call are
gone, so they do not run after it. They stay as the record of what ran.
