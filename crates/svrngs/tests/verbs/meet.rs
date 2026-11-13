//! A keeper changes where a group meets, and only a keeper may (relay
//! change; relay fallbacks), on the wire: meet.rs's claims whose decision
//! is `cli::link_to`'s, `Host::meet`'s and the record's fold, its relays
//! named and never dialled (docs/seams.tsv). What stays in meet.rs: a join
//! with every relay down, a relay lost, and `link --relay` asked of the
//! svrngs running here.

use svrngs::cli::{self, MeetAt, Verb};
use svrngs_node::iroh::{RelayConfig, RelayUrl};
use svrngs_node::link::Link;
use svrngs_node::node::{Node, Wire};
use svrngs_node::SigningKey;

use crate::computer;

fn argv(s: &str) -> Vec<String> {
    std::iter::once("svrngs".to_string())
        .chain(s.split_whitespace().map(str::to_string))
        .collect()
}

/// Three relays' addresses, the old, the new and the build's: named in
/// links and the record, never dialled on the wire.
fn relays() -> (String, String, String) {
    let at = |n: u8| format!("http://10.0.0.{n}:3340/");
    (at(1), at(2), at(3))
}

fn meets(node: &Node, group: &str) -> Vec<String> {
    node.rail()
        .register(group)
        .unwrap()
        .meets
        .unwrap_or_default()
        .into_iter()
        .map(|u| u.to_string())
        .collect()
}

fn acts(node: &Node, group: &str) -> usize {
    let dir = node.rail().group_dir(group).unwrap();
    std::fs::read_to_string(dir.join("ring_oplog.jsonl"))
        .unwrap()
        .lines()
        .count()
}

/// `link` takes `--relay`, and a keeper changes the group's own relays to
/// exactly what it names; the link it mints carries them, and a member who
/// is not the keeper is refused without writing an act.
#[tokio::test(flavor = "multi_thread")]
async fn a_keeper_changes_where_the_group_meets_and_a_member_may_not() {
    let wire = Wire::default();
    let (old, new, build) = relays();
    let ours = RelayConfig {
        relay_urls: vec![build.clone()],
        ..cli::reach(MeetAt::Svrngs)
    };

    let made = cli::parse(argv(&format!("new house --as Sam --relay {old}"))).unwrap();
    let sam = computer(&wire, 71).await.node;
    let Verb::New {
        name,
        person,
        relays,
        ..
    } = made
    else {
        panic!("new parses as new");
    };
    let (seed, printed) = cli::new_group(&sam, name, person.unwrap(), None, &relays, &ours)
        .await
        .expect("Sam makes house");
    let old_link = Link::parse(&printed.client).unwrap();
    assert!(
        old_link.token.relays.iter().any(|r| r.to_string() == old),
        "the first link carries the old relay"
    );

    let new: RelayUrl = new.parse().unwrap();
    let (_, printed_new, _) = cli::link_to(&sam, "house", None, &ours, false, &[new.clone()])
        .await
        .expect("Sam changes the relay and mints a link");
    let new_link = Link::parse(&printed_new.client).unwrap();
    assert!(
        new_link
            .token
            .relays
            .iter()
            .any(|r| r.to_string() == new.to_string()),
        "the new link carries the new relay"
    );
    assert_eq!(meets(&sam, &seed.id()), vec![new.to_string()]);

    // Two phases: keep the old beside the new, then drop it.
    let old: RelayUrl = old.parse().unwrap();
    sam.host()
        .meet(&seed.id(), &[old.clone(), new.clone()])
        .expect("the keeper adds the new relay beside the old");
    assert_eq!(
        meets(&sam, &seed.id()),
        vec![old.to_string(), new.to_string()]
    );
    sam.host()
        .meet(&seed.id(), &[new.clone()])
        .expect("the keeper drops the old relay");
    assert_eq!(meets(&sam, &seed.id()), vec![new.to_string()]);
    let quiet = acts(&sam, &seed.id());
    sam.host()
        .meet(&seed.id(), &[new.clone()])
        .expect("the same set writes nothing");
    assert_eq!(
        acts(&sam, &seed.id()),
        quiet,
        "an unchanged set writes no act"
    );

    let join = cli::parse(argv(&format!("join {} --as Alex", printed_new.client))).unwrap();
    let alex = computer(&wire, 72).await.node;
    let Verb::Join { link, person, .. } = join else {
        panic!("join parses as join");
    };
    let key = SigningKey::from_bytes(&[72; 32]);
    cli::join_group(&alex, &key, &link, &person.unwrap())
        .await
        .expect("Alex joins by the old link");

    let before = acts(&alex, &seed.id());
    let refused = alex.host().meet(&seed.id(), &[old.clone()]);
    assert!(refused.is_err(), "a member who is not keeper is refused");
    let refused = cli::link_to(&alex, "house", None, &ours, false, &[old.clone()]).await;
    assert!(
        matches!(refused, Err(svrngs::cli::Refusal::Meet(_))),
        "the command refuses before it mints a link"
    );
    assert_eq!(acts(&alex, &seed.id()), before, "the refusal writes no act");
    assert_eq!(
        meets(&alex, &seed.id()),
        vec![new.to_string()],
        "the group still meets where the keeper said"
    );
}

