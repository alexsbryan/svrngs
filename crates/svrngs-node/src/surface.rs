// SPDX-License-Identifier: AGPL-3.0-or-later
//! What an app on this computer asks of it beside the record (DESIGN §3;
//! u5-1): the groups it holds, and what each group's people lend. New: upstream's
//! apps read the mesh's doors, which did not come over (PLAN §4, U5's
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
//!   `null` likewise; and beside them this computer's key.
//! - `GET /v1/lends?namespace=<id>` — what the group's people lend now,
//!   this computer's own lends among them, each with its lender's person
//!   and key, its name, kind, limit, how many use it and its declared
//!   upload (`upload_bps`, `null` when its lender states none), and where
//!   this computer's browser opens it, its `origin` (u5-2,
//!   [`crate::origin`]); and the people whose computer did not say
//!   what they lend, named in [`crate::node::Unlisted`]'s four lists by how:
//!   `unanswered` within [`crate::node::ANSWER_WITHIN`], `unheard`,
//!   `refused` and `older` — their lends absent rather than stale. The body [`crate::node`]'s `GET /people/lends`
//!   answers a member, for one group.
//! - `GET /svrngs-reads`, at a lend's own origin and to its pages (mnp-2,
//!   [`reads`]): the lends of each kind its lender declared it reads, as a
//!   phone's lend worker answers it from what the group's page handed
//!   (`web/lend.js`, `told`).
//! - `POST /v1/sign?app=<app>` — the body's bytes signed for the app by
//!   this computer's own key, the one that signs its acts, answered as that
//!   key and the signature, each in hex (onair-sign, [`sign`]). The core
//!   says what is signed and how one is checked ([`svrngs_core::app`]).

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use svrngs_core::app::App;
use svrngs_core::pass::Label;

use crate::door::choices::{self, Said};
use crate::door::LendRefusal;
use crate::group::Namesake;
use crate::node::{Asking, Lending, Unlisted};
use crate::rail::{namespace_of, standing_of, RailQuery};
use crate::ring_routes::err;
use crate::seed::Seed;
use crate::{register::RemovalSaid, Person};

/// Where an app asks which groups this computer holds.
pub const GROUPS_ROUTE: &str = "/v1/groups";

/// Where an app asks what a group's people lend.
pub const LENDS_ROUTE: &str = "/v1/lends";

/// Where an app has this computer's key sign bytes for it.
pub const SIGN_ROUTE: &str = "/v1/sign";

/// Where a lent page asks, at its own origin, the lends it reads: the one
/// path svrngs answers there in the lend's place (DESIGN §3, "Lent apps",
/// call 6; `web/lend.js` spells it for a phone).
pub const READS_PATH: &str = "/svrngs-reads";

/// What ends `<READS_PATH>/<label>`, where a page posts its own stop at its
/// origin: `reads` tells it, `crate::origin`'s `lent` takes it
/// (`web/lend.js` spells it for a phone).
pub const STOP_SUFFIX: &str = "/svrngs-stop";

/// The routes, for [`crate::api::router`] to serve behind its check, on
/// the node's one state ([`Asking`]).
pub(crate) fn router(asking: Asking) -> Router {
    Router::new()
        .route(GROUPS_ROUTE, get(groups))
        .route(LENDS_ROUTE, get(lends))
        .route(SIGN_ROUTE, post(sign))
        .with_state(asking)
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
    /// Whether this computer's person left it (groups-leave), or keeps
    /// the order of removals in it, so hands that on before leaving; `null`
    /// beside `removed`'s.
    left: Option<bool>,
    keeps: Option<bool>,
    /// Who made it and since when this computer has held it, as two
    /// groups of one name are told apart (#138; [`Namesake`]); `null` with
    /// `name`.
    made: Option<Namesake>,
    /// Where this computer keeps the group's folders (v1-onboarding (d)):
    /// the home page says so. `null` when the list cannot be read here,
    /// said at warn, never read as none.
    folders: Option<Vec<std::path::PathBuf>>,
    /// Who is visiting now (guests-pages; [`crate::rail::visiting`]): `null`
    /// beside `removed`'s, or when this computer's clock cannot be read.
    visiting: Option<Vec<serde_json::Value>>,
}

