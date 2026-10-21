<!-- section: oidc after=intro -->
This queue makes svrngs an OpenID Connect provider for its lends: the
trial, after the demo evening. Before a row, read this queue's
`CHARTER.md` and `STATE.md`'s opening, and svrn-docs
`rings/current/SIGN_IN_WITH_SVRNGS.md` whole. This is the trial only:
what STATE.md lists under "Waits" is not built here.

Fences the rows keep:
1. *Instrument and reproduce first* (AGENTS.md 2 and 5). Each change starts
   red, named in its commit, and every new check fails first on a plant it
   names.
2. *The app learns only what it already hears* (SIGN_IN_WITH_SVRNGS §3 1,
   §5). The claims are the door's headers: `sub` the person's key,
   `preferred_username` their account, `name` their name, `groups` member
   or guest. No consent screen, nothing typed, no `email`.
3. *Nothing leaves the lender's computer* (§3 4). The ES256 key stays in
   the data dir at 0600; codes stay in this run's memory; tokens are never
   in the record, never on disk, and on the wire only to the app.
4. *Only people the door would carry* (§3 3, §8 7). A stranger or a
   removed member is refused before any code. A code is bound to the
   person, the client, the redirect and the PKCE challenge, and is used
   once.
5. *No new listener, no widowed floor* (§3 5). The four routes live on the
   record API's listener; the authorize path on the door. The trust floor
   (`scripts/trust_floor.txt`) is unchanged, and a widening is
   NEEDS_HUMAN.md.
6. *Standard OpenID Connect, nothing app-specific* (§3 5). svrngs learns no
   app's API; an app's quirk is met with standard claims, never with code
   in that app's words. A stock client that needs app code in svrngs is
   NEEDS_HUMAN.md.
7. *One decider* (AGENTS.md 8). The token's claims are svrngs-core's; the
   person is the door's `Asker`; the lender's declaration is the one place
   a lend's client id and secret live.
8. *Glassbox* (AGENTS.md 1). Every refusal traces at debug, naming the fact
   that refused it, and the person is told in words they can act on.
9. *The budget:* the economizing pass goes first; the ceiling is asked
   after it, and each row raises only what it needs, in its own commit.
   `apps/` is not counted.
10. *The apps are external and pinned.* Jellyfin's SSO plugin and Forgejo
    are installed at recorded versions for the run; Podman, never Docker
    (the operator, 2026-10-20).
11. *One host, one run of each:* not a rate (AGENTS.md 7). A run that could
    not finish is could-not-judge, never a pass (AGENTS.md 6).
