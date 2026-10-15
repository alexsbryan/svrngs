// SPDX-License-Identifier: AGPL-3.0-or-later
// `content_type`: ported from commonwealth-ai shared/crates/host-kit/src/shell/files.rs@c663d4004
// `FLOOR`'s policy: ported from commonwealth-ai shared/crates/sovereign-contracts/src/egress.rs@c663d4004 (`csp`)
//! An app the group carries (v1ui-shelf (a); svrn-docs
//! `rings/current/COMMUNITY_APPS.md` §6-§7, as `V1_UI_BUILD.md` §3.4
//! narrows it): a folder `apps/<kind>/` holding an `index.html`, in a
//! synced folder of one group, which already puts its files on every
//! member's computer by hash. Each computer holding it lends it as its own,
//! to that group alone ([`Folders::carried`], a [`crate::lend::Source::Carried`] lend), so
//! it opens while any of them is on; the door answers it here
//! ([`Folders::open`]), where a port's is carried to its service:
//!
//! - a file, from the store, by the hash the group's layout places at its
//!   path, `index.html` for a directory's;
//! - [`ACTS`], the app's route: `GET` its own acts — each whose `kind`
//!   begins `<kind>/`, the prefix taken off — in the record's order, as
//!   `{id, ts, by, op}`, with who asks, `you`, and the people in the group;
//!   `POST {op}` records an act, and `{corrects, op}` corrects one of its
//!   own, `op` null to void it: the prefix put on its kind, and
//!   `on_behalf_of` the person the door resolved, never a word of the
//!   body's.
//!
//! The route answers every act each time, from no cursor: a round may
//! place another computer's act before one already read, and a correction
//! voids an act read before, so the app asks again and folds them all.
//! Every answer carries [`FLOOR`], so a carried page loads nothing from
//! elsewhere (COMMUNITY_APPS §6.2). Upstream's policy, changed: its
//! connect-src is the page's own origin, `base-uri` none, a form posts and
//! a frame and a worker load only there.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::header::{self, HeaderName, HeaderValue};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use iroh_blobs::Hash;
use serde_json::{json, Map, Value};
use svrngs_core::Placed;

use super::Folders;
use crate::door::LendRefusal::{self, Away, Closed, Fault, Path as Malformed, Unlent};
use crate::lend::{Kind, Lend};
use crate::register::{Member, Standing};

/// Where a carried app's page asks its own acts, and writes one, at its
/// own origin.
pub const ACTS: &str = "/svrngs/acts";

/// The most a carried app's write reads: an act's most, and its envelope.
const WRITE_MOST: usize = svrngs_core::MAX_PAYLOAD_BYTES + 1024;

/// The containment floor (COMMUNITY_APPS §6.2), on every answer a carried
/// app is given here.
const FLOOR: [(&str, &str); 3] = [
    ("content-security-policy", POLICY),
    (
        "permissions-policy",
        "camera=(), microphone=(), geolocation=()",
    ),
    ("x-dns-prefetch-control", "off"),
];
const POLICY: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
    img-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'none'; form-action 'self'; \
    frame-src 'self'; worker-src 'self'";

