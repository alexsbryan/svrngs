// SPDX-License-Identifier: AGPL-3.0-or-later
//! The home page (sup-2; DESIGN §3, "The home page"; operator, 2026-10-09):
//! `http://svrngs.localhost:<port>/`, the person's own page on their
//! computer, on the record API's listener beside the record and the lends'
//! origins, routed by its `Host` ([`crate::api`]'s `to_a_lend`) and served
//! only at its exact host and port. No lend lives there: a label is 62
//! letters of base32, which `svrngs` never is.
//!
//! **It answers only itself** ([`only_itself`]): a read only from its own
//! origin or a typed address (`Sec-Fetch-Site: same-origin` or `none`), or
//! a navigation, which no other page can read; a write only from its own
//! origin, with its exact `Origin`, so a lend's page — same-site, never
//! same-origin — can neither read its list nor say a yes. Every answer
//! sends `frame-ancestors 'none'`, so no page frames it to borrow a click,
//! and `default-src 'self'`; its one page, its module and its look are
//! served from the node, and it runs no app's code.
//!
//! **What it shows** is what `/v1/groups`, `/v1/lends` and
//! `/v1/rail/people` answer, from the same functions ([`crate::surface`],
//! [`crate::rail`]; one decider): the person's groups, the people in each,
//! what each lends opened at its origin, and whose computer did not answer;
//! and, at `GET /lent`, what this computer lends, as a member is told it.
//! **Its only writes are the person's yes and stop** (`POST /choices`), and
//! **their change to what this computer lends** (`POST /lent`, bash-limit):
//! how many it lets in at once, a number or everyone, or no longer lent —
//! written to its declaration, which the door reads at its next request;
//! **a link to a group** (`POST /link`, start-home), made as `svrngs
//! link` makes it, in the group's record; **a guest's visit stopped** by
//! its host (`POST /visits`, guests-pages); **a group's folder opened**
//! in this computer's file browser (`POST /folder`, start-home); and **a
//! group made** by the svrngs running here, as `svrngs new` makes one
//! (`POST /group`, start-group; the node's group.rs `Asking::found`);
//! **a group joined** from a link someone sent, as `svrngs join` joins
//! (`POST /join`, once `GET /join?link=` has said what the link is for;
//! arch-command (c), the node's join.rs); **the shelf's apps brought**
//! (`POST /shelf`) and **the person's yes to a carried app's new version**
//! (`POST /runs`; canon-pin (c)), each in the group's folder or record; and
//! **a new svrngs fetched** from the page this computer came from, and this
//! one restarted into it (`GET` and `POST /update`; [`crate::update`]); and
//! **what runs on this computer lent** (`POST /running`, v1ui-lend (b)),
//! decided as `svrngs share` decides a lend ([`crate::share`]) from what
//! `GET /running` offers, every port that answers here and is not lent;
//! and **a lend opened to this WiFi, or closed** (`POST /v1/room`, the room's
//! one route, [`crate::room`], which `svrngs room` asks too); **a port
//! lent opened on this computer while Play is on, or closed** (`POST
//! /v1/play`, [`crate::play`]); and **an app
//! opened by its button, which is the person's yes to it reaching their
//! friends' lends as them** (`POST /reaching`, a form; capability-open),
//! **or that yes stopped**, each listed under "What acts as you" (`GET
//! /reaching`; capability-what-acts).
//! The person's yes and stop are
//! kept by this computer's door for its own lend, else dialled to the
//! lender's door on the members' protocol as this computer's key
//! ([`at_door`]). `GET /choices?lend=` asks that door, when the person
//! opens the list, what apps use. It asks at `/ask?lend=&app=&back=`.

use axum::extract::{rejection::JsonRejection, Query, Request, State};
use axum::http::header::{self, HeaderValue};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Form, Json, Router};
use serde::Deserialize;
use std::ffi::OsStr;
use std::num::NonZeroU16;
use std::process::{Command, Stdio};
use std::time::Duration;
use svrngs_core::link::{client_link, https_link, Link};
use svrngs_core::pass::Label;

use crate::door::choices::{own_choices, own_choose, Said, Saying};
use crate::door::{LendRefusal, PASS_HEADER, REFUSED_HEADER};
use crate::join::{Entered, NotJoined};
use crate::lend::{self, Audience, LendName, Program, Source::Port};
use crate::node::Asking;
use crate::share::{self, Share, ShareRefusal};

/// The home page's host, before its port.
const HOST: &str = "svrngs.localhost";

/// What a lend's change that could not be written says.
const UNCHANGED: &str = "that cannot be changed on this computer";

const POLICY: &str = "default-src 'self'; frame-ancestors 'none'";

/// When a person joined, as the page says it (home.js `people`) and
/// `svrngs remove --joined` takes it: this computer's time to the second
/// with its offset, so the hour a fall-back night repeats reads back to one
/// instant (ralph/REVIEW_FINDINGS.md #126). chrono's form; the page writes
/// it by hand.
pub const WHEN: &str = "%Y-%m-%d %H:%M:%S %z";

/// The home page's files, as built in: one table of each one's path, type
/// and body, its root module; svrngs's words, in the one wording the
/// group's page loads too (`scripts/build-web.sh` gives it the same file);
/// its look, svrngs's one look, a file of its own since its policy runs no
/// inline style, which the group's page is given too; and the tab's icon,
/// the svrngs mark's small cut, from its one home (docs/brand/). Then each
/// module the root imports, by name under its directory: a flow's at
/// `/page/`, the parts both pages draw with at `/ui/`, and the vendored
/// Preact and htm at `/vendor/` (web/vendor/README.md). A path no name
/// makes is 404, so nothing beside these files is reached
/// (tests/home.rs, `the_home_page_s_modules_are_served_each_as_its_file`).
macro_rules! served {
    ($($path:literal $kind:literal $body:literal)* $(; $at:literal $dir:literal [$($name:literal)*])*) => {
        Router::new()$(.route($path, get(|| async { ([(header::CONTENT_TYPE, $kind)], include_str!($body)) })))*
            $($(.route(concat!($at, $name, ".js"), get(|| async { ([(header::CONTENT_TYPE, "text/javascript")], include_str!(concat!($dir, $name, ".js"))) })))*)*
    };
}

