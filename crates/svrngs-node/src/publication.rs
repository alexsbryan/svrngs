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
fn fault(e: impl std::fmt::Display) -> Fault {
    Fault(e.to_string())
}

impl Receipt {
    fn read(host: &Host, group: &str) -> Result<Option<Self>, Fault> {
        let dir = host.rail.group_dir(group).map_err(fault)?;
        crate::declared::read_json(&dir.join(RECEIPT)).map_err(fault)
    }
    fn keep(&self, host: &Host, group: &str) -> Result<(), Fault> {
        let at = host.rail.group_dir(group).map_err(fault)?.join(RECEIPT);
        let bytes = serde_json::to_vec(self).map_err(fault)?;
        crate::declared::write_secret(&at, &bytes).map_err(|(_, e)| fault(e))
    }
    fn current(&self, host: &Host, group: &str) -> Result<bool, Fault> {
        let latest = host.rail.current_invite(group).map_err(fault)?;
        Ok(latest.is_some_and(|(_, i)| i.key == self.id))
    }
    /// The receipt as the page reads it, its expiry against the door's clock.
    fn shown(&self, host: &Host, group: &str) -> Result<serde_json::Value, Fault> {
        let current = self.current(host, group)?;
        let now =
            crate::door::unix_now().ok_or_else(|| Fault("the clock reads before 1970".into()))?;
        let state = match self.phase {
            Phase::Closed => Phase::Closed,
            _ if !current || now >= self.until => Phase::Ended,
            phase => phase,
        };
        // While the invitation admits, the long link /link gives this page
        // too (home.rs `link`, svrngs-core `https_link`'s spelling), and its
        // QR code: a phone scanning it never asks the resolver.
        let link = (self.page.as_ref())
            .filter(|_| matches!(state, Phase::Pending | Phase::Open))
            .map(|p| format!("{p}#{}", self.token));
        let qr = link.as_deref().and_then(qr);
        tracing::debug!(target: "publication", group, ?state, current, link = link.is_some(), qr = qr.is_some(), "publication: local status; no resolver request");
        let url = reqwest13::Url::parse(&self.resolver).map_err(fault)?;
        let at = url.origin().ascii_serialization();
        let address = self.phrase.as_ref().map(|phrase| format!("{at}/{phrase}"));
        Ok(
            serde_json::json!({"id": self.id, "state": state, "phrase": self.phrase, "address": address, "until": self.until, "link": link, "qr": qr}),
        )
    }
}

/// `link` as a QR code the home page draws (group-links.js): an SVG path,
/// a unit square for each dark module, in a square `size` wide with the
/// quiet zone of four around it. None when the link is too long for one.
fn qr(link: &str) -> Option<serde_json::Value> {
    let code = qrcodegen::QrCode::encode_text(link, qrcodegen::QrCodeEcc::Medium).ok()?;
    let n = code.size();
    let dark = |i: &i32| code.get_module(i % n, i / n);
    let square = |i: i32| format!("M{} {}h1v1h-1z", i % n + 4, i / n + 4);
    let path: String = (0..n * n).filter(dark).map(square).collect();
    Some(serde_json::json!({"size": n + 8, "path": path}))
}

/// svrngs's own resolver, for a computer whose `join-resolver` file names
/// none: batteries included, and never the only choice.
const RESOLVER: &str = "https://join.svrnme.sh/";

/// The resolver this computer's `join-resolver` file names, `named`, else
/// svrngs's own: a file that is there and cannot be read is said, never
/// passed over for ours.
fn resolver(named: std::io::Result<String>) -> Result<String, Fault> {
    match named {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(RESOLVER.into()),
        named => named.map_err(fault),
    }
}

fn answer(result: Result<serde_json::Value, Fault>) -> Response {
    match result {
        Ok(value) => Json(value).into_response(),
        Err(e) => {
            tracing::debug!(target: "publication", error = %e, "publication: unavailable; receipt retained for explicit retry");
            let said = Json(serde_json::json!({"error": UNAVAILABLE}));
            (StatusCode::SERVICE_UNAVAILABLE, said).into_response()
        }
    }
}

/// Status performs no resolver I/O, including after a restart.
pub(crate) async fn status(State(at): State<Asking>, Query(q): Query<Request>) -> Response {
    answer((|| {
        at.host.own_person(&q.group).map_err(fault)?;
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
        let _held = (host.minting.lock()).unwrap_or_else(std::sync::PoisonError::into_inner);
        host.own_person(&q.group).map_err(fault)?;
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
                let resolver = resolver(std::fs::read_to_string(
                    host.rail.root().join("join-resolver"),
                ))?;
                let url = reqwest13::Url::parse(resolver.trim()).map_err(fault)?;
                let local = matches!(url.host_str(), Some("127.0.0.1" | "[::1]"));
                if !(url.scheme() == "https" || url.scheme() == "http" && local)
                    || url.fragment().is_some()
                    || !url.username().is_empty()
                    || url.password().is_some()
                {
                    return Err(fault("resolver must use HTTPS (HTTP only on loopback)"));
                }
                let made = host.link_held(&q.group, false, None, None).map_err(fault)?;
                let latest = host.rail.current_invite(&q.group).map_err(fault)?;
                let (_, invite) = latest.ok_or_else(|| fault("invitation absent"))?;
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
        tracing::debug!(target: "publication", group = q.group, phase = ?r.phase, resolver = r.resolver, "publication: pending receipt kept before resolver request");
        r
    };
    // No invitation mutex crosses an await. A concurrent link or close wins
    // over this response, even if the resolver accepted the old publication.
    let phrase = send(host, &q.group, &receipt, closing).await?;
    let _held = (host.minting.lock()).unwrap_or_else(std::sync::PoisonError::into_inner);
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
            host.link_held(&q.group, false, None, None).map_err(fault)?;
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
    .map_err(fault)?;
    let app = svrngs_core::app::App::parse("onboarding").map_err(fault)?;
    let request = crate::http::builder()
        .map_err(Fault)?
        .redirect(reqwest13::redirect::Policy::none())
        .retry(reqwest13::retry::never())
        .build()
        .map_err(fault)?
        .post(&r.resolver)
        .header("x-svrngs-key", &host.actor)
        .header("x-svrngs-signature", app.sign(host.rail.signer(), &body))
        .header("content-type", "application/json")
        .body(body);
    let reply = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        let mut response = request.send().await.map_err(fault)?;
        if !response.status().is_success() || response.url().as_str() != r.resolver {
            return Err(Fault("resolver refused or redirected".into()));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(fault)? {
            if bytes.len() + chunk.len() > 8192 {
                return Err(Fault("resolver response exceeds 8 KiB".into()));
            }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice::<serde_json::Value>(&bytes).map_err(fault)
    })
    .await
    .map_err(fault)??;
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
    // Three words and a number (resolver/handler.mjs `phrase`), or five words
    // from a resolver not yet redeployed.
    if !(4..=5).contains(&phrase.split('-').count())
        || phrase.contains(|c: char| !matches!(c, 'a'..='z' | '0'..='9' | '-'))
        || phrase.split('-').any(str::is_empty)
    {
        return Err(Fault("resolver returned an invalid phrase".into()));
    }
    Ok(Some(phrase.into()))
}

#[cfg(test)]
#[path = "publication_tests.rs"]
mod tests;
