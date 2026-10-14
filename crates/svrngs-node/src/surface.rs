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
//!   `namespace` every other route names), its name, this computer's
//!   person in it, and whether its keeper removed this computer (U7),
//!   `null` when its register cannot be read here; and every removal and
//!   ask to remove in it, who of whom and when (v1-keeper-office (b)),
//!   `null` likewise.
//! - `GET /v1/lends?namespace=<id>` — what the group's people lend now,
//!   this computer's own lends among them, each with its lender's person
//!   and key, its name, kind, limit, how many use it and its declared
//!   upload (`upload_bps`, `null` when its lender states none), and where
//!   this computer's browser opens it, its `origin` (u5-2,
//!   [`crate::origin`]), and its `room` address while the room door serves
//!   it (u6-1, [`crate::room`]); and the people whose computer did not say
//!   what they lend, named in [`crate::node::Unlisted`]'s three lists by how:
//!   `unanswered` within [`crate::node::ANSWER_WITHIN`], `unheard` and
//!   `refused` — their lends absent rather than stale. The body [`crate::node`]'s `GET /people/lends`
//!   answers a member, for one group.
//! - `GET /svrngs-reads`, at a lend's own origin and to its pages (mnp-2,
//!   [`reads`]): the lends of each kind its lender declared it reads, as a
//!   phone's lend worker answers it from what the group's page handed
//!   (`web/lend.js`, `told`).

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use svrngs_core::pass::Label;

use crate::door::LendRefusal;
use crate::node::{Asking, Lending, Unlisted};
use crate::rail::{namespace_of, standing_of, RailQuery};
use crate::redeem::SeedText;
use crate::ring_routes::err;
use crate::seed::Seed;
use crate::Person;

/// Where an app asks which groups this computer holds.
pub const GROUPS_ROUTE: &str = "/v1/groups";

/// Where an app asks what a group's people lend.
pub const LENDS_ROUTE: &str = "/v1/lends";

/// Where a lent page asks, at its own origin, the lends it reads: the one
/// path svrngs answers there in the lend's place (DESIGN §3, "Lent apps",
/// call 6; `web/lend.js` spells it for a phone).
pub const READS_PATH: &str = "/svrngs-reads";

/// What ends `<READS_PATH>/<label>`, where a page posts its own stop at its
/// origin: `reads` tells it, `crate::origin`'s `lent` takes it
/// (`web/lend.js` spells it for a phone).
pub const STOP_SUFFIX: &str = "/svrngs-stop";

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
    /// Whether the group's keeper removed this computer (U7): the home
    /// page says so in a person's words. `null` when its register cannot
    /// be read here, said at warn: could-not-judge, never "not removed"
    /// (ralph/REVIEW_FINDINGS.md #123).
    removed: Option<bool>,
    /// Every removal and ask to remove, from the register, by time: the
    /// home page shows them to everyone in the group. `null` beside
    /// `removed`'s.
    removals: Option<Vec<svrngs_runtime::register::Removing>>,
    /// Where this computer keeps the group's folders (v1-onboarding (d)):
    /// the home page says so. `null` when the list cannot be read here,
    /// said at warn, never read as none.
    folders: Option<Vec<std::path::PathBuf>>,
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
        // One read for both: a group not held or with no seed is the empty
        // register, said by the rail (`RingRail::register`).
        let me = at.asking.host.actor();
        let (person, removed, removals) = match rail.register(&id) {
            Ok(r) => (
                r.members.get(me).map(|m| m.person.clone()),
                Some(r.removed.contains_key(me)),
                Some(r.removals),
            ),
            Err(e) => {
                tracing::warn!(target: "surface", namespace = id, error = %e, "surface: groups — its register is unreadable, so neither this computer's person nor whether it was removed is known");
                (None, None, None)
            }
        };
        let folders = crate::folder::dirs_of(&at.asking.door().data_dir, &id)
            .inspect_err(|e| tracing::warn!(target: "surface", namespace = id, error = %e, "surface: groups — where its folders are kept cannot be read"))
            .ok();
        tracing::debug!(target: "surface", namespace = id, name = ?name, person = ?person, removed = ?removed,
            removals = ?removals.as_ref().map(Vec::len), folders = ?folders, "surface: groups — a group held");
        held.push(Held {
            id,
            name,
            person,
            removed,
            removals,
            folders,
        });
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
async fn lends(
    State(at): State<Surface>,
    Query(q): Query<RailQuery>,
) -> Result<Response, Response> {
    let namespace = namespace_of(&q)?;
    let standing = standing_of(at.asking.host.rail(), &namespace)?;
    let Some(own) = standing
        .get(at.asking.host.actor())
        .map(|m| m.person.clone())
    else {
        tracing::debug!(target: "surface", namespace, "surface: lends refused — this computer does not stand in the group");
        return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": format!("this computer is not in {namespace}, so nobody in it lends to it"),
                "kind": "not_in_roster",
                "namespace": namespace,
            })),
        )
            .into_response());
    };
    let groups = [(namespace.clone(), standing)];
    let (lent, unlisted) = at.asking.lent(&groups, &own, None).await?;
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
    tracing::debug!(target: "surface", namespace, lent = lent.len(), ?unlisted,
        "surface: lends — answered what the group's people lend, this computer's own among them, each at its origin");
    let mut body = serde_json::json!(unlisted);
    body["namespace"] = namespace.into();
    body["lent"] = lent.into();
    Ok(Json(body).into_response())
}

