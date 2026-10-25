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
//! - a file, `index.html` for a directory's, by the hash the app was
//!   brought with at its path ([`svrngs_core::Runs`]; canon-pin), never by
//!   what a member's later change placed there: from the store, else
//!   fetched from a member's;
//! - [`ACTS`], the app's route: `GET` its own acts — each whose `kind`
//!   begins `<kind>/`, the prefix taken off, and each correction of one —
//!   as `ops`, in the record's order and the shape `/v1/rail/log` gives
//!   them, voided ones too, so an app's fold reads them as it reads the
//!   record; with who asks, `you`, the people in the group now, and
//!   `spans`, each person's time in it, `{person, joined, left}`, from
//!   svrngs-core's membership (canon-spans), null while who stands is
//!   unsettled (#124);
//!   `POST {op}` records an act, and `{corrects, op}` corrects one of its
//!   own, `op` null to void it, and absent never (#165): the prefix put on its kind, and
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
use std::fs::{create_dir_all, write};
use std::path::Path;

use axum::body::to_bytes;
use axum::extract::{Request, State};
use axum::http::header::{self, HeaderName, HeaderValue};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use iroh_blobs::Hash;
use serde_json::{json, Map, Value};
use svrngs_core::{Files, Placed, SyncPath};

use super::{blob::People, dirs_of, Folders};
use crate::door::LendRefusal::{self, Away, Closed, Fault, Path as Malformed, Unlent};
use crate::iroh::Dialer;
use crate::lend::{Kind, Lend};
use crate::node::Asking;
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
type Withheld = (LendRefusal, u16, &'static str);
const UNREAD: Withheld = (Fault, 500, "its group cannot be read on this computer");
const NOT_HERE: Withheld = (Unlent, 404, "nothing is here by that name");
const NOT_YET: Withheld = (Away, 503, "this computer does not hold it yet");
const UNSTORED: Withheld = (Fault, 500, "this computer cannot read it from its store");
const NOT_AN_ACT: Withheld = (Malformed, 422, "an act is JSON, {op} or {corrects, op}");
const TOO_LARGE: Withheld = (Malformed, 413, "the act is too large, or came cut short");
const NOT_ITS_OWN: Withheld = (Closed, 403, "an app corrects only its own acts");
const NOT_ITS_GROUP: Withheld = (Closed, 403, "this is its own group's alone");
const NOT_IN: Withheld = (
    Closed,
    403,
    "you can bring things only to a group you’re in",
);
const NO_FOLDER: Withheld = (Closed, 409, "this computer keeps no folder of that group");
const UNWRITTEN: Withheld = (Fault, 500, "your group’s folder can’t be written here");

/// svrngs's own answer for `refused`.
fn said((kind, status, why): Withheld) -> Response {
    kind.say(StatusCode::from_u16(status).expect("a status"), why)
}

/// `refused`, for `group`, its cause traced, which the asker is not told
/// (#163, #164).
fn traced(group: &str, refused: Withheld, why: &dyn std::fmt::Display) -> Withheld {
    tracing::warn!(target: "folder", group, %why, said = refused.2, "carried: refused, and the cause");
    refused
}

/// A carried app's answer, or why the door gives its own.
type Given = Result<Response, Withheld>;

/// The shelf (V1_UI_BUILD §3.4; v1ui-shelf (c)): svrngs's own reference
/// apps, which ship with it, and nothing else — nothing on it is a
/// stranger's (COMMUNITY_APPS §6.2). Each its name, what it is in a
/// sentence, and its files, as `src/shelf.txt` lists them and build.rs
/// embeds them.
const SHELF: &[(&str, &str, &[File])] = include!(concat!(env!("OUT_DIR"), "/shelf.rs"));

/// A file an app brings: its name in the app's folder, and its text.
type File = (&'static str, &'static str);

/// The shelf as the pages list it, beside what a group's people lend.
pub(crate) fn shelf() -> Value {
    (SHELF.iter())
        .map(|(name, words, _)| json!({ "name": name, "words": words }))
        .collect()
}

/// What a page asks brought to a group's table: the group's id, and the
/// shelf's apps by name.
type Ticks = Json<(String, Vec<String>)>;

/// `POST /shelf`: the shelf's apps a page names brought into its group's
/// folder on this computer ([`brought`]), from the home page or from a
/// phone of the group's dialling it (`by`).
pub(crate) async fn bring(at: State<Asking>, by: Option<Extension<Dialer>>, a: Ticks) -> Response {
    let (State(at), Json((group, apps))) = (at, a);
    let asker = by.map_or(at.host.actor.to_string(), |Extension(d)| d.actor());
    let brought = brought(&at, &group, &asker, &apps);
    tracing::debug!(target: "folder", group, asker, ?apps, status = ?brought.as_ref().map(|b| b.status()), "shelf: the apps asked brought to the group, or why none");
    brought.unwrap_or_else(said)
}

/// Each of `apps` on the shelf brought into `group`'s folder here for
/// `asker`, who stands in it: a folder `apps/<name>/` of its files, never
/// over one there already, which the folder then carries to the group as
/// it carries any file. Each said, and whether it was there already.
fn brought(at: &Asking, group: &str, asker: &str, apps: &[String]) -> Given {
    let stands = at.host.rail.standing(group).is_ok_and(|s| s.names(asker));
    stands.then_some(()).ok_or(NOT_IN)?;
    let dirs = dirs_of(&at.folders.data_dir, group).map_err(|e| traced(group, NO_FOLDER, &e))?;
    let dir = dirs.first().ok_or(NO_FOLDER)?;
    let mut said = Vec::new();
    for (name, _, files) in SHELF.iter().filter(|(n, ..)| apps.iter().any(|a| a == n)) {
        let at = dir.join("apps").join(name);
        let there = at.join("index.html").exists();
        let put = |(f, text): &(&str, &str)| {
            create_dir_all(at.join(f).with_file_name("")).and_then(|()| write(at.join(f), text))
        };
        let wrote: Result<(), _> = files.iter().filter(|_| !there).try_for_each(put);
        wrote.map_err(|e| traced(group, UNWRITTEN, &e))?;
        said.push(json!({ "app": name, "already": there }));
    }
    Ok(Json(said).into_response())
}

/// What a page says yes to: the group's id, and the app by its kind.
type Yes = Json<(String, String)>;

/// svrngs's answer, or its refusal.
type Handled = Result<Response, Response>;

/// `POST /runs`: the asker's yes to the version of an app carried in their
/// group that its files now are (canon-pin (c)), from the home page,
/// written as this computer's person; or, to a phone of the group's
/// dialling it (`by`), the act for the phone to sign with its own key and
/// carry back ([`crate::rail::carry`]): Decide is the person's own devices
/// alone (capability-signed).
pub(crate) async fn yes(at: State<Asking>, by: Option<Extension<Dialer>>, a: Yes) -> Handled {
    let (State(at), Json((group, app))) = (at, a);
    let asker = (by.as_ref()).map_or(at.host.actor.to_string(), |Extension(d)| d.actor());
    let act = says(&at, (&group, &app), &asker);
    tracing::debug!(target: "folder", group, asker, app, phone = by.is_some(), refused = act.as_ref().err().map(|r| r.2), "carried: a yes to an app's files now, to write here or for a phone to sign, or why not");
    let act = act.map_err(said)?;
    if by.is_some() {
        return Ok(Json(json!({ "sign": act })).into_response());
    }
    at.folders.nudge.notify_one();
    Ok(crate::rail::append_act(&at.host.rail, &group, act).await)
}

/// The act that says `asker`'s yes to the files of `app` in `group` now.
fn says(at: &Asking, (group, app): (&str, &str), asker: &str) -> Result<Value, Withheld> {
    let unread = |e: &dyn std::fmt::Display| traced(group, UNREAD, e);
    let standing = at.host.rail.standing(group).map_err(|e| unread(&e))?;
    standing.member(asker).ok_or(NOT_IN)?;
    let read = at.folders.layout(group).map_err(|e| unread(&e))?;
    let files = now(&read.placed, app);
    (!files.is_empty()).then_some(()).ok_or(NOT_HERE)?;
    Ok(json!({ "op": "runs", "app": app, "files": files }))
}

/// The files of the app `kind` that `placed` places now, under its path.
fn now(placed: &[Placed], kind: &str) -> Files {
    let under = |p: &&Placed| !p.beside && p.path.as_str().starts_with(&format!("apps/{kind}/"));
    let file = |p: &Placed| (p.path.clone(), p.hash);
    placed.iter().filter(under).map(file).collect()
}

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
        let (me, groups) = (&self.host.actor, self.host.standings().unwrap_or_default());
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

    /// Each app carried in `groups` whose files now are not those that run
    /// as the person of the key `asker` there (canon-pin (c)): its group,
    /// its kind, who changed its files last and when, and its files now,
    /// which a yes names; `null` when a group's record cannot be read here,
    /// said at warn, never as nothing changed.
    pub(crate) fn changed(&self, groups: &[(String, People)], asker: &str) -> Value {
        let mut out = Vec::new();
        for (group, people) in groups.iter().filter(|(_, p)| p.contains_key(asker)) {
            let Ok(read) = self.layout(group).map_err(|e| traced(group, UNREAD, &e)) else {
                return Value::Null;
            };
            let person = people[asker].person.as_str();
            let new = |k: &&str| read.runs.of(k, person) != Some(&now(&read.placed, k));
            for kind in read.placed.iter().filter_map(app_of).filter(new) {
                let (by, at) = &read.runs.changed[kind];
                out.push(json!({ "group": group, "app": kind, "by": by, "at": at }));
            }
        }
        tracing::debug!(target: "folder", asker, changed = out.len(), "carried: the apps whose files now are not those that run as the asker");
        Value::Array(out)
    }

    /// `req`, asked at `path` of the app `lend` carried in `group` by the
    /// key `key` (the door's): refused unless that key stands in the group,
    /// else the route's answer or a file's; each under [`FLOOR`].
    pub(crate) async fn open(&self, lend: &Lend, at: (&str, &str, &str), req: Request) -> Response {
        let (group, key, path) = at;
        let answered = match self.host.rail.standing(group) {
            Ok(Standing::Members(people)) if people.contains_key(key) => match path {
                ACTS => self.acts((&lend.kind, group), (&people, key), req).await,
                _ => self.file(group, &lend.kind, path, &people[key]).await,
            },
            Ok(_) => Err(NOT_ITS_GROUP),
            Err(e) => Err(traced(group, UNREAD, &e)),
        };
        let refused = answered.as_ref().err().map(|r| r.2);
        tracing::debug!(target: "folder", lend = %lend.name, group, key, path, ?refused, "carried: answered, or refused and why");
        let mut answer = answered.unwrap_or_else(said);
        let floor = FLOOR.map(|(n, v)| (HeaderName::from_static(n), HeaderValue::from_static(v)));
        answer.headers_mut().extend(floor);
        answer
    }

    /// The file at `path` of the app `kind` carried in `group`, `index.html`
    /// for a directory's, as the app runs as `asker`'s person
    /// ([`svrngs_core::Runs::of`]), from this computer's store, else
    /// fetched from a computer that stands in the group.
    async fn file(&self, group: &str, kind: &Kind, path: &str, asker: &Member) -> Given {
        let path = format!("apps/{kind}{path}");
        let path = (path.strip_suffix('/')).map_or(path.clone(), |dir| format!("{dir}/index.html"));
        let read = self.layout(group).map_err(|e| traced(group, UNREAD, &e))?;
        let files = read.runs.of(kind.as_str(), asker.person.as_str());
        let at = SyncPath::parse(&path)
            .ok()
            .and_then(|p| files?.get(&p).copied());
        let hash = Hash::from(*at.ok_or(NOT_HERE)?.as_bytes());
        let holders: Vec<_> = self.holders(group).into_iter().flatten().collect();
        let fetched = self.hold(hash, &holders).await;
        let held = self.store.get_bytes(hash).await;
        // Not held yet only when the store says it lacks the file; else it failed to read it.
        let lacks = held.is_err() && matches!(self.store.has(hash).await, Ok(false));
        tracing::debug!(target: "folder", path, %hash, lacks, ?fetched, why = ?held.as_ref().err(), "carried: the file as the app runs as the person, from the store, or why not");
        let held = held.map_err(|_| if lacks { NOT_YET } else { UNSTORED })?;
        let kind = content_type(Path::new(&path));
        Ok(([(header::CONTENT_TYPE, kind)], held).into_response())
    }

    /// The app `kind`'s route in `group`, for the key `key` of `people`:
    /// its own acts read, or one written as that key's person (the module
    /// docs).
    async fn acts(&self, app: (&Kind, &str), asker: (&People, &str), req: Request) -> Given {
        let ((kind, group), (people, key)) = (app, asker);
        let (prefix, rail, person) = (format!("{kind}/"), &self.host.rail, &people[key].person);
        let admission = rail.admission(group).ok();
        let read = (admission.as_ref()).and_then(|a| serde_json::to_value(a.wire()).ok());
        let ops = read.as_ref().and_then(Value::as_array).ok_or(UNREAD)?;
        let own =
            |o: &Value| Some((o["payload"]["kind"].as_str()?.strip_prefix(&prefix)?).to_string());
        if req.method() == Method::GET {
            let mine: BTreeSet<_> = (ops.iter().filter(|o| own(o).is_some()))
                .filter_map(|o| o["id"].as_str())
                .collect();
            let voids_mine = |o: &Value| o["corrects"].as_str().is_some_and(|id| mine.contains(id));
            let ops = (ops.iter().filter(|o| own(o).is_some() || voids_mine(o))).map(|o| {
                let mut o = o.clone();
                if let Some(kind) = own(&o) {
                    o["payload"]["kind"] = kind.into();
                }
                o
            });
            let (all, ops): (BTreeSet<_>, Vec<_>) =
                (people.values().map(|m| &m.person).collect(), ops.collect());
            // Who stood when is not judged on unsettled cuts (#124): none, never empty.
            let register = (admission.as_ref())
                .map(|a| a.membership())
                .filter(|m| !m.unsettled);
            let spans = register.map(|m| m.spans());
            tracing::debug!(target: "folder", group, %kind, spans = ?spans.as_ref().map(Vec::len), "carried: each person's time in the group, or none, their standing unsettled");
            let read =
                json!({ "you": person, "group": group, "people": all, "spans": spans, "ops": ops });
            return Ok(Json(read).into_response());
        }
        // Another member's computer cannot sign as them (capability-signed):
        // a computer writes on its own copy, and a phone, for now, here.
        let elsewhere = key != self.host.actor && people[key].device.dialled();
        tracing::debug!(target: "folder", group, %kind, %person, elsewhere, "carried: a write as the person, refused from another member's computer");
        if elsewhere {
            let why = format!("Open {kind} from your home page to write in it.");
            return Ok(Closed.say(StatusCode::FORBIDDEN, why));
        }
        let body = (to_bytes(req.into_body(), WRITE_MOST).await).map_err(|_| TOO_LARGE)?;
        let mut body: Map<String, Value> = serde_json::from_slice(&body).map_err(|_| NOT_AN_ACT)?;
        let mut op = body.remove("op").ok_or(NOT_AN_ACT)?;
        if let Some(fields) = op.as_object_mut() {
            let own_kind = (fields.get("kind").and_then(Value::as_str)).ok_or(NOT_AN_ACT)?;
            fields.insert("kind".into(), format!("{prefix}{own_kind}").into());
        }
        let act = match body.remove("corrects") {
            None => json!({ "op": "record", "payload": op }),
            Some(id) if ops.iter().any(|o| o["id"] == id && own(o).is_some()) => {
                json!({ "op": "correct", "corrects": id, "replacement": op })
            }
            Some(_) => return Err(NOT_ITS_OWN),
        };
        let answer = crate::rail::append_as(rail, group, act, Some(person.clone())).await;
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
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}
