//! A leave or a cut reaches a computer that was off as it landed
//! (#184, #190), on the wire: leave_page.rs's claims whose decision is the
//! record's telling and fold, each computer's round asked by the test
//! (docs/seams.tsv). Off is the node's place on the wire closed, so every
//! ask of it finds no path, and back on is a node started again over a copy
//! of what it held. What stays in leave_page.rs: each home page in a
//! browser.

use std::time::Duration;

use svrngs::cli;
use svrngs::group;
use svrngs_node::link::{client_link, Link};
use svrngs_node::node::{Node, Wire};
use svrngs_node::removal::{LeaveAsk, Outcome, LEAVES_ROUTE};
use svrngs_node::{actor_of, Person, SigningKey};

use crate::{ask, computer, name, no_relay};

/// A computer of `key`'s over `dir` on `wire`, as leave_page.rs's
/// `computer` starts one there.
async fn on(wire: &Wire, dir: &std::path::Path, key: &SigningKey) -> Node {
    Node::on_wire(wire, dir.to_path_buf(), key.clone(), group::seed_text)
        .await
        .expect("a computer starts on the wire")
}

/// `svrngs leave <group>` asked of `node`, as the command asks it: what
/// happened.
async fn leave_from(node: &Node, group: &str) -> Outcome {
    let body = serde_json::to_value(LeaveAsk {
        group: group.to_string(),
    });
    let (_, text) = ask(node, "POST", LEAVES_ROUTE, &body.unwrap()).await;
    serde_json::from_str(&text).unwrap()
}