/// Whether `host` names the home page, at any port: [`only_itself`] holds
/// it to this listener's.
pub(crate) fn named(host: Option<&HeaderValue>) -> bool {
    let host = host.and_then(|h| h.to_str().ok()).unwrap_or_default();
    host.split(':')
        .next()
        .is_some_and(|h| h.eq_ignore_ascii_case(HOST))
}

/// The home page's address, with the record API listening on `port`: the
/// one form the command and the node say it in.
pub fn address(port: u16) -> String {
    format!("http://{HOST}:{port}/")
}

/// The home page's routes, beside the lends' origins: a request naming it
/// is answered here, any other at its lend's origin.
pub(crate) fn router(at: Asking, rail: crate::rail::RailApi) -> Router {
    let page = || async { axum::response::Html(include_str!("home/index.html")) };
    let home = served! {
        "/home.js" "text/javascript" "home/home.js"
        "/question.js" "text/javascript" "home/question.js"
        "/look.css" "text/css" "home/look.css"
        "/favicon.svg" "image/svg+xml" "../../../docs/brand/favicon.svg";
        "/page/" "home/page/" ["acting" "acts" "acts-read" "address-copy" "ask" "asked" "asking" "door" "folder" "folders" "found" "group" "group-choice" "group-folder" "group-link" "group-links" "group-read" "group-reading" "group-screen" "group-told" "groups-left" "home-page" "home-reading" "how-it-opens" "index" "index-rows" "join" "join-form" "leave" "leave-group" "leaving" "lend-change" "lend-found" "lend-key" "lend-name" "lend-row" "lending" "lent-read" "lent-row" "link-box" "link-copy" "link-said" "links" "make" "make-form" "offered" "on-screen" "open" "open-link" "people" "play" "play-cell" "reach" "removals" "routes" "running" "running-here" "running-look" "screen" "screen-link" "shelf" "signing-on" "since-opened" "stop-visit" "update" "update-line" "visit" "visits" "what-you-lend" "wifi" "wifi-foot"];
        "/page/" "home/page/" ["grouped"];
        "/ui/" "home/ui/" ["acts" "changes" "copy" "door" "machine" "parts" "shelf" "trace" "uses" "view" "visitors"];
        "/vendor/" "../../../web/vendor/" ["hooks" "htm" "preact"]
    }
    .route("/", get(page))
    .route("/ask", get(page))
    .route("/to", get(to))
    .route("/choices", get(choices).post(choose))
    .route("/reaching", get(reached).post(reaching))
    .route("/lent", get(lent).post(change))
    .route("/running", get(running).post(lend_one))
    .route("/link", axum::routing::post(link))
    .route("/connection", axum::routing::post(connection))
    .route("/publication", get(crate::publication::status).post(crate::publication::publish))
    .route("/publication/close", axum::routing::post(crate::publication::close))
    .route("/visits", axum::routing::post(stop))
    .route("/folder", axum::routing::post(open))
    .route("/update", get(update_check).post(update_apply))
    .route("/shelf", axum::routing::post(crate::folder::bring))
    .route("/runs", axum::routing::post(crate::folder::yes))
    .route("/group", get(namesakes).post(found))
    .route("/join", get(asked).post(join))
    .with_state(at.clone())
    .merge(
        Router::new()
            .route("/v1/rail/people", get(crate::rail::people))
            .with_state(rail),
    )
    .merge(crate::surface::router(at.clone()))
    .merge(crate::removal::router(at.clone()))
    .merge(crate::room::route::router(at.clone()))
    .merge(crate::play::router(at.clone()));
    let home = home.layer(axum::middleware::from_fn_with_state(
        at.clone(),
        only_itself,
    ));
    let lends = crate::origin::router(at);
    Router::new().fallback(|req: Request| async move {
        use tower::ServiceExt;
        let to = if named(req.headers().get(header::HOST)) {
            home
        } else {
            lends
        };
        to.oneshot(req).await.unwrap_or_else(|never| match never {})
    })
}

/// The home page's one rule (the module doc), and its policy on every answer.
async fn only_itself(State(at): State<Asking>, mut req: Request, next: Next) -> Response {
    let own = format!("{HOST}:{}", at.host.api_port);
    let ([site, mode, origin, host], own_page) = crate::origin::fetch(req.headers());
    let at_own = host.is_some_and(|h| h.eq_ignore_ascii_case(&own));
    let reads = matches!(*req.method(), Method::GET | Method::HEAD);
    let answers = at_own
        && if reads {
            own_page
        } else {
            site == Some("same-origin") && origin.is_some_and(|o| o == format!("http://{own}"))
        };
    if at_own {
        if let Some(answer) = at.sessions.public(crate::session::HOME, &req) {
            return ([(header::CONTENT_SECURITY_POLICY, POLICY)], answer).into_response();
        }
    }
    let shown = at.sessions.shown(&req, crate::session::HOME);
    let opening = at_own && req.uri().path() == crate::session::OPEN_PATH;
    tracing::debug!(target: "home", path = %req.uri().path(), method = %req.method(), at_own, ?site, ?mode, ?origin, answers, ?shown, opening,
        "home: answered only to an open, then only with its session or the token, from itself, a typed address or, to read, a navigation");
    let intent = req
        .headers()
        .get("x-svrngs-intent")
        .is_some_and(|h| h == "yes");
    let closed = || LendRefusal::closed("this page answers only itself");
    let answer = if opening {
        at.sessions.redeem(crate::session::HOME, &req)
    } else if shown == crate::session::Shown::Neither {
        tracing::debug!(target: "home", path = %req.uri().path(), "home: refused — neither its session's proof nor the token");
        at.sessions.refused("")
    } else if !answers {
        tracing::debug!(target: "home", path = %req.uri().path(), ?site, ?origin, "home: refused — not from itself, a typed address or, to read, a navigation");
        closed()
    } else if shown == crate::session::Shown::Token || reads {
        next.run(req).await
    } else if !intent {
        tracing::debug!(target: "home", path = %req.uri().path(), "home: refused — a write its session proved, without its own page's intent");
        closed()
    } else if !at.sessions.body_matches(&mut req).await {
        tracing::debug!(target: "home", path = %req.uri().path(), "home: refused — a write whose body is not the one its proof signed");
        closed()
    } else {
        next.run(req).await
    };
    ([(header::CONTENT_SECURITY_POLICY, POLICY)], answer).into_response()
}

