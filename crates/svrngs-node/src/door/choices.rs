// SPDX-License-Identifier: AGPL-3.0-or-later
//! A person's yes and stop to an app, kept by the lend's door and decided
//! there (sup-2; svrn-docs `rings/current/SCOPED_USE_MADE_PRODUCTION.md` §4):
//! the one decision at the resource, on the lender's disk and never in the
//! record, since a private grant is not broadcast.
//!
//! Per person ([`PersonKey`]), per app, one [`Choice`], beside the lend's
//! declaration ([`crate::lend::choices_path`]), written through one task
//! ([`Choices`]) and read on each request for a lend that asks. A stop is
//! kept as a stop, so an app says "you stopped this" rather than asking
//! again; a yes replaces it. `svrngs unshare` removes them with the lend.
//!
//! Only the person says yes: `POST /choices/<lend>` on the members'
//! protocol as the dialler's person, `{app, said}`. With a pass — a phone's
//! on the pass holders' protocol, or a computer's own for its page — only
//! a stop, for the pass's app: that body's type has no yes ([`Stopping`]).
//! `GET /choices/<lend>` answers the dialler's own. A stop refuses every
//! request the door receives after it; one it is already carrying, a live
//! socket included, runs to its end.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use axum::body::Bytes;
use axum::extract::{Path as Segment, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use super::{find, holder_of, own_app, unix_now, Asker, Door, LendRefusal, PersonKey, PASS_HEADER};
use crate::iroh::Dialer;
use crate::lend::{choices_path, Lend};
use svrngs_core::pass::Label;

/// What a person says of an app: a closed set (AGENTS 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Said {
    Yes,
    Stop,
}

/// What an app holding a pass may say of itself: only stop, since giving
/// up access never widens it. No yes, so a pass's yes does not parse.
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Stopping {
    Stop,
}

/// A choice kept: what the person said, and when, in unix seconds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub said: Said,
    pub at: i64,
}

type Kept = BTreeMap<PersonKey, BTreeMap<Label, Choice>>;
type Done = oneshot::Sender<Result<(), String>>;

/// A person's body: the app, and what they say of it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Person {
    app: Label,
    said: Said,
}

/// An app's body, with its pass.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct App {
    #[expect(dead_code, reason = "its one variant is the whole of what it says")]
    said: Stopping,
}

/// Every choice kept at `path`; none when none was ever said. A file that
/// cannot be read is an error, never no choice.
fn load(path: &Path) -> Result<Kept, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).map_err(|e| e.to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Kept::new()),
        Err(e) => Err(e.to_string()),
    }
}

/// Every choice kept for `lend`, or svrngs's refusal.
fn read(door: &Door, lend: &Lend) -> Result<Kept, Response> {
    load(&choices_path(&door.data_dir, &lend.name)).map_err(|why| {
        tracing::warn!(target: "door", lend = %lend.name, why, "door: the lend's choices cannot be read");
        fault("who said yes to this cannot be read on the computer that lends it")
    })
}

fn fault(words: &str) -> Response {
    LendRefusal::Fault.say(StatusCode::INTERNAL_SERVER_ERROR, words.to_string())
}

/// The door's one check, by the design's §4 table: a lend that asks
/// nobody, or asked by its own pages, is carried; else the app the request
/// names must have its person's yes — `stopped` after a stop, `unasked`
/// with no app named or nothing said.
pub(crate) fn judge(
    asks: bool,
    own: &Label,
    app: Option<&Label>,
    said: Option<Said>,
) -> Result<(), LendRefusal> {
    match (app, said) {
        _ if !asks => Ok(()),
        (Some(app), _) if app == own => Ok(()),
        (Some(_), Some(Said::Yes)) => Ok(()),
        (Some(_), Some(Said::Stop)) => Err(LendRefusal::Stopped),
        _ => Err(LendRefusal::Unasked),
    }
}