/// `GET /v1/groups`.
async fn groups(State(at): State<Asking>) -> Response {
    let rail = &at.host.rail;
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
    let me = at.host.actor.as_str();
    for id in namespaces {
        let seed = seed_of(&at, &id);
        let name = seed.as_ref().map(|s| s.name().as_str().to_string());
        let made = seed.map(|s| Namesake::of(rail.root(), &s, me));
        // One read for both: a group not held or with no seed is the empty
        // register, said by the rail (`RingRail::register`).
        let (person, removed, removals, left, keeps, visiting) = match rail.register(&id) {
            Ok(r) => {
                let visiting = crate::rail::visiting(&r, me);
                let person_of = |k: &str| r.members.get(k).map(|m| &m.person_key);
                let keeps = r
                    .keeper
                    .as_deref()
                    .and_then(person_of)
                    .is_some_and(|p| Some(p) == person_of(me));
                let left = (r.removals.iter()).any(|x| x.key == me && x.said == RemovalSaid::Left);
                let person = r.members.get(me).map(|m| m.person.clone());
                (
                    person,
                    Some(r.removed.contains_key(me)),
                    Some(r.removals),
                    Some(left),
                    Some(keeps),
                    visiting,
                )
            }
            Err(e) => {
                tracing::warn!(target: "surface", namespace = id, error = %e, "surface: groups — its register is unreadable, so neither this computer's person nor whether it was removed is known");
                (None, None, None, None, None, None)
            }
        };
        let folders = crate::folder::dirs_of(&at.door.data_dir, &id)
            .inspect_err(|e| tracing::warn!(target: "surface", namespace = id, error = %e, "surface: groups — where its folders are kept cannot be read"))
            .ok();
        tracing::debug!(target: "surface", namespace = id, name = ?name, person = ?person, removed = ?removed, ?left, ?keeps,
            removals = ?removals.as_ref().map(Vec::len), folders = ?folders, "surface: groups — a group held");
        held.push(Held {
            id,
            name,
            made,
            person,
            removed,
            removals,
            left,
            keeps,
            folders,
            visiting,
        });
    }
    // This computer's key, so its home page tells its own devices from
    // others' among a group's people (capability-what-acts).
    Json(serde_json::json!({ "groups": held, "key": me })).into_response()
}

/// `POST /v1/sign?app=<app>`'s query: an app's name, refused unless it is
/// one ([`App::parse`]); none is the empty name, refused as that.
#[derive(Debug, Deserialize)]
struct SignQuery {
    #[serde(default)]
    app: String,
}

/// `POST /v1/sign?app=<app>`: `bytes`, as they came, signed for the app by
/// this computer's own key; `{key, sig}` in hex. Served to this computer's
/// apps alone, as the record is ([`crate::api::router`]): a page in a
/// browser is refused before it, so no lent page has the key sign.
async fn sign(
    State(at): State<Asking>,
    Query(q): Query<SignQuery>,
    bytes: axum::body::Bytes,
) -> Response {
    let app = match App::parse(&q.app) {
        Ok(app) => app,
        Err(why) => {
            tracing::debug!(target: "sign", app = q.app, %why, "sign: refused — not an app's name");
            return err(StatusCode::BAD_REQUEST, why.to_string());
        }
    };
    let (key, sig) = (&at.host.actor, app.sign(at.host.rail.signer(), &bytes));
    tracing::debug!(target: "sign", %app, bytes = bytes.len(), key, "sign: signed for the app with this computer's key");
    Json(serde_json::json!({ "key": key, "sig": sig })).into_response()
}

