// SPDX-License-Identifier: AGPL-3.0-or-later
//! The member door (DESIGN §3; §8 U3's inventory): a route on the members'
//! protocol that carries a request for a lend to its service on this
//! computer's loopback, and the answer back.
//!
//! `/lend/<lend>/<path>` reaches the lend's port as `/<path>`, the query
//! kept — the first segment names the service and is taken off, as
//! upstream's `split_app_name` did (`iroh_identity_forward.rs:181`).
//!
//! **Who is asking** is set by the door and nobody else. Every header a
//! client typed under [`IDENTITY_PREFIX`], and every header the lend's
//! secret is about to fill, is stripped, compared as upstream's
//! `rewrite_head` compared (`iroh_identity_forward.rs:297-328`: written as
//! the wire writes it, then lowercased, so case, padding and bytes the wire
//! drops dodge nothing); then the person and key are set from the
//! connection's verified [`Dialer`] and the register; then the lend's
//! secret ([`crate::declared`]). Exactly one of each reaches the service, and
//! it is the door's. The rules are upstream's, the code is not: upstream
//! rewrote request heads on a byte splice, and here hyper has parsed the
//! request before the door sees it (DESIGN §8, U1's inventory).
//!
//! **Bodies stream both ways**, neither buffered: a request's body is read
//! as the service reads it, a response's as the member reads it, so a large
//! upload or a whole file costs the door no more memory than a chunk.
//! `Range`, `If-Range` and `Content-Range` pass untouched, as every header
//! but the hop-by-hop ones does, so a seek is the service's to answer.
//! Upstream streamed a GET's response and passed `Range` alone
//! (`serve_client.rs:616-655`, `forward_stream`); it streamed no request body
//! anywhere.
//!
//! **The secret never goes back.** A response header holding the lend's
//! secret is dropped, named at warn by its header, never its value.
//!
//! **What it lends, asked** (u3-5; DESIGN §8, "Lends are asked, not
//! stored"): `GET /lends` answers a dialer standing in a group this
//! computer holds every lend declared here now, read from the declarations
//! at each ask, each with how many people use it now ([`Using`]): the
//! people with a request through the door to it in flight, its answer
//! still streaming included.
//!
//! **What the door refuses**, by name: a request not on an iroh stream; a
//! dialer standing in no group this computer holds; a path that can step
//! out of its lend — a `..` segment, an encoded `.` or `/` — as
//! `route_by_prefix` refuses one (`iroh_routed_forward.rs:318-337`); a lend
//! this computer does not declare; and a path the lend refuses
//! ([`crate::lend::RefusedPath::covers`]).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::header::{self, HeaderMap, HeaderName, HeaderValue};
use axum::http::{StatusCode, Version};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use axum::{Extension, Json};
use http_body_util::BodyExt;
use tokio::sync::{mpsc, oneshot};

use crate::declared::read_declared_in;
use crate::iroh::Dialer;
use crate::lend::{self, Lend, LendName, Offered};
use crate::ring_routes::err;
use crate::{Ed25519Verifier, Person, RingRail};

/// svrngs's own header prefix: every header under it on a request through
/// the door is the door's. A client's is stripped whole.
pub const IDENTITY_PREFIX: &str = "x-svrngs-";

/// The person asking, as the register names them: their name in the first
/// group this computer holds, in the groups' order, that they stand in.
pub const PERSON_HEADER: &str = "x-svrngs-person";

/// The key the connection verified, as the record spells an actor: its
/// lowercase hex.
pub const KEY_HEADER: &str = "x-svrngs-key";

/// Where a request for a lend starts.
pub const LEND_ROUTE: &str = "/lend/";

/// Where a member asks what this computer lends.
pub const LENDS_ROUTE: &str = "/lends";

/// How long the door waits for a lend's service to take a connection; once
/// connected, a body takes as long as it takes, as upstream's
/// `forward_stream` bounded only its connect.
const CONNECT_WITHIN: Duration = Duration::from_secs(10);

