# Agent instructions — svrngs

svrngs is held to two documents, pinned verbatim from commonwealth-ai in
`docs/principles/` (`SOURCE` names the commit each came from):

- `docs/principles/ARCH_PRINCIPLES.md` — the twelve principles. Read it whole in
  your first session here, and reread the principle at stake before deciding.
- `docs/principles/RUST_ARCH.md` — types encode the domain, ownership encodes
  the architecture, I/O lives at the edge.

Where this file and those disagree, they win. The design is `docs/DESIGN.md`.
The work queue is commonwealth-ai's `quality/campaigns/svrngs-e0.toml`: one rung
at a time, each naming the demo it moves.

## What they mean here

1. **Glassbox.** Every decision branch in `svrngs-node` and the command emits a
   tracing event; `svrngs-core` returns events and never logs.
2. **Don't whack moles.** Instrument and reproduce before a fix. A move that
   preserves behaviour is its own commit, apart from any change in behaviour.
3. **Write for the next reader.** A doc changes in the commit that changes what
   it describes; a comment says why, in a way someone can check.
4. **Cite, don't recall.** A claim in a commit body cites a file and line, a
   command, or a run.
5. **Watch every gate fail.** Every check here failed on a planted input before
   it passed; a new check lands with its plant named in the commit.
6. **Never silently substitute.** Four verdicts: passed, failed, could-not-judge,
   never-ran. An absence is reported, never defaulted.
7. **Validate the instrument.** A people bar is a recorded run (`demo/`); one
   run is not a rate.
8. **One decider.** Floors live in the campaign; values come from
   `scripts/e0_demo.py`; each threshold, key and schema has one home.
9. **Closed sets are enums.**
10. **Structural, not remembered** — and RUST_ARCH: parse, don't validate; the
    core is sans-IO and synchronous; ownership is a tree, no shared mutable
    state; a trait only at a real boundary; `thiserror` in crates, `anyhow` only
    in the `svrngs` command and the app.
11. **The inventory outranks the plan.** Check this repository, then
    commonwealth-ai's code to port by citation. Never link it. A port carries
    `ported from commonwealth-ai <path>@<commit>`; `scripts/ports.py` lists the
    pairs (DESIGN D2b, "Toward commonwealth-ai").
12. **Draw the line by ownership.** DESIGN D2b: app- and provider-specific code
    lives only in `apps/` and `providers/`.

## Done

`scripts/check.sh` exits 0. It runs the principle pins, the lean check
(standalone, no I/O in the core or engine, the RUST_ARCH rules a program can
decide, the size budget), rustfmt, clippy and the tests. A size-budget raise is
the operator's, with the reason in the commit.

Ship code, not prose. Commit freely; never push without the operator. Commit
messages carry no assistant attribution.
