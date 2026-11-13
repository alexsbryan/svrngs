//! Join by link, U2's bar (PLAN §4), where it crosses a process: `svrngs
//! join` with svrngs running here asks it, over a relay on 127.0.0.1, and
//! an older build that answers no join is said to be one. A link's form
//! parsed, and an issuer's seed believed only when it hashes to its group.
//! Every claim decided by `cli::join_group`, `cli::link_to` or the
//! issuer's routes is verbs/join.rs's, on the wire (docs/seams.tsv).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
use std::time::Duration;

use svrngs::cli::{self, Refusal, Verb};
use svrngs::group::{self, Name, Seed};
use svrngs_node::iroh::{RelayConfig, RelayUrl};
use svrngs_node::link::{client_link, https_link, Link, Origin, Token};
use svrngs_node::node::Node;
use svrngs_node::SigningKey;

#[path = "common/program.rs"]
mod program;
use program::Program;
#[path = "common/as_main.rs"]
mod as_main;

/// svrngs itself, as [`Program`] runs it (common/as_main.rs).
#[test]
#[ignore = "run by Program, in a process of its own, as svrngs"]
fn svrngs_here() {
    as_main::main()
}

fn no_relay() -> RelayConfig {
    RelayConfig {
        relay_urls: Vec::new(),
        n0_services: false,
        loopback_only: false,
    }
}

fn name(s: &str) -> Name {
    Name::parse(s).unwrap()
}

fn argv(rest: &[&str]) -> Vec<String> {
    std::iter::once("svrngs")
        .chain(rest.iter().copied())
        .map(String::from)
        .collect()
}