/// Why the door answers a carried app's request itself: its kind, status
/// and words, said and traced once ([`Folders::open`]).
type Refused = (LendRefusal, u16, &'static str);
const UNREAD: Refused = (Fault, 500, "its group cannot be read on this computer");
const NOT_HERE: Refused = (Unlent, 404, "nothing is here by that name");
const NOT_YET: Refused = (Away, 503, "this computer does not hold it yet");
const NOT_AN_ACT: Refused = (Malformed, 422, "an act is JSON, {op} or {corrects, op}");
const NOT_ITS_OWN: Refused = (Closed, 403, "an app corrects only its own acts");
const NOT_ITS_GROUP: Refused = (Closed, 403, "this is its own group's alone");

/// A carried app's answer, or why the door gives its own.
type Answer = Result<Response, Refused>;

/// The people standing in a group, by key.
type People = BTreeMap<String, Member>;

/// The kind of the app whose page `placed` is, `apps/<kind>/index.html`;
/// none for any other file, or one beside it as a conflict.
fn app_of(placed: &Placed) -> Option<&str> {
    let kind = (placed.path.as_str().strip_prefix("apps/")?).strip_suffix("/index.html")?;
    (!placed.beside && !kind.contains('/')).then_some(kind)
}

impl Folders {
    /// Every app carried in a group this computer stands in and keeps a
    /// folder of in step, so holds its files, each once by its group and
    /// kind, wherever the group's layout places its page, as a lend
    /// ([`Lend::carried`]). A group whose record cannot be read carries
    /// nothing here, said at warn.
    pub(crate) fn carried(&self) -> Vec<Lend> {
        let (me, groups) = (self.host.actor(), self.host.standings().unwrap_or_default());
        let mut found = BTreeMap::new();
        let syncing = self.syncing.borrow().clone().unwrap_or_default();
        let holds = |group: &String| syncing.keys().any(|(g, _)| g == group);
        for (group, _) in groups
            .iter()
            .filter(|(g, m)| m.contains_key(me) && holds(g))
        {
            let read = self.layout(group).inspect_err(|why| tracing::warn!(target: "folder", namespace = group, %why, "carried: the group's record cannot be read, so it carries nothing here"));
            let apps = read.iter().flat_map(|r| r.placed.iter().filter_map(app_of));
            let lends = apps.filter_map(|kind| Lend::carried(kind, group));
            found.extend(lends.map(|l| (l.name.clone(), l)));
        }
        tracing::debug!(target: "folder", found = ?found.keys(), "carried: the apps the groups' folders carry here");
        found.into_values().collect()
    }

    /// `req`, asked at `path` of the app `lend` carried in `group` by the
    /// key `key` (the door's): refused unless that key stands in the group,
    /// else the route's answer or a file's; each under [`FLOOR`].
    pub(crate) async fn open(&self, lend: &Lend, at: (&str, &str, &str), req: Request) -> Response {
        let (group, key, path) = at;
        let answered = match self.host.rail().standing(group) {
            Ok(Standing::Members(people)) if people.contains_key(key) => match path {
                ACTS => self.acts((&lend.kind, group), (&people, key), req).await,
                _ => self.file(group, &format!("apps/{}{path}", lend.kind)).await,
            },
            Ok(_) => Err(NOT_ITS_GROUP),
            Err(_) => Err(UNREAD),
        };
        let refused = answered.as_ref().err().map(|r| r.2);
        tracing::debug!(target: "folder", lend = %lend.name, group, key, path, ?refused, "carried: answered, or refused and why");
        let mut answer = answered.unwrap_or_else(|(kind, status, why)| {
            kind.say(StatusCode::from_u16(status).expect("a status"), why)
        });
        let floor = FLOOR.map(|(n, v)| (HeaderName::from_static(n), HeaderValue::from_static(v)));
        answer.headers_mut().extend(floor);
        answer
    }

    /// The file the group's layout places at `path`, `index.html` for a
    /// directory's, from this computer's store.
    async fn file(&self, group: &str, path: &str) -> Answer {
        let path = (path.strip_suffix('/')).map_or(path.into(), |dir| format!("{dir}/index.html"));
        let read = self.layout(group).map_err(|_| UNREAD)?;
        let at = |p: &&Placed| !p.beside && p.path.as_str() == path;
        let hash = read.placed.iter().find(at).ok_or(NOT_HERE)?.hash;
        let held = (self.store.get_bytes(Hash::from(*hash.as_bytes()))).await;
        let kind = content_type(Path::new(&path));
        Ok(([(header::CONTENT_TYPE, kind)], held.map_err(|_| NOT_YET)?).into_response())
    }

    /// The app `kind`'s route in `group`, for the key `key` of `people`:
    /// its own acts read, or one written as that key's person (the module
    /// docs).
    async fn acts(&self, app: (&Kind, &str), asker: (&People, &str), req: Request) -> Answer {
        let ((kind, group), (people, key)) = (app, asker);
        let (prefix, rail, person) = (format!("{kind}/"), self.host.rail(), &people[key].person);
        let read = (rail.admission(group).ok()).and_then(|a| serde_json::to_value(a.wire()).ok());
        let ops = read.as_ref().and_then(Value::as_array).ok_or(UNREAD)?;
        let own =
            |o: &Value| Some((o["payload"]["kind"].as_str()?.strip_prefix(&prefix)?).to_string());
        if req.method() == Method::GET {
            let acts = (ops.iter().filter(|o| o["voided"] == false)).filter_map(|o| {
                let (mut op, by) = (o["payload"].clone(), o.get("on_behalf_of"));
                op["kind"] = own(o)?.into();
                let by = by.unwrap_or(&o["person"]);
                Some(json!({ "id": o["id"], "ts": o["ts_unix"], "by": by, "op": op }))
            });
            let all: BTreeSet<_> = people.values().map(|m| &m.person).collect();
            let acts: Vec<Value> = acts.collect();
            return Ok(Json(json!({ "you": person, "people": all, "acts": acts })).into_response());
        }
        let body = (to_bytes(req.into_body(), WRITE_MOST).await).map_err(|_| NOT_AN_ACT)?;
        let mut body: Map<String, Value> = serde_json::from_slice(&body).map_err(|_| NOT_AN_ACT)?;
        let mut op = body.remove("op").unwrap_or(Value::Null);
        if let Some(fields) = op.as_object_mut() {
            let own_kind = (fields.get("kind").and_then(Value::as_str)).ok_or(NOT_AN_ACT)?;
            fields.insert("kind".into(), format!("{prefix}{own_kind}").into());
        }
        let mut act = match body.remove("corrects") {
            None => json!({ "op": "record", "payload": op }),
            Some(id) if ops.iter().any(|o| o["id"] == id && own(o).is_some()) => {
                json!({ "op": "correct", "corrects": id, "replacement": op })
            }
            Some(_) => return Err(NOT_ITS_OWN),
        };
        act["on_behalf_of"] = person.as_str().into();
        let answer = crate::rail::append_act(rail, group, act).await;
        tracing::debug!(target: "folder", group, %kind, %person, status = answer.status().as_u16(), "carried: an act written as the person");
        if answer.status().is_success() {
            self.nudge.notify_one();
        }
        Ok(answer)
    }
}

fn content_type(file: &Path) -> &'static str {
    match file.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}