/// Hop-by-hop headers (RFC 9110 §7.6.1), never carried across the door in
/// either direction, beside any a message's `Connection` names: the strip
/// lists of `sovereign-pods/src/worker_inference_proxy.rs:74-93`, less
/// `host` (the service builds its links for the address the member's
/// browser used) and `authorization` (a lend's secret displaces a client's,
/// and a lend declaring none leaves the client's alone) and
/// `content-length` (kept, so a `HEAD` and a `206` keep their length).
const HOP_BY_HOP: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authorization",
    "proxy-connection",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

/// What the door reads: the register, where the lends are declared, and
/// who is using each now.
#[derive(Clone)]
pub struct Door {
    pub rail: Arc<RingRail>,
    pub data_dir: PathBuf,
    pub using: Using,
}

/// The door's routes, for the members' protocol's router: a lend, every
/// method, and the list of what this computer lends.
pub fn router(door: Door) -> axum::Router {
    axum::Router::new()
        .route("/lend/{*rest}", any(through))
        .route(LENDS_ROUTE, get(lends))
        .with_state(door)
}

/// Who is using each lend now: for each, the people with a request
/// through the door to it in flight. The door's own state, owned by one
/// task and told by messages (RUST_ARCH 3, 4): a request carried to a
/// service begins a use, and its answer, dropped once streamed or
/// abandoned, ends it ([`InFlight`]). Never in the record.
#[derive(Clone)]
pub struct Using(mpsc::UnboundedSender<Use>);

enum Use {
    Began(LendName, String),
    Ended(LendName, String),
    People(LendName, oneshot::Sender<u32>),
}

impl Using {
    /// The task that keeps the count, ending when the last handle drops.
    pub fn spawn() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut flights: BTreeMap<LendName, BTreeMap<String, usize>> = BTreeMap::new();
            while let Some(u) = rx.recv().await {
                match u {
                    Use::Began(lend, actor) => {
                        *flights.entry(lend).or_default().entry(actor).or_default() += 1;
                    }
                    Use::Ended(lend, actor) => {
                        let people = flights.entry(lend).or_default();
                        if let Some(n) = people.get_mut(&actor) {
                            *n -= 1;
                            if *n == 0 {
                                people.remove(&actor);
                            }
                        }
                    }
                    Use::People(lend, answer) => {
                        let n = flights.get(&lend).map_or(0, |p| p.len());
                        let _ = answer.send(u32::try_from(n).unwrap_or(u32::MAX));
                    }
                }
            }
        });
        Self(tx)
    }

    /// `actor` begins a request to `lend`; it ends when the hold drops.
    fn begin(&self, lend: &LendName, actor: &str) -> InFlight {
        let _ = self.0.send(Use::Began(lend.clone(), actor.to_string()));
        InFlight {
            using: self.0.clone(),
            lend: lend.clone(),
            actor: actor.to_string(),
        }
    }

    /// How many people use `lend` now; `None` when the count's task is
    /// gone, which is not a count of none.
    pub async fn people(&self, lend: &LendName) -> Option<u32> {
        let (answer, counted) = oneshot::channel();
        self.0.send(Use::People(lend.clone(), answer)).ok()?;
        counted.await.ok()
    }
}

/// One request's use of a lend, ended when dropped.
struct InFlight {
    using: mpsc::UnboundedSender<Use>,
    lend: LendName,
    actor: String,
}

impl Drop for InFlight {
    fn drop(&mut self) {
        let _ = self.using.send(Use::Ended(
            self.lend.clone(),
            std::mem::take(&mut self.actor),
        ));
    }
}

