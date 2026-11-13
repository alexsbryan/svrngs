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
//! - `<LEND_PATH>/<label>/…`, at a carried app's own origin and to its
//!   page ([`LEND_PATH`]; `crate::origin`'s `routed`): a lend it reads,
//!   reached there, since its page may ask nowhere else.
//! - `GET /svrngs-to?lender=&lend=`, at a carried app's own origin and to
//!   its page ([`to`]; docs/APP_ACCESS_AND_STOP.md A4): one lend of the
//!   app's group, sent on to its plain origin, or, with `whether`, told
//!   only whether it opens.
//!
//!   These three at an app's origin, and a carried app's own `/svrngs/acts`
//!   (`crate::folder::carried`), are parsed once, into [`AppRoute`].
//! - `POST /v1/sign?app=<app>` — the body's bytes signed for the app by
//!   this computer's own key, the one that signs its acts, answered as that
//!   key and the signature, each in hex (onair-sign, [`sign`]). The core
//!   says what is signed and how one is checked ([`svrngs_core::app`]).

use std::collections::BTreeMap;

use axum::extract::{Query, State};
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use eudoxia_community::lend;
use serde::{Deserialize, Serialize};
use svrngs_core::app::App;
use svrngs_core::pass::{Label, Whose};

use crate::door::choices;
use crate::door::LendRefusal;
use crate::folder::carried::{Handled, ACTS, SENDS};
use crate::group::Namesake;
use crate::lend::{Program, SendRow, Source};
use crate::node::{Asking, Groups, Lending, Unlisted};
use crate::rail::{namespace_of, refused_as, standing_of, RailQuery};
use crate::ring_routes::err;
use crate::seed::Seed;
use crate::{origin::origin, session::AS_PATH};
use crate::{register::RemovalSaid, Office, Person};

/// The guide to building an app, docs/guide/build.md, in the binary, so
/// `svrngs guide` prints it to an agent holding only the command (svrn-docs
/// `rings/current/EXTENSION_POINT.md` §7, G4: in one agent run `--help` was
/// all it read).
pub const GUIDE: &str = include_str!("../../../docs/guide/build.md");

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

/// Where a carried app's page opens one lend of its own group, at its own
/// origin, by its lender's key and its name (docs/APP_ACCESS_AND_STOP.md
/// A4): no registry, frame or read; svrngs knows nothing of why it opens.
pub const TO_PATH: &str = "/svrngs-to";

/// Where a carried app's page reaches a lend it reads, at its own origin,
/// `<LEND_PATH>/<label>/<path>`: its page's CSP lets it ask nowhere else
/// (COMMUNITY_APPS §6.2), so svrngs carries it ([`crate::origin`]).
pub const LEND_PATH: &str = "/svrngs-lend";

/// What a page asks of svrngs at its app's own origin, parsed once from
/// the path and query asked, a closed set (ARCH 9; svrn-docs
/// `rings/current/EXTENSION_POINT.md` §3, §8): [`crate::origin`]'s `lent`
/// and `routed`, and the door's carried app, `crate::folder::carried`,
/// each read this, never the path again.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AppRoute {
    /// [`ACTS`]: a carried app's own part of the record.
    Acts,
    /// [`READS_PATH`]: the lends it reads ([`reads`]).
    Reads,
    /// `<READS_PATH>/<label><STOP_SUFFIX>`: its own stop of that lend.
    Stop(Label),
    /// [`TO_PATH`]: one lend of its group, opened ([`to`]).
    To,
    /// `<LEND_PATH>/<label>/<path>`: a read of that lend at that path, its
    /// query kept; no label when the path names none.
    Lend(Option<Label>, String),
    /// Anything else: the app's own, a file or its lend's own path.
    Own,
}

