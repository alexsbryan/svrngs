//! `svrngs remove` and `svrngs handover` (U7; DESIGN §6 "Removal has one
//! decider"; PLAN §3, 2026-10-09: removing someone stays in the command), asked
//! of svrngs running here as `share --folder` is. Here: the verbs' parse
//! and their words; the asks and the fold's answers are tests/verbs/remove.rs's.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use svrngs::cli::{self, RemoveKind, Said, Verb};
use svrngs::group::Name;
use svrngs_node::removal::Outcome;
use svrngs_node::{Joined, Person};

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

/// **`svrngs leave` asks a group, and says what stays** (groups-leave (b)):
/// `leave trip` parses, `leave` alone is refused, and a leave is told that
/// its copy and folder stay and that this computer cannot rejoin yet. (Kept
/// from `the_keeper_hands_on_then_leaves`, which pinned the old fold's
/// keeper rules and went with it at release 2; the keeping scene,
/// scenes/keeping.rs, plays a hand-on, its Accept and a leave.)
#[test]
fn leave_asks_a_group_and_says_what_stays() {
    assert_eq!(
        cli::parse(argv(&["leave", "trip"])),
        Ok(Verb::Leave("trip".into()))
    );
    assert!(cli::parse(argv(&["leave"])).is_err(), "a group is asked");
    let words = Said::Left {
        group: name("trip"),
        outcome: Outcome::Left,
    }
    .to_string();
    for said in [
        "stay on this computer",
        "new invitation",
        "cannot rejoin yet",
    ] {
        assert!(words.contains(said), "{words}");
    }
}
