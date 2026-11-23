# Lending without shared membership — due diligence

Surveyed 2026-11-20, starting at svrngs `3d96e775` on a shared worktree.
Recommendation and proposed proof, not a change to the design in force or
an implementation queue. The existing meaning of a lend to groups is in
[LENDING_SCOPE.md](LENDING_SCOPE.md).

## 1. The boundary

**A provider runs a service; a group can keep its address; the provider
decides who gets to use it.** A library can be the only place that takes a
recording-booth booking without being the place the block or the film club
meets. Two groups can keep the same offer without knowing of each other.
The library going away takes its service away, not either group's record
or ability to meet. A provider is a way to use a node, not a kind of node.

| Fact | Who owns it | What crosses the boundary |
|---|---|---|
| What is offered, at which interface, and who may use it | The provider | A reference people can carry; permission is decided at the service. |
| What a group wants to keep handy | The group, by its existing rules | A reference only. Removing it does not revoke a person's independent access. |
| Who belongs to a group | That group's authority | Nothing by default. Neither a group's ID nor a copy of its offer is a membership proof. |
| Card eligibility, a reservation, capacity and cancellation | The library app or its existing system | Only what that service needs. No card data or booking history in a group record. |

Invitation stays a deliberate act. A neighbor passes on an offer, or the
library publishes it on its own site or at the desk. Neither event installs
it in anyone's group or authorizes other apps to call it. Start with no
provider directory, group-to-group synchronization, or search fan-out.

## 2. What actually exists