/// `GET /lends`: what this computer lends, to a dialer standing in a group
/// it holds, read from the declarations now. A declaration that cannot be
/// read is left out, named at warn; a lend whose count cannot be had is
/// left out too, never answered as used by none.
async fn lends(State(door): State<Door>, dialer: Option<Extension<Dialer>>) -> Response {
    let Some(Extension(dialer)) = dialer else {
        tracing::warn!(target: "door", "door: lends refused — no verified dialer, so the request did not come on an iroh stream");
        return err(
            StatusCode::FORBIDDEN,
            "what a computer lends is asked only from another computer of its group",
        );
    };
    let actor = dialer.actor();
    if door
        .rail
        .standing_anywhere(&actor, &Ed25519Verifier)
        .is_none()
    {
        tracing::debug!(target: "door", actor, "door: lends refused — the dialer stands in no group held here");
        return err(StatusCode::FORBIDDEN, "this computer lends nothing to you");
    }
    let declared = match lend::all(&door.data_dir) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(target: "door", error = %e, "door: lends — the declarations cannot be read");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "what this computer lends cannot be read on it",
            );
        }
    };
    let mut offered = Vec::new();
    for lend in declared {
        let lend = match lend {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!(target: "door", error = %e, "door: lends — a declaration cannot be read, so it is left out");
                continue;
            }
        };
        let Some(using) = door.using.people(&lend.name).await else {
            tracing::warn!(target: "door", lend = %lend.name, "door: lends — no count of who uses it, so it is left out");
            continue;
        };
        offered.push(Offered {
            name: lend.name,
            kind: lend.kind,
            limit: lend.limit,
            using,
        });
    }
    tracing::debug!(target: "door", actor, lends = offered.len(), "door: lends — answered what this computer lends");
    Json(serde_json::json!({ "lends": offered })).into_response()
}

/// A name as upstream's `rewrite_head` compared it (`compare_name`,
/// `iroh_identity_forward.rs:274`): visible ASCII only, trimmed, lowercased.
fn compare_name(name: &str) -> String {
    name.chars()
        .filter(|c| (' '..='~').contains(c))
        .collect::<String>()
        .trim()
        .to_ascii_lowercase()
}

/// Whether `path` can step out of the lend it names: a `..` segment, or an
/// encoded `.` or `/` anywhere (`route_by_prefix`, upstream's guard).
fn can_step_out(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    path.split('/').any(|seg| seg == "..") || lower.contains("%2e") || lower.contains("%2f")
}

/// The lend a request's path names, and the path its service is asked for.
fn split_lend(path: &str) -> Option<(&str, String)> {
    let rest = path.strip_prefix(LEND_ROUTE)?;
    let (name, tail) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let tail = if tail.is_empty() { "/" } else { tail };
    Some((name, tail.to_string()))
}

/// Remove the hop-by-hop headers from `headers`, and any its `Connection`
/// names; answer how many went.
fn strip_hop_by_hop(headers: &mut HeaderMap) -> usize {
    let named: Vec<String> = headers
        .get_all(header::CONNECTION)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .map(compare_name)
        .collect();
    let gone: Vec<HeaderName> = headers
        .keys()
        .filter(|n| {
            let name = compare_name(n.as_str());
            HOP_BY_HOP.contains(&name.as_str()) || named.contains(&name)
        })
        .cloned()
        .collect();
    let mut n = 0;
    for name in gone {
        n += headers.get_all(&name).iter().count();
        headers.remove(&name);
    }
    n
}

/// The request's head as the service is to see it: the client's headers
/// but the hop-by-hop ones, those under [`IDENTITY_PREFIX`] and those a
/// secret fills; then who is asking; then the secret. Answers how many the
/// client typed that the door displaced.
fn rewrite_headers(
    headers: &mut HeaderMap,
    identity: [(HeaderName, HeaderValue); 2],
    secret: &[(HeaderName, HeaderValue)],
) -> usize {
    strip_hop_by_hop(headers);
    let declared: Vec<String> = secret
        .iter()
        .map(|(n, _)| compare_name(n.as_str()))
        .collect();
    let typed: Vec<HeaderName> = headers
        .keys()
        .filter(|n| {
            let name = compare_name(n.as_str());
            name.starts_with(IDENTITY_PREFIX) || declared.contains(&name)
        })
        .cloned()
        .collect();
    let mut displaced = 0;
    for name in typed {
        displaced += headers.get_all(&name).iter().count();
        headers.remove(&name);
    }
    for (name, value) in identity.into_iter().chain(secret.iter().cloned()) {
        headers.append(name, value);
    }
    displaced
}

