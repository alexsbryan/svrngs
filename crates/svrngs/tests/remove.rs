//! `svrngs remove` and `svrngs handover` (U7; DESIGN §6 "Removal has one
//! decider"; §10, 2026-10-09: removing someone stays in the command), asked
//! of svrngs running here as `share --folder` is.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use svrngs::cli::{self, Refusal, RemoveKind, Said, Verb};
use svrngs::group::{Name, Seed};
use svrngs_node::iroh::RelayConfig;
use svrngs_node::node::Node;
use svrngs_node::removal::Outcome;
use svrngs_node::{actor_of, Joined, Person, RailAct, SigningKey};

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

/// Sam's computer, running here in `dir`, and the group trip it made.
async fn sams_trip(dir: &std::path::Path) -> (Node, Seed) {
    let node = Node::start(
        dir.to_path_buf(),
        SigningKey::from_bytes(&[5; 32]),
        &no_relay(),
        svrngs::group::seed_text,
        cli::API,
    )
    .await
    .unwrap();
    let (seed, _) = cli::new_group(&node, name("trip"), name("Sam"), None, None, &no_relay())
        .await
        .unwrap();
    (node, seed)
}

/// Bo let into trip as `bo` by hand, with no link.
fn let_bo_in(node: &Node, seed: &Seed, bo: &SigningKey) {
    let admit = RailAct::Admit {
        person: Person::parse("Bo").unwrap(),
        key: actor_of(bo),
        invite: None,
    };
    node.rail()
        .commit(&seed.id(), &svrngs_core::tests_support::Batch(vec![admit]))
        .unwrap()
        .unwrap();
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
            kind: RemoveKind::Remove,
            person: name("Bo"),
            group: "trip".into(),
            joined: None,
        })
    );
    assert_eq!(
        cli::parse(argv(&["handover", "Ana", "--group", "trip"])),
        Ok(Verb::Remove {
            kind: RemoveKind::Handover,
            person: name("Ana"),
            group: "trip".into(),
            joined: None,
        })
    );
    assert_eq!(
        cli::parse(argv(&[
            "remove", "Bo", "--group", "trip", "--joined", "started"
        ])),
        Ok(Verb::Remove {
            kind: RemoveKind::Remove,
            person: name("Bo"),
            group: "trip".into(),
            joined: Some(Joined::Started),
        })
    );
    assert!(
        matches!(
            cli::parse(argv(&[
                "remove",
                "Bo",
                "--group",
                "trip",
                "--joined",
                "2026-10-12 14:03:22 -0700"
            ])),
            Ok(Verb::Remove {
                kind: RemoveKind::Remove,
                joined: Some(Joined::At(_)),
                ..
            })
        ),
        "a time with its offset"
    );
    for wrong in [
        "yesterday",
        "2026-10-12",
        "1791840217",
        "2026-10-12 14:03:22",
    ] {
        assert!(
            cli::parse(argv(&[
                "remove", "Bo", "--group", "trip", "--joined", wrong
            ]))
            .is_err(),
            "--joined {wrong}"
        );
    }
    assert!(
        cli::parse(argv(&["link", "trip", "--joined", "started"])).is_err(),
        "--joined is remove's and handover's"
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
        Outcome::Which {
            joined: vec![Joined::Started, Joined::At(1_791_840_217)],
        },
        Outcome::Decides,
        Outcome::NotDecider { decides: sam },
        Outcome::NotDeciderUnread,
        Outcome::NotIn,
        Outcome::Own,
        Outcome::Left,
        Outcome::Fault,
    ];
    for outcome in outcomes {
        let removal = Said::Removal {
            group: name("trip"),
            person: name("Bo"),
            outcome: outcome.clone(),
        };
        let left = Said::Left {
            group: name("trip"),
            outcome: outcome.clone(),
        };
        for said in [removal.to_string(), left.to_string()] {
            assert!(said.contains("trip"), "{outcome:?}: {said}");
            for kept in ["key", "member", "keeper", "ring", " act", "journal"] {
                assert!(
                    !said.to_lowercase().contains(kept),
                    "{outcome:?} says {kept}: {said}"
                );
            }
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
        kind: RemoveKind::Remove,
        person: name(who),
        group: "trip".into(),
        joined: None,
    };
    let (node, seed) = sams_trip(dir.path()).await;
    let bo = SigningKey::from_bytes(&[6; 32]);
    let_bo_in(&node, &seed, &bo);
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

/// What a refusal's words say to add to the command, between backticks,
/// as a shell would split it: each `"…"` one argument.
fn ways(words: &str) -> Vec<Vec<String>> {
    (words.split('`').skip(1).step_by(2))
        .map(|way| {
            (way.split('"').enumerate())
                .flat_map(|(i, part)| match i % 2 {
                    0 => part.split_whitespace().map(str::to_string).collect(),
                    _ => vec![part.to_string()],
                })
                .collect()
        })
        .collect()
}

/// **The keeper removes one of two people of one name** (bash-same-name
/// (a)). Sam lets two people into trip as Bo, a second apart, each by
/// hand, as a member's hand or a race at two computers gives them
/// (bash-devices made them two people, `a1d0ba9`). Sam's `remove Bo` and
/// `handover Bo` are refused — svrngs never guesses which Bo — in words
/// that name, between backticks, what to add to the command to name one;
/// following the first, one Bo is out and the other stands, and following
/// the second, the other is out. Planted: two people of one name, the
/// keeper unable to remove either by any means the command offers.
#[tokio::test]
async fn the_keeper_removes_one_of_two_people_of_one_name() {
    let dir = tempfile::tempdir().unwrap();
    let relay = no_relay();
    let ask = |verb: Verb| cli::run(verb, dir.path().to_path_buf(), &relay);
    let (node, seed) = sams_trip(dir.path()).await;
    let bos = [6u8, 7].map(|b| SigningKey::from_bytes(&[b; 32]));
    for bo in &bos {
        let_bo_in(&node, &seed, bo);
        tokio::time::sleep(std::time::Duration::from_millis(1_100)).await;
    }
    let stand = || {
        let members = node.rail().register(&seed.id()).unwrap().members;
        bos.each_ref().map(|bo| members.contains_key(&actor_of(bo)))
    };
    assert_eq!(stand(), [true, true], "control: both Bos are in");

    let verb = |v: &str, more: &[String]| {
        let mut args = argv(&[v, "Bo", "--group", "trip"]);
        args.extend(more.iter().cloned());
        cli::parse(args)
    };
    for v in ["handover", "remove"] {
        let refused = ask(verb(v, &[]).unwrap()).await;
        let Err(Refusal::Removal(words)) = &refused else {
            panic!("{v} Bo: {refused:?}")
        };
        eprintln!("{v} Bo: {words}");
        for kept in ["key", "member", "keeper", "ring", " act", "journal"] {
            assert!(
                !words.to_lowercase().contains(kept),
                "{v} says {kept}: {words}"
            );
        }
        assert_eq!(stand(), [true, true], "{v} Bo changed nothing");
        assert_eq!(
            ways(words).len(),
            2,
            "{v} Bo: the refusal names a way to name each Bo: {words}"
        );
    }
    let Err(Refusal::Removal(words)) = ask(verb("remove", &[]).unwrap()).await else {
        unreachable!("refused above")
    };
    let [first, second] = <[Vec<String>; 2]>::try_from(ways(&words)).unwrap();
    let out = ask(verb("remove", &first).unwrap()).await;
    assert!(out.is_ok(), "remove Bo {first:?}: {out:?}");
    assert_eq!(stand(), [false, true], "the first Bo out, the other stands");
    let out = ask(verb("remove", &second).unwrap()).await;
    assert!(out.is_ok(), "remove Bo {second:?}: {out:?}");
    assert_eq!(stand(), [false, false], "the second Bo out");
    node.endpoint().close().await;
}

/// **`svrngs leave` asks svrngs running here, and whoever decides who is
/// removed hands that on first** (groups-leave (b)). `leave trip` parses;
/// Sam, who made trip, is refused while Bo is in it, in words that say
/// to hand on first and how; Sam hands on to Bo and leaves, told that
/// their copy and folder stay and that this computer cannot come back yet;
/// Sam's copy, read, holds Sam out and Bo deciding. At 1b19d80f it does not compile: no
/// `leave` verb.
#[tokio::test]
async fn the_keeper_hands_on_then_leaves() {
    assert_eq!(
        cli::parse(argv(&["leave", "trip"])),
        Ok(Verb::Leave("trip".into()))
    );
    assert!(cli::parse(argv(&["leave"])).is_err(), "a group is asked");
    let dir = tempfile::tempdir().unwrap();
    let relay = no_relay();
    let ask = |verb: Verb| cli::run(verb, dir.path().to_path_buf(), &relay);
    let (node, seed) = sams_trip(dir.path()).await;
    let bo = SigningKey::from_bytes(&[6; 32]);
    let_bo_in(&node, &seed, &bo);

    let refused = ask(Verb::Leave("trip".into())).await;
    assert!(
        matches!(&refused, Err(Refusal::Removal(w)) if w.starts_with("You decide who is removed from trip, so you cannot leave it yet")),
        "{refused:?}"
    );
    let to_bo = Verb::Remove {
        kind: RemoveKind::Handover,
        person: name("Bo"),
        group: "trip".into(),
        joined: None,
    };
    assert!(ask(to_bo).await.is_ok());
    assert!(ask(Verb::Leave("trip".into())).await.is_ok(), "Sam leaves");
    let register = node.rail().register(&seed.id()).unwrap();
    let sam = actor_of(&SigningKey::from_bytes(&[5; 32]));
    assert!(register.removed.contains_key(&sam), "Sam is out");
    assert_eq!(register.keeper, Some(actor_of(&bo)), "Bo decides");
    let words = Said::Left {
        group: name("trip"),
        outcome: Outcome::Left,
    }
    .to_string();
    for said in [
        "stay on this computer",
        "new invitation",
        "cannot come back yet",
    ] {
        assert!(words.contains(said), "{words}");
    }
    node.endpoint().close().await;
}
