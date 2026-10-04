// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-rails/src/api.rs@e1a43bcd2
//! The loopback routes a shim author touches, and nothing else.
//!
//! The ring rail's doors, `/v1/rail/{append,log}`, in [`crate::rail`].
//! (svrngs: the mesh, media, publish, origin, membership, store and ledger
//! doors beside them stay behind; DESIGN §8, U3's inventory.)
//!
//! **Loopback IS the auth.** There is no bearer here, and the bind refuses
//! any address that is not loopback rather than serving an unauthenticated
//! API to a network.

use std::net::SocketAddr;

use axum::routing::{get, post};
use axum::Router;

use crate::rail::RailApi;

/// Why the API did not start. (Upstream's were two variants of the daemon's
/// `Refusal`, `lib.rs:122-123`, which did not come over.)
#[derive(Debug, thiserror::Error)]
pub enum Refusal {
    #[error("{0} is not a loopback address; this API has no auth because loopback is the auth")]
    NotLoopback(SocketAddr),
    #[error("cannot listen on {0}: {1}")]
    Listen(SocketAddr, std::io::Error),
}

/// The routes this API serves.
pub fn router(api: RailApi) -> Router {
    Router::new()
        // The ring rail's doors (FIVE_PROGRAMS fp-44): durable append and
        // the log. The bodies mirror the daemon's `routes_rail` doors.
        .route("/v1/rail/append", post(crate::rail::append))
        .route("/v1/rail/log", get(crate::rail::log))
        .with_state(api)
}

/// The bind guard, as a function so it has a failing input that can be
/// named without standing an iroh endpoint up. This API has no auth because
/// loopback IS the auth; anywhere else is publishing it.
pub fn check_loopback(listen: SocketAddr) -> Result<(), Refusal> {
    if listen.ip().is_loopback() {
        Ok(())
    } else {
        Err(Refusal::NotLoopback(listen))
    }
}

/// Bind and serve. Refuses a non-loopback address by name.
pub async fn serve(
    api: RailApi,
    listen: SocketAddr,
) -> Result<tokio::task::JoinHandle<()>, Refusal> {
    check_loopback(listen)?;
    let listener = tokio::net::TcpListener::bind(listen)
        .await
        .map_err(|e| Refusal::Listen(listen, e))?;
    let router = router(api);
    Ok(tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            tracing::error!(target: "rails", error = %e, "api: listener stopped");
        }
    }))
}