/// The lend's secret as headers. A declared name or value that cannot be a
/// header is skipped, named at warn by its header, never its value.
fn secret_headers(door: &Door, lend: &LendName) -> Vec<(HeaderName, HeaderValue)> {
    read_declared_in(&lend::secret_dir(&door.data_dir, lend))
        .into_iter()
        .filter_map(|(name, value)| {
            match (
                HeaderName::try_from(name.as_str()),
                HeaderValue::try_from(value),
            ) {
                (Ok(n), Ok(v)) => Some((n, v)),
                _ => {
                    tracing::warn!(target: "door", lend = %lend, header = %name,
                        "door: a declared secret cannot be a header, so it is not sent");
                    None
                }
            }
        })
        .collect()
}

/// Drop every response header holding one of `secret`'s values; answer the
/// names dropped.
fn drop_echoed_secret(
    headers: &mut HeaderMap,
    secret: &[(HeaderName, HeaderValue)],
) -> Vec<String> {
    let holds = |v: &HeaderValue| {
        secret.iter().any(|(_, s)| {
            let s = s.as_bytes();
            !s.is_empty() && v.as_bytes().windows(s.len()).any(|w| w == s)
        })
    };
    let echoed: Vec<HeaderName> = headers
        .iter()
        .filter(|(_, v)| holds(v))
        .map(|(n, _)| n.clone())
        .collect();
    for name in &echoed {
        headers.remove(name);
    }
    echoed.iter().map(|n| n.to_string()).collect()
}

/// One request through the door: refused by name, or carried to the lend's
/// service and its answer streamed back.
async fn through(
    State(door): State<Door>,
    dialer: Option<Extension<Dialer>>,
    req: Request,
) -> Response {
    let Some(Extension(dialer)) = dialer else {
        tracing::warn!(target: "door", "door: refused — no verified dialer, so the request did not come on an iroh stream");
        return err(
            StatusCode::FORBIDDEN,
            "a lend is reached only from another computer of its group",
        );
    };
    let actor = dialer.actor();
    let path = req.uri().path().to_string();
    if can_step_out(&path) {
        tracing::debug!(target: "door", actor, path, "door: refused — a path that can step out of its lend");
        return err(
            StatusCode::BAD_REQUEST,
            "a path that can step outside a lend is refused",
        );
    }
    let Some((raw_name, service_path)) = split_lend(&path) else {
        tracing::debug!(target: "door", actor, path, "door: refused — no lend named in the path");
        return err(StatusCode::NOT_FOUND, "nothing is lent here by that name");
    };
    let Some((group, person)) = door.rail.standing_anywhere(&actor, &Ed25519Verifier) else {
        tracing::debug!(target: "door", actor, "door: refused — the dialer stands in no group held here");
        return err(StatusCode::FORBIDDEN, "this computer lends nothing to you");
    };
    let lend = match LendName::parse(raw_name).map(|n| lend::read(&door.data_dir, &n)) {
        Ok(Ok(Some(lend))) => lend,
        Ok(Ok(None)) | Err(_) => {
            tracing::debug!(target: "door", actor, lend = raw_name, "door: refused — no lend declared by that name");
            return err(StatusCode::NOT_FOUND, "nothing is lent here by that name");
        }
        Ok(Err(e)) => {
            tracing::warn!(target: "door", lend = raw_name, error = %e, "door: refused — the lend's declaration cannot be read");
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "this lend cannot be read on the computer that lends it",
            );
        }
    };
    if let Some(refused) = lend.refusing(&service_path) {
        tracing::debug!(target: "door", actor, lend = %lend.name, path = service_path, refused = refused.as_str(),
            "door: refused — the lend refuses this path");
        return err(
            StatusCode::FORBIDDEN,
            format!("{} is lent without {}", lend.name, refused.as_str()),
        );
    }
    tracing::debug!(target: "door", actor, person = person.as_str(), group, lend = %lend.name,
        method = %req.method(), path = service_path, "door: carrying a request to the lend's service");
    forward(&door, &lend, (&actor, &person), service_path, req).await
}