/// [`judge`] for one request for `lend` by `person` from `app`: `None` to
/// carry it, else svrngs's refusal.
pub(super) fn check(
    door: &Door,
    lend: &Lend,
    person: &PersonKey,
    app: Option<&Label>,
) -> Option<Response> {
    let own = Label::of(&door.key, lend.name.as_str());
    let choice = match (&lend.asks, app) {
        (Some(_), Some(app)) => match read(door, lend) {
            Ok(mut kept) => kept.remove(person).and_then(|mut k| k.remove(app)),
            Err(refused) => return Some(refused),
        },
        _ => None,
    };
    let judged = judge(
        lend.asks.is_some(),
        &own,
        app,
        choice.as_ref().map(|c| c.said),
    );
    tracing::debug!(target: "door", lend = %lend.name, %person, app = ?app.map(Label::as_str), asks = lend.asks.is_some(),
        ?choice, ?judged, "door: the person's choice for the app, judged");
    let refused = judged.err()?;
    let words = match refused {
        LendRefusal::Stopped => format!("you stopped this app using {}", lend.name),
        _ => format!("only an app you said yes to uses {}", lend.name),
    };
    Some(refused.say(StatusCode::FORBIDDEN, words))
}

/// The task every choice is written through, so two never interleave
/// (RUST_ARCH 3, 4), ending when the last handle drops.
#[derive(Clone)]
pub struct Choices(mpsc::UnboundedSender<(PathBuf, Kept, Done)>);

impl Choices {
    pub fn spawn() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<(PathBuf, Kept, Done)>();
        tokio::spawn(async move {
            while let Some((path, kept, done)) = rx.recv().await {
                let _ = done.send(write(&path, kept));
            }
        });
        Self(tx)
    }
}

/// `said` merged into what is kept at `path`, through a file renamed into
/// place.
fn write(path: &Path, said: Kept) -> Result<(), String> {
    let mut kept = load(path)?;
    for (person, choices) in said {
        kept.entry(person).or_default().extend(choices);
    }
    let partial = path.with_extension("json.partial");
    let text = serde_json::to_string_pretty(&kept).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(path.parent().ok_or("no directory")?)
        .and_then(|()| std::fs::write(&partial, text))
        .and_then(|()| std::fs::rename(&partial, path))
        .map_err(|e| e.to_string())
}

