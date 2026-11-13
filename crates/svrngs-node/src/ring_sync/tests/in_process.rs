// SPDX-License-Identifier: AGPL-3.0-or-later
//! The round's exchange and its peer route with the member asked in this
//! process (docs/CHECKS.md, harness 4): the peer's own `ring_routes`
//! router, asked by `oneshot` through [`InProcess`] with the asker's
//! `Dialer` inserted as the acceptor inserts it, its body limit and
//! standing check as served. No endpoint, no socket, no clock. That a
//! request rides QUIC at all stays iroh/tests.rs's.

use super::*;
use crate::iroh::tests::InProcess;
use crate::iroh::{Contact, Fetch};

/// A member answering as one router, and the contact it is asked by.
struct Member {
    asked: InProcess,
    contact: Contact,
}

impl Member {
    /// `router`, asked by `asker`.
    fn of(router: axum::Router, asker: &SigningKey) -> Self {
        let mut asked = InProcess::asked_by(asker);
        let contact = asked.serving(0xEE, router);
        Self { asked, contact }
    }

    fn to(&self) -> To<'_> {
        (&self.asked, &self.contact)
    }
}

/// `rail`'s ring routes, as the node serves them, asked by a dialer
/// holding `asker`: the route answers only a dialer its roster names.
fn serve(rail: Arc<RingRail>, asker: &SigningKey) -> Member {
    Member::of(ring_router(rail), asker)
}

/// One raw exchange call: the status, and the answer when it is a 200.
async fn sync_raw(member: &Member, body: &RingSyncRequest) -> (u16, Option<RingSyncResponse>) {
    let posted = crate::iroh::posted(SYNC_ROUTE, serde_json::to_vec(body).unwrap());
    let answer = Fetch::fetch(&member.asked, &member.contact, posted)
        .await
        .unwrap();
    let status = answer.status().as_u16();
    let read = (status == 200).then(|| serde_json::from_slice(answer.body()).unwrap());
    (status, read)
}

/// A route answering `answer` to every exchange, asked by any dialer.
fn stand_in(
    answer: impl Fn(RingSyncRequest) -> axum::response::Response + Clone + Send + Sync + 'static,
) -> Member {
    let app = axum::Router::new().route(
        SYNC_ROUTE,
        axum::routing::post(move |body: axum::body::Bytes| {
            let answer = answer.clone();
            async move { answer(serde_json::from_slice(&body).unwrap()) }
        }),
    );
    Member::of(app, &SigningKey::from_bytes(&[1u8; 32]))
}

