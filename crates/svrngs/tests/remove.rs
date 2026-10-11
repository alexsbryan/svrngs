//! `svrngs remove` and `svrngs handover` (U7; DESIGN §6 "Removal has one
//! decider"; §10, 2026-10-09: removing someone stays in the command), asked
//! of svrngs running here as `share --folder` is.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use svrngs::cli::{self, Refusal, Said, Verb};
use svrngs::group::Name;
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::removal::Outcome;
use svrngs_node::{actor_of, Person, RailAct, SigningKey};

fn no_relay() -> RelayConfig {
    RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: true,
    }
}

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

fn argv(rest: &[&str]) -> Vec<String> {
    std::iter::once("svrngs".to_string())
        .chain(rest.iter().map(|s| s.to_string()))
        .collect()
}

/// **The verbs parse** (U7 (e)): a name and `--group`, nothing else.
#[test]
fn remove_and_handover_parse() {
    assert_eq!(
        cli::parse(argv(&["remove", "Bo", "--group", "trip"])),
        Ok(Verb::Remove {
            person: name("Bo"),
            group: "trip".into()
        })
    );
    assert_eq!(
        cli::parse(argv(&["handover", "Ana", "--group", "trip"])),
        Ok(Verb::Handover {
            person: name("Ana"),
            group: "trip".into()
        })
    );
    assert!(
        cli::parse(argv(&["remove", "Bo"])).is_err(),
        "--group is asked"
    );
    assert!(
        cli::parse(argv(&["remove", "--group", "trip"])).is_err(),
        "a name is asked"
    );
    assert!(cli::parse(argv(&["remove", "Bo", "--group", "trip", "--folder", "x"])).is_err());
    assert!(cli::parse(argv(&["link", "trip", "--group", "trip"])).is_err());
}

/// **A person reads names and the group** (DESIGN §1): every outcome's
/// words name the person, the group or who decides, and none of the words
/// kept from people.
#[test]
fn every_outcome_is_said_in_a_persons_words() {
    let sam = Some(Person::parse("Sam").unwrap());
    let outcomes = [
        Outcome::Out,
        Outcome::Asked {
            decides: sam.clone(),
        },
        Outcome::Asked { decides: None },
        Outcome::Handed,
        Outcome::NotInGroup,
        Outcome::Decides,
        Outcome::NotDecider { decides: sam },
        Outcome::NotIn,
        Outcome::Fault,
    ];
    for outcome in outcomes {
        let said = Said::Removal {
            group: name("trip"),
            person: name("Bo"),
            outcome: outcome.clone(),
        }
        .to_string();
        assert!(said.contains("trip"), "{outcome:?}: {said}");
        for kept in ["key", "member", "keeper", "ring", " act", "journal"] {
            assert!(
                !said.to_lowercase().contains(kept),
                "{outcome:?} says {kept}: {said}"
            );
        }
    }
    let asked = Said::Removal {
        group: name("trip"),
        person: name("Bo"),
        outcome: Outcome::Asked {
            decides: Some(Person::parse("Sam").unwrap()),
        },
    };
    assert_eq!(
        asked.to_string(),
        "You asked Sam to remove Bo from trip. Bo stays in it until Sam does.\n"
    );
}

/// **The command asks svrngs running here, and the fold decides** (U7
/// (e)). Sam makes trip and lets Bo in; with no svrngs running the ask is a
/// precondition absent; Sam's `remove` of somebody not in trip is refused
/// by name, of Sam is refused as Sam decides, and of Bo is the cut — Bo
/// stands no more. Planted: the ask posted where nothing answers.
#[tokio::test]
async fn the_keepers_remove_is_the_cut() {
    let dir = tempfile::tempdir().unwrap();
    let relay = no_relay();
    let ask = |verb: Verb| cli::run(verb, dir.path().to_path_buf(), &relay);
    let remove = |who: &str| Verb::Remove {
        person: name(who),
        group: "trip".into(),
    };
    let node = Node::start(
        dir.path().to_path_buf(),
        SigningKey::from_bytes(&[5; 32]),
        &no_relay(),
        svrngs::group::seed_text,
        cli::API,
    )
    .await
    .unwrap();
    let (seed, _) = cli::new_group(&node, name("trip"), name("Sam"), None, &no_relay())
        .await
        .unwrap();
    let bo = SigningKey::from_bytes(&[6; 32]);
    let admit = RailAct::Admit {
        person: Person::parse("Bo").unwrap(),
        key: actor_of(&bo),
        invite: None,
    };
    node.rail()
        .commit(&seed.id(), &svrngs_core::tests_support::Batch(vec![admit]))
        .unwrap()
        .unwrap();
    let stands = || {
        node.rail()
            .register(&seed.id())
            .unwrap()
            .members
            .contains_key(&actor_of(&bo))
    };
    assert!(stands(), "control: Bo is in");

    let nobody = ask(remove("Dee")).await;
    assert!(
        matches!(&nobody, Err(Refusal::Removal(w)) if w == "Nobody called Dee is in trip."),
        "{nobody:?}"
    );
    let own = ask(remove("Sam")).await;
    assert!(
        matches!(&own, Err(Refusal::Removal(w)) if w.starts_with("Sam decides who is removed from trip")),
        "{own:?}"
    );
    assert!(ask(remove("Bo")).await.is_ok());
    assert!(!stands(), "Bo is out");
    node.endpoint().close().await;

    std::fs::write(
        dir.path().join(svrngs_node::api::API_FILE),
        "http://127.0.0.1:9",
    )
    .unwrap();
    let away = ask(remove("Bo")).await;
    assert!(matches!(away, Err(Refusal::NotRunning)), "{away:?}");
    assert_eq!(away.unwrap_err().exit_code(), 3);
}
