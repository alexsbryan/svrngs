//! Give's carried page, brought to a group: its files written into the
//! group's synced folder at `apps/give/`, which carries them to every
//! member's computer, where svrngs opens it as an app of the group's
//! (crates/svrngs-node/src/carried.rs: a folder `apps/<kind>/` holding an
//! `index.html`). That is the synced folder on disk, the one way of
//! svrngs's give uses for it (CHARTER fence 4). The shelf it shows is
//! written beside it, `cards.json`, from the cards this program knows.

use std::path::{Path, PathBuf};

use crate::shelf::Shelf;

/// The page's own files: each name in `apps/give/`, and its text.
const FILES: [(&str, &str); 5] = [
    ("index.html", include_str!("../page/index.html")),
    ("index.js", include_str!("../page/index.js")),
    ("look.css", include_str!("../page/look.css")),
    ("vendor/call.mjs", include_str!("../page/vendor/call.mjs")),
    ("vendor/fold.js", include_str!("../page/vendor/fold.js")),
];

/// Where in a group's folder the page goes: the folder svrngs reads a
/// carried app called `give` from.
pub const AT: &str = "apps/give";

/// The page and `shelf`'s cards written into the group folder `folder`,
/// each file only where its text differs: the files written.
pub fn bring(folder: &Path, shelf: &Shelf) -> std::io::Result<Vec<PathBuf>> {
    let cards = serde_json::to_string_pretty(&shelf.to_json()).expect("the shelf is JSON");
    let mut wrote = Vec::new();
    for (name, text) in FILES.iter().copied().chain([("cards.json", cards.as_str())]) {
        let at = folder.join(AT).join(name);
        if std::fs::read_to_string(&at).is_ok_and(|was| was == text) {
            continue;
        }
        std::fs::create_dir_all(at.with_file_name(""))?;
        std::fs::write(&at, text)?;
        tracing::debug!(at = %at.display(), "page: written");
        wrote.push(at);
    }
    Ok(wrote)
}
