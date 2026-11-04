// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native short invitations. The resolver owns phrase reservations; this
//! computer owns the invitation and its recoverable receipt (ONBOARDING §3).
use crate::{
    actor_of,
    node::{Asking, Host},
};
use axum::http::StatusCode;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

const RECEIPT: &str = "publication.json";
const UNAVAILABLE: &str =
    "joining is temporarily unavailable; retry in a minute from this computer";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Phase {
    Pending,
    Open,
    Closing,
    Closed,
    Ended,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    id: String,
    token: String,
    resolver: String,
    until: i64,
    page: Option<String>,
    phase: Phase,
    phrase: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    group: String,
    #[serde(default)]
    id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct Fault(String);
impl<E: std::fmt::Display> From<&E> for Fault {
    fn from(e: &E) -> Self {
        Self(e.to_string())
    }
}

impl Receipt {
    fn read(host: &Host, group: &str) -> Result<Option<Self>, Fault> {
        let dir = host.rail.group_dir(group).map_err(|e| Fault::from(&e))?;
        match std::fs::read(dir.join(RECEIPT)) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|e| Fault::from(&e)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Fault::from(&e)),
        }
    }
    fn keep(&self, host: &Host, group: &str) -> Result<(), Fault> {
        let at = host
            .rail
            .group_dir(group)
            .map_err(|e| Fault::from(&e))?
            .join(RECEIPT);
        let bytes = serde_json::to_vec(self).map_err(|e| Fault::from(&e))?;
        crate::declared::write_secret(&at, &bytes).map_err(|(_, e)| Fault::from(&e))
    }
    fn current(&self, host: &Host, group: &str) -> Result<bool, Fault> {
        let latest = host
            .rail
            .current_invite(group)
            .map_err(|e| Fault::from(&e))?;
        Ok(latest.is_some_and(|(_, i)| i.key == self.id))
    }
    #[expect(
        clippy::disallowed_methods,
        reason = "status compares the persisted invitation expiry at the HTTP edge"
    )]
    fn shown(&self, host: &Host, group: &str) -> Result<serde_json::Value, Fault> {
        let current = self.current(host, group)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| Fault::from(&e))?
            .as_secs() as i64;
        let state = if self.phase == Phase::Closed {
            Phase::Closed
        } else if !current || now >= self.until {
            Phase::Ended
        } else {
            self.phase
        };
        tracing::debug!(target: "publication", group, ?state, current, "publication: local status; no resolver request");
        Ok(
            serde_json::json!({"id": self.id, "state": state, "phrase": self.phrase, "until": self.until}),
        )
    }
}

fn answer(result: Result<serde_json::Value, Fault>) -> Response {
    match result {
        Ok(value) => Json(value).into_response(),
        Err(e) => {
            tracing::debug!(target: "publication", error = %e, "publication: unavailable; receipt retained for explicit retry");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": UNAVAILABLE})),
            )
                .into_response()
        }
    }
}

/// Status performs no resolver I/O, including after a restart.
pub(crate) async fn status(State(at): State<Asking>, Query(q): Query<Request>) -> Response {
    answer((|| {
        at.host.own_person(&q.group).map_err(|e| Fault::from(&e))?;
        Receipt::read(&at.host, &q.group)?
            .map_or(Ok(serde_json::Value::Null), |r| r.shown(&at.host, &q.group))
    })())
}

/// An explicit retry uses the same receipt and publication id. A new address
/// requires the previous id, so reloading cannot accidentally mint again.
pub(crate) async fn publish(State(at): State<Asking>, Json(q): Json<Request>) -> Response {
    answer(change(&at.host, &q, false).await)
}
pub(crate) async fn close(State(at): State<Asking>, Json(q): Json<Request>) -> Response {
    answer(change(&at.host, &q, true).await)
}