/// `POST /connection`, the form of "Check the connection now": this
/// computer's fallback asked now ([`crate::node::Host::check_now`]), a round
/// fired, and This computer, where the button is, its address saying what
/// happened.
async fn connection(State(at): State<Asking>) -> Redirect {
    let said = at.host.check_now().await;
    at.door.folders.nudge.notify_one();
    Redirect::to(&format!("/?view=computer&{said}"))
}

/// What the person says of an app on a lend.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Choosing {
    lend: Label,
    app: Label,
    said: Said,
}

/// `POST /choices`: the person's yes or stop, kept by the lend's door.
async fn choose(State(at): State<Asking>, Json(c): Json<Choosing>) -> Response {
    say(&at, &c.lend, c.app, c.said, None).await
}

/// The app the person opens from its button on this page, or stops from
/// "What acts as you", `said` stop.
#[derive(Deserialize)]
struct Opening {
    app: Label,
    said: Option<Said>,
    /// The lends its open showed and left ticked, labels apart by spaces.
    reads: Option<String>,
    /// The lends its open showed and the person unticked, the same way.
    left: Option<String>,
}

/// `POST /reaching`, a form the open button posts in a tab of its own:
/// the person's yes to the app reaching others' lends as them, kept
/// ([`crate::door::choices::own_reaching`]), then the app's own page; or
/// its Stop, then this page again, the stop held from the app's next
/// request ([`crate::surface::reaches`]).
async fn reaching(State(at): State<Asking>, Form(o): Form<Opening>) -> Response {
    let said = o.said.unwrap_or(Said::Yes);
    // A word no label parses (a tab from before a yes named lends, posting
    // kinds) names no lend: the yes covers less, never more.
    let labels = |field: Option<String>| -> Vec<Label> {
        field
            .iter()
            .flat_map(|f| f.split_whitespace())
            .filter_map(Label::parse)
            .collect()
    };
    let told = (said, labels(o.reads), labels(o.left));
    let named = (told.1.len(), told.2.len());
    let kept = crate::door::choices::own_reaching(&at.door, o.app.clone(), told).await;
    tracing::debug!(target: "home", app = %o.app, ?said, ?named, status = kept.status().as_u16(), "home: an app opened from its button or stopped, what was said kept");
    if !kept.status().is_success() {
        return kept;
    }
    let to = match said {
        Said::Yes => at.sessions.open(o.app.as_str(), None),
        Said::Stop => "/".into(),
    };
    Redirect::to(&to).into_response()
}

/// A lend the page opens, at a path there.
#[derive(Deserialize)]
struct To {
    lend: Label,
    at: Option<String>,
}

/// `GET /to?lend=&at=`: the lend opened from this page at its origin, with
/// a code for that origin alone, which becomes its session there
/// ([`crate::session`]; give-session).
async fn to(State(at): State<Asking>, Query(t): Query<To>) -> Redirect {
    Redirect::to(&at.sessions.open(t.lend.as_str(), t.at.as_deref()))
}

/// `GET /reaching`: what the person said of each app reaching others'
/// lends as them, and when, for "What acts as you"; nothing on a computer
/// in no group, which has no person to say it.
async fn reached(State(at): State<Asking>) -> Response {
    let kept = crate::door::choices::reachings(&at.door);
    let answered = match kept {
        Ok(kept) => Json(kept).into_response(),
        Err(r) if r.status() == StatusCode::FORBIDDEN => {
            Json(serde_json::json!({})).into_response()
        }
        Err(refused) => refused,
    };
    tracing::debug!(target: "home", status = answered.status().as_u16(), "home: what was said of apps reaching others' lends, asked");
    answered
}

/// What `/update` answers, its one field said once.
fn update_state(state: &str) -> Response {
    Json(serde_json::json!({ "state": state })).into_response()
}

/// `GET /update`: whether the page this computer came from offers a svrngs
/// it does not run; `unknown` when that cannot be judged, never `current`.
async fn update_check(State(at): State<Asking>) -> Response {
    match crate::update::check(&at.door.data_dir).await {
        Ok(crate::update::Check::Current) => update_state("current"),
        Ok(crate::update::Check::Available) => update_state("available"),
        Err(e) => {
            tracing::debug!(target: "home", error = %e, "home: an update cannot be judged");
            Json(serde_json::json!({ "state": "unknown", "why": e.to_string() })).into_response()
        }
    }
}