impl AppRoute {
    /// The route `asked`, a path and its query, names.
    pub(crate) fn of(asked: &str) -> Self {
        let path = asked.split('?').next().unwrap_or_default();
        let stop = (path.strip_prefix(READS_PATH)).and_then(|p| p.strip_prefix('/'));
        let stop = stop.and_then(|p| Label::parse(p.strip_suffix(STOP_SUFFIX)?));
        match path {
            ACTS => Self::Acts,
            READS_PATH => Self::Reads,
            TO_PATH => Self::To,
            _ if path.starts_with(LEND_PATH) => {
                let named = (asked.strip_prefix(LEND_PATH)).and_then(|r| r.strip_prefix('/'));
                let (read, tail) = named.and_then(|r| r.split_once('/')).unwrap_or_default();
                Self::Lend(Label::parse(read), format!("/{tail}"))
            }
            _ => stop.map_or(Self::Own, Self::Stop),
        }
    }
}

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
    /// Which files no member gave, by their writer's person and how many
    /// (synced.rs `waiting_in`): `null` when the register cannot be read,
    /// said at warn, never read as none waiting.
    waiting: Option<Vec<(Option<Person>, usize)>>,
    /// Who is visiting now (guests-pages; [`crate::rail::visiting`]): `null`
    /// beside `removed`'s, or when this computer's clock cannot be read.
    visiting: Option<Vec<serde_json::Value>>,
    /// Why this computer meets the group where it does now: the group names
    /// relays of its own, and this computer is at the build's fallback
    /// because that answers (relay fallbacks). The home page says so in a
    /// person's words; `null` when it meets the group at its own.
    fallback: Option<svrngs_core::relay::Tier>,
    /// A hand-on of the order of removals waiting on its Accept, to this
    /// computer or from it, with the person at its other end; absent when
    /// none waits here, or beside `removed`'s `null`.
    #[serde(skip_serializing_if = "Option::is_none")]
    offered: Option<Offer>,
    /// Whether this computer's person keeps the order of removals in a
    /// group others stand in, so leaving without handing it on leaves
    /// nobody keeping it: the home page says so before Leave. Absent when
    /// not.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    vacates: bool,
    /// What the home page says of a group an older svrngs made, in
    /// `svrngs_core::made_older`'s words; absent for every other group.
    #[serde(skip_serializing_if = "Option::is_none")]
    older: Option<String>,
}