/// The seed this computer keeps for `id`, or `None`, at warn, when it
/// keeps none, it cannot be read, or it is another group's.
pub(crate) fn seed_of(at: &Asking, id: &str) -> Option<Seed> {
    let dir = at.host.rail.group_dir(id).map_err(|e| e.to_string());
    let text = dir.and_then(|dir| (at.seeds)(&dir));
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
    Some(seed)
}

/// `GET /v1/lends?namespace=<id>`: what the group's people lend, the shelf,
/// and each carried app whose files changed since the version that runs as
/// this computer's person (canon-pin (c)). A group not held, or held with no seed,
/// is refused as the rail's doors refuse it ([`standing_of`]); one this
/// computer does not stand in, since no lender's computer would answer it.
async fn lends(State(at): State<Asking>, Query(q): Query<RailQuery>) -> Result<Response, Response> {
    let namespace = namespace_of(&q)?;
    let standing = standing_of(&at.host.rail, &namespace)?;
    let Some(own) = standing.get(&at.host.actor).map(|m| m.person.clone()) else {
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
    let (lent, unlisted) = at.lent(&groups, &own, None).await?;
    let host = &at.host;
    let lent: Vec<serde_json::Value> = lent
        .into_iter()
        .filter_map(|l| {
            let origin = host.origin(&l.key, l.offered.name.as_str())?;
            let mut l = serde_json::to_value(l).ok()?;
            l["origin"] = origin.into();
            Some(l)
        })
        .collect();
    tracing::debug!(target: "surface", namespace, lent = lent.len(), ?unlisted,
        "surface: lends — answered what the group's people lend, this computer's own among them, each at its origin");
    let mut body = serde_json::json!(unlisted);
    body["namespace"] = namespace.into();
    body["lent"] = lent.into();
    body["shelf"] = crate::folder::shelf();
    body["changed"] = at.folders.changed(&groups, &at.host.actor);
    Ok(Json(body).into_response())
}

/// What the page of the lend `app` reads now: each lend of a kind its
/// lender declared it reads ([`crate::lend::Offered::reads_one`]), in the
/// groups its lender and this computer stand in, and whose computer did not
/// say what it lends. The one decider of what `/svrngs-reads` tells a page
/// ([`reads`]) and what it reaches at another lend's origin ([`reaches`]).
/// A lender whose computer did not say — [`Unlisted`]'s four lists — is
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
    groups.retain(|(_, s)| s.contains_key(&host.actor));
    let Some(own) = own else {
        tracing::debug!(target: "surface", %app, "surface: reads refused — its lender and this computer share no group");
        return Err(LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing is lent here"));
    };
    let (lent, unlisted) = at.lent(&groups, &own, None).await?;
    let silent: Vec<String> = (groups.iter().flat_map(|(_, s)| s.iter()))
        .filter(|(_, m)| unlisted.names(&m.person))
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
/// ([`crate::origin`]): `label` among what it reads ([`read_by`]), and,
/// for another computer's app, its reach the person's yes, which opening
/// it from the home page says (capability-open; [`choices::reaching`]):
/// this computer's own app reads what its person declared. A lend whose
/// lender did not say is could-not-judge, refused `away`.
pub(crate) async fn reaches(at: &Asking, app: &Label, label: &Label) -> Result<(), Response> {
    let (read, unlisted, silent) = read_by(at, app).await?;
    if read.iter().any(|l| named(l).as_ref() == Some(label)) {
        let own = app.lender() == at.door.key;
        let said = if own {
            None
        } else {
            choices::reaching(&at.door, app)?
        };
        tracing::debug!(target: "surface", %app, %label, own, ?said, "surface: the page reads the lend's kind; reached when the app is this computer's own or its person said yes to it");
        if !own && said != Some(Said::Yes) {
            return Err(LendRefusal::Unasked.say(
                StatusCode::FORBIDDEN,
                "Say yes on your home page: open this app from there.",
            ));
        }
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