/// What the page of the lend `app` reads now: each lend of a kind its
/// lender declared it reads ([`crate::lend::Offered::reads_one`]), in the
/// groups its lender and this computer stand in, and whose computer did not
/// say what it lends. The one decider of what `/svrngs-reads` tells a page
/// ([`reads`]) and what it reaches at another lend's origin ([`reaches`]).
/// A lender whose computer did not say — [`Unlisted`]'s three lists — is
/// could-not-judge, never read as lending nothing: refused `away` where the
/// answer hangs on it, named where it is one of many (onair-economize (a)).
/// Known by person, so a lend not found whose person has a computer that
/// did not say is `away`, never "not lent".
async fn read_by(
    at: &Asking,
    app: &Label,
) -> Result<(Vec<Lending>, Unlisted, Vec<String>), Response> {
    let reader = hex::encode(app.lender());
    let host = &at.host;
    let (mut groups, own) = host.shared_with(&reader)?;
    groups.retain(|(_, s)| s.contains_key(host.actor()));
    let Some(own) = own else {
        tracing::debug!(target: "surface", %app, "surface: reads refused — its lender and this computer share no group");
        return Err(LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing is lent here"));
    };
    let (lent, unlisted) = at.lent(&groups, &own, None).await?;
    let Unlisted {
        unanswered,
        unheard,
        refused,
    } = &unlisted;
    let silent: Vec<String> = (groups.iter().flat_map(|(_, s)| s.iter()))
        .filter(|(_, m)| {
            [unanswered, unheard, refused]
                .iter()
                .any(|l| l.contains(&m.person))
        })
        .map(|(actor, _)| actor.clone())
        .collect();
    let Some(reading) = lent.iter().find(|l| named(l).as_ref() == Some(app)) else {
        if silent.contains(&reader) {
            tracing::debug!(target: "surface", %app, ?unlisted, "surface: reads refused — the page's lender did not say what it lends, so what it reads cannot be judged");
            return Err(crate::origin::away());
        }
        tracing::debug!(target: "surface", %app, "surface: reads refused — the lend is not lent now");
        return Err(LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing is lent here"));
    };
    let read = (lent.iter())
        .filter(|l| named(l).as_ref() != Some(app) && reading.offered.reads_one(&l.offered))
        .cloned()
        .collect();
    Ok((read, unlisted, silent))
}

/// A lend's label, from its lender's key and its name.
fn named(l: &Lending) -> Option<Label> {
    Label::of_hex(&l.key, l.offered.name.as_str())
}

/// Whether the page of the lend `app` may reach the lend `label`
/// ([`crate::origin`]): `label` among what it reads ([`read_by`]); a lend
/// whose lender did not say is could-not-judge, refused `away`.
pub(crate) async fn reaches(at: &Asking, app: &Label, label: &Label) -> Result<(), Response> {
    let (read, unlisted, silent) = read_by(at, app).await?;
    if read.iter().any(|l| named(l).as_ref() == Some(label)) {
        tracing::debug!(target: "surface", %app, %label, "surface: the page reads the lend's kind, so it reaches it");
        return Ok(());
    }
    if silent.contains(&hex::encode(label.lender())) {
        tracing::debug!(target: "surface", %app, %label, ?unlisted, "surface: refused — the lend's lender did not say what it lends, so whether the page reads it cannot be judged");
        return Err(crate::origin::away());
    }
    tracing::debug!(target: "surface", %app, %label, "surface: refused — the page does not read the lend, or it is not lent now");
    Err(LendRefusal::Closed.say(
        StatusCode::FORBIDDEN,
        "this page does not read that; only what its lender declared it reads",
    ))
}

/// `GET /svrngs-reads` at the origin of the lend `label` (mnp-2): what it
/// reads ([`read_by`]), each lend's kind, name, lender's person and the
/// origin this computer's browser asks it at; for a lend that asks, its
/// lender's words, `asks`, `ask`, where the page sends its person to say
/// yes, on the home page ([`crate::home`]; the page adds `&back=<its
/// path>`), and `stop`, where the page posts its own stop at its origin
/// ([`crate::origin`]) — and whose computer did not answer. No key: a page
/// needs none to ask, and the address is all it builds on.
pub(crate) async fn reads(at: &Asking, label: &Label) -> Result<Response, Response> {
    let host = &at.host;
    let (lent, Unlisted { unanswered, .. }, _) = read_by(at, label).await?;
    let read: Vec<serde_json::Value> = (lent.iter())
        .filter_map(|l| {
            let address = host.origin(&l.key, l.offered.name.as_str())?;
            let mut read = serde_json::json!({ "kind": l.offered.kind, "name": l.offered.name, "lender": l.person, "address": address });
            if let Some(words) = &l.offered.asks {
                let lend = named(l)?;
                read["asks"] = words.as_str().into();
                read["ask"] = crate::home::ask_address(host.api_port, &lend, label).into();
                let page = crate::origin::origin(label, host.api_port);
                read["stop"] = format!("{page}{}/{lend}{STOP_SUFFIX}", &READS_PATH[1..]).into();
            }
            Some(read)
        })
        .collect();
    tracing::debug!(target: "surface", %label, read = read.len(), unanswered = unanswered.len(),
        "surface: reads — answered a lent page the lends of the kinds it reads, each at its origin here");
    Ok(Json(serde_json::json!({ "lends": read, "unanswered": unanswered })).into_response())
}