/// `POST /update`: the page's release put in place, then this node stopped
/// so the one that starts in its place is the new binary. A refusal keeps
/// the running one and says why.
async fn update_apply(State(at): State<Asking>) -> Response {
    match crate::update::apply(&at.door.data_dir).await {
        Ok(crate::update::Outcome::Current) => update_state("current"),
        Ok(crate::update::Outcome::Updated) => {
            let answer = update_state("updated");
            tokio::spawn(async {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                tracing::info!(target: "home", "home: the new svrngs is in place, so this one stops for it to start");
                std::process::exit(75);
            });
            answer
        }
        Err(e) => {
            tracing::debug!(target: "home", error = %e, judged = e.judged(), "home: an update was refused");
            let status = if e.judged() {
                StatusCode::CONFLICT
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            };
            LendRefusal::Closed.say(status, e.to_string())
        }
    }
}

/// What the person changes of a lend this computer lends.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Changing {
    lend: LendName,
    to: LendChange,
}

/// A closed set (AGENTS 9): let in this many at once, none for everyone in
/// its groups; the groups it is lent to (LENDING_SCOPE); the account apps
/// read it through; or lend it no longer. No `Debug`, so a password typed
/// here can't be traced.
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum LendChange {
    Limit(Option<NonZeroU16>),
    /// The groups it is lent to, by stable ID: an audience-only edit that
    /// keeps its secret, sign-in client and choices (SCOPE_PARALLEL.md).
    Groups(Vec<String>),
    /// The lender turns on signing people in with svrngs for it.
    #[serde(rename = "sign-in")]
    SignIn,
    /// The account apps reading it are let in as, for one sign-in now.
    Readers {
        name: String,
        password: String,
    },
    Stop,
}

/// `GET /lent`: what this computer lends, each with how many it lets in at
/// once and how many use it now, as a member asking is told it, and the
/// lends that sign people in with svrngs: their secret is never in this
/// answer (SIGN_IN_WITH_SVRNGS §5, "Turning it on").
async fn lent(State(at): State<Asking>) -> Response {
    let (lent, all) = match (
        crate::door::offered(&at.door).await,
        lend::all(&at.door.data_dir),
    ) {
        (Ok(lent), Ok(all)) => (lent, all),
        (Err(refused), _) => return refused,
        (_, Err(e)) => return unread("it cannot read what you lend already", &e),
    };
    let all: Vec<_> = all.into_iter().flatten().collect();
    let signing: Vec<_> = (all.iter().filter(|l| l.sign_in_with_svrngs.is_some()))
        .map(|l| l.name.to_string())
        .collect();
    let readers = |l: &lend::Lend| {
        let kept = lend::readers_dir(&at.door.data_dir, &l.name).exists();
        let said = serde_json::json!({ "kept": kept, "signs": share::accounts_of(&l.kind) });
        share::readers_of(&l.kind).map(|_| (l.name.to_string(), said))
    };
    let readers: serde_json::Map<_, _> = all.iter().filter_map(readers).collect();
    Json(serde_json::json!({ "lent": lent, "signing": signing, "readers": readers }))
        .into_response()
}

/// `POST /lent`: the person's change, written to the lend's declaration.
async fn change(State(at): State<Asking>, Json(c): Json<Changing>) -> Response {
    let dir = &at.door.data_dir;
    let to = kind_of(&c.to);
    let done = match c.to {
        LendChange::Limit(limit) => lend::limit(dir, &c.lend, limit),
        LendChange::Groups(groups) => match chosen(&at, &groups) {
            Ok(()) => lend::set_audience(dir, &c.lend, groups),
            Err(refused) => {
                tracing::debug!(target: "home", lend = %c.lend, "home: refused — an audience names a group this computer does not hold");
                return refused;
            }
        },
        LendChange::Stop => lend::withdraw(dir, &c.lend),
        LendChange::SignIn => return sign_in(&at, &c.lend).await,
        LendChange::Readers { name, password } => {
            return readers(&at, &c.lend, &name, &password).await
        }
    };
    tracing::debug!(target: "home", lend = %c.lend, to, done = ?done.as_ref().ok(), "home: the person changed a lend");
    match done {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        unchanged => unchanged_said(&c.lend, unchanged),
    }
}

/// A change not made, said: nothing lent by that name, another change
/// holding it (409, try again), or a declaration that cannot be changed.
fn unchanged_said(name: &LendName, done: Result<bool, lend::LendError>) -> Response {
    match done {
        Ok(_) => LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "you lend nothing by that name"),
        Err(e @ lend::LendError::Busy { .. }) => {
            tracing::debug!(target: "home", lend = %name, error = %e, "home: refused — another change to it is being saved");
            let words = "another change to it is being saved: try again";
            LendRefusal::Closed.say(StatusCode::CONFLICT, words)
        }
        Err(e) => {
            tracing::warn!(target: "home", lend = %name, error = %e, "home: refused — the lend's declaration cannot be changed");
            LendRefusal::fault(UNCHANGED)
        }
    }
}

/// A short name for a change, for the trace: only the kind is kept, never a
/// group list a person chose.
fn kind_of(change: &LendChange) -> &'static str {
    match change {
        LendChange::Limit(_) => "limit",
        LendChange::Groups(_) => "groups",
        LendChange::SignIn => "sign-in",
        LendChange::Readers { .. } => "readers",
        LendChange::Stop => "stop",
    }
}

/// A selected audience names only groups this computer holds, by stable ID
/// (LENDING_SCOPE): an unknown ID is refused, never stored, so a misspelt
/// or foreign group can never become a grant. Empty is allowed (paused).
fn chosen(at: &Asking, groups: &[String]) -> Result<(), Response> {
    let held = at.door.rail.namespaces().map_err(|e| {
        tracing::warn!(target: "home", error = %e, "home: refused — the groups held cannot be listed");
        LendRefusal::fault("this computer cannot read which groups it holds")
    })?;
    let stands = |id: &String| crate::door::stands_here(&at.door.rail, id);
    match groups.iter().find(|id| !held.contains(id) || !stands(id)) {
        None => Ok(()),
        Some(unknown) => {
            tracing::debug!(target: "home", group = unknown, "home: refused — an audience names a group not held here, or one this computer does not stand in");
            Err(LendRefusal::Closed.say(
                StatusCode::BAD_REQUEST,
                format!(
                    "{unknown} is not a group you are in on this computer: choose one of the groups shown."
                ),
            ))
        }
    }
}