/// A waiting hand-on as the home page says it (`Office::Pending`).
#[derive(Debug, Serialize)]
struct Offer {
    to_you: bool,
    person: Option<Person>,
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
        let older = (seed.as_ref())
            .filter(|s| s.older())
            .map(|s| crate::made_older(s.name().as_str()));
        let name = seed.as_ref().map(|s| s.name().as_str().to_string());
        let made = seed.map(|s| Namesake::of(rail.root(), &s, me));
        // One read for both: a group not held or with no seed is the empty
        // register, said by the rail (`RingRail::register`).
        let (person, removed, removals, left, keeps, visiting, offered, vacates) = match rail
            .register(&id)
        {
            Ok(r) => {
                let other = match &r.office {
                    Office::Pending { to, by, .. } if to == me => Some((true, by)),
                    Office::Pending { to, by, .. } if by == me => Some((false, to)),
                    _ => None,
                };
                let named = |k: &str| {
                    (r.members.get(k).map(|m| m.person.clone()))
                        .or_else(|| r.removed.get(k).cloned())
                };
                let offered = other.map(|(to_you, k)| Offer {
                    to_you,
                    person: named(k),
                });
                let visiting = crate::rail::visiting(&r, me);
                let person_of = |k: &str| r.members.get(k).map(|m| &m.person_key);
                let keeper = r.keeper.as_ref().and_then(|k| r.members.get(k));
                let keeps = keeper.is_some_and(|k| Some(&k.person_key) == person_of(me));
                let left = (r.removals.iter()).any(|x| x.key == me && x.said == RemovalSaid::Left);
                let others = (r.members.values()).any(|m| Some(&m.person_key) != person_of(me));
                let person = r.members.get(me).map(|m| m.person.clone());
                (
                    person,
                    Some(r.removed.contains_key(me)),
                    Some(r.removals),
                    Some(left),
                    Some(keeps),
                    visiting,
                    offered,
                    keeps && others,
                )
            }
            Err(e) => {
                tracing::warn!(target: "surface", namespace = id, error = %e, "surface: groups — its register is unreadable, so neither this computer's person nor whether it was removed is known");
                (None, None, None, None, None, None, None, false)
            }
        };
        let folders = crate::folder::dirs_of(&at.door.data_dir, &id)
            .inspect_err(|e| tracing::warn!(target: "surface", namespace = id, error = %e, "surface: groups — where its folders are kept cannot be read"))
            .ok();
        let waiting = at.door.folders.waiting_in(&id);
        tracing::debug!(target: "surface", namespace = id, name = ?name, person = ?person, removed = ?removed, ?left, ?keeps, ?offered, older = older.is_some(),
            removals = ?removals.as_ref().map(Vec::len), folders = ?folders, "surface: groups — a group held");
        held.push(Held {
            fallback: at.host.at_fallback(&id),
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
            waiting,
            offered,
            vacates,
            older,
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
        let why = format!("this computer is not in {namespace}, so nobody in it lends to it");
        let refused = refused_as(StatusCode::CONFLICT, &namespace, "not_in_roster", why);
        return Err(refused);
    };
    let groups: Groups = vec![(namespace.clone(), standing)];
    let (lent, unlisted) = at.lent(&groups, &own, None).await?;
    let host = &at.host;
    // What each open says it reads needs the person's yeses: unreadable,
    // it says only the kinds, and its open then names no lend.
    let yeses = choices::reachings(&at.door).ok();
    tracing::debug!(target: "surface", namespace, yeses = ?yeses.as_ref().map(BTreeMap::len), "surface: lends — the yeses each open is told, none when they cannot be read");
    // What each app carried here sends to, by its files as this computer's
    // person runs them, never as another computer lists it (CONTRACTS §7):
    // unreadable, it shows no send.
    let mut sending = BTreeMap::new();
    for lend in at.door.folders.carried() {
        let kinds = (at.door.folders).words(&lend, SENDS).await;
        sending.insert(lend.label(&at.door.key), kinds.unwrap_or_default());
    }
    let lent: Vec<serde_json::Value> = (lent.iter())
        .filter_map(|l| {
            let origin = host.origin(&l.key, &l.offered, l.version)?;
            let mut shown = serde_json::to_value(l).ok()?;
            shown["origin"] = origin.into();
            // What `svrngs check` finds in a carried app's folder, for its card.
            let (group, kind) = (l.offered.group.as_deref(), l.offered.kind.as_str());
            shown["check"] = group.and_then(|g| at.door.folders.checked(g, kind)).into();
            let yes = yeses.as_ref();
            if let Some(reading) = opening(l, &lent, &groups, yes, &host.actor, &sending) {
                shown["reading"] = reading;
            }
            Some(shown)
        })
        .collect();
    tracing::debug!(target: "surface", namespace, lent = lent.len(), ?unlisted,
        "surface: lends — answered what the group's people lend, this computer's own among them, each at its origin");
    let mut body = serde_json::json!(unlisted);
    body["namespace"] = namespace.into();
    body["lent"] = lent.into();
    body["shelf"] = crate::folder::shelf();
    body["changed"] = at.door.folders.changed(&groups, &at.host.actor);
    Ok(Json(body).into_response())
}

/// What the page of the lend `app` reads now: each lend of a kind its
/// lender declared it reads (`Offered.reads`), in the
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
    let reader = match crate::origin::serving(at, app).await? {
        crate::origin::Serving::Own => at.host.actor.clone(),
        crate::origin::Serving::At(_, lender) => lender,
    };
    let host = &at.host;
    let (mut groups, own) = host.shared_with(&reader)?;
    groups.retain(|(_, s)| s.contains_key(&host.actor));
    let Some(own) = own else {
        tracing::debug!(target: "surface", %app, "surface: reads refused — its lender and this computer share no group");
        return Err(LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing is lent here"));
    };
    let (lent, unlisted) = at.lent(&groups, &own, None).await?;
    let silent: Vec<String> = (groups.iter().flat_map(|(_, s)| s.iter()))
        .filter(|(_, m)| unlisted.names(&m.person_key))
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
    let read: Vec<_> = reads_among(app, reading, &lent, &groups).cloned().collect();
    tracing::debug!(target: "surface", %app, group = ?reading.offered.group, read = read.len(), "surface: what the page reads, its kinds, a carried app's only in its own group");
    Ok((read, unlisted, silent))
}

/// What the lend `reading`, whose label is `app`, reads among `lent`: each
/// lend of a kind it declares, never itself, a carried app's only in its
/// own group, lent to it, by one standing in it. The one decider of
/// [`read_by`] and of what an open names ([`opening`]).
fn reads_among<'a>(
    app: &'a Label,
    reading: &'a Lending,
    lent: &'a [Lending],
    groups: &'a Groups,
) -> impl Iterator<Item = &'a Lending> {
    lent.iter().filter(move |l| {
        let in_group = (reading.offered.group.as_ref()).is_none_or(|g| {
            let lent = (l.offered.groups.as_ref()).is_some_and(|ids| ids.contains(g));
            lent && (groups.iter()).any(|(ns, s)| ns == g && s.contains_key(&l.key))
        });
        in_group
            && named(l).as_ref() != Some(app)
            && reading.offered.reads.contains(&l.offered.kind)
    })
}

