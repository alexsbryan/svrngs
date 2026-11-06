// SPDX-License-Identifier: AGPL-3.0-or-later
//! The room's store: which lends are on, in [`super::ROOM_FILE`] in the data
//! dir, by the lender's key and the lend's name with the network each was
//! turned on at, so the room opens again as svrngs starts, of whatever a
//! store keeps so. Written whole, through a file renamed into place
//! ([`crate::declared::write_whole`]). And what both of a store's owners
//! are told by, the room's ([`super::owner`]) and Play's ([`crate::play`]).

use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::{mpsc, oneshot};

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

/// Where an owner answers what stands after a turn, or why that cannot be
/// read.
type Done<A> = oneshot::Sender<Result<Vec<A>, String>>;

/// What an owner reads its asks from.
pub(crate) type Asks<T, A> = mpsc::UnboundedReceiver<(T, Done<A>)>;

/// An owner's handle, held by the node's routers, of turns `T` answered by
/// what stands, `A`s.
pub(crate) struct Owner<T, A>(mpsc::UnboundedSender<(T, Done<A>)>);

impl<T, A> Clone for Owner<T, A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, A> Owner<T, A> {
    /// A handle, and what its owner takes.
    pub(crate) fn new() -> (Self, Asks<T, A>) {
        let (owner, asked) = mpsc::unbounded_channel();
        (Self(owner), asked)
    }

    /// `turn` made by the owner, and what stands then, or why it cannot be
    /// read; `None` once it has stopped.
    pub(crate) async fn turn(&self, turn: T) -> Option<Result<Vec<A>, String>> {
        let (done, made) = oneshot::channel();
        self.0.send((turn, done)).ok()?;
        made.await.ok()
    }
}

/// An owner's wait for its next look: at `every`'s tick, nobody to answer;
/// at an ask, its turn made and written to `file`, and who to answer. A
/// turn, not a read, writes over `on`: one that could not be read too.
/// `None` once no handle is left.
pub(crate) async fn next<T, A, On: Serialize>(
    every: &mut tokio::time::Interval,
    asked: &mut Asks<T, A>,
    file: &Path,
    on: &mut Result<Vec<On>, String>,
    made: impl FnOnce(T, &[On]) -> Option<Vec<On>>,
) -> Option<Option<Done<A>>> {
    tokio::select! {
        _ = every.tick() => Some(None),
        turned = asked.recv() => {
            let (turn, done) = turned?;
            if let Some(made) = made(turn, on.as_deref().unwrap_or_default()) {
                write(file, &made);
                *on = Ok(made);
            }
            Some(Some(done))
        }
    }
}
