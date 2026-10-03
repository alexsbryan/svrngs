//! The journal actor (DESIGN D2a): one thread owns the group's directory and
//! its [`Journal`], and is its one writer. Everything else holds a
//! [`JournalHandle`] and sends it messages.

use std::thread::JoinHandle;

use ed25519_dalek::SigningKey;
use serde::Deserialize;
use svrngs_core::{
    ActId, Actor, AppId, Body, Guest, GuestChange, Ingest, Journal, Name, OnBehalfOf, Payload, Record, Speaker,
};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

use crate::home::sign;
use crate::runner::{Input, Proposal, Reminder, Reply, Runner, RunnerError};
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

/// A chat input for an app to answer (DESIGN C5): the bridge sends one for
/// every command or tap, and the handler runner decides what it writes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invocation {
    pub input: Input,
    pub on_behalf_of: OnBehalfOf,
}

/// An append's outcome: a new act, or the act this interaction already made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Appended {
    New(ActId),
    Taken(ActId),
}

/// A guest act's outcome: the change, and the act that wrote it if it wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuestWritten {
    pub change: GuestChange,
    pub id: Option<ActId>,
}

/// What the chat sees of the group: its name, how many people it has, and
/// whether anyone has ever entered it from the chat.
#[derive(Debug, Clone)]
pub struct GroupView {
    pub name: Name,
    pub people: usize,
    pub entered: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum AppendError {
    #[error("{} was removed from the group in the chat, and only rejoining lifts that", .0.as_str())]
    Removed(Speaker),
    #[error("the group's journal could not be written: {0}")]
    Store(String),
    #[error("the journal refused its own act: {0}")]
    Refused(String),
    #[error("the app could not answer: {0}")]
    App(#[from] RunnerError),
    #[error("the journal has stopped")]
    Stopped,
}

enum Msg {
    Append(BridgeAppend, oneshot::Sender<Result<Appended, AppendError>>),
    Run(Invocation, oneshot::Sender<Result<Reply, AppendError>>),
    Due(oneshot::Sender<Result<Vec<Reminder>, AppendError>>),
    Guest(Guest, oneshot::Sender<Result<GuestWritten, AppendError>>),
    Group(oneshot::Sender<GroupView>),
}

#[derive(Clone)]
pub struct JournalHandle {
    tx: mpsc::Sender<Msg>,
}

impl JournalHandle {
    pub async fn append(&self, req: BridgeAppend) -> Result<Appended, AppendError> {
        self.ask(|reply| Msg::Append(req, reply)).await?
    }

    /// Answer a chat input through its app's handlers.
    pub async fn run(&self, inv: Invocation) -> Result<Reply, AppendError> {
        self.ask(|reply| Msg::Run(inv, reply)).await?
    }

    /// The reminders due now, each fulfilled in the journal before it is
    /// handed back to be posted.
    pub async fn due(&self) -> Result<Vec<Reminder>, AppendError> {
        self.ask(Msg::Due).await?
    }

    /// Write what the chat reported about a speaker into the guest book.
    pub async fn guest(&self, guest: Guest) -> Result<GuestWritten, AppendError> {
        self.ask(|reply| Msg::Guest(guest, reply)).await?
    }

    pub async fn group(&self) -> Result<GroupView, AppendError> {
        self.ask(Msg::Group).await
    }

    async fn ask<T>(&self, msg: impl FnOnce(oneshot::Sender<T>) -> Msg) -> Result<T, AppendError> {
        let (reply, rx) = oneshot::channel();
        self.tx.send(msg(reply)).await.map_err(|_| AppendError::Stopped)?;
        rx.await.map_err(|_| AppendError::Stopped)
    }
}

struct JournalActor {
    dir: GroupDir,
    journal: Journal,
    bridge: SigningKey,
    clock: fn() -> u64,
    runner: Runner,
}

/// Start the actor on its own thread; it ends when the last handle drops.
pub fn spawn(
    dir: GroupDir,
    journal: Journal,
    bridge: SigningKey,
    clock: fn() -> u64,
    runner: Runner,
) -> (JournalHandle, JoinHandle<()>) {
    let (tx, mut rx) = mpsc::channel(64);
    let mut actor = JournalActor {
        dir,
        journal,
        bridge,
        clock,
        runner,
    };
    let thread = std::thread::spawn(move || {
        while let Some(msg) = rx.blocking_recv() {
            match msg {
                Msg::Append(req, reply) => drop(reply.send(actor.append(req))),
                Msg::Run(inv, reply) => drop(reply.send(actor.run(inv))),
                Msg::Due(reply) => drop(reply.send(actor.due())),
                Msg::Guest(guest, reply) => drop(reply.send(actor.guest(guest))),
                Msg::Group(reply) => drop(reply.send(GroupView {
                    name: actor.journal.genesis().name.clone(),
                    people: actor.journal.guests().present().count(),
                    entered: !actor.journal.guests().is_empty(),
                })),
            }
        }
        info!(dir = %actor.dir.path().display(), "journal:stop");
    });
    (JournalHandle { tx }, thread)
}

impl JournalActor {
    fn append(&mut self, req: BridgeAppend) -> Result<Appended, AppendError> {
        let interaction = req.on_behalf_of.interaction.clone();
        // A retried interaction — the adapter resending after a restart, on
        // either side — is answered with the act it already made. `taken` is
        // rebuilt from the directory on open, so this survives a restart.
        if let Some(id) = self.journal.taken(&interaction) {
            debug!(?id, interaction = interaction.as_str(), "journal:append_taken");
            return Ok(Appended::Taken(*id));
        }
        // After the retry check, so a tap taken before a ban still answers
        // with its act; nothing new is written for a removed speaker.
        let speaker = &req.on_behalf_of.speaker;
        if !self.journal.guests().may_speak(speaker) {
            warn!(speaker = speaker.as_str(), "journal:append_speaker_removed");
            return Err(AppendError::Removed(speaker.clone()));
        }
        let body = Body::Record(Record {
            app: req.app,
            op: req.op,
            on_behalf_of: Some(req.on_behalf_of),
        });
        self.write(body)
    }

    fn run(&mut self, inv: Invocation) -> Result<Reply, AppendError> {
        let interaction = inv.on_behalf_of.interaction.clone();
        // A retried tap is answered with the view of the act it already
        // made, as `append` answers one, and writes nothing.
        if let Some(id) = self.journal.taken(&interaction).copied() {
            let app = self.journal.get(&id).and_then(|a| match a.body() {
                Body::Record(r) => Some(r.app.clone()),
                _ => None,
            });
            debug!(?id, interaction = interaction.as_str(), "journal:run_taken");
            let app = app.ok_or_else(|| AppendError::Refused(format!("{id} answers no app's input")))?;
            return Ok(self.runner.say(&self.journal, &app, Some(&id))?);
        }
        let speaker = inv.on_behalf_of.speaker.clone();
        if !self.journal.guests().may_speak(&speaker) {
            warn!(speaker = speaker.as_str(), "journal:run_speaker_removed");
            return Err(AppendError::Removed(speaker));
        }
        match self.runner.propose(&self.journal, &inv.input, &speaker)? {
            (_, Proposal::Answer { text, private }) => {
                debug!(speaker = speaker.as_str(), private, "journal:run_answered");
                Ok(Reply {
                    text,
                    buttons: Vec::new(),
                    private,
                })
            }
            (app, Proposal::Write { op }) => {
                let body = Body::Record(Record {
                    app: app.clone(),
                    op,
                    on_behalf_of: Some(inv.on_behalf_of),
                });
                let (Appended::New(id) | Appended::Taken(id)) = self.write(body)?;
                info!(speaker = speaker.as_str(), app = app.as_str(), ?id, "journal:run_wrote");
                Ok(self.runner.say(&self.journal, &app, Some(&id))?)
            }
        }
    }

    /// Each due item's fulfilment is written before it is handed out, so a
    /// reminder is posted at most once; one the chat then fails to post is
    /// the bridge's to report (`bridge:due_post_failed`).
    fn due(&mut self) -> Result<Vec<Reminder>, AppendError> {
        let items = self.runner.due(&self.journal, (self.clock)())?;
        let mut out = Vec::with_capacity(items.len());
        for (app, item) in items {
            let body = Body::Record(Record {
                app,
                op: item.fulfil,
                on_behalf_of: None,
            });
            let (Appended::New(id) | Appended::Taken(id)) = self.write(body)?;
            info!(
                person = item.person.as_str(),
                due = item.id,
                ?id,
                "journal:due_fulfilled"
            );
            out.push(Reminder {
                person: item.person,
                text: item.text,
            });
        }
        Ok(out)
    }

    fn guest(&mut self, guest: Guest) -> Result<GuestWritten, AppendError> {
        let change = self.journal.guests().change(&guest);
        let speaker = guest.speaker.as_str().to_string();
        match change {
            GuestChange::RemovalSticks => {
                warn!(speaker, event = ?guest.event, "journal:guest_removal_sticks");
                Err(AppendError::Removed(guest.speaker))
            }
            GuestChange::Unchanged => {
                debug!(speaker, event = ?guest.event, "journal:guest_unchanged");
                Ok(GuestWritten { change, id: None })
            }
            _ => {
                let (Appended::New(id) | Appended::Taken(id)) = self.write(Body::Guest(guest))?;
                info!(speaker, ?change, ?id, "journal:guest_written");
                Ok(GuestWritten { change, id: Some(id) })
            }
        }
    }

    /// Sign `body` with the Bridge key, hold it, and append its line.
    fn write(&mut self, body: Body) -> Result<Appended, AppendError> {
        let now = (self.clock)();
        let actor = Actor::from_key(&self.bridge.verifying_key());
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
                debug!(?id, "journal:write_new");
                Ok(Appended::New(id))
            }
            Ingest::AlreadyHeld(id) => {
                debug!(?id, "journal:write_already_held");
                Ok(Appended::Taken(id))
            }
            Ingest::OfferAgain(why) => {
                error!(%why, "journal:own_act_not_holdable");
                Err(AppendError::Refused(why.to_string()))
            }
        }
    }
}