/// What opening `l` reads, for its line and ticks on the home page: each
/// lend it would name, whose, and whether it is ticked or new since the
/// person's last yes; and, for a lend with `sends:` rows of a kind the app
/// sends to (`sending`, by app), the rows, ticked apart only where that yes
/// named it to send to (CONTRACTS §6.6). `None` for a lend reading nothing;
/// for this computer's own app, which needs no yes ([`reaches`]), so it
/// lists no ticks; and where the yeses cannot be read, as its open then
/// names no lend.
fn opening(
    l: &Lending,
    lent: &[Lending],
    groups: &Groups,
    yeses: Option<&BTreeMap<Label, choices::Choice>>,
    me: &str,
    sending: &BTreeMap<Label, Vec<Program>>,
) -> Option<serde_json::Value> {
    let own = l.key == me && l.offered.group.is_none();
    if own || l.offered.reads.is_empty() {
        return None;
    }
    let app = named(l)?;
    // A yes from before this names no lend, so it says nothing is new.
    let named_any = |c: &&choices::Choice| !(c.reads.is_empty() && c.left.is_empty());
    let yes = yeses?
        .get(&app)
        .filter(named_any)
        .filter(|c| c.said == choices::Said::Yes);
    let to = sending.get(&app).map_or(&[][..], Vec::as_slice);
    let reading: Vec<serde_json::Value> = reads_among(&app, l, lent, groups)
        .filter_map(|read| {
            let label = named(read)?;
            let left = yes.is_some_and(|c| c.left.contains(&label));
            let new = !left && yes.is_some_and(|c| !c.reads.contains(&label));
            let rows = read.offered.sends.iter().filter(|_| to.contains(&read.offered.kind));
            let sends: Vec<&str> = rows.map(SendRow::as_str).collect();
            let sent = !sends.is_empty() && yes.is_some_and(|c| c.sends.contains(&label));
            Some(serde_json::json!({ "label": label, "person": read.person, "name": read.offered.name,
                "kind": read.offered.kind, "ticked": !left, "new": new, "sends": sends, "sending": sent }))
        })
        .collect();
    tracing::debug!(target: "surface", %app, reading = reading.len(), yes = yes.is_some(), sends_to = ?to, "surface: lends — what its open names, ticked or new since the person's last yes, and what it may send to, ticked apart");
    Some(reading.into())
}

/// A lend's label, as its lender lists it ([`crate::lend::Offered::label`]).
fn named(l: &Lending) -> Option<Label> {
    l.offered.label(&l.key, l.version)
}

/// What the person said of `app` reaching others' lends: `None` for this
/// computer's own app, which reads what its person declared; a carried app
/// is its group's code, not this computer's person's, so it reaches on
/// their yes, as another computer's app does.
async fn said_of(at: &Asking, app: &Label) -> Result<Option<Option<choices::Choice>>, Response> {
    let serving = crate::origin::serving(at, app).await?;
    let own =
        matches!(app.whose(), Whose::Computer(_)) && matches!(serving, crate::origin::Serving::Own);
    let reaching = (!own).then(|| choices::reaching(&at.door, app));
    reaching.transpose()
}

