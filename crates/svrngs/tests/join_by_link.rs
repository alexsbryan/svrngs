//! Join by link, U2's bar (PLAN §4): a link's form parsed, and an
//! issuer's seed believed only when it hashes to its group. Every claim
//! decided by `cli::join_group`, `cli::link_to`, the issuer's routes or
//! the svrngs running here is verbs/join.rs's, on the wire
//! (docs/seams.tsv).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use svrngs::cli::{self, Verb};
use svrngs::group::{self, Name, Seed};
use svrngs_node::link::{client_link, https_link, Link, Origin, Token};
use svrngs_node::SigningKey;

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

fn argv(rest: &[&str]) -> Vec<String> {
    std::iter::once("svrngs")
        .chain(rest.iter().copied())
        .map(String::from)
        .collect()
}

/// The issuer's answer is believed only when its seed hashes to the group
/// it names: an issuer answering another group's seed is not believed.
#[test]
fn an_issuer_answering_another_groups_seed_is_not_believed() {
    let dir = tempfile::tempdir().unwrap();
    let founder = SigningKey::from_bytes(&[1; 32]).verifying_key();
    let house = Seed::from_parts(name("house"), name("Sam"), founder, [9; 16]);
    let work = Seed::from_parts(name("work"), name("Sam"), founder, [9; 16]);
    group::write_seed(dir.path(), &house).unwrap();
    let text = group::seed_text(dir.path()).unwrap().expect("a seed kept");
    assert_eq!(group::answered(&house.id(), &text), Some(house.clone()));
    assert_eq!(group::answered(&work.id(), &text), None);
    assert_eq!(group::answered(&house.id(), "{}"), None);
}

/// `join` parses a link in each of its forms and `--as`; a link cut short
/// is refused before anything is dialled.
#[test]
fn join_parses_its_link() {
    let founder = SigningKey::from_bytes(&[1; 32]);
    let token = Token {
        issuer: founder.verifying_key(),
        secret: [7; 16],
        relays: Vec::new(),
    };
    let page = Origin::parse("https://x.example/").unwrap();
    for (raw, origin) in [
        (client_link(&token), None),
        (https_link(&page, &token), Some(page.clone())),
        (token.encode(), None),
    ] {
        assert_eq!(
            cli::parse(argv(&["join", &raw, "--as", "Alex"])).unwrap(),
            Verb::Join {
                link: Box::new(Link {
                    token: token.clone(),
                    origin,
                }),
                person: Some(name("Alex")),
                run: true,
            },
            "{raw}"
        );
        let cut = &raw[..raw.len() - 4];
        assert!(cli::parse(argv(&["join", cut])).is_err(), "{cut}");
    }
    let url = client_link(&token);
    assert!(matches!(
        cli::parse(argv(&["join", &url, "--no-run"])),
        Ok(Verb::Join { run: false, .. })
    ));
    assert!(cli::parse(argv(&["new", "x", "--no-run"])).is_err());
    assert!(cli::parse(argv(&["join"])).is_err());
    assert!(cli::parse(argv(&["join", &url, "--origin", "https://x.example/"])).is_err());
}