/// `svrngs join <raw>` parsed: the link the command would redeem.
fn typed(raw: &str) -> Link {
    match cli::parse(argv(&["join", raw])).expect("join parses") {
        Verb::Join { link, .. } => *link,
        other => panic!("{other:?}"),
    }
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

/// A plain-HTTP relay on 127.0.0.1, as `svrngs relay --at` serves one, so
/// the program in another process reaches a node in this one: the relay,
/// and its URL for a link to carry.
async fn a_relay() -> (iroh_relay::server::Server, RelayUrl) {
    let mut config = iroh_relay::server::ServerConfig::default();
    config.relay = Some(iroh_relay::server::RelayConfig::new(([127, 0, 0, 1], 0)));
    let server = iroh_relay::server::Server::spawn(config)
        .await
        .expect("a relay starts on 127.0.0.1");
    let url: RelayUrl = format!("http://{}/", server.http_addr().unwrap())
        .parse()
        .unwrap();
    (server, url)
}

/// **A join the svrngs running here did not answer says it is too old**
/// (join-while-running): svrngs holds the data dir and answers no
/// `/v1/joins`, a build from before the route, stood in for by a socket that
/// answers 404, and `svrngs join` gives the way on. Watched failing with the
/// fallback at `Refusal::Busy`, the refusal this was until an operator hit it.
#[tokio::test(flavor = "multi_thread")]
async fn a_join_an_older_svrngs_did_not_answer_says_it_is_too_old() {
    // The svrngs of an older build: a socket answering 404 to everything,
    // and the data dir's `api` naming it.
    let older = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let at = older.local_addr().unwrap();
    std::thread::spawn(move || {
        use std::io::{Read, Write};
        for asked in older.incoming() {
            let mut asked = match asked {
                Ok(a) => a,
                Err(_) => return,
            };
            let mut read = [0u8; 1024];
            let _ = asked.set_read_timeout(Some(Duration::from_secs(5)));
            let _ = asked.read(&mut read);
            let _ = asked.write_all(
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
        }
    });
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(svrngs_node::api::API_FILE),
        format!("http://{at}\n"),
    )
    .unwrap();
    std::fs::write(
        dir.path().join(svrngs_node::api::TOKEN_FILE),
        "not-checked-by-404",
    )
    .unwrap();
    let _held = cli::claim(dir.path()).expect("the older svrngs holds the data dir");

    let token = Token {
        issuer: SigningKey::from_bytes(&[1; 32]).verifying_key(),
        secret: [7; 16],
        relays: Vec::new(),
    };
    let refused = cli::run(
        Verb::Join {
            link: Box::new(typed(&client_link(&token))),
            person: Some(name("Alex")),
            run: true,
        },
        dir.path().to_path_buf(),
        &no_relay(),
    )
    .await;
    let words = refused
        .as_ref()
        .expect_err("no join, for nothing answered it")
        .to_string();
    assert!(matches!(refused, Err(Refusal::Older)), "{words}");
    assert!(
        words.contains("svrngs update") && words.contains("svrngs restart"),
        "{words}"
    );
}

/// **`svrngs join`, with svrngs running here, joins the group through it**
/// (join-while-running; DESIGN §3 "Joining"): Alex's computer joins house
/// and keeps running, holding the data dir's lock — as a computer runs
/// whose svrngs starts at login — and Alex then joins Sam's party from a
/// terminal: the second join asks the svrngs running here, as `room` and
/// `share` do, says "You're in party, as Alex." with party's folder placed
/// under Alex's own home, and leaves, exit 0, running nothing. The svrngs
/// already running keeps running, for both groups now. Planted, watched
/// failing: the second join refused, "svrngs is already running on this
/// computer", and party was never joined.
#[tokio::test(flavor = "multi_thread")]
async fn join_asks_the_svrngs_running_here_for_another_group() {
    let (relay, at) = a_relay().await;
    let ours = RelayConfig {
        relay_urls: vec![at.to_string()],
        n0_services: false,
        loopback_only: false,
    };
    let sams_dir = tempfile::tempdir().unwrap();
    let sam = Node::start(
        sams_dir.path().to_path_buf(),
        SigningKey::from_bytes(&[1; 32]),
        &ours,
        group::seed_text,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("Sam's computer starts");
    tokio::time::timeout(Duration::from_secs(20), sam.endpoint().online())
        .await
        .expect("Sam's computer reaches the relay");
    let (house, printed) =
        cli::new_group(&sam, name("house"), name("Sam"), None, &[at.clone()], &ours)
            .await
            .expect("Sam makes house");
    let (party, party_link) =
        cli::new_group(&sam, name("party"), name("Sam"), None, &[at.clone()], &ours)
            .await
            .expect("Sam makes party");

    // Alex's first join runs the computer's svrngs, holding the data dir.
    let alex = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    let logs = tempfile::tempdir().unwrap();
    let mut running = Program::at(
        alex.path(),
        &logs.path().join("first"),
        &["join", &printed.client, "--as", "Alex"],
        "",
        Some(home.path()),
    );
    running.until("Leave this running");

    // The second join, of party, asks the svrngs running here.
    let home_said = home.path().display().to_string();
    let (code, said) = Program::at(
        alex.path(),
        &logs.path().join("second"),
        &["join", &party_link.client, "--as", "Alex"],
        "",
        Some(home.path()),
    )
    .ends();
    eprintln!("svrngs join party, svrngs running: {said:#?}");
    assert_eq!(code, 0, "{said:?}");
    assert!(
        said.iter().any(|l| l.contains("You're in party, as Alex.")),
        "{said:?}"
    );
    assert!(
        said.iter()
            .any(|l| l.starts_with("party's folder is") && l.contains(&home_said)),
        "{said:?}"
    );
    assert!(
        !said.iter().any(|l| l.contains("already running")),
        "the second join is not refused: {said:?}"
    );
    assert!(
        !said.iter().any(|l| l.contains("Leave this running")),
        "the second join runs nothing: {said:?}"
    );

    // The svrngs already running keeps running, for both groups now.
    assert!(
        running.child.try_wait().unwrap().is_none(),
        "the first join's svrngs still runs"
    );
    let api = std::fs::read_to_string(alex.path().join(svrngs_node::api::API_FILE)).unwrap();
    let token = std::fs::read_to_string(alex.path().join(svrngs_node::api::TOKEN_FILE)).unwrap();
    let groups: serde_json::Value = {
        let answer = reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get(format!("{}/v1/groups", api.trim()))
            .bearer_auth(token.trim())
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap_or_default();
        serde_json::from_str(&answer).unwrap_or(serde_json::Value::Null)
    };
    let held: Vec<&str> = groups["groups"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|g| g["id"].as_str())
        .collect();
    assert!(held.contains(&house.id().as_str()), "{groups}");
    assert!(held.contains(&party.id().as_str()), "{groups}");
    drop(running);
    sam.endpoint().close().await;
    drop(relay);
}
