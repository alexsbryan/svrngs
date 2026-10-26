//! A card is read once, and what give will not run is refused by name
//! (GIVE.md §4). No podman here: these read text.

use std::time::Duration;

use give::card::{Card, HealthTest, Protocol, Refused};

const STANDIN: &str = include_str!("standin/compose.yaml");

fn refused(card: &str) -> (String, String) {
    match Card::parse(card) {
        Err(Refused::Field { field, why }) => (field, why),
        other => panic!("expected a refusal by field, got {other:?}"),
    }
}

fn with(from: &str, to: &str) -> String {
    assert!(STANDIN.contains(from), "the stand-in has no {from:?}");
    STANDIN.replacen(from, to, 1)
}

#[test]
fn the_stand_in_is_read_whole() {
    let card = Card::parse(STANDIN).unwrap();
    assert_eq!(card.title, "Stand-in");
    assert_eq!((card.door.port, card.door.protocol), (8080, Protocol::Http));
    assert_eq!(card.state, ["data"]);
    assert_eq!(card.services.len(), 1);
    let web = &card.services[0];
    assert!(web.image.as_str().contains("@sha256:"));
    let health = web.health.as_ref().unwrap();
    assert!(matches!(&health.test, HealthTest::Exec(w) if w[0] == "wget"));
    assert_eq!(
        (health.interval, health.retries, health.start_period),
        (Duration::from_secs(1), 10, Duration::ZERO)
    );
    assert_eq!(web.stop_grace, Duration::from_secs(10), "compose's default");
}

/// The five refusals GIVE.md §4 names, each for what it would let an app do
/// to the computer that keeps it, each naming its field.
#[test]
fn what_could_reach_the_computer_is_refused_by_name() {
    let svc = "    image: docker.io";
    let cases = [
        (
            with(svc, "    build: .\n    image: docker.io"),
            "services.web.build",
            "never builds",
        ),
        (
            with(svc, "    privileged: true\n    image: docker.io"),
            "services.web.privileged",
            "privileged",
        ),
        (
            with(svc, "    network_mode: host\n    image: docker.io"),
            "services.web.network_mode",
            "host networking",
        ),
        (
            with(svc, "    networks: [back]\n    image: docker.io"),
            "services.web.networks",
            "custom network",
        ),
        (
            format!("{STANDIN}networks:\n  back: {{}}\n"),
            "networks",
            "custom network",
        ),
        (
            with("[\"data:/data\"]", "[\"/Users/me:/data\"]"),
            "services.web.volumes",
            "bind mount",
        ),
        (
            with("[\"data:/data\"]", "[\"./here:/data\"]"),
            "services.web.volumes",
            "bind mount",
        ),
        (
            with("[\"data:/data\"]", "[{type: bind, source: /etc, target: /data}]"),
            "services.web.volumes",
            "bind mount",
        ),
    ];
    for (card, field, says) in cases {
        let (f, why) = refused(&card);
        assert_eq!(f, field, "{why}");
        assert!(why.contains(says), "{field}: {why}");
    }
}

#[test]
fn a_card_runs_only_what_it_can_say() {
    let cases = [
        (
            with(
                "@sha256:bdf57e528e45e4433820e045b29b4597825a1c9e38353532d90a01445013f82e",
                "",
            ),
            "services.web.image",
        ),
        (
            with("    image: docker.io", "    cap_add: [ALL]\n    image: docker.io"),
            "services.web.cap_add",
        ),
        (with("state: [data]", "state: [world]"), "x-svrngs.state"),
        (
            with("port: 8080, protocol: http", "port: 9090, protocol: http"),
            "x-svrngs.door",
        ),
        (
            with("port: 8080, protocol: http", "port: 8080, protocol: udp"),
            "x-svrngs.door",
        ),
        (
            with("port: 8080, protocol: http", "port: 8080, protocol: gopher"),
            "x-svrngs.door.protocol",
        ),
        (
            with("  state: [data]", "  state: [data]\n  hooks: [x]"),
            "x-svrngs.hooks",
        ),
        (with("  state: [data]\n", ""), "x-svrngs"),
        (
            with(
                "[\"sh\", \"-c\", \"date >> /data/starts && exec httpd -f -p 8080 -h /data\"]",
                "httpd -f",
            ),
            "services.web.command",
        ),
        (with("data: {}", "data: {external: true}"), "volumes.data"),
        (
            with("interval: 1s", "interval: soon"),
            "services.web.healthcheck.interval",
        ),
    ];
    for (card, field) in cases {
        assert_eq!(refused(&card).0, field);
    }
    assert!(matches!(
        Card::parse("services: [unclosed"),
        Err(Refused::Unreadable(_))
    ));
}

#[test]
fn what_is_harmless_is_read() {
    let card = Card::parse(&with(
        "    image: docker.io",
        "    privileged: false\n    restart: unless-stopped\n    image: docker.io",
    ));
    assert!(card.is_ok(), "{card:?}");
    let card = Card::parse(&with("interval: 1s", "interval: 1m30s\n      start_period: 500ms")).unwrap();
    let h = card.services[0].health.as_ref().unwrap();
    assert_eq!(
        (h.interval, h.start_period),
        (Duration::from_secs(90), Duration::from_millis(500))
    );
    let card = Card::parse(&with("    image: docker.io", "    mem_limit: 3g\n    image: docker.io")).unwrap();
    assert_eq!(card.memory(), 3 << 30);
    let card = Card::parse(&with(
        "    image: docker.io",
        "    platform: linux/amd64\n    image: docker.io",
    ))
    .unwrap();
    assert_eq!(card.services[0].platform.as_deref(), Some("linux/amd64"));
}

#[test]
fn an_ask_carries_its_link_and_what_its_yes_sets() {
    let card = Card::parse(&format!(
        "{STANDIN}  asks:\n    - say: Its makers ask whoever runs it to agree to their terms.\n      link: https://example.org/terms\n      sets: {{ AGREED: \"TRUE\" }}\n"
    ))
    .unwrap();
    assert_eq!(card.asks.len(), 1);
    assert_eq!(card.asks[0].link, "https://example.org/terms");
    assert_eq!(card.asks[0].sets, [("AGREED".to_string(), "TRUE".to_string())]);
    let plain = with(
        "  state: [data]",
        "  state: [data]\n  asks: [{say: x, link: http://example.org, sets: {}}]",
    );
    assert_eq!(refused(&plain).0, "x-svrngs.asks.link");
}
