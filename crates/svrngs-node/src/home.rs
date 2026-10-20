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
//! link` makes it, in the group's record; **a group's folder opened**
//! in this computer's file browser (`POST /folder`, start-home); and **a
//! group made** by the svrngs running here, as `svrngs new` makes one
//! (`POST /group`, start-group; the node's group.rs `Asking::found`);
//! **a group joined** from a link someone sent, as `svrngs join` joins
//! (`POST /join`, once `GET /join?link=` has said what the link is for;
//! arch-command (c), the node's join.rs); **the shelf's apps brought**
//! (`POST /shelf`) and **the person's yes to a carried app's new version**
//! (`POST /runs`; canon-pin (c)), each in the group's folder or record; and
//! **what runs on this computer lent** (`POST /running`, v1ui-lend (b)),
//! decided as `svrngs share` decides a lend ([`crate::share`]) from what
//! `GET /running` offers, every port that answers here and is not lent,
//! nor signs people in to a lend ([`share::signs_in`], said once why).
//! The person's yes and stop are
//! kept by this computer's door for its own lend, else dialled to the
//! lender's door on the members' protocol as this computer's key
//! ([`at_door`]). `GET /choices?lend=` asks that door, when the person
//! opens the list, what apps use. It asks at `/ask?lend=&app=&back=`.

use axum::extract::{rejection::JsonRejection, Query, Request, State};
use axum::http::header::{self, HeaderValue};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use std::num::NonZeroU16;
use std::process::Command;
use svrngs_core::link::{client_link, https_link, Link};
use svrngs_core::pass::Label;

use crate::door::choices::{own_choices, own_choose, Said, Saying};
use crate::door::{LendRefusal, PASS_HEADER, REFUSED_HEADER};
use crate::join::{Entered, NotJoined};
use crate::lend::{self, Kind, LendName, Source::Port};
use crate::node::Asking;
use crate::share::{self, Share, ShareRefusal};

/// The home page's host, before its port.
const HOST: &str = "svrngs.localhost";

const POLICY: &str = "default-src 'self'; frame-ancestors 'none'";

/// When a person joined, as the page says it (home.js `people`) and
/// `svrngs remove --joined` takes it: this computer's time to the second
/// with its offset, so the hour a fall-back night repeats reads back to one
/// instant (ralph/REVIEW_FINDINGS.md #126). chrono's form; the page writes
/// it by hand.
pub const WHEN: &str = "%Y-%m-%d %H:%M:%S %z";

/// The home page's one module.
const MODULE: ([(header::HeaderName, &str); 1], &str) = (
    [(header::CONTENT_TYPE, "text/javascript")],
    include_str!("home/home.js"),
);

/// svrngs's question and what apps use, in the one wording the group's page
/// loads too (`scripts/build-web.sh` gives it the same file).
const QUESTION: ([(header::HeaderName, &str); 1], &str) = (
    [(header::CONTENT_TYPE, "text/javascript")],
    include_str!("home/question.js"),
);

/// Its look, svrngs's one look, a file of its own since its policy runs no
/// inline style; the group's page is given the same file (`scripts/build-web.sh`).
const LOOK: ([(header::HeaderName, &str); 1], &str) = (
    [(header::CONTENT_TYPE, "text/css")],
    include_str!("home/look.css"),
);

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

/// Where the page of the lend `app` sends its person to say yes to `lend`.
pub(crate) fn ask_address(port: u16, lend: &Label, app: &Label) -> String {
    format!("{}ask?lend={lend}&app={app}", address(port))
}

/// The home page's routes, beside the lends' origins: a request naming it
/// is answered here, any other at its lend's origin.
pub(crate) fn router(at: Asking, rail: crate::rail::RailApi) -> Router {
    let page = || async { axum::response::Html(include_str!("home/index.html")) };
    let home = Router::new()
        .route("/", get(page))
        .route("/ask", get(page))
        .route("/home.js", get(|| async { MODULE }))
        .route("/question.js", get(|| async { QUESTION }))
        .route("/look.css", get(|| async { LOOK }))
        .route("/choices", get(choices).post(choose))
        .route("/lent", get(lent).post(change))
        .route("/running", get(running).post(lend_one))
        .route("/helper/{kind}", get(steps))
        .route("/link", axum::routing::post(link))
        .route("/folder", axum::routing::post(open))
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
        .layer(axum::middleware::from_fn_with_state(
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
async fn only_itself(State(at): State<Asking>, req: Request, next: Next) -> Response {
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
    tracing::debug!(target: "home", path = %req.uri().path(), method = %req.method(), at_own, ?site, ?mode, ?origin, answers,
        "home: answered only from itself, a typed address or, to read, a navigation");
    let mut answer = if answers {
        next.run(req).await
    } else {
        LendRefusal::Closed.say(StatusCode::FORBIDDEN, "this page answers only itself")
    };
    answer.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(POLICY),
    );
    answer
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

/// What the person changes of a lend this computer lends.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Changing {
    lend: LendName,
    to: LendChange,
}

/// A closed set (AGENTS 9): let in this many at once, none for everyone in
/// its groups; or lend it no longer.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum LendChange {
    Limit(Option<NonZeroU16>),
    Stop,
}

/// `GET /lent`: what this computer lends, each with how many it lets in at
/// once and how many use it now, as a member asking is told it.
async fn lent(State(at): State<Asking>) -> Response {
    match crate::door::offered(&at.door).await {
        Ok(lent) => Json(serde_json::json!({ "lent": lent })).into_response(),
        Err(refused) => refused,
    }
}

/// `POST /lent`: the person's change, written to the lend's declaration.
async fn change(State(at): State<Asking>, Json(c): Json<Changing>) -> Response {
    let dir = &at.door.data_dir;
    let done = match c.to {
        LendChange::Limit(limit) => lend::limit(dir, &c.lend, limit),
        LendChange::Stop => lend::withdraw(dir, &c.lend),
    };
    tracing::debug!(target: "home", lend = %c.lend, to = ?c.to, done = ?done.as_ref().ok(), "home: the person changed a lend");
    match done {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => {
            LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "you lend nothing by that name")
        }
        Err(e) => {
            tracing::warn!(target: "home", lend = %c.lend, error = %e, "home: refused — the lend's declaration cannot be changed");
            LendRefusal::Fault.say(
                StatusCode::INTERNAL_SERVER_ERROR,
                "that cannot be changed on this computer",
            )
        }
    }
}

