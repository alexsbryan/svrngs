//! The shelf's two cards read as give runs them, each pinned (GIVE.md §4).
//! Whether each runs is `give check`'s, recorded in
//! demo/runs/give/2026-10-26-cards/.

use give::card::{Card, Protocol};

fn card(kind: &str) -> Card {
    let at = format!("{}/cards/{kind}/compose.yaml", env!("CARGO_MANIFEST_DIR"));
    Card::parse(&std::fs::read_to_string(&at).unwrap()).unwrap_or_else(|e| panic!("{at}: {e}"))
}

#[test]
fn minecraft_is_pinned_and_asks_for_its_terms() {
    let c = card("minecraft");
    assert_eq!(
        (c.title.as_str(), c.door.port, c.door.protocol),
        ("Minecraft", 25565, Protocol::Tcp)
    );
    assert_eq!(c.state, ["world"]);
    let server = &c.services[0];
    assert!(server.image.as_str().contains("@sha256:"));
    let version = server
        .environment
        .iter()
        .find(|(k, _)| k == "VERSION")
        .map(|(_, v)| v.as_str());
    assert!(
        matches!(version, Some(v) if v != "LATEST" && !v.is_empty()),
        "VERSION pinned: {version:?}"
    );
    assert_eq!(c.asks.len(), 1);
    assert_eq!(c.asks[0].link, "https://aka.ms/MinecraftEULA");
    assert_eq!(c.asks[0].sets, [("EULA".to_string(), "TRUE".to_string())]);
    assert!(
        !server.environment.iter().any(|(k, _)| k == "EULA"),
        "the EULA is the keeper's yes, never the card's"
    );
}

#[test]
fn factorio_is_pinned_and_asks_nothing() {
    let c = card("factorio");
    assert_eq!(
        (c.title.as_str(), c.door.port, c.door.protocol),
        ("Factorio", 34197, Protocol::Udp)
    );
    assert_eq!(c.state, ["world"]);
    assert!(c.services[0].image.as_str().contains("@sha256:"));
    assert!(c.services[0].health.is_some());
    assert!(c.asks.is_empty());
}

#[test]
fn each_card_carries_its_nouns() {
    for kind in ["minecraft", "factorio"] {
        let at = format!("{}/cards/{kind}/nouns.txt", env!("CARGO_MANIFEST_DIR"));
        let nouns = std::fs::read_to_string(&at).unwrap();
        assert!(nouns.lines().any(|l| l.trim() == kind), "{at} names {kind}");
    }
}