/// `POST /lent` with `to: "sign-in"`: a client id and secret for `name`,
/// kept in its declaration at 0600, answered with the issuer and the
/// deviations once (SIGN_IN_WITH_SVRNGS §5 "Turning it on", §8 9). The
/// page shows them the once; `GET /lent` says only that it signs people in.
async fn sign_in(at: &Asking, name: &LendName) -> Response {
    let dir = &at.door.data_dir;
    let client = match crate::oidc::client() {
        Ok(client) => client,
        Err(e) => {
            tracing::warn!(target: "home", lend = %name, error = %e, "home: refused — no sign-in client can be made");
            return LendRefusal::fault("this computer cannot make a sign-in for it");
        }
    };
    // Read, changed and written under the lend's lock, as every change is
    // (`lend::mutate`), so it never writes back an audience read stale.
    let mut had = false;
    let done = lend::mutate(dir, name, |lend| {
        had = lend.sign_in_with_svrngs.is_some();
        lend.sign_in_with_svrngs
            .get_or_insert_with(|| client.clone());
    });
    match done {
        Ok(true) if had => {
            tracing::debug!(target: "home", lend = %name, "home: refused — it signs people in with svrngs already");
            return LendRefusal::Closed.say(
                StatusCode::CONFLICT,
                format!("{name} already signs people in with svrngs: make a new secret with `svrngs share <port> --sign-in-with-svrngs`."),
            );
        }
        Ok(true) => {}
        unchanged => return unchanged_said(name, unchanged),
    }
    let issuer = crate::oidc::issuer(at.host.api_port, name.as_str());
    tracing::debug!(target: "home", lend = %name, %issuer, "home: it signs people in with svrngs now");
    Json(serde_json::json!({
        "issuer": issuer,
        "client_id": client.client_id,
        "client_secret": client.client_secret,
        "deviations": crate::oidc::DEVIATIONS,
    }))
    .into_response()
}

/// `POST /lent` with `to: {"readers": ...}`: the account apps reading `name`
/// are let in as, signed in to once as `svrngs share --readers` does, and
/// what it gives kept ([`lend::keep_readers`]); the password nowhere.
async fn readers(at: &Asking, name: &LendName, account: &str, password: &str) -> Response {
    let dir = &at.door.data_dir;
    let found = lend::read(dir, name);
    let Ok(Some(lent)) = found else {
        return unchanged_said(name, found.map(|_| false));
    };
    let (Port(port), Some(how)) = (lent.source, share::readers_of(&lent.kind)) else {
        tracing::debug!(target: "home", lend = %name, "home: refused — no account can be given for this program");
        return LendRefusal::closed(format!("svrngs doesn’t know how to sign in to {name}."));
    };
    let line = match share::sign_in_readers(port, how, account, password).await {
        Ok(line) => line,
        Err(why) => {
            tracing::debug!(target: "home", lend = %name, why, "home: refused — the account was not let in");
            return LendRefusal::closed(format!("{name} didn’t let that account in: {why}."));
        }
    };
    tracing::debug!(target: "home", lend = %name, "home: the apps' account signed in once, what it gave kept");
    match lend::keep_readers(dir, name, (lend::SECRET_HEADER, &line)) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => unchanged_said(name, Err(e)),
    }
}

/// A port as the page shows it ([`share::listening`]): the kind the table
/// knows, how that kind signs each person in, the paths svrngs keeps back
/// of it, and its name once lent.
fn shown(port: NonZeroU16, kind: Option<&Program>, name: Option<&LendName>) -> serde_json::Value {
    let (signs, keeps) = (kind.and_then(share::accounts_of), kind.map(share::kept_of));
    let unknown = kind.and_then(share::unrecognized);
    serde_json::json!({ "port": port, "kind": kind, "name": name, "signs": signs, "keeps": keeps, "unknown": unknown })
}

/// Which read failed, said, and traced with its error (#160).
fn unread(why: &str, error: &dyn std::fmt::Display) -> Response {
    tracing::warn!(target: "home", why, %error, "home: refused — what runs here not offered, nor lent");
    LendRefusal::fault(why)
}

/// `GET /running`: what answers on this computer, less svrngs's own
/// listener and what it lends already, each [`shown`]; and the kinds the
/// table knows by their name alone, for the person to say.
async fn running(State(at): State<Asking>) -> Response {
    let found = tokio::task::spawn_blocking(share::listening).await;
    let (all, found) = match (lend::all(&at.door.data_dir), found) {
        (Ok(all), Ok(found)) => (all, found),
        (Err(e), _) => return unread("it cannot read what you lend already", &e),
        (_, Err(e)) => return unread("the look stopped before it finished", &e),
    };
    let lent: Vec<_> = all.into_iter().flatten().collect();
    let taken =
        |p: &NonZeroU16| p.get() == at.host.api_port || lent.iter().any(|l| l.source == Port(*p));
    let rows: Vec<_> = (found.into_iter())
        .filter(|p| !taken(p))
        .map(|p| shown(p, share::well_known_kind(p).as_ref(), None))
        .collect();
    let named: Vec<Program> = (share::rows().filter(|(at, ..)| *at == share::At::Named))
        .map(|(_, kind, _)| kind)
        .collect();
    tracing::debug!(target: "home", offered = rows.len(), lent = lent.len(), "home: what runs here, offered to lend");
    Json(serde_json::json!({ "running": rows, "named": named })).into_response()
}

