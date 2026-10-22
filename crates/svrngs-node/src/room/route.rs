// SPDX-License-Identifier: AGPL-3.0-or-later
//! The room's one route, [`super::ROUTE`], loopback alone behind the
//! record's and the home page's checks: the home page's button and
//! `svrngs room` ask it. It translates: a name to the lends of it offered
//! now or on already ([`named`]), the network read now to a lend turned on
//! there, and the owner's answer to JSON or a refusal in its words. The
//! room decides nothing here; its owner and [`super::serves`] do.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use svrngs_core::pass::Label;

use super::{On, RoomAsk, Turn, ROUTE};
use crate::door::LendRefusal;
use crate::lend::LendName;
use crate::node::Asking;

/// The routes, for [`crate::api::router`] and the home page to serve.
pub(crate) fn router(asking: Asking) -> Router {
    Router::new()
        .route(ROUTE, get(listed).post(turn))
        .with_state(asking)
}

/// `GET /v1/room`: each lend on for the room, where it is served or why not.
async fn listed(State(at): State<Asking>) -> Response {
    turned(&at, Turn::Read).await
}

/// `POST /v1/room`: `ask` turned on or off, and the room as it stands then
/// ([`listed`]). A lend is turned on at the network read now, so one that
/// cannot be read turns nothing on; a name lent to this computer by nobody
/// now, and not on already, turns nothing on either.
async fn turn(State(at): State<Asking>, Json(ask): Json<RoomAsk>) -> Response {
    let lends = match ask {
        RoomAsk::Off { key, name } => return turned(&at, Turn::Off(key, name)).await,
        RoomAsk::On { key, name } => (Label::of_hex(&key, name.as_str())
            .map(|_| vec![(key, name)]))
        .ok_or_else(|| "That is not something lent in your groups.".to_string()),
        RoomAsk::Names(names) => named(&at, &names).await,
    };
    let (network, _) = crate::network::now().await;
    tracing::debug!(target: "room", ?lends, ?network, "room: turned on at the network read now, or refused");
    let refused = |words: &str| LendRefusal::Closed.say(StatusCode::CONFLICT, words);
    match (lends, network) {
        (Err(words), _) => refused(&words),
        (_, Err(_)) => refused("This computer cannot tell which WiFi it is on, so nothing was turned on for your TV: try again once it is on one."),
        (Ok(lends), Ok(network)) => {
            let on = |(key, name)| On { key, name, network };
            turned(&at, Turn::On(lends.into_iter().map(on).collect())).await
        }
    }
}

/// `turn` made by the room's owner, and the room as it stands then; why
/// what is on cannot be read, which serves nothing until a turn writes it
/// over; or refused when its owner has stopped.
async fn turned(at: &Asking, turn: Turn) -> Response {
    let room = at.room.turn(turn).await;
    tracing::debug!(target: "room", made = room.is_some(), unread = ?room.as_ref().and_then(|r| r.as_ref().err()), "room: asked of its owner: the room, why it cannot be read, or refused as it has stopped");
    match room {
        Some(Ok(room)) => Json(room).into_response(),
        Some(Err(why)) => LendRefusal::Fault.say(StatusCode::INTERNAL_SERVER_ERROR, format!("This computer cannot read what you turned on for your TV ({why}), so it shows nothing on your TV now. Turn a lend on for your TV again, and it starts over from that one.")),
        None => LendRefusal::Fault.say(StatusCode::INTERNAL_SERVER_ERROR, "the room cannot be changed on this computer now"),
    }
}

/// The lends `names` names, by the lender's key and the lend's name: each
/// lent to this computer now, its own and the other computers' of its
/// groups, each asked now, or on already, so one whose lender is away
/// stays on; or the words of a name none is.
async fn named(at: &Asking, names: &[LendName]) -> Result<Vec<(String, LendName)>, String> {
    let groups = at.host.standings().unwrap_or_default();
    let own = crate::door::offered(&at.door).await.unwrap_or_default();
    let own = own.into_iter().map(|o| (at.host.actor.clone(), o.name));
    let others = at.host.lends_in(&groups, None).await.lent;
    let others = others.into_iter().map(|l| (l.key, l.offered.name));
    let on = at.room.turn(Turn::Read).await.and_then(Result::ok);
    let on = on.unwrap_or_default();
    let on = on.into_iter().map(|r| (r.key, r.name));
    let found: Vec<_> = own
        .chain(others)
        .chain(on)
        .filter(|(_, n)| names.contains(n))
        .collect();
    let none = names.iter().find(|n| !found.iter().any(|(_, f)| f == *n));
    tracing::debug!(target: "room", ?names, found = found.len(), ?none, "room: the names turned on, each lent now or on already");
    match none {
        Some(none) => Err(format!(
            "nobody in your groups lends {none} to this computer now"
        )),
        None => Ok(found),
    }
}
