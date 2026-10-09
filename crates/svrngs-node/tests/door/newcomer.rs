//! A newcomer let in by every lender at once (mnp-1 (b);
//! `demo/runs/mn/2026-10-08/README.md` §9, 3). Ada's computer, A, admits
//! nia; bo's computer, B, which holds the group and lends `shelf`, has not
//! heard of her, and its own round runs every 60 s
//! (`ring_sync::DEFAULT_RING_SYNC_INTERVAL`). Nia's page's first request
//! through B's door, on the pass holders' protocol, is timed from her
//! admission until B's door answers it as nia.

use std::time::{Duration, Instant};

use super::common::loopback_sockets;
use super::{
    bridge_as, header_values, last, lends_on, origins, raw_to, signing, unix_now, Pair, NS,
};
use svrngs_core::alpn::Alpn;
use svrngs_core::pass::{Label, Pass};
use svrngs_node::iroh::EndpointAddr;
use svrngs_node::{actor_of, Person, RailAct, Roster};

/// Nia, admitted at A, asks B's `shelf` with her pass seconds after, and
/// is answered as nia within [`WITHIN`]: B's door, meeting a signer it does
/// not know, asks the group's ring once before it answers. A pass whose
/// signer stands in no group is still refused, after the same ask.
/// Planted: the door refusing at once (door.rs before mnp-1), which
/// refuses nia until B's round.
#[tokio::test]
async fn a_newcomer_s_first_request_through_a_second_lender_is_answered_within_seconds() {
    const WITHIN: Duration = Duration::from_secs(10);
    let pair = Pair::new().await;
    let mut at_a = EndpointAddr::new(pair.a.endpoint().id());
    for sock in loopback_sockets(pair.a.endpoint()) {
        at_a = at_a.with_ip_addr(sock);
    }
    let seed = Roster::new(
        [(Person::parse("ada").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    pair.b.rail().set_seed(NS, &seed).unwrap();
    let synced = pair.b.sync_with(NS, &at_a).await;
    assert!(synced.stop.is_none(), "B holds the group: {synced:?}");
    let (shelf_origin, heads) = origins::head_origin().await;
    lends_on(pair._db.path(), "shelf", shelf_origin.port());
    let shelf = Label::of(pair.b.endpoint().id().as_bytes(), "shelf");

    let holder = signing(9);
    let bridge = bridge_as(&holder, &pair.b, Alpn::Lend).await;
    let ask = |signer: u8| {
        let pass = Pass::sign(
            &signing(signer),
            &actor_of(&holder),
            (shelf.clone(), vec![]),
            unix_now() + 3600,
        );
        raw_to(
            bridge.local_addr(),
            format!(
                "GET /lend/{shelf}/whoami HTTP/1.1\r\nHost: local\r\nx-svrngs-pass: {}\r\nConnection: close\r\n\r\n",
                pass.encode()
            ),
        )
    };

    // B's round at its start, which runs as the node starts, is over, as
    // Dee's was when Movie Night's people joined: a round in flight as nia
    // is admitted would carry her.
    tokio::time::sleep(Duration::from_secs(2)).await;
    let admit = svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse("nia").unwrap(),
            key: actor_of(&signing(7)),
            invite: None,
        },
        on_behalf_of: None,
    };
    pair.a.rail().commit(NS, &admit).unwrap().unwrap();
    let joined = Instant::now();
    let mut refused = 0;
    let answered = loop {
        let reply = ask(7).await;
        if reply.status == 200 {
            break joined.elapsed();
        }
        assert_eq!(reply.status, 403, "{}", reply.text);
        refused += 1;
        assert!(
            joined.elapsed() < Duration::from_secs(75),
            "nia was never let in: {}",
            reply.text
        );
        tokio::time::sleep(Duration::from_secs(1)).await;
    };
    eprintln!("nia answered {answered:?} after joining, refused {refused} times before");
    assert!(
        answered < WITHIN && refused == 0,
        "nia's first request must be answered within {WITHIN:?} of joining: answered {answered:?} after, refused {refused} times before"
    );
    let head = last(&heads);
    assert_eq!(
        header_values(&head, "x-svrngs-person"),
        vec!["nia"],
        "{head}"
    );

    let stranger = ask(5).await;
    assert_eq!(stranger.status, 403, "{}", stranger.text);
    assert!(
        stranger.text.contains("x-svrngs-refused: pass"),
        "{}",
        stranger.text
    );
    pair.close().await;
}

/// As above, with Cy in the group and Cy's computer asleep
/// (asleep-1-none-held (a); `demo/runs/mnp/2026-10-09/README.md`, "What no
/// prediction named" 1): nia's first request through B's door is answered
/// as nia within [`SOON`] of joining, refused never — let in as soon as A,
/// which knows her, has answered B's round, not when Cy's dial gives up.
/// Cy's key sorts before A's, so a round dialling its members one by one in
/// key order dials Cy first. Planted: the round's members dialled one by
/// one and the door waiting on the round's end (ring_sync.rs and door.rs
/// before this row), which refused nia once at about 10 s, as Dee's door
/// refused Ben.
#[tokio::test]
async fn a_newcomer_is_let_in_while_a_computer_of_the_group_sleeps() {
    const SOON: Duration = Duration::from_secs(2);
    let pair = Pair::new().await;
    let ada = actor_of(&signing(1));
    let cy = (10..=255u8)
        .find(|&s| actor_of(&signing(s)) < ada)
        .expect("a key sorting before ada's");
    let seed = Roster::new(
        [(Person::parse("ada").unwrap(), vec![ada])]
            .into_iter()
            .collect(),
    );
    pair.b.rail().set_seed(NS, &seed).unwrap();
    let admit = |person: &str, key: u8| svrngs_core::tests_support::Fixture {
        act: RailAct::Admit {
            person: Person::parse(person).unwrap(),
            key: actor_of(&signing(key)),
            invite: None,
        },
        on_behalf_of: None,
    };
    pair.a.rail().commit(NS, &admit("cy", cy)).unwrap().unwrap();
    let (_asleep, at_cy) = super::common::never_answers();
    let cy_key =
        svrngs_node::iroh::PublicKey::from_bytes(signing(cy).verifying_key().as_bytes()).unwrap();
    pair.b.transport().add_known_peer(cy_key, at_cy);
    let mut at_a = EndpointAddr::new(pair.a.endpoint().id());
    for sock in loopback_sockets(pair.a.endpoint()) {
        at_a = at_a.with_ip_addr(sock);
    }
    let synced = pair.b.sync_with(NS, &at_a).await;
    assert!(
        synced.stop.is_none(),
        "B holds the group, Cy in it: {synced:?}"
    );
    let (shelf_origin, heads) = origins::head_origin().await;
    lends_on(pair._db.path(), "shelf", shelf_origin.port());
    let shelf = Label::of(pair.b.endpoint().id().as_bytes(), "shelf");

    let holder = signing(9);
    let bridge = bridge_as(&holder, &pair.b, Alpn::Lend).await;
    let pass = Pass::sign(
        &signing(7),
        &actor_of(&holder),
        (shelf.clone(), vec![]),
        unix_now() + 3600,
    );
    let ask = || {
        raw_to(
            bridge.local_addr(),
            format!(
                "GET /lend/{shelf}/whoami HTTP/1.1\r\nHost: local\r\nx-svrngs-pass: {}\r\nConnection: close\r\n\r\n",
                pass.encode()
            ),
        )
    };

    tokio::time::sleep(Duration::from_secs(2)).await;
    pair.a.rail().commit(NS, &admit("nia", 7)).unwrap().unwrap();
    let joined = Instant::now();
    let mut refused = 0;
    let answered = loop {
        let reply = ask().await;
        if reply.status == 200 {
            break joined.elapsed();
        }
        assert_eq!(reply.status, 403, "{}", reply.text);
        refused += 1;
        eprintln!("nia refused {:?} after joining", joined.elapsed());
        assert!(
            joined.elapsed() < Duration::from_secs(75),
            "nia was never let in: {}",
            reply.text
        );
        tokio::time::sleep(Duration::from_secs(1)).await;
    };
    eprintln!("nia answered {answered:?} after joining, Cy asleep, refused {refused} times before");
    assert!(
        answered < SOON && refused == 0,
        "nia's first request must be answered within {SOON:?} of joining, Cy asleep: answered {answered:?} after, refused {refused} times before"
    );
    assert_eq!(header_values(&last(&heads), "x-svrngs-person"), vec!["nia"]);
    pair.close().await;
}