async fn change(host: &Host, q: &Request, closing: bool) -> Result<serde_json::Value, Fault> {
    let mut receipt = {
        let _held = host
            .minting
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        host.own_person(&q.group).map_err(|e| Fault::from(&e))?;
        let old = Receipt::read(host, &q.group)?;
        let mut r = match old {
            Some(r)
                if closing
                    || q.id.as_deref() != Some(&r.id)
                    || (r.phase != Phase::Closed && r.current(host, &q.group)?) =>
            {
                r
            }
            _ if closing => return Err(Fault("no publication to close".into())),
            _ => {
                let resolver = std::fs::read_to_string(host.rail.root().join("join-resolver"))
                    .map_err(|e| Fault::from(&e))?;
                let url = reqwest13::Url::parse(resolver.trim()).map_err(|e| Fault::from(&e))?;
                let local = url
                    .host_str()
                    .is_some_and(|h| h == "127.0.0.1" || h == "[::1]");
                if !(url.scheme() == "https" || url.scheme() == "http" && local)
                    || url.fragment().is_some()
                    || !url.username().is_empty()
                    || url.password().is_some()
                {
                    return Err(Fault(
                        "resolver must use HTTPS (HTTP only on loopback)".into(),
                    ));
                }
                let made = host
                    .link_held(&q.group, false, None, None)
                    .map_err(|e| Fault::from(&e))?;
                let (_, invite) = host
                    .rail
                    .current_invite(&q.group)
                    .map_err(|e| Fault::from(&e))?
                    .ok_or_else(|| Fault("invitation absent".into()))?;
                Receipt {
                    id: actor_of(&made.token.key()),
                    token: made.token.encode(),
                    resolver: url.into(),
                    until: invite.until,
                    page: made.page.map(|p| p.as_str().into()),
                    phase: Phase::Pending,
                    phrase: None,
                }
            }
        };
        if closing {
            if q.id.as_deref() != Some(&r.id) {
                return Err(Fault("stale close".into()));
            }
            if r.phase == Phase::Closed {
                return r.shown(host, &q.group);
            }
            r.phase = Phase::Closing;
        } else if r.phase != Phase::Pending || !r.current(host, &q.group)? {
            return r.shown(host, &q.group);
        }
        r.keep(host, &q.group)?;
        tracing::debug!(target: "publication", group = q.group, phase = ?r.phase, "publication: pending receipt kept before resolver request");
        r
    };
    // No invitation mutex crosses an await. A concurrent link or close wins
    // over this response, even if the resolver accepted the old publication.
    let phrase = send(host, &q.group, &receipt, closing).await?;
    let _held = host
        .minting
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let kept = Receipt::read(host, &q.group)?.ok_or_else(|| Fault("receipt disappeared".into()))?;
    if kept.id != receipt.id || kept.phase != receipt.phase {
        tracing::debug!(target: "publication", group = q.group, "publication: late response superseded by local receipt");
        return kept.shown(host, &q.group);
    }
    let current = receipt.current(host, &q.group)?;
    if closing {
        if current {
            // A fresh, undisclosed invitation supersedes this one by the
            // existing fold; a stale close never touches a newer link.
            host.link_held(&q.group, false, None, None)
                .map_err(|e| Fault::from(&e))?;
        }
        receipt.phase = Phase::Closed;
    } else {
        if !current {
            tracing::debug!(target: "publication", group = q.group, "publication: late publication no longer current");
            return receipt.shown(host, &q.group);
        }
        receipt.phase = Phase::Open;
        receipt.phrase = phrase;
    }
    receipt.keep(host, &q.group)?;
    tracing::debug!(target: "publication", group = q.group, phase = ?receipt.phase, "publication: resolver acknowledged; local invitation rechecked");
    receipt.shown(host, &q.group)
}

async fn send(
    host: &Host,
    group: &str,
    r: &Receipt,
    closing: bool,
) -> Result<Option<String>, Fault> {
    let action = if closing { "close" } else { "publish" };
    let body = serde_json::to_vec(&serde_json::json!({
        "action": action,
        "id": r.id,
        "group": group,
        "token": if closing { None } else { Some(&r.token) },
        "until": r.until,
        "page": r.page,
    }))
    .map_err(|e| Fault::from(&e))?;
    let app = svrngs_core::app::App::parse("onboarding").map_err(|e| Fault::from(&e))?;
    let request = crate::http::builder()
        .map_err(Fault)?
        .redirect(reqwest13::redirect::Policy::none())
        .retry(reqwest13::retry::never())
        .build()
        .map_err(|e| Fault::from(&e))?
        .post(&r.resolver)
        .header("x-svrngs-key", &host.actor)
        .header("x-svrngs-signature", app.sign(host.rail.signer(), &body))
        .header("content-type", "application/json")
        .body(body);
    let reply = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        let mut response = request.send().await.map_err(|e| Fault::from(&e))?;
        if !response.status().is_success() || response.url().as_str() != r.resolver {
            return Err(Fault("resolver refused or redirected".into()));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|e| Fault::from(&e))? {
            if bytes.len() + chunk.len() > 8192 {
                return Err(Fault("resolver response exceeds 8 KiB".into()));
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|e| Fault::from(&e))
    })
    .await
    .map_err(|e| Fault::from(&e))??;
    if reply["id"].as_str() != Some(&r.id) {
        return Err(Fault("resolver acknowledged another publication".into()));
    }
    if closing {
        return (reply["closed"] == true)
            .then_some(None)
            .ok_or_else(|| Fault("resolver did not acknowledge closure".into()));
    }
    let phrase = reply["phrase"]
        .as_str()
        .ok_or_else(|| Fault("resolver omitted phrase".into()))?;
    if phrase.split('-').count() != 5
        || !phrase.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
        || phrase.split('-').any(str::is_empty)
    {
        return Err(Fault("resolver returned an invalid phrase".into()));
    }
    Ok(Some(phrase.into()))
}
