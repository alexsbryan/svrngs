//! The shelf: the cards give knows, each by its kind, the name of its
//! folder. Give's own cards are built into the program (`cards/`), so a
//! computer that keeps needs only svrngs, Podman and give (CHARTER); an
//! author adds one by naming its folder (`--card`), as `give check` takes it.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value, json};

use crate::card::{Card, Protocol, Refused};

/// Give's own cards: each kind, and its compose.yaml.
const OWN: [(&str, &str); 2] = [
    ("factorio", include_str!("../cards/factorio/compose.yaml")),
    ("minecraft", include_str!("../cards/minecraft/compose.yaml")),
];

#[derive(Debug, thiserror::Error)]
pub enum ShelfError {
    #[error("{0}: no card here ({1})")]
    Unread(String, String),
    #[error("{0}: {1}")]
    Refused(String, Refused),
    #[error("{0} is not a kind of app: lowercase letters, digits and -")]
    Kind(String),
}

/// The cards, by kind.
#[derive(Debug, Clone, Default)]
pub struct Shelf(BTreeMap<String, Card>);

impl Shelf {
    /// Give's own cards, and one more from each folder in `more`, which
    /// stands in for an own card of its kind.
    pub fn with(more: &[impl AsRef<Path>]) -> Result<Shelf, ShelfError> {
        let mut shelf = BTreeMap::new();
        for (kind, text) in OWN {
            let card = Card::parse(text).map_err(|e| ShelfError::Refused(kind.into(), e))?;
            shelf.insert(kind.to_string(), card);
        }
        for dir in more {
            let dir = dir.as_ref();
            let kind = kind_of(dir)?;
            let file = dir.join("compose.yaml");
            let text = std::fs::read_to_string(&file)
                .map_err(|e| ShelfError::Unread(file.display().to_string(), e.to_string()))?;
            let card = Card::parse(&text).map_err(|e| ShelfError::Refused(kind.clone(), e))?;
            tracing::debug!(kind, version = card.version, "shelf: a card from its folder");
            shelf.insert(kind, card);
        }
        Ok(Shelf(shelf))
    }

    pub fn get(&self, kind: &str) -> Option<&Card> {
        self.0.get(kind)
    }

    /// Each card, by kind, in the shelf's order.
    pub fn cards(&self) -> impl Iterator<Item = (&String, &Card)> {
        self.0.iter()
    }

    /// What the page shows of each card, in the shelf's order: its kind,
    /// title, about, door, the say and link of each ask, and its version,
    /// which a start names.
    pub fn to_json(&self) -> Value {
        (self.0.iter())
            .map(|(kind, c)| {
                let asks: Vec<_> = c.asks.iter().map(|a| json!({ "say": a.say, "link": a.link })).collect();
                let door = match c.door.protocol {
                    Protocol::Http => "http",
                    Protocol::Tcp => "tcp",
                    Protocol::Udp => "udp",
                };
                json!({ "kind": kind, "title": c.title, "about": c.about, "door": door, "asks": asks, "version": c.version })
            })
            .collect()
    }
}

/// A card's kind: its folder's name, lowercased, as `give check` names it.
pub fn kind_of(dir: &Path) -> Result<String, ShelfError> {
    let name = dir
        .canonicalize()
        .ok()
        .and_then(|d| d.file_name().map(|n| n.to_string_lossy().to_lowercase()));
    let name = name.unwrap_or_default();
    let ok = !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    ok.then_some(name.clone()).ok_or(ShelfError::Kind(name))
}
