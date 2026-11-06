# Audience UI — independent parallel work, no new feature scope

Read `../onboarding/SCOPE_PARALLEL.md`, `CHARTER.md`, and
`docs/LENDING_SCOPE.md`. This queue supplies only the UI half of the
main queue's scoped-lending implementation. Work only in this worktree.

- [x] onboarding-scope-ui 2f22da567 — depends [] — implement the existing scope UI against the frozen API; meaningful focused Node checks and commits, no cargo or full check.sh:
  - Explicit stable-ID group choices for new lends; no implicit all-groups
    selection. Show the selected rings, paused [] and legacy null requiring
    selection on owner rows. An audience-only edit preserves other settings;
    a new join never silently selects the new ring. Reuse HOME's group data,
    current disambiguation and typed machines/refusal/save controls.
  - Use exactly the `/running`, `/lent` and discovery-version contract in
    `../onboarding/SCOPE_PARALLEL.md`. Own UI JS/JSDoc, the phone discovery
    header, Node/browser test drivers and registration, and only needed
    `home.rs` served-module entries. Backend owns other Rust and the wasm
    `current_version()` export; do not invent a different endpoint or shim.
  - First prove the old all-groups/default behavior fails the new check.
    Add Node laws/fixtures for choices, sent IDs, no expansion, paused/legacy
    state, save refusal/retry and retained controls. Prepare the authenticated
    browser proof/registration, but defer its Rust compilation/native run and
    backend-generated wasm types to final integration. Run no cargo here and
    never claim those pending integration checks passed. Keep existing welcome
    and grouped-home surfaces intact.
  - Run the relevant Node laws, JS lint/format and applicable types, lean
    before/after, and the named JS budget plant. Only necessary actual growth
    within the original combined +235JS bound is approved, with incremental
    one-past JS cap adjustments. Do not edit the non-JS/core budget rungs.
    Record actual growth, checks and pending integration in a scoped run
    README, commit explicit owned paths, then `ralph-result done`.
  - Do not merge/push main, change personal data, start services or deploy.
    Main integration waits for this queue's done marker and clean branch,
    then merges and runs the final integrated gate and real-node scene.