/// What the person lends from the page: a port, and for one the table
/// does not know, its name and what it is in their words, each parsed
/// apart so each is refused in its own (#161).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ToLend {
    port: NonZeroU16,
    name: Option<String>,
    kind: Option<String>,
    /// The groups to lend it to, required (SCOPE_PARALLEL.md): an absent
    /// field is `None` and is refused, never defaulted to all groups; an
    /// explicit empty list is paused. `Option` distinguishes absent from
    /// paused, which a plain `Vec` could not.
    groups: Option<Vec<String>>,
}

/// `POST /running`: the lend decided as `svrngs share` decides one
/// ([`share::lend_of`]: the table's kind and kept paths), lent only with a
/// name and what it is unless the table knows its port, never in place of
/// a lend of its name, then declared ([`lend::declare`]) and [`shown`].
async fn lend_one(
    State(at): State<Asking>,
    asked: Result<Json<ToLend>, JsonRejection>,
) -> Result<Response, Response> {
    // Each refusal in the person's words, traced with what refused it, the
    // two closed sets met here each said apart (#161).
    let refused = |why: &str, error: &dyn std::fmt::Display| {
        tracing::debug!(target: "home", why, %error, "home: refused — not lent from the page");
        LendRefusal::Closed.say(StatusCode::BAD_REQUEST, why)
    };
    // What the page sent: a port that is none, 0 or past 65535, else a
    // body the page never sends.
    let Json(ToLend { port, name, kind, groups }) = asked.map_err(|e| match e {
        JsonRejection::JsonDataError(_) => refused("That is not something running on this computer: look again, then lend it from the list.", &e),
        _ => refused("This computer could not read what the page sent: reload the page, then try again.", &e),
    })?;
    let one = "one word: letters and digits, with - or _ between.";
    let name = name.as_deref().map(LendName::parse).transpose();
    let name = name.map_err(|e| refused(&format!("Give it a name of {one}"), &e))?;
    let kind = kind.as_deref().map(Program::parse).transpose();
    let kind = kind.map_err(|e| refused(&format!("Say which program it is in {one}"), &e))?;
    let (known, dir) = (share::well_known_kind(port), &at.door.data_dir);
    tracing::debug!(target: "home", port, ?known, ?name, ?kind, "home: asked to lend what runs here");
    let unknown =
        "Give it a name and say which program it is first: svrngs does not know what runs there.";
    if known.is_none() && (name.is_none() || kind.is_none()) {
        return Err(refused(unknown, &port));
    }
    // The audience is required and explicit (SCOPE_PARALLEL.md): an absent
    // selection is refused, never defaulted to all groups; every ID is
    // checked against the groups this computer holds. A present empty list
    // is a deliberate pause.
    let Some(groups) = groups else {
        return Err(refused(
            "Choose which groups can open this first.",
            &"no selection",
        ));
    };
    chosen(&at, &groups)?;
    let asked = Share {
        port,
        name,
        kind,
        limit: None,
        refused: Vec::new(),
        upload: None,
        reads: Vec::new(),
        asks: None,
        sign_in: false,
        readers: false,
        raw: None,
        audience: Audience::Selected(groups),
    };
    // Each of the lend's refusals in a person's words: a kind is known or
    // given above, so `NoKind` is not met here.
    let silent = format!("Nothing answers at http://localhost:{port} now: start it, then lend it.");
    let lend = share::lend_of(&asked, share::listens, known).map_err(|e| match e {
        ShareRefusal::NothingListens(_) => refused(&silent, &e),
        ShareRefusal::NoKind(_) => refused(unknown, &e),
    })?;
    let stands = lend::read(dir, &lend.name).map(|had| had.is_some());
    tracing::debug!(target: "home", lend = %lend.name, ?stands, "home: lent from the page unless a lend of its name stands");
    let declared = stands.map(|stands| (!stands).then(|| lend::declare(dir, &lend, None)));
    let name = &lend.name;
    Ok(match declared {
        Ok(Some(Ok(()))) => Json(shown(port, Some(&lend.kind), Some(name))).into_response(),
        Ok(None) => refused(
            &format!("You already lend something called {name}: give it another name."),
            name,
        ),
        Err(e) | Ok(Some(Err(e))) => LendRefusal::fault(e.to_string()),
    })
}

/// What the person asks a link for: to invite someone to `group`, or,
/// with `device`, to add a computer or phone of their own, or, with
/// `visit`, to have a guest for that many seconds (guests-link).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Linking {
    group: String,
    device: bool,
    #[serde(default)]
    visit: Option<i64>,
    /// What `link --relay` asks ([`crate::node::Host::meet`]): none on the
    /// page, which keeps where a group meets as it is.
    #[serde(default)]
    relays: Vec<crate::iroh::RelayUrl>,
}

/// Where the command asks a link of the svrngs running here, as it asks a
/// join ([`crate::join::JOINS_ROUTE`]): [`link`], behind the record's checks.
pub const LINKS_ROUTE: &str = "/v1/links";

