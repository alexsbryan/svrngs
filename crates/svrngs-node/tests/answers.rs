// SPDX-License-Identifier: AGPL-3.0-or-later
//! What svrngs running here answers the command, byte for byte: each
//! outcome of a join it was asked (join.rs, `JOINS_ROUTE`) and a link it
//! made (home.rs, `LINKS_ROUTE`), as an older command reads them. Each is
//! read back as itself. Red on the plant of any field renamed, reordered
//! or dropped, or a variant's tag respelt.

use svrngs_node::home::LinkMade;
use svrngs_node::join::Outcome;

#[test]
fn each_join_outcome_keeps_its_bytes() {
    let seed = || "{\"seed\":1}".to_string();
    let dir = || "/home/ana/svrngs/house".to_string();
    let group = || "house".to_string();
    let cases = [
        (
            Outcome::In {
                seed: seed(),
                person: "Ana".into(),
                dir: dir(),
                placed: None,
            },
            r#"{"outcome":"in","seed":"{\"seed\":1}","person":"Ana","dir":"/home/ana/svrngs/house"}"#,
        ),
        (
            Outcome::In {
                seed: seed(),
                person: "Ana".into(),
                dir: dir(),
                placed: Some("taken".into()),
            },
            r#"{"outcome":"in","seed":"{\"seed\":1}","person":"Ana","dir":"/home/ana/svrngs/house","placed":"taken"}"#,
        ),
        (
            Outcome::AlreadyIn {
                seed: seed(),
                dir: dir(),
                placed: None,
            },
            r#"{"outcome":"already_in","seed":"{\"seed\":1}","dir":"/home/ana/svrngs/house"}"#,
        ),
        (Outcome::LinkRefused, r#"{"outcome":"link_refused"}"#),
        (
            Outcome::NameTaken {
                holder: "Sam".into(),
                group: group(),
            },
            r#"{"outcome":"name_taken","holder":"Sam","group":"house"}"#,
        ),
        (Outcome::IssuerAway, r#"{"outcome":"issuer_away"}"#),
        (
            Outcome::Keep { group: group() },
            r#"{"outcome":"keep","group":"house"}"#,
        ),
        (
            Outcome::Left { group: group() },
            r#"{"outcome":"left","group":"house"}"#,
        ),
        (
            Outcome::Older { group: group() },
            r#"{"outcome":"older","group":"house"}"#,
        ),
    ];
    for (outcome, bytes) in cases {
        assert_eq!(serde_json::to_string(&outcome).unwrap(), bytes);
        assert_eq!(serde_json::from_str::<Outcome>(bytes).unwrap(), outcome);
    }
}

#[test]
fn a_link_made_keeps_its_bytes() {
    let made = LinkMade {
        client: "svrngs://join/abc".into(),
        device: Some(svrngs_node::Person::parse("Sam").unwrap()),
        https: Some("https://group.svrnme.sh/#abc".into()),
        online: false,
    };
    let bytes = r#"{"client":"svrngs://join/abc","device":"Sam","https":"https://group.svrnme.sh/#abc","online":false}"#;
    assert_eq!(serde_json::to_string(&made).unwrap(), bytes);
    assert_eq!(serde_json::from_str::<LinkMade>(bytes).unwrap(), made);
    let plain = r#"{"client":"svrngs://join/abc","device":null,"https":null,"online":true}"#;
    let made = LinkMade {
        device: None,
        https: None,
        online: true,
        ..made
    };
    assert_eq!(serde_json::to_string(&made).unwrap(), plain);
}