| Seam | Evidence | Consequence |
|---|---|---|
| Peer discovery | [`Node::lends_in`](../crates/svrngs-node/src/node.rs#L881-L913) asks standing peers; `relay_of` finds their relay in held groups. [`handing`](../web/page/hand.js#L47-L84) opens a listed lend, signs a member pass, and gets relay information from the group. | An independent offer needs an explicitly supplied reference. Adding a fake entry to the member list will not make the open safe or the dial self-contained. |
| Address | [`Label::of` / `whose`](../crates/svrngs-core/src/pass.rs#L85-L146) round-trip the lender key and distinguish a computer's lend from a carried app. | Reuse the label for an exact offer and give the client the provider's dial hint from an intentionally obtained reference. A relay hint is routing, not a claim about the provider's name. |
| Transport | [`admit_dialer`](../crates/svrngs-node/src/node.rs#L1228-L1259) already lets an outsider reach `Alpn::Lend`. [`holders_router`](../crates/svrngs-node/src/door.rs#L453-L461) is the narrow HTTP router. Member, blob and raw protocols still require standing at ingress. | Reuse authenticated iroh and the lend ingress. No new ALPN, relay role, member protocol or general unauthenticated router. |
| Relay admission | The [group-only relay](../crates/svrngs/src/relay.rs#L102-L157) admits a member or a pass whose signer stands in a held group; an open relay admits either endpoint without this relationship. | Use a direct dial in the seam test and a reachable open relay for the first cross-network demo. A provider grant opens a service, not a group's relay. |
| Admission | [`reached`](../crates/svrngs-node/src/door.rs#L422-L439) needs the lender's standing; `member_for` needs the caller's. [`holder_of`](../crates/svrngs-node/src/door.rs#L1276-L1338) checks a `Pass`, then resolves its issuer in an eligible held group or a scoped visit. [`standing`](../crates/svrngs-runtime/src/register.rs#L218-L247) is a local fold, not a remote proof. | `groups: []` is paused, not public. A valid pass only represents its signer; it cannot give them rights the provider never gave. A visit is a particular group's grant, not a provider-local grant. |
| Effects after admission | [`Asker`](../crates/svrngs-node/src/door.rs#L1194-L1207) assumes a group person. [`lend_for`](../crates/svrngs-node/src/door.rs#L1575-L1645) handles sign-in and selects credentials; Eudoxia's `lend::admit` gives `Admit::Own` when no other app is named. [`forward`](../crates/svrngs-node/src/door.rs#L1943-L1988) supplies identity and group-page headers. | A foreign holder cannot just become an `Asker` with an empty group or unnamed app. That can inject the lender's own credential, claim the caller is a group person, or pick the wrong origin behavior. Model the admitted use explicitly. |
| Lifetime and target | [`withdrawn` / `cut_by`](../crates/svrngs-node/src/door.rs#L841-L925) recheck member or visit evidence and cut streams independently of the reader. [`door::bound`](../crates/svrngs-node/src/door/bound.rs#L94-L139) checks what the provider approved on its port. | Keep the approved localhost target and one upstream forwarder. Extend the evidence the recheck consumes; don't use group standing to recheck a provider lease. |

Upstream surveyed: commonwealth-ai `3a05f1e89`,
`cmnwlth/crates/commonwealth-media/src/identity.rs::{admit_offer,admit_spliced_origin,admit_app}`
already separate the origin, verified requester and injected credentials,
but the offer and app paths reject non-members before forwarding. Its
`svrn/crates/sovereign-grants/src/guest_grant.rs::GuestGrantStore` has a
token-indexed, in-memory lifetime/revocation model for the upstream guest
door. Neither replaces the narrow native lend door or a provider-local
decision bound to its verified caller. Grapevine already demonstrates the
[deliberate human carry](../apps/grapevine/README.md#grapevine); its relay
and sealed-reply protocol are unnecessary for passing a reference.

The pinned Eudoxia (`71490cf`) already owns lend consent, request policy
and recheck (`community/src/lend.rs::{choose,admit,reach,recheck}`). Its
`src/rules.rs` can decide on verified facts; it doesn't produce those facts
or give the library anyone's standing. The native-pass projection in
svrn-docs `rings/current/eudoxia/integration/REACH_PASS.md` explicitly
assumes issuer legitimacy and lender consent. Use that distinction, and
the separation of resource selection and admission in svrn-docs
`rings/current/CAPABILITY_CONNECTIONS.md`; neither requires a second
policy engine, signature-chain format or a change to Eudoxia's kernel.

## 3. What to do in which order

**First, let the group carry an ordinary HTTPS offer.** The library's own
URL is the service reference. The same URL in two groups and at the desk
opens the same service on its own origin. Let the public see availability;
defer to the library for booking, sign-in and eligibility. Use a link in
the group's existing chat before building a shared shelf. A link with an
existing login is the right answer for a provider that already runs the
service on the web. A provider-operated group is also possible today for
invite-only trials: it keeps the original groups separate, but makes every
consumer join another group and gives the provider a membership list it
doesn't otherwise need. It does not prove independent service admission.

**The first native slice is one HTTP lend, granted to a holder by the
provider.** Do it for a provider who wants to run a service on localhost
without exposing a public app origin on their own network. A first probe
should use a read-only, dedicated interface with no upstream credential.
Use the existing `Alpn::Lend` and verified dialer. The provider explicitly
approves that key for that label until a given time. Provision the key by
hand in the probe; the holder already proves possession on the connection,
so there is no need to invent a signed grant, pass in a URL, or a second
login mechanism. Keep grants alongside the provider's local declaration,
never in a rail. A missing, expired or revoked grant fails closed. Address,
policy and grant are different facts.

```text
offer reference  = native label + bounded provider contact + display text
verified caller  = key from the lend connection
admitted use    = current approved interface + live grant for that label and key
                   + permitted operation; otherwise refused before forwarding
```

- Add an explicit provider mode only for a declared HTTP port. Its creation
  must go through the normal local, authenticated approval of the port,
  without the [`share` group selection](../crates/svrngs/src/cli.rs#L1977-L1987).
  Validate the mode at load and write. Keep `groups: []` paused, existing
  group and visit rules intact, and provider-mode files closed on downgrade.
  Don't overload `Offered.group` or mix the two kinds of audience in one
  interface in the first slice.
- Resolve verified group/visit versus provider evidence at the existing
  lend door. One resource decision selects the operation, forwarding
  identity/headers, credential and recheck provenance. Reuse the community
  profile for policy, the existing target binding and the transport. No
  invented group/person header, OIDC claim, owner credential, record write,
  other-lend reads, or upgrade for this provider mode. Strip client-supplied
  authority headers. The provider may see a service-specific holder key;
  it is not a patron account, and need not reach the upstream app. Record
  typed outcomes at debug without putting credentials or group identities
  in logs. For the first probe, admit one exact read route on a backend that
  implements no mutations. Recheck admission on each request and the same
  admitting grant under the existing stream cancellation owner. Revoking and
  reissuing must not restore an old stream; distinguish grant generations.
- Use the library's HTTPS site or a physical handoff to establish the
  association between the institution and the node key. A key proves which
  node answered, not that its operator is the public library. The label is
  stable across address changes with the same key/name. Rekeying requires
  explicit republishing; do not silently replace the node behind a bookmark.
  Keep referrals passive until a person opens one. Never put a pass, a
  redemption secret, or a patron identifier in a group record. A fresh
  grant is required when a retired interface is recreated with materially
  different authority, even if the label is reused.
- After the native door is proven, add a phone open at an isolated lend
  origin using a per-offer holder. The [current WASM holder binder](../web/src/lib.rs#L493)
  takes a pass for relay authentication; the provider open needs the same
  binder with no such token. Keep origin isolation without the rest of the
  group handoff: no group token, member signer, read list, or write approver
  to this open. Confirm dial and local origin handling on both phone and
  computer; today the desktop origin also builds its lend map from the
  peers that answered. Keep the HTTPS fallback for walk-ins; don't ship a
  second anonymous native ingress just to serve a QR.

**Only add group-issued permissions when a provider wants the group to
choose who gets a benefit.** If the library wants to set aside two slots,
first let it allocate or approve two people in its own booking flow. A
reservation counter and duplicate-booking behavior stay with that app.
When a group actually needs to nominate on its own, the provider can trust
an explicitly enrolled group issuer *for that benefit only*. Require a
fresh assertion of the exact offer and holder, from an issuer the provider
authorized for it, with expiry and with the provider still able to close
the allocation. A signed assertion is a statement at a time, not current
standing: decide the maximum stale-access window or require a live verifier
with a fail-closed timeout. Name who can rotate the issuer. No group roster
or full journal for the provider, no transferable group-wide secret.
[`Seed::id`](../crates/svrngs-core/src/seed.rs#L53-L67) is a hash of seed
fields, not an issuer key. Neither the ID nor a founder's bare signature
settles current authority. Revisit delegation machinery only at this step.

## 4. Proof and stop conditions

On the checkout surveyed, `cargo test -p svrngs-node --test door` passed **8/8**,
including a stranger's correctly signed pass refused, scoped refusals with
no upstream hit, and revocation cutting a live answer.
`cargo test -p svrngs-node --test accept` passed **2/2**, including verified dialer identity
coming from the connection rather than a header. These are baseline
witnesses; no provider-mode implementation or real library run was tested.
The fast gate stopped on the shared worktree's non-JavaScript ceiling
(24,195 against 24,174); the other fast checks passed.

1. **Public scene:** two groups with no members in common share the same
   library URL; a person without an account opens it from the desk QR. One
   update appears at all three opens. The provider never needs a group
   identifier. Measure what people did, rather than counting links shared.
2. **Native seam:** a node that holds neither group's record runs one
   approved localhost origin. Two granted keys from separate groups each
   open it. An ungranted key, a copied referral, an expired or withdrawn
   grant, a replay from another key, or a grant for another label cannot
   make the protected origin count a request. An ordinary member pass is
   not an alternative way into provider mode. Revoke one key on an open
   connection; it cannot make another request and its live response ends
   within a measured bound, while the other key continues. Use the door's
   counting origin and live-answer fixture; plant a dropped key/label bind
   and a skipped recheck and watch each test fail.
3. **Boundary seam:** from that same granted holder, there is still no
   access to the provider's groups, blobs, carried apps or raw ports. The
   upstream origin sees no group metadata, signer or injected owner
   secret; a forged identity header is absent. A changed port owner and an
   older build fail closed. Once the phone is hooked up, exercise a clean
   browser, a bookmark and re-open after revocation at the real origins.

If the public scene supplies the value, stop there for that library. The
native slice earns its place when the provider wants the service on its own
computer, reached by exact address and verified key through the existing
network. Ordinary HTTPS can already do access control. Size the native
slice with a deletion ledger around admission and forwarding before asking
for budget. Keep HTTP-only, one interface, manual provisioning and no
delegation until the native seam passes; build no new shared service to
prove that a provider can run one on its own computer.