/// Whether the page of the lend `app` may reach the lend `label`
/// ([`crate::origin`]), a request that sends if `send`, by the community
/// profile's rules (`lend::reach`): `label` among what it reads
/// ([`read_by`]), and, for another computer's app or one its group carries,
/// named by the person's yes, which their last open from the home page said,
/// ticking only what they left ticked (capability-open;
/// [`choices::reaching`]), and named apart to send to where a carried app
/// sends (CONTRACTS §6.6): this computer's own app reads what its person
/// declared. A lend whose lender did not say is could-not-judge, refused
/// `away`.
pub(crate) async fn reaches(
    at: &Asking,
    app: &Label,
    label: &Label,
    send: bool,
) -> Result<(), Response> {
    let (read, unlisted, silent) = read_by(at, app).await?;
    let reads = read.iter().any(|l| named(l).as_ref() == Some(label));
    // The person's yes is read only for a lend the page reads.
    let said = if reads { said_of(at, app).await? } else { None };
    let yes = said.map(|c| c.map_or((false, false), |c| (c.names(label), c.sends_to(label))));
    // A carried app's computers are each listed or not: none stands for it.
    let lender = match label.whose() {
        Whose::Computer(key) => silent.contains(&hex::encode(key)),
        Whose::Carried => false,
    };
    let sending = (send, matches!(app.whose(), Whose::Carried));
    let reached = lend::reach(&at.door.key, label.as_str(), (reads, lender), yes, sending);
    tracing::debug!(target: "surface", %app, %label, reads, lender, ?yes, ?sending, ?unlisted, ?reached, "surface: whether the page reaches the lend, by the community profile's rules");
    reached.map_err(|refused| match refused {
        lend::Refusal::Silent => crate::origin::away(),
        lend::Refusal::NotRead => LendRefusal::closed(refused.to_string()),
        _ => LendRefusal::Unasked.say(StatusCode::FORBIDDEN, refused.to_string()),
    })
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
    let (lent, Unlisted { unanswered, .. }, ..) = read_by(at, label).await?;
    let said = said_of(at, label).await?;
    let yes = said.as_ref().and_then(Option::as_ref);
    // This computer's own app reads what its person declared, so every
    // lend is named; another's, those its person's yes names ([`reaches`]).
    let names = |l: &&Lending| {
        named(l).is_some_and(|lend| said.is_none() || yes.is_some_and(|c| c.names(&lend)))
    };
    let read: Vec<serde_json::Value> = (lent.iter())
        .filter(&names)
        .filter_map(|l| {
            let lend = named(l)?;
            // A carried page reaches it at its own origin, holding no capability.
            let address = match label.whose() {
                Whose::Carried => format!("{}{}/{lend}", origin(label, host.api_port), &LEND_PATH[1..]),
                Whose::Computer(_) => origin(&lend, host.api_port) + &AS_PATH[1..] + &at.sessions.cap(label, &lend),
            } + "/";
            let mut read = serde_json::json!({ "kind": l.offered.kind, "name": l.offered.name, "lender": l.person, "address": address });
            // A carried page opens a lender's own page by svrngs's `to`, no capability in it.
            if matches!(label.whose(), Whose::Carried) {
                read["to"] = format!("{}{}?lender={}&lend={}", origin(label, host.api_port), &TO_PATH[1..], l.key, l.offered.name).into();
            }
            if let Some(words) = &l.offered.asks {
                read["asks"] = words.as_str().into();
                read["ask"] = format!("{}ask?lend={lend}&app={label}", crate::home::address(host.api_port)).into();
                let page = origin(label, host.api_port);
                read["stop"] = format!("{page}{}/{lend}{STOP_SUFFIX}", &READS_PATH[1..]).into();
            }
            Some(read)
        })
        .collect();
    // What the person's yes does not name: no address, enough for the app to
    // say to open it from the home page. A lend they unticked is not said.
    let left = |l: &Lending| named(l).is_some_and(|x| yes.is_some_and(|c| c.left.contains(&x)));
    let new: Vec<serde_json::Value> = (lent.iter())
        .filter(|l| !names(l) && !left(l))
        .map(|l| serde_json::json!({ "kind": l.offered.kind, "name": l.offered.name, "lender": l.person }))
        .collect();
    tracing::debug!(target: "surface", %label, own = said.is_none(), read = read.len(), new = new.len(), unanswered = unanswered.len(),
        "surface: reads — answered a lent page the lends its person's yes names, each at its origin here, and how many more it does not name");
    // A lent page is told each lender by name, as it always was.
    let unanswered: Vec<_> = unanswered.iter().map(|n| &n.person).collect();
    let mut body = serde_json::json!({ "lends": read, "unanswered": unanswered });
    if !new.is_empty() {
        body["new"] = new.into();
    }
    Ok(Json(body).into_response())
}