/// **The gate the ceiling measured.** A 10,000-op journal, past the
/// one-exchange ceiling, converges onto a peer that has never seen this
/// ring, through the real route and the real loop.
#[tokio::test]
async fn a_journal_past_the_one_exchange_ceiling_converges_onto_a_fresh_peer() {
    const N: usize = 10_000;
    let key = SigningKey::from_bytes(&[1u8; 32]);
    let (sender_dir, peer_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (rail, journal) = node(sender_dir.path(), &key, N);
    let (peer_rail, peer_journal) = node(peer_dir.path(), &key, 0);

    let served = serve(peer_rail, &key);
    let out = exchange(served.to(), rail.as_ref(), NS).await;

    assert!(out.stop.is_none(), "the exchange failed: {:?}", out.stop);
    assert_eq!(out.pushed, N, "every op landed on the peer");
    assert_eq!(out.pulled, 0, "a peer holding nothing has nothing to give");
    let admitted = peer_journal
        .admit(&solo_roster(&key), &Ed25519Verifier)
        .unwrap();
    assert_eq!(svrngs_core::tests_support::wire_ops(&admitted).len(), N);
    assert!(admitted.is_complete(), "gaps: {:?}", admitted.gaps());
    assert_eq!(
        peer_journal.digest(&crate::Ed25519Verifier).unwrap(),
        journal.digest(&crate::Ed25519Verifier).unwrap(),
        "two nodes, one claim"
    );
}

/// **A seal another member signs retires nothing here, and every act under
/// it still counts** (arch-seal (c); svrn-docs A_RECORD_FOR_YEARS §6 q2).
/// Bo, a member, writes three records and then a seal; Ana's computer
/// pulls the seal and keeps all four lines, the seal held as an act that
/// does nothing.
#[tokio::test]
async fn a_seal_another_member_signs_retires_nothing_here() {
    let (ana, bo) = (
        SigningKey::from_bytes(&[1u8; 32]),
        SigningKey::from_bytes(&[2u8; 32]),
    );
    let mut seed = solo_roster(&ana);
    seed.members
        .insert(Person::parse("bo").unwrap(), vec![actor_of(&bo)]);
    let (mine, theirs) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let duo = |dir: &std::path::Path, key: &SigningKey| {
        let rail = Arc::new(RingRail::new(dir, Arc::new(key.clone())));
        rail.set_seed(NS, &seed).unwrap();
        assert_eq!(rail.ingest_all(NS, &ops_in(NS, &bo, 3)).unwrap(), 3);
        (rail.clone(), Journal(rail))
    };
    let (rail, journal) = duo(mine.path(), &ana);
    let (peer_rail, peer_journal) = duo(theirs.path(), &bo);
    peer_journal
        .ingest(&next_in(&peer_journal, &bo, RailAct::Seal))
        .unwrap();

    let served = serve(peer_rail, &ana);
    let out = exchange(served.to(), rail.as_ref(), NS).await;

    assert!(out.stop.is_none(), "the exchange failed: {:?}", out.stop);
    assert_eq!(out.pulled, 1, "one op came over, and it was the seal");
    let held = journal.read().unwrap().0;
    assert_eq!(held.len(), 4, "the seal retired nothing: {held:?}");
    let admitted = journal.admit(&seed, &Ed25519Verifier).unwrap();
    assert!(admitted.is_complete(), "gaps: {:?}", admitted.gaps());
    let ops = svrngs_core::tests_support::wire_ops(&admitted);
    assert_eq!(ops.len(), 4, "the seal is held as an act: {ops:?}");
    assert_eq!(
        ops.iter().filter(|o| o.applies()).count(),
        3,
        "every record under it still counts"
    );
    assert_eq!(
        journal.digest(&crate::Ed25519Verifier).unwrap(),
        peer_journal.digest(&crate::Ed25519Verifier).unwrap(),
        "two nodes, one claim"
    );
}

/// The control for the seal test above: the identical exchange with an
/// ordinary act pulls the same one op and deletes nothing.
#[tokio::test]
async fn an_ordinary_op_arriving_prunes_nothing() {
    let key = SigningKey::from_bytes(&[1u8; 32]);
    let (mine, theirs) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (rail, journal) = node(mine.path(), &key, 3);
    let (peer_rail, peer_journal) = node(theirs.path(), &key, 3);
    peer_journal
        .ingest(&next_in(
            &peer_journal,
            &key,
            RailAct::Record {
                payload: body_of_size(FIXTURE_BODY_BYTES),
            },
        ))
        .unwrap();

    let served = serve(peer_rail, &key);
    let out = exchange(served.to(), rail.as_ref(), NS).await;
    assert_eq!(out.pulled, 1);
    assert_eq!(journal.read().unwrap().0.len(), 4, "nothing was retired");
}

/// The route's own limit: an unbudgeted push of 8,000 ops is served, 10,000
/// is refused 413, and the budgeted chunk of those 10,000 is served.
#[tokio::test]
async fn the_budgeted_chunk_is_served_where_the_whole_journal_is_refused() {
    let key = SigningKey::from_bytes(&[1u8; 32]);
    for (n, want) in [(8_000usize, 200u16), (10_000, 413)] {
        let (s, r) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let (_rail, journal) = node(s.path(), &key, n);
        let (peer, _) = node(r.path(), &key, 0);
        let push = RingSyncRequest {
            namespace: NS.to_string(),
            digest: journal.digest(&crate::Ed25519Verifier).unwrap(),
            ops: journal
                .ops_missing_from(&crate::Ed25519Verifier, &Digest::default())
                .unwrap(),
        };
        let (status, _) = sync_raw(&serve(peer, &key), &push).await;
        assert_eq!(status, want, "{n} unbudgeted ops");
    }
    let (s, r) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (_rail, journal) = node(s.path(), &key, 10_000);
    let (peer, _) = node(r.path(), &key, 0);
    let (ops, more) = journal
        .ops_missing_from_within(
            &crate::Ed25519Verifier,
            &Digest::default(),
            RING_SYNC_OPS_BUDGET_BYTES,
        )
        .unwrap();
    assert!(more, "the control: 10,000 ops must not fit one chunk");
    let push = RingSyncRequest {
        namespace: NS.to_string(),
        digest: journal.digest(&crate::Ed25519Verifier).unwrap(),
        ops,
    };
    let (status, _) = sync_raw(&serve(peer, &key), &push).await;
    assert_eq!(status, 200, "the budgeted chunk is served");
}

/// The pull direction is budgeted too, and converges by repeating: every
/// answer fits the limit, and each carries ops the puller lacked.
#[tokio::test]
async fn the_pull_direction_is_budgeted_and_converges_by_repeating() {
    const N: usize = 10_000;
    let key = SigningKey::from_bytes(&[1u8; 32]);
    let (h, f) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let (holder, journal) = node(h.path(), &key, N);
    let (_fresh, fresh_journal) = node(f.path(), &key, 0);
    let served = serve(holder, &key);
    let mut rounds = 0usize;
    loop {
        let ask = RingSyncRequest {
            namespace: NS.to_string(),
            digest: fresh_journal.digest(&crate::Ed25519Verifier).unwrap(),
            ops: Vec::new(),
        };
        let (status, body) = sync_raw(&served, &ask).await;
        assert_eq!(status, 200);
        let body = body.unwrap();
        let bytes = serde_json::to_vec(&body).unwrap().len();
        assert!(
            bytes <= MAX_REQUEST_BODY_BYTES,
            "round {rounds}: {bytes} B — the pull direction is unbounded again"
        );
        if body.ops.is_empty() {
            break;
        }
        assert!(
            fresh_journal.ingest_all(&body.ops).unwrap() > 0,
            "round {rounds} carried ops the puller already held — the spin"
        );
        rounds += 1;
        assert!(rounds < 64, "the pull did not converge in 64 rounds");
    }
    assert!(
        rounds > 1,
        "the control: {N} ops must take more than one pull"
    );
    let admitted = fresh_journal
        .admit(&solo_roster(&key), &Ed25519Verifier)
        .unwrap();
    assert_eq!(svrngs_core::tests_support::wire_ops(&admitted).len(), N);
    assert!(admitted.is_complete(), "gaps: {:?}", admitted.gaps());
    assert_eq!(
        fresh_journal.digest(&crate::Ed25519Verifier).unwrap(),
        journal.digest(&crate::Ed25519Verifier).unwrap()
    );
}

/// A real pull is not discarded by a later failure: call 1's ops are on
/// disk and counted although call 2 is refused.
#[tokio::test]
async fn a_second_call_that_fails_still_reports_what_the_first_call_pulled() {
    let gift = ops_in(NS, &SigningKey::from_bytes(&[9u8; 32]), 5);
    let stood = stand_in(move |req| {
        if !req.ops.is_empty() {
            return axum::http::StatusCode::PAYLOAD_TOO_LARGE.into_response();
        }
        axum::Json(RingSyncResponse {
            namespace: req.namespace,
            digest: Digest::default(),
            ops: gift.clone(),
            ingested: 0,
        })
        .into_response()
    });
    let dir = tempfile::tempdir().unwrap();
    let (rail, journal) = node(dir.path(), &SigningKey::from_bytes(&[1u8; 32]), 3);

    let out = exchange(stood.to(), rail.as_ref(), NS).await;
    assert!(
        matches!(out.stop, Some(ExchangeStop::Refused { .. })),
        "a 413 is a refusal: {:?}",
        out.stop
    );
    assert_eq!(out.pulled, 5, "call 1's five ops are counted");
    assert_eq!(journal.read().unwrap().0.len(), 8, "3 held + 5 pulled");
}

/// 403 and 413 are answers, not unreachability; a member no path reaches
/// is Failed. (That a dial to a dead address is no answer at all is
/// iroh/tests.rs's, over QUIC; here, what the exchange makes of none.)
#[tokio::test]
async fn a_peer_that_answers_403_or_413_is_refused_rather_than_unreachable() {
    let dir = tempfile::tempdir().unwrap();
    let (rail, _journal) = node(dir.path(), &SigningKey::from_bytes(&[1u8; 32]), 3);
    for code in [403u16, 413] {
        let stood = stand_in(move |_| {
            axum::http::StatusCode::from_u16(code)
                .unwrap()
                .into_response()
        });
        let out = exchange(stood.to(), rail.as_ref(), NS).await;
        match out.stop {
            Some(ExchangeStop::Refused { sent_bytes, status }) => {
                assert_eq!(status, code, "the status picks the sentence");
                assert!(
                    sent_bytes > 0,
                    "the refused size is what makes it actionable"
                );
            }
            other => panic!("{code}: expected Refused, got {other:?}"),
        }
    }
    let nobody = InProcess::asked_by(&SigningKey::from_bytes(&[9u8; 32]));
    let dead = crate::iroh::tests::bare(crate::iroh::SecretKey::from_bytes(&[8; 32]).public());
    let out = exchange((&nobody, &dead), rail.as_ref(), NS).await;
    assert!(
        matches!(out.stop, Some(ExchangeStop::Failed(_))),
        "an unreachable member is not a refusal: {:?}",
        out.stop
    );
}

/// A peer that answers but never ingests cannot make the loop spin: the
/// stall check sees an unmoved digest and hands the round back.
#[tokio::test]
async fn a_peer_whose_digest_never_moves_stops_the_loop_instead_of_spinning() {
    let hits = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = hits.clone();
    let stood = stand_in(move |req| {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        axum::Json(RingSyncResponse {
            namespace: req.namespace,
            digest: Digest::default(),
            ops: Vec::new(),
            ingested: 0,
        })
        .into_response()
    });
    let dir = tempfile::tempdir().unwrap();
    let (rail, _journal) = node(dir.path(), &SigningKey::from_bytes(&[1u8; 32]), 3);

    let out = exchange(stood.to(), rail.as_ref(), NS).await;
    assert!(out.stop.is_none());
    assert_eq!((out.pulled, out.pushed), (0, 0));
    let calls = hits.load(std::sync::atomic::Ordering::SeqCst);
    assert!(
        calls <= 4,
        "the stall check must stop after the second chunk, not run all \
         {MAX_CHUNKS_PER_EXCHANGE} — {calls} calls"
    );
}