/// `POST /link`: a link to the group, as `svrngs link` makes it
/// ([`crate::node::Host::link`]), its home relay waited for as there
/// ([`crate::node::Host::online`]); a guest's, `<page>#visit=<token>`, so
/// the page tells it from a join link before it asks anything.
pub(crate) async fn link(State(at): State<Asking>, Json(l): Json<Linking>) -> Response {
    if let Err(why) = at.host.meet(&l.group, &l.relays) {
        tracing::debug!(target: "home", group = l.group, why, "home: refused — where the group meets stays");
        return LendRefusal::closed(why);
    }
    // Its home first, as `svrngs link` waits (cli.rs `link_to`): with none
    // yet, a link names a relay this computer may never register at.
    let online = at.host.online().await;
    tracing::debug!(target: "home", group = l.group, online, "home: a link, once this computer's home relay is reached or waited for");
    match at.host.link(&l.group, l.device, None, l.visit) {
        Ok(made) => {
            let https = made.page.map(|p| https_link(&p, &made.token));
            let https = https.map(|h| {
                if l.visit.is_some() {
                    h.replacen('#', "#visit=", 1)
                } else {
                    h
                }
            });
            let (client, device) = (client_link(&made.token), made.device);
            let made = serde_json::json!({ "client": client, "https": https, "device": device });
            Json(made).into_response()
        }
        Err(crate::invite::Unminted::NotIn) => {
            tracing::debug!(target: "home", group = l.group, "home: refused — no link, this computer is not in that group");
            LendRefusal::closed("you are not in that group on this computer")
        }
        Err(e) => {
            tracing::warn!(target: "home", group = l.group, error = %e, "home: refused — no link made");
            LendRefusal::fault(format!("this computer cannot make a link: {e}"))
        }
    }
}

/// `POST /visits`, `[group, visit]`: the host's Stop (guests-pages; GUESTS
/// §5), a `Correct` of a visit this computer hosts, written as an app's act
/// is ([`crate::rail::append_act`]), so the fold ends it on every copy;
/// any other id refused, so a Stop voids nothing else.
async fn stop(State(at): State<Asking>, Json([group, visit]): Json<[String; 2]>) -> Response {
    let hosts = (at.host.rail.register(&group)).map(|r| {
        (r.visits.iter()).any(|(id, g)| id.to_string() == visit && g.host == at.host.actor)
    });
    tracing::debug!(target: "home", group, visit, hosts = ?hosts.as_ref().ok(), "home: a visit stopped only by the computer that hosts it");
    match hosts {
        Ok(true) => {
            let act =
                serde_json::json!({ "op": "correct", "corrects": visit, "replacement": null });
            crate::rail::append_act(&at.host.rail, &group, act, None).await
        }
        Ok(false) => LendRefusal::closed("you host no such visit here"),
        Err(e) => LendRefusal::fault(e.to_string()),
    }
}

/// `POST /group`, `[name, person]`: a group made, as `svrngs new` makes
/// one ([`Asking::found`]): its name, why its folder was not placed, and
/// the groups called so already.
async fn found(State(at): State<Asking>, Json([name, person]): Json<[String; 2]>) -> Response {
    let made = at.found(&name, &person);
    tracing::debug!(target: "home", ?made, "home: a group made, or why not");
    match made {
        Ok(((_, placed), seed, beside)) => {
            Json((seed.name().as_str(), placed.err(), beside)).into_response()
        }
        Err(why) => LendRefusal::Closed.say(StatusCode::CONFLICT, why),
    }
}

/// A link someone sent, and the name the person goes by in its group.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Joining {
    link: String,
    #[serde(default)]
    person: String,
}

/// `link`, or the page's refusal of what is not a whole one.
fn link_of(link: &str) -> Result<Link, Response> {
    Link::parse(link).ok_or_else(|| {
        tracing::debug!(target: "home", "home: refused — not a whole link");
        LendRefusal::Closed.say(
            StatusCode::BAD_REQUEST,
            "That is not a whole link: paste all of it, to its end.",
        )
    })
}

/// Why a link let nobody in, in the page's words ([`NotJoined`]).
fn not_joined(why: NotJoined) -> Response {
    LendRefusal::Closed.say(StatusCode::CONFLICT, why.to_string())
}

/// `GET /join?link=`: what a link someone sent is for, asked of the
/// computer that sent it ([`Host::asked`]), for the page to say on its
/// button before it joins: the group's name, and the person a link to add
/// a computer of theirs joins as, else none.
async fn asked(State(at): State<Asking>, Query(j): Query<Joining>) -> Result<Response, Response> {
    let asked = at.host.asked(&link_of(&j.link)?).await;
    let (asked, seed) = asked.map_err(not_joined)?;
    tracing::debug!(target: "home", group = %seed.id(), person = ?asked.person, "home: what a link is for, asked");
    Ok(Json((seed.name().as_str(), asked.person)).into_response())
}

/// `POST /join`: the group a link is for joined by the svrngs running
/// here, as `svrngs join` joins ([`Host::join`], arch-command (c)), and
/// its folder placed as a joiner's: its name, whom this computer joined
/// as, none when it was in already, and why its folder was not placed.
async fn join(State(at): State<Asking>, Json(j): Json<Joining>) -> Result<Response, Response> {
    let name = crate::group::Name::parse(&j.person).map_err(|why| {
        tracing::debug!(target: "home", %why, "home: refused — no name to join as");
        LendRefusal::Closed.say(StatusCode::BAD_REQUEST, why.to_string())
    })?;
    let entered = at.host.join(&link_of(&j.link)?, &name).await;
    tracing::debug!(target: "home", ?entered, "home: a group joined by its link, or why not");
    let (seed, person) = match entered.map_err(not_joined)? {
        Entered::In { seed, person } => (seed, Some(person.to_string())),
        Entered::AlreadyIn(seed) => (seed, None),
    };
    let (_, placed) = at.door.folders.place(&seed);
    Ok(Json((seed.name().as_str(), person, placed.err())).into_response())
}

#[derive(Deserialize)]
struct Naming {
    name: String,
}

/// `GET /group?name=`: the groups called so already, for the page to say
/// before a group is made ([`crate::group::namesakes`]); a name that is
/// not one refused in its words.
async fn namesakes(State(at): State<Asking>, Query(n): Query<Naming>) -> Response {
    let root = at.host.rail.root();
    match crate::group::Name::parse(&n.name) {
        Ok(name) => Json(crate::group::namesakes(root, &name, &at.host.actor)).into_response(),
        Err(why) => LendRefusal::Closed.say(StatusCode::BAD_REQUEST, why.to_string()),
    }
}

