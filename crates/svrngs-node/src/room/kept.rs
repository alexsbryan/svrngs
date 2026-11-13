// SPDX-License-Identifier: AGPL-3.0-or-later
//! The room's store: which lends are on, in [`super::ROOM_FILE`] in the data
//! dir, by the lender's key and the lend's name with the network each was
//! turned on at, so the room opens again as svrngs starts, of whatever a
//! store keeps so. Written whole, through a file renamed into place
//! ([`crate::declared::write_whole`]). And what both of a store's owners
//! are told by, the room's ([`super::owner`]) and Play's ([`crate::play`]).

use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::{mpsc, oneshot};

use super::owner::LOOK_EVERY;
use crate::lend::LendName;

/// A change to the lends on, sent to their owner: these on, this one off,
/// or none, to read them.
pub(crate) enum Turn<On> {
    On(Vec<On>),
    Off(String, LendName),
    Read,
}

impl<On: Clone> Turn<On> {
    /// The lends `on` once turned, each lend, as `id` tells them apart, on
    /// once; `None` for a read, which changes nothing.
    fn made(self, on: &[On], id: fn(&On) -> (&String, &LendName)) -> Option<Vec<On>> {
        let mut on = on.to_vec();
        match self {
            Turn::On(lends) => lends.into_iter().for_each(|l| {
                on.retain(|o| id(o) != id(&l));
                on.push(l);
            }),
            Turn::Off(key, name) => on.retain(|o| id(o) != (&key, &name)),
            Turn::Read => return None,
        }
        Some(on)
    }
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

/// A store's owner, for as long as a handle is left: the lends on, read
/// from `file` as it starts; at each tick of [`LOOK_EVERY`], each `serve`d;
/// at an ask, its turn made and written to `file`, then served, and its
/// asker answered with what stands, or why it cannot be read. A turn, not
/// a read, writes over what is on: one that could not be read too.
pub(crate) async fn run<A, On: Clone + Serialize + DeserializeOwned>(
    file: PathBuf,
    mut asked: Asks<Turn<On>, A>,
    id: fn(&On) -> (&String, &LendName),
    mut serve: impl AsyncFnMut(&[On]) -> Vec<A>,
) {
    // None before the first is turned on; why, when it cannot be read or
    // parsed, which the room says and serves nothing from (AGENTS.md 6).
    let mut on = crate::declared::read_json(&file).map(Option::unwrap_or_default);
    tracing::debug!(target: "room", file = %file.display(), error = ?on.as_ref().err(), "room: the lends on, as kept: none before the first, and one unread said, serving nothing");
    let mut every = tokio::time::interval(LOOK_EVERY);
    loop {
        let done = tokio::select! {
            _ = every.tick() => None,
            turned = asked.recv() => {
                let Some((turn, done)) = turned else { return };
                if let Some(made) = turn.made(on.as_deref().unwrap_or_default(), id) {
                    // Written whole, so it opens again as svrngs starts.
                    let bytes = serde_json::to_vec(&made).expect("the room is JSON");
                    let kept = crate::declared::write_whole(&file, &bytes);
                    tracing::debug!(target: "room", file = %file.display(), on = made.len(), ?kept, "room: turned, and kept so it opens again as svrngs starts");
                    on = Ok(made);
                }
                Some(done)
            }
        };
        let served = serve(on.as_deref().unwrap_or_default()).await;
        done.map(|d| d.send(on.as_ref().map(|_| served).map_err(String::clone)));
    }
}