/// Up to 30 s: whether `node` holds `key` out of `group`.
async fn out_of(node: &Node, group: &str, key: &str) -> bool {
    let out = || {
        node.rail()
            .register(group)
            .unwrap()
            .removed
            .contains_key(key)
    };
    let held = tokio::time::timeout(Duration::from_secs(30), async {
        while !out() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
    held.await.is_ok()
}

/// What a computer off holds, `from`, copied to `to` but its blob store,
/// which the stopped node in this process still keeps open: started there
/// with its key, it comes back on holding what it held.
fn held_again(from: &std::path::Path, to: &std::path::Path) {
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let (at, name) = (e.path(), e.file_name());
        if name == "blobs" {
            continue;
        }
        if at.is_dir() {
            std::fs::create_dir_all(to.join(&name)).unwrap();
            held_again(&at, &to.join(&name));
        } else {
            std::fs::copy(&at, to.join(&name)).unwrap();
        }
    }
}

/// **A cut by a keeper made by a hand-on the cut computer never heard
/// reaches it** (gv-leave (c), #184). Sam makes Trip; Bo and Ana join from
/// his links, and Ana's second computer from her first's link for her
/// devices. While her second is off, Sam hands the order of removals on to
/// Bo and Bo removes Ana. Back on, within 90 s its copy holds her cut, as
/// its home page reads it (`/v1/groups`' `removed`). Red at 7f2a8d1d, told
/// Bo's `Remove` and Bo's acts alone (ring_routes.rs `told_cut`): its copy
/// still named Sam the keeper, so read the cut as an ask, and was refused
/// from then on, untold, for all 90 s.
#[tokio::test(flavor = "multi_thread")]
async fn a_cut_by_a_handed_on_keeper_reaches_a_computer_off_as_it_landed() {
    let wire = Wire::default();
    let sam = computer(&wire, 6).await.node;
    let (seed, _) = cli::new_group(&sam, name("Trip"), name("Sam"), None, &[], &no_relay())
        .await
        .expect("Sam makes Trip");
    let trip = seed.id();
    let dirs = [(); 3].map(|_| tempfile::tempdir().unwrap());
    let keys = [9, 7, 8].map(|b| SigningKey::from_bytes(&[b; 32]));
    let bo = on(&wire, dirs[0].path(), &keys[0]).await;
    let one = on(&wire, dirs[1].path(), &keys[1]).await;
    let two = on(&wire, dirs[2].path(), &keys[2]).await;
    for (who, as_) in [(&bo, "Bo"), (&one, "Ana")] {
        let made = sam
            .host()
            .link(&trip, false, None, None)
            .expect("Sam makes a link");
        let sent = Link::parse(&client_link(&made.token)).unwrap();
        who.host()
            .join(&sent, &name(as_))
            .await
            .expect("joins Trip");
    }
    let made = one
        .host()
        .link(&trip, true, None, None)
        .expect("Ana's link for her devices");
    let sent = Link::parse(&client_link(&made.token)).unwrap();
    two.host()
        .join(&sent, &name("whoever"))
        .await
        .expect("Ana's second computer joins as Ana");
    let [bo_actor, two_actor] = [&keys[0], &keys[2]].map(actor_of);
    let all = [&sam, &bo, &one, &two];
    let held = || {
        all.iter().all(|n| {
            let s = n.rail().standing(&trip).unwrap();
            s.member(&two_actor)
                .is_some_and(|m| m.person.as_str() == "Ana")
                && s.names(&bo_actor)
        })
    };
    let settled = tokio::time::timeout(Duration::from_secs(60), async {
        while !held() {
            for n in all {
                let _ = tokio::time::timeout(Duration::from_secs(10), n.round()).await;
            }
        }
    });
    assert!(
        settled.await.is_ok(),
        "control: all four hold Bo, and Ana's second as Ana"
    );

    // Off as the hand-on and the cut land.
    two.net().close().await;
    drop(two);
    let to_bo = svrngs_core::HandOnTo {
        person: Person::parse("Bo").unwrap(),
        joined: None,
    };
    let handed = sam.rail().commit(&trip, &to_bo).unwrap();
    assert!(handed.is_ok(), "control: Sam hands on to Bo: {handed:?}");
    // Trip is sources-v1: the hand-on passes with Bo's own yes, once Bo's
    // computer holds it (refused, writing nothing, until then).
    let keeps = || {
        let _ = bo
            .rail()
            .commit(&trip, &svrngs_core::Take(bo_actor.clone()));
        bo.rail().register(&trip).unwrap().keeper == Some(bo_actor.clone())
    };
    let kept = tokio::time::timeout(Duration::from_secs(30), async {
        while !keeps() {
            let _ = tokio::time::timeout(Duration::from_secs(10), bo.round()).await;
        }
    });
    assert!(
        kept.await.is_ok(),
        "control: Bo's computer says Bo keeps it"
    );
    let ana = svrngs_node::Removal {
        person: Person::parse("Ana").unwrap(),
        joined: None,
    };
    let cut = bo.rail().commit(&trip, &ana).unwrap();
    assert!(cut.is_ok(), "control: Bo cuts Ana: {cut:?}");
    let _ = tokio::time::timeout(Duration::from_secs(10), bo.round()).await;
    assert!(
        out_of(&sam, &trip, &two_actor).await,
        "control: Sam's computer holds Ana's second out of Trip within 30 s"
    );

    let again = tempfile::tempdir().unwrap();
    held_again(dirs[2].path(), again.path());
    let two = on(&wire, again.path(), &keys[2]).await;
    let cut_here = || {
        let r = two.rail().register(&trip).unwrap();
        r.removed.contains_key(&two_actor)
    };
    let told = tokio::time::timeout(Duration::from_secs(90), async {
        while !cut_here() {
            let _ = tokio::time::timeout(Duration::from_secs(10), two.round()).await;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    let keeper = two.rail().register(&trip).unwrap().keeper;
    assert!(
        told.await.is_ok(),
        "back on, Ana's second computer holds her cut; its copy's keeper {keeper:?}, Bo {bo_actor}"
    );
}

/// **A leave reaches a computer off as it lands, with nobody else to tell
/// it** (#190). Sam makes Trip and Ana joins from his link: a group of two.
/// Sam's computer is off as Ana leaves from hers (no path to it on the
/// wire); back on, from a copy of what it held, within 90 s of each computer's rounds its copy holds Ana
/// out — told by the one computer that holds the leave, Ana's, whose round
/// tells it until Sam's answers. Red at fc2ad0ec: told once as it landed,
/// to a computer that was off, and never again.
#[tokio::test]
async fn a_leave_reaches_a_computer_off_as_it_landed() {
    let wire = Wire::default();
    let sam_dir = tempfile::tempdir().unwrap();
    let sam = on(&wire, sam_dir.path(), &SigningKey::from_bytes(&[6; 32])).await;
    let (seed, _) = cli::new_group(&sam, name("Trip"), name("Sam"), None, &[], &no_relay())
        .await
        .expect("Sam makes a group");
    let trip = seed.id();
    let ana_dir = tempfile::tempdir().unwrap();
    let ana_key = SigningKey::from_bytes(&[7; 32]);
    let ana = on(&wire, ana_dir.path(), &ana_key).await;
    let made = sam
        .host()
        .link(&trip, false, None, None)
        .expect("Sam makes a link");
    let sent = Link::parse(&client_link(&made.token)).unwrap();
    ana.host()
        .join(&sent, &name("Ana"))
        .await
        .expect("Ana joins");
    let ana_actor = actor_of(&ana_key);
    assert!(
        sam.rail().standing(&trip).unwrap().names(&ana_actor),
        "control: Sam's computer holds Ana in Trip"
    );

    sam.net().close().await;
    drop(sam);
    assert_eq!(
        leave_from(&ana, &trip).await,
        Outcome::Left,
        "Ana leaves Trip while Sam's computer is off"
    );
    // Off past the leave's first telling, which found no path to it: on
    // one thread, a pause lets every task that is ready run.
    tokio::time::sleep(Duration::from_millis(500)).await;
    let again = tempfile::tempdir().unwrap();
    held_again(sam_dir.path(), again.path());
    let sam = on(&wire, again.path(), &SigningKey::from_bytes(&[6; 32])).await;
    let out = || {
        sam.rail()
            .register(&trip)
            .unwrap()
            .removed
            .contains_key(&ana_actor)
    };
    let told = tokio::time::timeout(Duration::from_secs(90), async {
        while !out() {
            for n in [&ana, &sam] {
                let _ = tokio::time::timeout(Duration::from_secs(10), n.round()).await;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    assert!(
        told.await.is_ok(),
        "back on, Sam's computer holds Ana's leave"
    );
}