/// A port as the page shows it ([`share::listening`]): the kind the table
/// knows, how that kind signs each person in and the paths svrngs keeps
/// back of it, whether its helper answers, if it has one, and its name
/// once lent.
fn shown(port: NonZeroU16, kind: Option<&Kind>, name: Option<&LendName>) -> serde_json::Value {
    let (signs, keeps) = (kind.and_then(share::accounts_of), kind.map(share::kept_of));
    let helper = kind.and_then(share::helper_of).map(share::listens); // whether it answers
    serde_json::json!({ "port": port, "kind": kind, "name": name, "signs": signs, "keeps": keeps, "helper": helper })
}

/// `GET /helper/<kind>`: the steps to start the kind's sign-in helper, as
/// the table names them ([`share::HELPERS`]), for the page to link.
async fn steps(axum::extract::Path(kind): axum::extract::Path<String>) -> Response {
    let none = || LendRefusal::Unlent.say(StatusCode::NOT_FOUND, "svrngs knows no helper for that");
    let steps = share::HELPERS.iter().find(|(k, _)| *k == kind);
    tracing::debug!(target: "home", kind, found = steps.is_some(), "home: a kind's helper's steps asked, refused where svrngs knows none");
    steps.map_or_else(none, |(_, s)| s.into_response())
}