/// The lend `raw` names here, or svrngs's refusal.
pub(super) fn found(door: &Door, raw: &str) -> Result<Lend, Response> {
    match find(door, raw) {
        Ok(Some(lend)) => Ok(lend),
        Ok(None) => {
            tracing::debug!(target: "door", lend = raw, "door: refused — no lend declared by that name");
            Err(LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing is lent here by that name"))
        }
        Err(e) => {
            tracing::warn!(target: "door", lend = raw, error = %e, "door: refused — the lend's declaration cannot be read");
            Err(fault(
                "this lend cannot be read on the computer that lends it",
            ))
        }
    }
}

/// Keep what `person` said of `app` on `lend`.
async fn keep(door: &Door, lend: &Lend, person: PersonKey, app: Label, said: Said) -> Response {
    let choice = Choice {
        said,
        at: unix_now(),
    };
    let one = [(person.clone(), [(app.clone(), choice.clone())].into())].into();
    let (done, kept) = oneshot::channel();
    let _ = door
        .choices
        .0
        .send((choices_path(&door.data_dir, &lend.name), one, done));
    match kept.await {
        Ok(Ok(())) => {
            tracing::info!(target: "door", lend = %lend.name, %person, %app, ?choice, "door: a choice kept");
            Json(serde_json::json!({ "app": app, "choice": choice })).into_response()
        }
        failed => {
            tracing::warn!(target: "door", lend = %lend.name, ?failed, "door: a choice could not be kept");
            fault("this could not be kept on the computer that lends it")
        }
    }
}

/// An app's stop, the only thing its body may say.
async fn stop(door: &Door, lend: &Lend, person: PersonKey, app: Label, body: &[u8]) -> Response {
    if let Err(e) = serde_json::from_slice::<App>(body) {
        tracing::debug!(target: "door", lend = %lend.name, %app, error = %e, "door: refused — an app's pass says only stop");
        return LendRefusal::Closed.say(
            StatusCode::FORBIDDEN,
            "an app may only stop; its person says yes",
        );
    }
    keep(door, lend, person, app, Said::Stop).await
}

/// `person`'s own choices on `lend`.
fn kept(door: &Door, lend: &Lend, person: &PersonKey) -> Response {
    match read(door, lend) {
        Ok(mut kept) => {
            Json(serde_json::json!({ "choices": kept.remove(person).unwrap_or_default() }))
                .into_response()
        }
        Err(refused) => refused,
    }
}

/// A dialler standing in a group held here, as the person it is, and the
/// lend `raw` names.
fn member(
    door: &Door,
    dialer: Option<Extension<Dialer>>,
    raw: &str,
) -> Result<(Dialer, PersonKey, Lend), Response> {
    let closed =
        || LendRefusal::Closed.say(StatusCode::FORBIDDEN, "this computer lends nothing to you");
    let Extension(dialer) = dialer.ok_or_else(closed)?;
    let key = dialer.actor();
    if door.rail.standing_anywhere(&key).is_none() {
        tracing::debug!(target: "door", actor = key, "door: choices refused — the dialer stands in no group held here");
        return Err(closed());
    }
    Ok((dialer, PersonKey::resolved(key), found(door, raw)?))
}

/// `POST /choices/<lend>` on the members' protocol: the person's yes or
/// stop; with a computer's own pass, its page's stop alone.
pub(super) async fn choose(
    State(door): State<Door>,
    dialer: Option<Extension<Dialer>>,
    Segment(raw): Segment<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let (dialer, person, lend) = match member(&door, dialer, &raw) {
        Ok(m) => m,
        Err(refused) => return refused,
    };
    if headers.contains_key(PASS_HEADER) {
        return match own_app(&door, &dialer, &headers, &lend) {
            Ok(app) => stop(&door, &lend, person, app.expect("a pass is present"), &body).await,
            Err(refused) => refused,
        };
    }
    match serde_json::from_slice::<Person>(&body) {
        Ok(Person { app, said }) => keep(&door, &lend, person, app, said).await,
        Err(e) => {
            tracing::debug!(target: "door", lend = %lend.name, error = %e, "door: refused — not a choice");
            LendRefusal::Path.say(
                StatusCode::BAD_REQUEST,
                "a choice is {app, said: yes or stop}",
            )
        }
    }
}

/// `POST /choices/<lend>` on the pass holders' protocol: the pass's app's
/// stop, as the person who signed it.
pub(super) async fn stop_with_pass(
    State(door): State<Door>,
    dialer: Option<Extension<Dialer>>,
    Segment(raw): Segment<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let (Some(Extension(dialer)), Ok(lend)) = (dialer, found(&door, &raw)) else {
        return LendRefusal::Closed
            .say(StatusCode::FORBIDDEN, "nothing is lent to you by that name");
    };
    match holder_of(&door, &dialer, &headers, &lend).await {
        Ok(Asker { actor, app, .. }) => {
            stop(
                &door,
                &lend,
                actor,
                app.expect("a pass names its app"),
                &body,
            )
            .await
        }
        Err(refused) => refused,
    }
}

/// `GET /choices/<lend>`: the dialler's own.
pub(super) async fn choices_of(
    State(door): State<Door>,
    dialer: Option<Extension<Dialer>>,
    Segment(raw): Segment<String>,
) -> Response {
    match member(&door, dialer, &raw) {
        Ok((_, person, lend)) => kept(&door, &lend, &person),
        Err(refused) => refused,
    }
}

#[cfg(test)]
mod tests;
