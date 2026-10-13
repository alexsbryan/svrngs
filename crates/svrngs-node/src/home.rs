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
//! written to its declaration, which the door reads at its next request.
//! The person's yes and stop are
//! kept by this computer's door for its own lend, else dialled to the
//! lender's door on the members' protocol as this computer's key
//! ([`at_door`]). `GET /choices?lend=` asks that door, when the person
//! opens the list, what apps use. It asks at `/ask?lend=&app=&back=`.

use axum::extract::{Query, Request, State};
use axum::http::header::{self, HeaderValue};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use std::num::NonZeroU16;
use svrngs_core::pass::Label;

use crate::door::choices::{own_choices, own_choose, Said, Saying};
use crate::door::{LendRefusal, PASS_HEADER, REFUSED_HEADER};
use crate::lend::{self, LendName};
use crate::node::Asking;

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

/// Its look, a file of its own, since its policy runs no inline style.
const LOOK: ([(header::HeaderName, &str); 1], &str) = (
    [(header::CONTENT_TYPE, "text/css")],
    include_str!("home/home.css"),
);

/// Whether `host` names the home page, at any port: [`only_itself`] holds
/// it to this listener's.
pub(crate) fn named(host: Option<&HeaderValue>) -> bool {
    let host = host.and_then(|h| h.to_str().ok()).unwrap_or_default();
    host.split(':')
        .next()
        .is_some_and(|h| h.eq_ignore_ascii_case(HOST))
}

/// Where the page of the lend `app` sends its person to say yes to `lend`.
pub(crate) fn ask_address(port: u16, lend: &Label, app: &Label) -> String {
    format!("http://{HOST}:{port}/ask?lend={lend}&app={app}")
}

/// The home page's routes, beside the lends' origins: a request naming it
/// is answered here, any other at its lend's origin.
pub(crate) fn router(
    at: Asking,
    seeds: crate::redeem::SeedText,
    rail: crate::rail::RailApi,
) -> Router {
    let page = || async { axum::response::Html(include_str!("home/index.html")) };
    let home = Router::new()
        .route("/", get(page))
        .route("/ask", get(page))
        .route("/home.js", get(|| async { MODULE }))
        .route("/question.js", get(|| async { QUESTION }))
        .route("/home.css", get(|| async { LOOK }))
        .route("/choices", get(choices).post(choose))
        .route("/lent", get(lent).post(change))
        .with_state(at.clone())
        .merge(
            Router::new()
                .route("/v1/rail/people", get(crate::rail::people))
                .with_state(rail),
        )
        .merge(crate::surface::router(at.clone(), seeds))
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
    match crate::door::offered(at.door()).await {
        Ok(lent) => Json(serde_json::json!({ "lent": lent })).into_response(),
        Err(refused) => refused,
    }
}

/// `POST /lent`: the person's change, written to the lend's declaration.
async fn change(State(at): State<Asking>, Json(c): Json<Changing>) -> Response {
    let dir = &at.door().data_dir;
    let done = match c.to {
        LendChange::Limit(limit) => lend::limit(dir, &c.lend, limit),
        LendChange::Stop => lend::withdraw(dir, &c.lend),
    };
    tracing::debug!(target: "home", lend = %c.lend, to = ?c.to, done = ?done.as_ref().ok(), "home: the person changed a lend");
    match done {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => LendRefusal::Unlent.say(
            StatusCode::NOT_FOUND,
            "this computer lends nothing by that name",
        ),
        Err(e) => {
            tracing::warn!(target: "home", lend = %c.lend, error = %e, "home: refused — the lend's declaration cannot be changed");
            LendRefusal::Fault.say(
                StatusCode::INTERNAL_SERVER_ERROR,
                "that cannot be changed on this computer",
            )
        }
    }
}

#[derive(Deserialize)]
struct Of {
    lend: Label,
}

/// `GET /choices?lend=`: the person's choices on a lend, asked of its door now.
async fn choices(State(at): State<Asking>, Query(Of { lend }): Query<Of>) -> Response {
    if lend.lender() == at.door().key {
        return own_choices(at.door(), &lend);
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
    if lend.lender() == at.door().key {
        return own_choose(at.door(), lend, app, said).await;
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
    let (base, http) = match (
        crate::origin::dialled(at, lend).await,
        crate::ring_sync::peer_client(),
    ) {
        (Ok(base), Ok(http)) => (base, http),
        (Err(refused), _) => return refused,
        (_, Err(_)) => return crate::origin::away(),
    };
    let url = format!("{base}/choices/{lend}");
    let ask = match &body {
        Some(body) => http.post(url).json(body),
        None => http.get(url),
    };
    let Ok(answer) = pass
        .into_iter()
        .fold(ask, |a, p| a.header(PASS_HEADER, p))
        .send()
        .await
    else {
        return crate::origin::away();
    };
    let (status, heads) = (answer.status(), answer.headers().clone());
    tracing::debug!(target: "home", %lend, status = status.as_u16(), refused = ?heads.get(REFUSED_HEADER), "home: the lender's door answered");
    let mut said = (status, answer.bytes().await.unwrap_or_default()).into_response();
    for (name, value) in heads
        .iter()
        .filter(|(n, _)| [header::CONTENT_TYPE.as_str(), REFUSED_HEADER].contains(&n.as_str()))
    {
        said.headers_mut().insert(name, value.clone());
    }
    said
}