/// Carry `req` to `lend`'s service as `path` (with its query), the head
/// rewritten, and stream the answer back.
async fn forward(
    door: &Door,
    lend: &Lend,
    (actor, person): (&str, &Person),
    path: String,
    req: Request,
) -> Response {
    let (mut parts, body) = req.into_parts();
    let target = match parts.uri.query() {
        Some(q) => format!("{path}?{q}"),
        None => path,
    };
    let (Ok(person_value), Ok(key_value)) = (
        HeaderValue::from_bytes(person.as_str().as_bytes()),
        HeaderValue::from_str(actor),
    ) else {
        tracing::warn!(target: "door", actor, "door: refused — the person's name cannot be a header");
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "who is asking cannot be told to this lend",
        );
    };
    let secret = secret_headers(door, &lend.name);
    let displaced = rewrite_headers(
        &mut parts.headers,
        [
            (HeaderName::from_static(PERSON_HEADER), person_value),
            (HeaderName::from_static(KEY_HEADER), key_value),
        ],
        &secret,
    );
    if displaced > 0 {
        tracing::info!(target: "door", actor, lend = %lend.name, displaced,
            "door: dropped headers the client typed in the door's or the secret's name before setting them");
    }
    let mut request = match axum::http::Request::builder()
        .method(parts.method)
        .uri(&target)
        .version(Version::HTTP_11)
        .body(body)
    {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!(target: "door", actor, error = %e, "door: refused — the request target is not a path");
            return err(
                StatusCode::BAD_REQUEST,
                "the request is not one a lend can be asked",
            );
        }
    };
    *request.headers_mut() = parts.headers;
    // A use begins as the request leaves for the service, and lasts until
    // the member has the whole answer or leaves it: the answer's body holds it.
    let held = door.using.begin(&lend.name, actor);

    let unanswered = || {
        err(
            StatusCode::BAD_GATEWAY,
            format!(
                "{} is not answering on the computer that lends it",
                lend.name
            ),
        )
    };
    let origin = std::net::SocketAddr::from(([127, 0, 0, 1], lend.port.get()));
    let tcp = match tokio::time::timeout(CONNECT_WITHIN, tokio::net::TcpStream::connect(origin))
        .await
    {
        Ok(Ok(tcp)) => tcp,
        Ok(Err(e)) => {
            tracing::warn!(target: "door", lend = %lend.name, %origin, error = %e, "door: the lend's service refused a connection");
            return unanswered();
        }
        Err(_) => {
            tracing::warn!(target: "door", lend = %lend.name, %origin, within = ?CONNECT_WITHIN, "door: the lend's service took no connection in time");
            return unanswered();
        }
    };
    let (mut sender, conn) = match hyper::client::conn::http1::handshake(
        hyper_util::rt::TokioIo::new(tcp),
    )
    .await
    {
        Ok(pair) => pair,
        Err(e) => {
            tracing::warn!(target: "door", lend = %lend.name, error = %e, "door: no HTTP/1 connection to the lend's service");
            return unanswered();
        }
    };
    // Drives the connection until the response body is read to its end.
    let name = lend.name.clone();
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            tracing::debug!(target: "door", lend = %name, error = %e, "door: the connection to the lend's service ended in error");
        }
    });
    let answer = match sender.send_request(request).await {
        Ok(a) => a,
        Err(e) => {
            tracing::warn!(target: "door", lend = %lend.name, error = %e, "door: the lend's service did not answer");
            return unanswered();
        }
    };
    let (mut parts, body) = answer.into_parts();
    strip_hop_by_hop(&mut parts.headers);
    let echoed = drop_echoed_secret(&mut parts.headers, &secret);
    if !echoed.is_empty() {
        tracing::warn!(target: "door", lend = %lend.name, headers = ?echoed,
            "door: the lend's service answered with its secret in a header — dropped, never sent on");
    }
    tracing::debug!(target: "door", actor, lend = %lend.name, status = parts.status.as_u16(),
        "door: the lend's service answered; streaming it back");
    let body = body.map_frame(move |frame| {
        let _ = &held;
        frame
    });
    let mut response = Response::new(Body::new(body));
    *response.status_mut() = parts.status;
    *response.headers_mut() = parts.headers;
    response.into_response()
}

#[cfg(test)]
mod tests;
