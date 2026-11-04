// SPDX-License-Identifier: AGPL-3.0-or-later
//! The room's store: which lends are on, in [`super::ROOM_FILE`] in the data
//! dir, by the lender's key and the lend's name with the network each was
//! turned on at, so the room opens again as svrngs starts, of whatever a
//! store keeps so. Written whole, through a file renamed into place
//! ([`crate::declared::write_whole`]).

use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};

/// The lends on, as `file` keeps them: none before the first is turned on;
/// why, when it cannot be read or parsed, which the room says and serves
/// nothing from (AGENTS.md 6).
pub(crate) fn read<T: DeserializeOwned>(file: &Path) -> Result<Vec<T>, String> {
    let on = match std::fs::read(file) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        read => (read.map_err(|e| e.to_string()))
            .and_then(|b| serde_json::from_slice(&b).map_err(|e| e.to_string())),
    };
    tracing::debug!(target: "room", file = %file.display(), error = ?on.as_ref().err(), "room: the lends on, as kept: none before the first, and one unread said, serving nothing");
    on
}

/// `on` written to `file` whole, so it opens again as svrngs starts.
pub(crate) fn write<T: Serialize>(file: &Path, on: &[T]) {
    let bytes = serde_json::to_vec(on).expect("the room is JSON");
    let kept = crate::declared::write_whole(file, &bytes);
    tracing::debug!(target: "room", file = %file.display(), on = on.len(), ?kept, "room: turned, and kept so it opens again as svrngs starts");
}
