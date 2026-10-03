//! The journal actor (DESIGN D2a): one thread owns the group's directory and
//! its [`Journal`], and is its one writer. Everything else holds a
//! [`JournalHandle`] and sends it messages.

use std::thread::JoinHandle;

use ed25519_dalek::SigningKey;
use serde::Deserialize;
use svrngs_core::{ActId, Actor, AppId, Body, Ingest, Journal, OnBehalfOf, Payload, Record};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info};

use crate::home::sign;
use crate::store::GroupDir;

/// What the bridge asks to write (DESIGN C2, the bridge append): an app's op,
/// for a chat speaker, answering one interaction. Every field is required, so
/// an append with no interaction id never reaches the journal.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgeAppend {
    pub app: AppId,
    pub op: Payload,
    pub on_behalf_of: OnBehalfOf,
}

/// An append's outcome: a new act, or the act this interaction already made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appended {
    New(ActId),
    Taken(ActId),
}

#[derive(Debug, thiserror::Error)]
pub enum AppendError {
    #[error("the group's journal could not be written: {0}")]
    Store(String),
    #[error("the journal refused its own act: {0}")]
    Refused(String),
    #[error("the journal has stopped")]
    Stopped,
}

enum Msg {
    Append(BridgeAppend, oneshot::Sender<Result<Appended, AppendError>>),
}

#[derive(Clone)]
pub struct JournalHandle {
    tx: mpsc::Sender<Msg>,
}

impl JournalHandle {
    pub async fn append(&self, req: BridgeAppend) -> Result<Appended, AppendError> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(Msg::Append(req, reply))
            .await
            .map_err(|_| AppendError::Stopped)?;
        rx.await.map_err(|_| AppendError::Stopped)?
    }
}

struct JournalActor {
    dir: GroupDir,
    journal: Journal,
    bridge: SigningKey,
    clock: fn() -> u64,
}

/// Start the actor on its own thread; it ends when the last handle drops.
pub fn spawn(
    dir: GroupDir,
    journal: Journal,
    bridge: SigningKey,
    clock: fn() -> u64,
) -> (JournalHandle, JoinHandle<()>) {
    let (tx, mut rx) = mpsc::channel(64);
    let mut actor = JournalActor {
        dir,
        journal,
        bridge,
        clock,
    };
    let thread = std::thread::spawn(move || {
        while let Some(Msg::Append(req, reply)) = rx.blocking_recv() {
            let _ = reply.send(actor.append(req));
        }
        info!(dir = %actor.dir.path().display(), "journal:stop");
    });
    (JournalHandle { tx }, thread)
}

impl JournalActor {
    fn append(&mut self, req: BridgeAppend) -> Result<Appended, AppendError> {
        let now = (self.clock)();
        let actor = Actor::from_key(&self.bridge.verifying_key());
        let interaction = req.on_behalf_of.interaction.clone();
        // A retried interaction — the adapter resending after a restart, on
        // either side — is answered with the act it already made. `taken` is
        // rebuilt from the directory on open, so this survives a restart.
        if let Some(id) = self.journal.taken(&interaction) {
            debug!(?id, interaction = interaction.as_str(), "journal:append_taken");
            return Ok(Appended::Taken(*id));
        }
        let body = Body::Record(Record {
            app: req.app,
            op: req.op,
            on_behalf_of: Some(req.on_behalf_of),
        });
        let draft = self
            .journal
            .draft(&actor, &body, now)
            .map_err(|e| AppendError::Refused(e.to_string()))?;
        let act = draft
            .clone()
            .seal(sign(&self.bridge, draft.id().as_bytes()))
            .map_err(|e| AppendError::Refused(e.to_string()))?;
        match self.journal.decide(act, now) {
            Ingest::Held(h) => {
                self.dir.append(h.act()).map_err(|e| {
                    error!(error = %e, "journal:write_failed");
                    AppendError::Store(e.to_string())
                })?;
                let id = *h.act().id();
                self.journal.apply(h);
                debug!(?id, interaction = interaction.as_str(), "journal:append_new");
                Ok(Appended::New(id))
            }
            Ingest::AlreadyHeld(id) => {
                debug!(?id, "journal:append_already_held");
                Ok(Appended::Taken(id))
            }
            Ingest::OfferAgain(why) => {
                error!(%why, "journal:own_act_not_holdable");
                Err(AppendError::Refused(why.to_string()))
            }
        }
    }
}