/// The programs that open an address or a folder: macOS's, or the freedesktop
/// one anywhere else, whose `open` can be something else (`openvt`).
const OPENERS: [&str; 2] = ["open", "xdg-open"];

/// How long an opener has to refuse before it is taken to have opened.
const OPENER_ANSWERS: Duration = Duration::from_secs(1);

/// Whether an opener took `what`. `xdg-open` stays until the browser or file
/// manager it started closes, so one still running has taken it: it is never
/// waited for, though a thread reaps it. Its output goes nowhere, so nothing it starts holds a pipe its
/// caller reads to the end, as install.sh's `$(svrngs open)` does.
pub fn opened(what: &OsStr) -> Option<&'static str> {
    let opener = OPENERS[usize::from(!cfg!(target_os = "macos"))];
    let mut opening = Command::new(opener);
    opening
        .arg(what)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let answer = opening.spawn().and_then(|mut opening| {
        std::thread::sleep(OPENER_ANSWERS);
        let answer = opening.try_wait();
        std::thread::spawn(move || opening.wait());
        answer
    });
    let took = answer.is_ok_and(|done| done.is_none_or(|said| said.success()));
    took.then_some(opener)
}

/// `POST /folder`: the folder `dir` opened, asked as `/v1/folders` is asked
/// to keep it, and only one `/v1/groups` lists for the group
/// ([`crate::folder::dirs_of`]), so the page opens nothing else.
async fn open(State(at): State<Asking>, Json(o): Json<crate::folder::FolderAsk>) -> Response {
    let listed =
        crate::folder::dirs_of(&at.door.data_dir, &o.group).is_ok_and(|d| d.contains(&o.dir));
    let by = listed.then(|| opened(o.dir.as_os_str())).flatten();
    tracing::debug!(target: "home", group = o.group, dir = %o.dir.display(), listed, ?by, "home: a folder asked opened");
    match (listed, by) {
        (true, Some(_)) => StatusCode::NO_CONTENT.into_response(),
        (false, _) => LendRefusal::closed("that is not this group's folder"),
        _ => LendRefusal::fault("no program opens it"),
    }
}

#[derive(Deserialize)]
struct Of {
    lend: Label,
}

/// `GET /choices?lend=`: the person's choices on a lend, asked of its door now.
async fn choices(State(at): State<Asking>, Query(Of { lend }): Query<Of>) -> Response {
    if let Ok(crate::origin::Serving::Own) = crate::origin::serving(&at, &lend).await {
        return own_choices(&at.door, &lend);
    }
    at_door(&at, &lend, None, None).await
}

/// `said` of `app` on `lend`, kept by this computer's door for its own
/// lend, else carried to the lender's: as the person, or with `pass`, the
/// one this computer signs for an app's page stopping itself at its origin
/// (`/svrngs-reads/<label>/svrngs-stop`, [`crate::origin`]).
pub(crate) async fn say(
    at: &Asking,
    lend: &Label,
    app: Label,
    said: Said,
    pass: Option<HeaderValue>,
) -> Response {
    tracing::debug!(target: "home", %lend, %app, ?said, by_app = pass.is_some(), "home: a choice said");
    if let Ok(crate::origin::Serving::Own) = crate::origin::serving(at, lend).await {
        return own_choose(&at.door, lend, app, said).await;
    }
    let body = match pass {
        Some(_) => serde_json::json!({ "said": said }),
        None => serde_json::json!(Saying { app, said }),
    };
    at_door(at, lend, Some(body), pass).await
}

/// `/choices/<lend>` at the lender's door, dialled as this computer's key
/// ([`crate::origin::serving`]; its own lend is not dialled): posting
/// `body`, else asking; with `pass` for an app's stop. Its answer, svrngs's
/// mark kept.
async fn at_door(
    at: &Asking,
    lend: &Label,
    body: Option<serde_json::Value>,
    pass: Option<HeaderValue>,
) -> Response {
    let contact = match crate::origin::serving(at, lend).await {
        Ok(crate::origin::Serving::At(contact, _)) => contact,
        Ok(crate::origin::Serving::Own) => {
            return LendRefusal::fault("this computer's own lend is not dialled")
        }
        Err(refused) => return refused,
    };
    let method = body.as_ref().map_or(Method::GET, |_| Method::POST);
    let ask = pass
        .into_iter()
        .fold(axum::http::Request::builder(), |a, p| {
            a.header(PASS_HEADER, p)
        });
    let ask = ask.method(method).uri(format!("/choices/{lend}"));
    let ask = ask.header(header::CONTENT_TYPE, "application/json");
    let ask = ask.body(body.map_or_else(Default::default, |b| b.to_string().into()));
    let answer = match ask {
        Ok(ask) => (at.host.transport.fetch(&contact, ask).await).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    };
    let answer = match answer {
        Ok(answer) => answer,
        Err(error) => {
            tracing::debug!(target: "home", %lend, error, "home: the lender's door did not answer");
            return crate::origin::away();
        }
    };
    let (head, bytes) = answer.into_parts();
    tracing::debug!(target: "home", %lend, status = head.status.as_u16(), refused = ?head.headers.get(REFUSED_HEADER), "home: the lender's door answered");
    let mut said = (head.status, bytes).into_response();
    for (name, value) in head
        .headers
        .iter()
        .filter(|(n, _)| [header::CONTENT_TYPE.as_str(), REFUSED_HEADER].contains(&n.as_str()))
    {
        said.headers_mut().insert(name, value.clone());
    }
    said
}
