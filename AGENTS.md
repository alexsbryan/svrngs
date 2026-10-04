# Agent instructions — svrngs

Read `docs/DESIGN.md` first: §1 says what svrngs is, §2 how we build and the
missteps each rule came from, §8 the order of work. svrngs is also held to two
documents pinned verbatim from commonwealth-ai in `docs/principles/` (`SOURCE`
names the commit each came from):

- `docs/principles/ARCH_PRINCIPLES.md` — the twelve principles. Read it whole in
  your first session here, and reread the principle at stake before deciding.
- `docs/principles/RUST_ARCH.md` — types encode the domain, ownership encodes
  the architecture, I/O lives at the edge.

Where this file and those disagree, they win. Forked code cites principles by
their old section numbers (ARCH §18.1); the table at the end of
`ARCH_PRINCIPLES.md` maps them. Work is the next unit in DESIGN §8; each unit
there moves the demo that follows it, and work that moves no demo waits.

## What they mean here

1. **Glassbox.** Every decision branch traces at debug, the core's refusals
   included; the engine, which is pure, returns its decisions for its caller
   to trace.
2. **Don't whack moles.** Instrument and reproduce before a fix. A move that
   preserves behaviour is its own commit, apart from any change in behaviour —
   a forked piece lands with its behaviour kept, then changes (DESIGN §4).
3. **Write for the next reader.** A doc changes in the commit that changes what
   it describes; a comment says why, in a way someone can check. A change of
   direction rewrites `docs/DESIGN.md`; it never adds a section that overrides
   the rest.
4. **Cite, don't recall.** A claim in a commit body cites a file and line, a
   command, or a run.
5. **Watch every gate fail.** Every check here failed on a planted input before
   it passed; a new check lands with its plant named in the commit.
6. **Never silently substitute.** Four verdicts: passed, failed, could-not-judge,
   never-ran. An absence is reported, never defaulted.
7. **Validate the instrument.** A people bar is a recorded run (`demo/`); one
   run is not a rate.
8. **One decider.** Each threshold, key and schema has one home. A forked file
   and its original are a fork across two systems, not two deciders in one.
9. **Closed sets are enums.**
10. **Structural, not remembered** — and RUST_ARCH: parse, don't validate; the
    core is pure and synchronous; ownership is a tree, no shared mutable state;
    a trait only at a real boundary; `thiserror` in crates, `anyhow` only in the
    `svrngs` command and the app. New code is held to this from its first line,
    forked code as it becomes ours (DESIGN §5).
11. **The inventory outranks the plan.** Before writing a type or module, look
    here, then in commonwealth-ai's ring code. What exists there is forked, not
    rewritten and never linked: it lands with `ported from commonwealth-ai
    <path>@<commit>` on every file, and `scripts/ports.py` lists the pairs
    (DESIGN §4).
12. **Draw the line by ownership.** A group is not a mesh; app-specific code
    lives only in `apps/`; a chat adapter lives in its own package.

## Done

`scripts/check.sh` exits 0. It runs the principle pins, the lean check
(standalone, no I/O in the core or engine, the RUST_ARCH rules a program can
decide, the size budget), rustfmt, clippy and the tests. A size-budget raise is
the operator's, with the reason in the commit.

Ship code, not prose. Commit freely; never push without the operator. Commit
messages carry no assistant attribution.
