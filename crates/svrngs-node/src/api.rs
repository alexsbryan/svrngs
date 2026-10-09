// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/api.rs@e1a43bcd2
//! The loopback routes a shim author touches, and nothing else.
//!
//! The ring rail's doors, `/v1/rail/{append,log,live,people}`, in [`crate::rail`];
//! beside them (svrngs, u5-1) the groups this computer holds and what their
//! people lend, `/v1/{groups,lends}`, in [`crate::surface`]. The address it
//! is served on is written to [`API_FILE`] in the data dir.
//! (svrngs: the mesh, media, publish, origin, membership, store and ledger
//! doors beside them stay behind; DESIGN §8, U3's inventory.)
//!
//! **Loopback IS the auth.** There is no bearer here, and the bind refuses
//! any address that is not loopback rather than serving an unauthenticated
//! API to a network.

use std::net::SocketAddr;

use axum::extract::{ConnectInfo, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::rail::RailApi;

/// Why the API did not start. (Upstream's were two variants of the daemon's
/// `Refusal`, `lib.rs:122-123`, which did not come over; named here as the
/// crate names its errors, since `Refusal` is the command's, cli.rs.)
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0} is not a loopback address; this API has no auth because loopback is the auth")]
    NotLoopback(SocketAddr),
    #[error("cannot listen on {0}: {1}")]
    Listen(SocketAddr, std::io::Error),
    #[error("cannot write where the API listens to {0}: {1}")]
    AddressFile(std::path::PathBuf, std::io::Error),
}

/// The file in the data dir naming where the API listens, as a URL and a
/// newline (`http://127.0.0.1:<port>`), so an app the node did not start
/// finds it (u5-1; the API binds any free port, `svrngs::cli::API`).
/// Written at each start; a node that has stopped leaves it naming a port
/// that refuses, which is what an app asking it then hears.
pub const API_FILE: &str = "api";

/// Write `addr` to [`API_FILE`] in `data_dir`, through a file renamed into
/// place, so an app never reads half an address.
pub fn write_address(data_dir: &std::path::Path, addr: SocketAddr) -> Result<(), ApiError> {
    let path = data_dir.join(API_FILE);
    let partial = data_dir.join(format!(".{API_FILE}.partial"));
    std::fs::write(&partial, format!("http://{addr}\n"))
        .and_then(|()| std::fs::rename(&partial, &path))
        .map_err(|e| ApiError::AddressFile(path.clone(), e))?;
    tracing::debug!(target: "rails", path = %path.display(), %addr, "api: wrote where it listens, for apps it did not start");
    Ok(())
}

/// The routes this API serves: the rail's, and `apps` — the routes an app
/// asks beside the record ([`crate::surface::router`]) — refused to a page
/// in a browser ([`not_from_a_page`]); and, for a request whose `Host`
/// names a lend, `lends`, the lend's origin ([`crate::origin`], u5-2), which
/// answers no route of the record's. All behind the one loopback check.
pub fn router(api: RailApi, apps: Router, lends: Router) -> Router {
    Router::new()
        // The ring rail's doors (FIVE_PROGRAMS fp-44): durable append and
        // the log, and the live lane's local buffer half. The bodies mirror
        // the daemon's `routes_rail` doors.
        .route("/v1/rail/append", post(crate::ring_sync::append))
        .route("/v1/rail/log", get(crate::rail::log))
        .route(
            "/v1/rail/live",
            post(crate::rail::live_push).get(crate::rail::live_drain),
        )
        // svrngs: the group's people with their keys, from the register
        // (DESIGN §3), so an app needs no second source.
        .route("/v1/rail/people", get(crate::rail::people))
        .with_state(api)
        .merge(apps)
        .layer(axum::middleware::from_fn(not_from_a_page))
        .layer(axum::middleware::from_fn_with_state(lends, to_a_lend))
        // svrngs: every request's caller is asked too, so a listener that
        // ever served beyond loopback would still answer no one else.
        .layer(axum::middleware::from_fn(from_this_computer))
}

/// A request whose `Host` is a lend's, `<label>.localhost`, goes to the
/// lend's origin and never to the record (u5-2); one whose `Host` is the
/// home page's, `svrngs.localhost`, to the home page (sup-2,
/// [`crate::home`]), which `lends` serves beside the origins.
async fn to_a_lend(State(lends): State<Router>, request: Request, next: Next) -> Response {
    use tower::ServiceExt;
    let host = request.headers().get(axum::http::header::HOST);
    if crate::origin::label_of(host).is_none() && !crate::home::named(host) {
        return next.run(request).await;
    }
    match lends.oneshot(request).await {
        Ok(answer) => answer,
        Err(never) => match never {},
    }
}