/// Which read failed, said, and traced with its error (#160).
fn unread(why: &str, error: &dyn std::fmt::Display) -> Response {
    tracing::warn!(target: "home", why, %error, "home: refused — what runs here not offered, nor lent");
    LendRefusal::Fault.say(StatusCode::INTERNAL_SERVER_ERROR, why)
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
    // A sign-in is left out, the page saying once why (arch-helper (b)).
    let (signing, found): (Vec<_>, Vec<_>) =
        (found.into_iter().filter(|p| !taken(p))).partition(|p| share::signs_in(*p, &lent));
    let rows: Vec<_> = (found.into_iter())
        .map(|p| shown(p, share::well_known_kind(p).as_ref(), None))
        .collect();
    let named: Vec<Kind> = (share::rows().filter(|(at, ..)| *at == share::At::Named))
        .map(|(_, kind, _)| kind)
        .collect();
    tracing::debug!(target: "home", offered = rows.len(), lent = lent.len(), ?signing, "home: what runs here, offered to lend, sign-ins left out");
    Json(serde_json::json!({ "running": rows, "named": named, "signing": signing })).into_response()
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
    let Json(ToLend { port, name, kind }) = asked.map_err(|e| match e {
        JsonRejection::JsonDataError(_) => refused("That is not something running on this computer: look again, then lend it from the list.", &e),
        _ => refused("This computer could not read what the page sent: reload the page, then try again.", &e),
    })?;
    let one = "one word: letters and digits, with - or _ between.";
    let name = name.as_deref().map(LendName::parse).transpose();
    let name = name.map_err(|e| refused(&format!("Give it a name of {one}"), &e))?;
    let kind = kind.as_deref().map(Kind::parse).transpose();
    let kind = kind.map_err(|e| refused(&format!("Say what it is in {one}"), &e))?;
    let (known, dir) = (share::well_known_kind(port), &at.door.data_dir);
    let lent = lend::all(dir).map_err(|e| unread("it cannot read what you lend already", &e))?;
    let lent: Vec<_> = lent.into_iter().flatten().collect();
    tracing::debug!(target: "home", port, ?known, ?name, ?kind, "home: asked to lend what runs here");
    let unknown = "Give it a name and say what it is first: svrngs does not know what runs there.";
    if known.is_none() && (name.is_none() || kind.is_none()) {
        return Err(refused(unknown, &port));
    }
    let asked = Share {
        port,
        name,
        kind,
        limit: None,
        refused: Vec::new(),
        upload: None,
        reads: Vec::new(),
        sign_in: None,
        asks: None,
    };
    // Each of the lend's refusals in a person's words: a kind is known or
    // given above, and no sign-in asked, so `NoKind` and `NoSignIn` are not met here.
    let silent = format!("Nothing answers at http://localhost:{port} now: start it, then lend it.");
    let no_sign_in = "Nothing answers at its sign-in now: start it, then lend it.";
    let signs = "It signs people in to something you lend, and only svrngs on this computer asks it: lend what it signs them in to instead.";
    let lend = share::lend_of(&asked, share::listens, known, &lent).map_err(|e| match e {
        ShareRefusal::SignsIn(_) => refused(signs, &e),
        ShareRefusal::NothingListens(_) => refused(&silent, &e),
        ShareRefusal::NoKind(_) => refused(unknown, &e),
        ShareRefusal::NoSignIn(_) => refused(no_sign_in, &e),
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
        Err(e) | Ok(Some(Err(e))) => {
            LendRefusal::Fault.say(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })
}

/// What the person asks a link for: to invite someone to `group`, or,
/// with `device`, to add a computer or phone of their own, or, with
/// `visit`, to have a guest for that many seconds (guests-link).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Linking {
    group: String,
    device: bool,
    #[serde(default)]
    visit: Option<i64>,
}

/// `POST /link`: a link to the group, as `svrngs link` makes it
/// ([`crate::node::Host::link`]); a guest's, `<page>#visit=<token>`, so
/// the page tells it from a join link before it asks anything.
async fn link(State(at): State<Asking>, Json(l): Json<Linking>) -> Response {
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
            Json(serde_json::json!({ "client": client_link(&made.token), "https": https }))
                .into_response()
        }
        Err(crate::invite::Unminted::NotIn) => {
            tracing::debug!(target: "home", group = l.group, "home: refused — no link, this computer is not in that group");
            LendRefusal::Closed.say(
                StatusCode::FORBIDDEN,
                "you are not in that group on this computer",
            )
        }
        Err(e) => {
            tracing::warn!(target: "home", group = l.group, error = %e, "home: refused — no link made");
            LendRefusal::Fault.say(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("this computer cannot make a link: {e}"),
            )
        }
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
    let (_, placed) = at.folders.place(&seed);
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

/// The programs that open a folder in its file browser, in turn: macOS's,
/// then the freedesktop one Linux desktops answer.
const OPENERS: [&str; 2] = ["open", "xdg-open"];

/// `POST /folder`: the folder `dir` opened, asked as `/v1/folders` is asked
/// to keep it, and only one `/v1/groups` lists for the group
/// ([`crate::folder::dirs_of`]), so the page opens nothing else.
async fn open(State(at): State<Asking>, Json(o): Json<crate::folder::FolderAsk>) -> Response {
    let listed =
        crate::folder::dirs_of(&at.door.data_dir, &o.group).is_ok_and(|d| d.contains(&o.dir));
    let opens = |p: &&str| matches!(Command::new(p).arg(&o.dir).status(), Ok(s) if s.success());
    let by = OPENERS.iter().find(|p| listed && opens(p));
    tracing::debug!(target: "home", group = o.group, dir = %o.dir.display(), listed, ?by, "home: a folder asked opened");
    match (listed, by) {
        (true, Some(_)) => StatusCode::NO_CONTENT.into_response(),
        (false, _) => {
            LendRefusal::Closed.say(StatusCode::FORBIDDEN, "that is not this group's folder")
        }
        _ => LendRefusal::Fault.say(StatusCode::INTERNAL_SERVER_ERROR, "no program opens it"),
    }
}

#[derive(Deserialize)]
struct Of {
    lend: Label,
}

/// `GET /choices?lend=`: the person's choices on a lend, asked of its door now.
async fn choices(State(at): State<Asking>, Query(Of { lend }): Query<Of>) -> Response {
    if lend.lender() == at.door.key {
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
    if lend.lender() == at.door.key {
        return own_choose(&at.door, lend, app, said).await;
    }
    let body = match pass {
        Some(_) => serde_json::json!({ "said": said }),
        None => serde_json::json!(Saying { app, said }),
    };
    at_door(at, lend, Some(body), pass).await
}

/// `/choices/<lend>` at the lender's door, dialled as this computer's key
/// ([`crate::origin::dialled`]): posting `body`, else asking; with `pass`
/// for an app's stop. Its answer, svrngs's mark kept.
async fn at_door(
    at: &Asking,
    lend: &Label,
    body: Option<serde_json::Value>,
    pass: Option<HeaderValue>,
) -> Response {
    let contact = match crate::origin::dialled(at, lend) {
        Ok(contact) => contact,
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
    let ask = ask.body(body.map_or_else(axum::body::Body::empty, |b| b.to_string().into()));
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
