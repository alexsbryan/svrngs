// SPDX-License-Identifier: AGPL-3.0-or-later
//! When someone joined, said so it reads back to one instant
//! (ralph/REVIEW_FINDINGS.md #126): in the hour a fall-back night repeats,
//! two people of one name let in an hour apart are offered apart, and
//! `--joined` takes each as offered. A binary of its own: it sets `TZ`
//! before any clock is read, which no other test of the process may race.

use svrngs::cli::{self, Removal, Said, Verb};
use svrngs::group::Name;
use svrngs_node::removal::Outcome;
use svrngs_node::Joined;

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

/// 2026-11-01 01:30 in Los Angeles, twice: 08:30 UTC, still daylight
/// time, then 09:30 UTC, an hour after the clocks fell back. Planted: the
/// time written in this computer's zone with no offset, as before #126,
/// which says both "2026-11-01 01:30:00" and reads neither back.
#[test]
fn two_of_one_name_an_hour_apart_on_a_fall_back_night_are_told_apart() {
    std::env::set_var("TZ", "America/Los_Angeles");
    let (first, second) = (1_793_521_800, 1_793_525_400);
    let said = Said::Removal {
        group: name("trip"),
        person: name("Bo"),
        outcome: Outcome::Which {
            joined: vec![Joined::At(first), Joined::At(second)],
        },
    }
    .to_string();
    let offered: Vec<&str> = (said.split('`'))
        .filter_map(|s| s.strip_prefix("--joined "))
        .map(|s| s.trim_matches('"'))
        .collect();
    assert_eq!(offered.len(), 2, "{said}");
    assert_ne!(
        offered[0], offered[1],
        "two an hour apart said alike: {said}"
    );
    for (when, at) in offered.into_iter().zip([first, second]) {
        let argv = [
            "svrngs", "remove", "Bo", "--group", "trip", "--joined", when,
        ];
        assert_eq!(
            cli::parse(argv.map(String::from).to_vec()),
            Ok(Verb::Remove {
                kind: Removal::Remove,
                person: name("Bo"),
                group: "trip".into(),
                joined: Some(Joined::At(at)),
            }),
            "--joined {when}, as offered in: {said}"
        );
    }
}
