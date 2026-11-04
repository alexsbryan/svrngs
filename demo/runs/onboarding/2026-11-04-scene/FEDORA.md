# Mac–Fedora arrival observation, 2026-11-04

This is a two-machine observation, not the completed three-laptop scene.
The Mac serves the prepared canonical page and resolver over tailnet HTTPS;
the operator enabled certificate issuance. Fedora is x86_64, already runs
svrngs through its systemd user service, and its binary is
`/home/alexbryan/.local/bin/svrngs` despite not being on the interactive PATH.
The operator reported HTTP 200 from Fedora for the staged installer with
normal certificate verification.

The dedicated Mac host created **Welcome room Nov4**, as **Mac host**, using
the prepared scene build. Its real folder is
`/Users/alexsbryan/svrngs/Welcome room Nov4`. Publication through the native
authenticated boundary returned an open short address. The folder contains
`WELCOME.txt` and `mac.txt`. Invitation capabilities and credentials are not
included in this record.

After opening the invitation on Fedora and following the installation path,
the operator reported:

> It seemed to pretty much work, but once we're in the browser page (where I
> have three different rings) It becomes a bit overwhelming and I don't get
> any of that sense of omotenashi / welcome to your ring, add a haiku vibe

This establishes an observed arrival/orientation failure with three groups.
The existing home-page implementation scrolls to `#group-<id>` after reading the groups
(`home/page/home-page.js:48-51`), then renders all group sections and the
management places (`:70-103`). An anchor is not a focused welcome.

The operator then clarified:

> But it's worth stating that the file sync did work (I ended up adding the
> haiku via cli)

The Mac's shared folder contains `alex_file.txt`, with the operator's
three-line contribution. Its observed SHA-256 is
`38b734f1fb7c8bf5b582eda83191628245858193f472ca71ba6174c2544941bf`
(`shasum -a 256` on that file, exit 0). This corroborates the operator's
report that the Fedora contribution arrived on the Mac. **File sync worked;
the arrival/welcome experience fell short.** Creating the contribution via
CLI is not evidence of an unassisted browser-to-folder welcome. No transfer
latency or success rate is inferred. Exact membership count, interrupted
recovery and closure are still unobserved in this two-machine run.

The next correction is `onboarding-welcome` in the queue. Keep the original
scene predictions intact; this observation is evidence against treating the
previous arrival implementation as a finished welcome.