/// The record is served to this computer's apps, never to a page in a
/// browser (u5-2): a request carrying `Origin` or `Sec-Fetch-Site`, which
/// a browser sends and an app's own HTTP client does not, is refused 403
/// before any door runs. So a lent page, at its origin on this same
/// listener, cannot write the record, nor can any site this computer's
/// browser opens: loopback alone let both in, since a browser on this
/// computer is a caller on loopback.
async fn not_from_a_page(request: Request, next: Next) -> Response {
    let headers = request.headers();
    let Some(page) = headers
        .get(axum::http::header::ORIGIN)
        .or_else(|| headers.get("sec-fetch-site"))
    else {
        return next.run(request).await;
    };
    tracing::warn!(target: "rails", page = ?page, path = %request.uri().path(), "api: refused — a page in a browser asked the record");
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({
            "error": "the record is served to this computer's apps, not to a page in a browser",
            "kind": "from_a_page",
        })),
    )
        .into_response()
}

/// Serve a request only from a caller on this computer: the bind guard's
/// one decision ([`check_loopback`]) asked of the caller's address, which
/// [`serve`] hands every request. A caller elsewhere, or one whose address
/// is unknown, is refused 403 naming why, before any door runs. (svrngs:
/// upstream relied on the bind alone; DESIGN §8, "Localhost is the trust
/// boundary".)
async fn from_this_computer(request: Request, next: Next) -> Response {
    let caller = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0);
    let refused = |why: String| {
        (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": why, "kind": "not_loopback" })),
        )
            .into_response()
    };
    match caller {
        Some(addr) => match check_loopback(addr) {
            Ok(()) => {
                tracing::debug!(target: "rails", caller = %addr, path = %request.uri().path(), "api: a caller on this computer");
                next.run(request).await
            }
            Err(refusal) => {
                tracing::warn!(target: "rails", caller = %addr, path = %request.uri().path(), "api: refused — the caller is not on this computer");
                refused(refusal.to_string())
            }
        },
        None => {
            tracing::warn!(target: "rails", path = %request.uri().path(), "api: refused — the caller's address is unknown, so it cannot be shown to be on this computer");
            refused("the caller's address is unknown; the record is served to this computer's apps alone".into())
        }
    }
}

/// The bind guard, as a function so it has a failing input that can be
/// named without standing an iroh endpoint up. This API has no auth because
/// loopback IS the auth; anywhere else is publishing it.
pub fn check_loopback(listen: SocketAddr) -> Result<(), ApiError> {
    // svrngs: canonical first, so `::ffff:127.0.0.1` reads as the loopback
    // it is, where `Ipv6Addr::is_loopback` alone says no.
    if listen.ip().to_canonical().is_loopback() {
        Ok(())
    } else {
        Err(ApiError::NotLoopback(listen))
    }
}

/// The API being served, on the address it is bound to. Dropping it stops
/// the listener. (svrngs: upstream's `serve` answered the task alone, for a
/// port its config named; the node binds whatever port the command asks,
/// `0` among them, so the address bound is answered too.)
pub struct Serving {
    addr: SocketAddr,
    task: tokio::task::JoinHandle<()>,
}

impl Serving {
    /// Where an app on this computer reaches the API.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Drop for Serving {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// A listener bound on loopback, not yet served. (svrngs: upstream's
/// `serve` bound and served in one call; the node binds before its
/// endpoint, so a refused address leaves no endpoint open, and serves once
/// the endpoint's transport exists, which the live lane dials through.)
pub struct Bound {
    listener: tokio::net::TcpListener,
    addr: SocketAddr,
}

impl Bound {
    /// Where it listens.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

/// Bind. Refuses a non-loopback address by name.
pub async fn bind(listen: SocketAddr) -> Result<Bound, ApiError> {
    check_loopback(listen)?;
    let listener = tokio::net::TcpListener::bind(listen)
        .await
        .map_err(|e| ApiError::Listen(listen, e))?;
    let addr = listener
        .local_addr()
        .map_err(|e| ApiError::Listen(listen, e))?;
    Ok(Bound { listener, addr })
}

/// Serve `router` ([`router`]'s) on what [`bind`] bound.
pub fn serve(router: Router, bound: Bound) -> Serving {
    let Bound { listener, addr } = bound;
    tracing::info!(target: "rails", %addr, "api: serving the record to this computer's apps");
    let task = tokio::spawn(async move {
        let served = router.into_make_service_with_connect_info::<SocketAddr>();
        if let Err(e) = axum::serve(listener, served).await {
            tracing::error!(target: "rails", error = %e, "api: listener stopped");
        }
    });
    Serving { addr, task }
}
