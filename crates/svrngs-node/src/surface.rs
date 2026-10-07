// SPDX-License-Identifier: AGPL-3.0-or-later
//! What an app on this computer asks of it beside the record (DESIGN §3;
//! u5-1): the groups it holds, and what each group's people lend. New: upstream's
//! apps read the mesh's doors, which did not come over (DESIGN §8, U5's
//! inventory). Served on the record's loopback listener, behind its one
//! check ([`crate::api::router`]), and asked of the runtime and of the
//! lenders' computers now; nothing is kept between asks, and no event is
//! sent.
//!
//! - `GET /v1/groups` — each group this computer holds: its id (the
//!   `namespace` every other route names), its name, and this computer's
//!   person in it.
//! - `GET /v1/lends?namespace=<id>` — what the group's people lend now,
//!   this computer's own lends among them, each with its lender's person
//!   and key, its name, kind, limit, how many use it and its declared
//!   upload (`upload_bps`, `null` when its lender states none), and where
//!   this computer's browser opens it, its `origin` (u5-2,
//!   [`crate::origin`]), and its `room` address while the room door serves
//!   it (u6-1, [`crate::room`]); and the people whose computer did not answer,
//!   named, their lends absent rather than stale. The body [`crate::node`]'s `GET /people/lends`
//!   answers a member, for one group.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use crate::node::Asking;
use crate::rail::{namespace_of, standing_of, RailQuery};
use crate::redeem::SeedText;
use crate::register::Standing;
use crate::ring_routes::err;
use crate::seed::Seed;
use crate::{Ed25519Verifier, Person};

/// Where an app asks which groups this computer holds.
pub const GROUPS_ROUTE: &str = "/v1/groups";

/// Where an app asks what a group's people lend.
pub const LENDS_ROUTE: &str = "/v1/lends";

/// What the routes read: the node's groups and transport and its own door
/// ([`Asking`]), and how it reads a group's seed, for its name.
#[derive(Clone)]
struct Surface {
    asking: Asking,
    seeds: SeedText,
}

/// The routes, for [`crate::api::router`] to serve behind its check.
pub(crate) fn router(asking: Asking, seeds: SeedText) -> Router {
    Router::new()
        .route(GROUPS_ROUTE, get(groups))
        .route(LENDS_ROUTE, get(lends))
        .with_state(Surface { asking, seeds })
}

/// A group as `GET /v1/groups` answers it.
#[derive(Debug, Serialize)]
struct Held {
    id: String,
    /// `null` when its seed cannot be read on this computer, or does not
    /// hash to its id: said so at warn, never a name guessed.
    name: Option<String>,
    /// `null` when this computer does not stand in it — not let in yet, or
    /// taken out — so it is nobody there.
    person: Option<Person>,
}

/// `GET /v1/groups`.
async fn groups(State(at): State<Surface>) -> Response {
    let rail = at.asking.host.rail();
    let namespaces = match rail.namespaces() {
        Ok(n) => n,
        Err(e) => {
            tracing::warn!(target: "surface", error = %e, "surface: groups — the groups held cannot be listed");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "the groups this computer is in cannot be read on it",
            );
        }
    };
    let mut held = Vec::with_capacity(namespaces.len());
    for id in namespaces {
        let name = name_of(&at, &id);
        let person = match rail.standing(&id, &Ed25519Verifier) {
            Ok(Standing::Members(people)) => {
                people.get(at.asking.host.actor()).map(|m| m.person.clone())
            }
            Ok(other) => {
                tracing::debug!(target: "surface", namespace = id, standing = ?other, "surface: groups — nobody can be shown to stand in it");
                None
            }
            Err(e) => {
                tracing::warn!(target: "surface", namespace = id, error = %e, "surface: groups — its register is unreadable, so this computer's person is not known");
                None
            }
        };
        tracing::debug!(target: "surface", namespace = id, name = ?name, person = ?person, "surface: groups — a group held");
        held.push(Held { id, name, person });
    }
    Json(serde_json::json!({ "groups": held })).into_response()
}

/// The name in the seed this computer keeps for `id`, or `None`, at warn,
/// when it keeps none, it cannot be read, or it is another group's.
fn name_of(at: &Surface, id: &str) -> Option<String> {
    let text = at
        .asking
        .host
        .rail()
        .group_dir(id)
        .map_err(|e| e.to_string())
        .and_then(|dir| (at.seeds)(&dir));
    let seed = match text.map(|t| t.map(|t| Seed::from_text(&t))) {
        Ok(Some(Ok(seed))) => seed,
        Ok(None) => {
            tracing::warn!(target: "surface", namespace = id, "surface: groups — no seed kept, so its name is not known");
            return None;
        }
        Ok(Some(Err(why))) | Err(why) => {
            tracing::warn!(target: "surface", namespace = id, why, "surface: groups — its seed cannot be read, so its name is not known");
            return None;
        }
    };
    if seed.id() != id {
        tracing::warn!(target: "surface", namespace = id, seed = seed.id(), "surface: groups — its seed is another group's, so its name is not known");
        return None;
    }
    Some(seed.name().as_str().to_string())
}

/// `GET /v1/lends?namespace=<id>`. A group not held, or held with no seed,
/// is refused as the rail's doors refuse it ([`standing_of`]); one this
/// computer does not stand in, since no lender's computer would answer it.
async fn lends(State(at): State<Surface>, Query(q): Query<RailQuery>) -> Response {
    let namespace = match namespace_of(&q) {
        Ok(ns) => ns,
        Err(refusal) => return refusal,
    };
    let standing = match standing_of(at.asking.host.rail(), &namespace) {
        Ok(people) => people,
        Err(refusal) => return refusal,
    };
    let Some(own) = standing
        .get(at.asking.host.actor())
        .map(|m| m.person.clone())
    else {
        tracing::debug!(target: "surface", namespace, "surface: lends refused — this computer does not stand in the group");
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": format!("this computer is not in {namespace}, so nobody in it lends to it"),
                "kind": "not_in_roster",
                "namespace": namespace,
            })),
        )
            .into_response();
    };
    let groups = [(namespace.clone(), standing)];
    let (lent, unanswered) = match at.asking.lent(&groups, &own, None).await {
        Ok(answer) => answer,
        Err(refused) => return refused,
    };
    let host = &at.asking.host;
    let lent: Vec<serde_json::Value> = lent
        .into_iter()
        .filter_map(|l| {
            let origin = host.origin(&l.key, l.offered.name.as_str())?;
            let room = host.room_address(&l.key, l.offered.name.as_str());
            let mut l = serde_json::to_value(l).ok()?;
            l["origin"] = origin.into();
            if let Some(room) = room {
                l["room"] = room.into();
            }
            Some(l)
        })
        .collect();
    tracing::debug!(target: "surface", namespace, lent = lent.len(), unanswered = unanswered.len(),
        "surface: lends — answered what the group's people lend, this computer's own among them, each at its origin");
    Json(serde_json::json!({ "namespace": namespace, "lent": lent, "unanswered": unanswered }))
        .into_response()
}