/// The lend a carried app's page names at [`TO_PATH`]: its lender's key, in
/// hex as the record spells an actor, and its name; and `whether`, true for
/// a page asking only whether it opens.
#[derive(Deserialize)]
struct To {
    lender: String,
    lend: String,
    #[serde(default)]
    whether: bool,
}

/// `GET /svrngs-to?lender=&lend=` at the origin of the carried app `app`:
/// the lend `lend` of the computer `lender`, a page and no app a group
/// carries, as this computer's person is told it in the app's group
/// ([`lends`]' list, so only a computer standing in that group, and only
/// what it lends to that group). It is sent on to the lend's plain origin,
/// which opens as a bookmark of it does ([`crate::origin`]'s `homeward`):
/// the home page's session decides, its code is minted there and nowhere
/// else, and any yes the lend asks is still asked. The address carries no
/// code, capability or proof, and a page's own fetch cannot read where a
/// redirect goes; so a script gets nothing to open with. With `whether`,
/// a page is told only whether it would open, 204. A page's own fetch and
/// its tab look alike here, its session worker sending both as fetches,
/// so the page says which it asks. Not lent there is 404; a lender who did
/// not say what it lends, `away`.
pub(crate) async fn to(at: &Asking, app: &Label, uri: Uri) -> Handled {
    let unlent = || LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "nothing is lent here");
    let Ok(Query(to)) = Query::<To>::try_from_uri(&uri) else {
        return Err(LendRefusal::bad_path("name a lender and a lend"));
    };
    let carried = (at.door.folders.carried().into_iter()).find_map(|l| match l.source {
        Source::Carried(a, g) if Label::carried(a.as_str(), &g) == *app => Some(g),
        _ => None,
    });
    let Some(group) = carried else {
        tracing::debug!(target: "surface", %app, "surface: to refused — not an app a group carries here");
        return Err(LendRefusal::closed("only a carried app opens this"));
    };
    let mut standing = standing_of(&at.host.rail, &group)?;
    let own = standing.get(&at.host.actor).map(|m| m.person.clone());
    let own = own.ok_or_else(unlent)?;
    standing.retain(|key, _| *key == to.lender);
    let groups = [(group.clone(), standing)];
    let (lent, unlisted) = at.lent(&groups, &own, None).await?;
    let found = (lent.iter())
        .filter(|l| l.offered.group.is_none() && l.offered.raw.is_none())
        .find(|l| l.key == to.lender && l.offered.name.as_str() == to.lend);
    let origin = found.and_then(|l| at.host.origin(&l.key, &l.offered, l.version));
    let silent = groups[0].1.values().any(|m| unlisted.names(&m.person_key));
    tracing::debug!(target: "surface", %app, group, lender = to.lender, lend = to.lend, found = origin.is_some(), whether = to.whether, silent,
        "surface: to — one lend of the carried app's group, sent on to its plain origin, or told only whether it opens");
    match origin {
        Some(_) if to.whether => Ok(StatusCode::NO_CONTENT.into_response()),
        Some(origin) => Ok(Redirect::to(&origin).into_response()),
        None if silent => Err(crate::origin::away()),
        None => Err(unlent()),
    }
}
