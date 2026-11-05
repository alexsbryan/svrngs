# Native-home working-tree preview — deferred until after the demo

Scheduled from the operator's group-UI worker handback, 2026-11-04.
This queue is not installed or launched; it must not extend the current
onboarding/scoped-lending demo critical path.

The handback distinguishes evidence correctly: B's browser parity proof used
synthetic three-group data and selected mocked actions; broader tests used
isolated real nodes. The worker did not evaluate B against the operator's
actual running groups. Native HTML/JS/CSS are embedded in the executable;
`web/tests/dev.mjs` previews the phone page, not this native home.

- [ ] native-home-dev-preview — depends [] — ONE ROW, give the native UI a usable development loop on real existing groups/identity:
  - **Inventory/economy first.** Read the handback above, native `home.rs`
    asset serving/version replacement, session/authentication policy,
    existing folder confinement and `web/tests/dev.mjs`. Reuse them and the
    current CLI/native boundary; no separate dev web server, frontend build
    system, identity, group copy or unauthenticated API proxy. Survey supported
    cuts before requesting any cap growth; onboarding/lending approvals do
    not fund this deferred feature automatically.
  - **Outcome.** One explicit development command serves working-tree native
    HTML/JS/CSS through the node's existing authenticated origin and real APIs.
    Save a frontend file, refresh, see the change against the real groups and
    identity. A one-time node update/setup may be needed to enable it; routine
    UI edits require neither binary installation nor launchd management.
    Provide an obvious way back to embedded/release assets. Default released
    behavior remains self-contained embedded assets.
  - **Boundary.** This is an explicit owner-selected dev mode, not a group
    content setting. Keep the exact served-asset allowlist, source-root
    confinement, existing origin/session/intent checks and policy headers.
    Preserve version substitution and avoid stale dev caching. Missing or
    unreadable working-tree assets are visible errors, not silent embedded
    substitution. No generic filesystem route or extra listener/CORS grant.
  - **Proof.** Red-first actual native-browser check: edit one served UI file
    on disk, refresh and observe the changed bytes while the same real node,
    identity/groups and authenticated API still work. Release/default mode
    continues serving embedded bytes. Cover denied unauthenticated/foreign
    requests, path escape, missing file and dev-mode exit. Synthetic navigation
    fixtures alone cannot establish this preview workflow.
  - **Close.** Document the one command and minimal one-time prerequisite.
    Provide the operator a working B preview against their existing groups
    without manually deploying a second test installation. Run affected
    checks and the required final gate for the resulting product; keep this
    separate from today's gated demo snapshot. No unsolicited modification,
    migration or restart of the operator's running personal node.