/// A keeper's change reaches a member on the next exchange, and the member
/// still reaches the keeper.
#[tokio::test(flavor = "multi_thread")]
async fn a_member_folds_the_new_relay_from_the_old_one() {
    let wire = Wire::default();
    let (old, new, build) = relays();
    let ours = RelayConfig {
        relay_urls: vec![build.clone()],
        ..cli::reach(MeetAt::Svrngs)
    };
    let made = cli::parse(argv(&format!("new house --as Sam --relay {old}"))).unwrap();
    let sam = computer(&wire, 73).await.node;
    let Verb::New {
        name,
        person,
        relays,
        ..
    } = made
    else {
        panic!("new parses as new");
    };
    let (seed, printed) = cli::new_group(&sam, name, person.unwrap(), None, &relays, &ours)
        .await
        .expect("Sam makes house");
    let join = cli::parse(argv(&format!("join {} --as Alex", printed.client))).unwrap();
    let alex = computer(&wire, 74).await.node;
    let Verb::Join { link, person, .. } = join else {
        panic!("join parses as join");
    };
    let key = SigningKey::from_bytes(&[74; 32]);
    cli::join_group(&alex, &key, &link, &person.unwrap())
        .await
        .expect("Alex joins");

    let new: RelayUrl = new.parse().unwrap();
    sam.host()
        .meet(&seed.id(), &[new.clone()])
        .expect("Sam changes the relay");
    let at = svrngs_node::iroh::EndpointAddr::new(sam.net().id())
        .with_relay_url(new.clone())
        .with_relay_url(build.parse().unwrap());
    let synced = alex.host().sync_with(&seed.id(), &at).await;
    assert!(synced.stop.is_none(), "Alex reaches Sam: {:?}", synced.stop);
    assert_eq!(meets(&alex, &seed.id()), vec![new.to_string()]);
}

/// **`link --relay` while svrngs runs asks it** (KEEP_GOING_AT_HOME 28): the
/// command as a person types it, beside a svrngs holding the data dir's
/// lock and serving the group's record, asked through `cli::Here` of its
/// routes. Sam's asks his: where house meets changes and the link printed
/// carries it. Alex's, not the keeper's, is refused in the keeper rule's
/// words, exit 1, and writes nothing. Watched failing on two plants: the
/// busy arm's `link` dropped, so the lock's refusal stands (cli.rs
/// `answer`), and its refusal's words read from no field, so it reads as
/// an older build (cli.rs `link_running`).
#[tokio::test]
async fn link_relay_asks_the_svrngs_running_here() {
    let wire = Wire::default();
    let (old, new, build) = relays();
    let ours = RelayConfig {
        relay_urls: vec![build.clone()],
        ..cli::reach(MeetAt::Svrngs)
    };
    let sam = computer(&wire, 78).await;
    let (name, person) = (
        svrngs::group::Name::parse("house").unwrap(),
        svrngs::group::Name::parse("Sam"),
    );
    let relays = [old.parse().unwrap()];
    let (seed, printed) = cli::new_group(&sam.node, name, person.unwrap(), None, &relays, &ours)
        .await
        .expect("Sam makes house");
    let Verb::Join { link, person, .. } =
        cli::parse(argv(&format!("join {} --as Alex", printed.client))).unwrap()
    else {
        panic!("join parses as join");
    };
    let alex = computer(&wire, 79).await;
    cli::join_group(&alex.node, &alex.key, &link, &person.unwrap())
        .await
        .expect("Alex joins");
    let _held = [&sam, &alex].map(|c| cli::claim(c.dir.path()).expect("the lock is free"));
    let link = async |who: &crate::Computer| {
        let verb = cli::parse(argv(&format!("link house --relay {new}"))).unwrap();
        cli::answer(verb, &crate::Running(who), &ours).await
    };

    let before = acts(&alex.node, &seed.id());
    let refused = link(&alex).await;
    let said = refused.as_ref().expect_err("Alex is refused").to_string();
    assert_eq!(refused.unwrap_err().exit_code(), 1);
    assert!(
        said.contains("only the group's keeper decides where it meets"),
        "{said}"
    );
    assert_eq!(
        acts(&alex.node, &seed.id()),
        before,
        "the refusal writes no act"
    );

    let said = link(&sam).await.expect("Sam's link");
    assert_eq!(meets(&sam.node, &seed.id()), vec![new.clone()]);
    let line = said
        .lines()
        .find(|l| l.contains("svrngs://join/"))
        .expect(&said);
    let minted = Link::parse(line.trim()).expect("the link printed is whole");
    assert!(
        minted.token.relays.iter().any(|r| r.to_string() == new),
        "the link printed carries the new relay: {:?}",
        minted.token.relays
    );
}
