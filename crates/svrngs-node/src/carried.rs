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
//!   record; with who asks, `you`, and their first key, `you_key`, which
//!   the fold keeps them by (keys-names), the people in the group now, and
//!   `spans`, each person's time in it, `{person, joined, left,
//!   person_key}`, from svrngs-core's membership (canon-spans), null while
//!   who stands is unsettled (#124); and `page`, the page the group's https
//!   links print on (`Host::printed_page`), null where none is kept and
//!   `{unread}` where it cannot be read (give-page-host);
//!   `POST {op}` records an act, and `{corrects, op}` corrects one of its
//!   own, `op` null to void it, and absent never (#165): the prefix put on its kind
//!   ([`svrngs_core::grant::app_act`]), signed by this computer as its own
//!   person, and refused for any other device's, which signs its own and
//!   carries it (`/carry`, capability-signed).
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
use axum::extract::{Request, State};
use axum::http::header::{self, HeaderName, HeaderValue};
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use iroh_blobs::Hash;
use serde_json::{json, Value};
use svrngs_core::{Admission, Append, AppendRefusal, Command, Files, Op, Placed};
use svrngs_core::{RailAct, Reads, SignedOp, SyncPath};

use super::{blob::People, dirs_of, Folders};
use crate::door::LendRefusal::{self, Away, Closed, Fault, Path as Malformed, Unlent};
use crate::iroh::Dialer;
use crate::lend::{AppName, Lend, Program, Source};
use crate::node::Asking;
use crate::register::Standing;
use crate::surface::AppRoute;

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

/// `POST /v1/programs`, `[group, word]`: the text of `programs/<word>.txt`
/// in `group` and the name of who wrote it ([`Folders::terms`]), both empty
/// for none; or, 409, why not.
pub(super) async fn terms(State(f): State<Folders>, Json(at): Json<(String, Program)>) -> Response {
    match f.terms(&at.0, &at.1).await {
        Ok(said) => Json(said).into_response(),
        Err(why) => (StatusCode::CONFLICT, why).into_response(),
    }
}

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

/// What the door asks of a carried app: the key asking, the path asked,
/// and the route that path names, parsed once by the door.
pub(crate) type Opened<'a> = (&'a str, &'a str, AppRoute);

/// A carried app's answer, or why the door gives its own.
type Given = Result<Response, Withheld>;

/// Who asks: the people standing in the group, and the key asking among
/// them.
type Who<'a> = (&'a People, &'a str);

/// A file of a carried app, none where its files have none, or why not.
type Held = Result<Option<axum::body::Bytes>, Withheld>;

/// Where a carried app declares the kinds of lend it reads, beside its
/// `index.html`: each a kind, apart by spaces or lines (DESIGN §3, "Or a
/// folder the group carries").
pub const READS: &str = "reads.txt";
const NOT_PROGRAMS: Withheld = (Fault, 500, "a word in its reads.txt isn’t a program’s name");

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
    let dirs =
        dirs_of(&at.door.folders.data_dir, group).map_err(|e| traced(group, NO_FOLDER, &e))?;
    let own = crate::surface::seed_of(at, group).map(|s| crate::group::folder_name(&s));
    let dir = dirs
        .iter()
        .find(|d| own == Some(super::synced::folder_of(d, group)));
    tracing::debug!(target: "folder", group, ?own, ?dir, kept = dirs.len(), "shelf: brought into the group's own folder, never another it shares; refused when this computer keeps none of its own");
    let dir = dir.ok_or(NO_FOLDER)?;
    let root = super::confined::Root::open(dir).map_err(|e| traced(group, UNWRITTEN, &e))?;
    let mut said = Vec::new();
    for (name, _, files) in SHELF.iter().filter(|(n, ..)| apps.iter().any(|a| a == n)) {
        let at = Path::new("apps").join(name);
        let there = root.metadata(&at.join("index.html")).is_ok();
        let put = |(f, text): &(&str, &str)| root.write(&at.join(f), text.as_bytes());
        let wrote: Result<(), _> = files.iter().filter(|_| !there).try_for_each(put);
        wrote.map_err(|e| traced(group, UNWRITTEN, &e))?;
        said.push(json!({ "app": name, "already": there }));
    }
    Ok(Json(said).into_response())
}

/// What a page says yes to: the group's id, and the app by its kind.
type Yes = Json<(String, String)>;

/// svrngs's answer, or its refusal.
pub(crate) type Handled = Result<Response, Response>;

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
    at.door.folders.nudge.notify_one();
    Ok(crate::rail::append_act(&at.host.rail, &group, act, None).await)
}

/// The act that says `asker`'s yes to the files of `app` in `group` now.
fn says(at: &Asking, (group, app): (&str, &str), asker: &str) -> Result<Value, Withheld> {
    let unread = |e: &dyn std::fmt::Display| traced(group, UNREAD, e);
    let standing = at.host.rail.standing(group).map_err(|e| unread(&e))?;
    standing.member(asker).ok_or(NOT_IN)?;
    let read = at.door.folders.layout(group).map_err(|e| unread(&e))?;
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

/// The files of the app `app` that run as `asker`, the person of `people`
/// whose first key it is: those they last said yes to, else those it was
/// brought with ([`svrngs_core::Runs::of`]); or its files now, when every
/// file that differs from those is one a device of theirs placed, and none
/// is gone (svrn-docs `rings/current/EXTENSION_POINT.md` §7, G5). So an
/// author's own change runs for them at once, as an agent editing and
/// retrying needs, and anyone else's change still runs for them only after
/// their yes: rule 2 is untouched, and so is everyone else's yes.
fn runs_as(read: &super::Layout, (people, key): Who, app: &str) -> Option<Files> {
    let first = people.get(key)?.person_key.as_str();
    let (said, now) = (read.runs.of(app, first)?, now(&read.placed, app));
    let mine = |k: &String| people.get(k).is_some_and(|m| m.person_key == first);
    let new = |p: &&Placed| now.get(&p.path) == Some(&p.hash) && said.get(&p.path) != Some(&p.hash);
    let gone = said.keys().any(|path| !now.contains_key(path));
    let own = !gone && read.placed.iter().filter(new).all(|p| mine(&p.entry.key));
    tracing::debug!(target: "folder", app, first, own, "carried: the app's files now run as the person when every change since their yes is their own, else their yes");
    Some(if own { now } else { said.clone() })
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
        for (group, _) in (groups.iter()).filter(|(g, m)| m.contains_key(me) && holds(g)) {
            let read = self.layout(group).inspect_err(|why| tracing::warn!(target: "folder", namespace = group, %why, "carried: the group's record cannot be read, so it carries nothing here"));
            let apps = read.iter().flat_map(|r| r.placed.iter().filter_map(app_of));
            let lends = apps.filter_map(|kind| Lend::carried(kind, group));
            found.extend(lends.map(|l| (l.name.clone(), l)));
        }
        tracing::debug!(target: "folder", found = ?found.keys(), "carried: the apps the groups' folders carry here");
        found.into_values().collect()
    }

    /// The text of `programs/<word>.txt` in `group` where its record places
    /// it, and the name of who wrote it, from this computer's store, else fetched from a computer that
    /// stands in the group, never read from a directory, so half a sync is
    /// never read as terms (CONTRACTS §2, "Reading a contract"); empty when
    /// it places none. Refused, never read as no terms, and said with what to
    /// do: two copies in conflict, a writer who no longer stands in the
    /// group, bytes not here yet, or terms a group's file can't hold
    /// ([`crate::share::fits`]).
    async fn terms(&self, group: &str, word: &Program) -> Result<(String, String), String> {
        let path = format!("programs/{word}.txt");
        let read = self.layout(group).map_err(|e| e.to_string())?;
        let holders = self.holders(group).map_err(|e| e.to_string())?;
        let at = |p: &&Placed| p.entry.path.as_str() == path;
        let copies: Vec<&Placed> = read.placed.iter().filter(at).collect();
        let standing = copies.iter().all(|p| holders.contains(&p.entry.key));
        tracing::debug!(target: "folder", group, path, copies = copies.len(), standing, "carried: a group's programs file, as its record places it");
        let (hash, key) = match copies[..] {
            [] => return Ok(Default::default()),
            [_, _, ..] => return Err(format!("{path} has two copies in conflict. Keep the one you mean, delete the other, then share again.")),
            [_] if !standing => return Err(format!("{path} was last written by someone no longer in the group. Someone in it writes it again, or deletes it, then share again.")),
            [one] => (Hash::from(*one.hash.as_bytes()), &one.entry.key),
        };
        let named =
            (self.host.rail.standing(group)).map(|s| Some(s.member(key)?.person.to_string()));
        let fetched = self.hold(hash, &Vec::from_iter(holders)).await;
        let held = self.store.get_bytes(hash).await;
        tracing::debug!(target: "folder", group, path, %hash, ?fetched, why = ?held.as_ref().err(), "carried: a group's programs file from the store, or why not");
        let held = held.map_err(|_| format!("{path} isn't on this computer yet: it is still being fetched from the group. Wait a moment, then share again."))?;
        let text = String::from_utf8(held.to_vec()).map_err(|_| format!("{path} is not text."));
        let by = named.ok().flatten().unwrap_or_default();
        let fits = text.and_then(|t| crate::share::fits(word, &t).map(|()| (t, by)));
        fits.map_err(|why| format!("{path} can't be used: {why}. Change it, then share again."))
    }

    /// The kinds `lend` reads: a carried app's from its [`READS`] as its
    /// files run as this computer's person, so a version that reads more
    /// reads it only once they said yes to it (canon-pin); none without
    /// one, as the shelf's apps have; refused, never read as fewer, when a
    /// word in it is not a kind (ARCH 6). A declared lend's, its own.
    pub(crate) async fn reads(&self, lend: &Lend) -> Result<Vec<Program>, Withheld> {
        let Source::Carried(app, group) = &lend.source else {
            return Ok(lend.reads.clone());
        };
        let standing = (self.host.rail.standing(group)).map_err(|e| traced(group, UNREAD, &e))?;
        standing.member(&self.host.actor).ok_or(NOT_IN)?;
        let Standing::Members(people) = &standing else {
            return Err(NOT_IN);
        };
        let path = format!("apps/{app}/{READS}");
        let text = self
            .runs_at(group, app, &path, (people, &self.host.actor))
            .await?;
        let words = String::from_utf8_lossy(text.as_deref().unwrap_or_default());
        let kinds = (words.split_whitespace())
            .map(Program::parse)
            .collect::<Result<_, _>>();
        tracing::debug!(target: "folder", lend = %lend.name, group, declared = text.is_some(), ?kinds, "carried: the kinds the app reads as it runs as this computer's person, none undeclared, or refused");
        kinds.map_err(|e| traced(group, NOT_PROGRAMS, &e))
    }

    /// Each app carried in `groups` whose files now are not those that run
    /// as the person of the key `asker` there (canon-pin (c)): its group,
    /// its kind, who changed its files last, by name and first key, and
    /// when, and its files now,
    /// which a yes names; `null` when a group's record cannot be read here,
    /// said at warn, never as nothing changed.
    pub(crate) fn changed(&self, groups: &[(String, People)], asker: &str) -> Value {
        let mut out = Vec::new();
        for (group, people) in groups.iter().filter(|(_, p)| p.contains_key(asker)) {
            let Ok(read) = self.layout(group).map_err(|e| traced(group, UNREAD, &e)) else {
                return Value::Null;
            };
            let new = |k: &&str| runs_as(&read, (people, asker), k) != Some(now(&read.placed, k));
            for kind in read.placed.iter().filter_map(app_of).filter(new) {
                let (key, by, at) = &read.runs.changed[kind];
                out.push(json!({ "group": group, "app": kind, "by": by, "by_key": key, "at": at }));
            }
        }
        tracing::debug!(target: "folder", asker, changed = out.len(), "carried: the apps whose files now are not those that run as the asker");
        Value::Array(out)
    }

    /// What `svrngs check` finds in the app `kind` in a folder of `group`
    /// here, in words for its card, so a page the policy blanks is told so
    /// before anyone opens it (svrn-docs `rings/current/EXTENSION_POINT.md`
    /// §6.3); none when it finds nothing, or no folder here holds its page.
    pub(crate) fn checked(&self, group: &str, kind: &str) -> Option<String> {
        let dirs = dirs_of(&self.data_dir, group).unwrap_or_default();
        let mut apps = dirs.iter().map(|d| d.join("apps").join(kind));
        let app = apps.find(|d| d.join("index.html").is_file());
        let found = app
            .as_deref()
            .and_then(super::check::check)
            .unwrap_or_default();
        let words: BTreeSet<_> = found.iter().map(|(.., refused)| refused.words()).collect();
        tracing::debug!(target: "folder", group, kind, ?app, found = found.len(), "carried: what svrngs check finds in the app's folder here, for its card");
        (!words.is_empty()).then(|| words.into_iter().collect::<Vec<_>>().join(" "))
    }

    /// `req`, asked at `path` of the app `lend` carried in `group` by the
    /// key `key` (the door's), the route it names `route`: refused unless
    /// that key stands in the group, else the route's answer or a file's;
    /// each under [`FLOOR`].
    pub(crate) async fn open(&self, lend: &Lend, at: Opened<'_>, req: Request) -> Response {
        let (Source::Carried(app, group), (key, path, route)) = (&lend.source, at) else {
            return said(NOT_ITS_GROUP);
        };
        let answered = match self.host.rail.standing(group) {
            Ok(Standing::Members(people)) if people.contains_key(key) => match route {
                AppRoute::Acts => self.acts((app, group), (&people, key), req).await,
                _ => self.file(group, app, path, (&people, key)).await,
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
    /// ([`runs_as`]), from this computer's store, else fetched from a
    /// computer that stands in the group.
    async fn file(&self, group: &str, app: &AppName, path: &str, asker: Who<'_>) -> Given {
        let path = format!("apps/{app}{path}");
        let path = (path.strip_suffix('/')).map_or(path.clone(), |dir| format!("{dir}/index.html"));
        let held = self.runs_at(group, app, &path, asker).await?;
        let kind = content_type(Path::new(&path));
        Ok(([(header::CONTENT_TYPE, kind)], held.ok_or(NOT_HERE)?).into_response())
    }

    /// The bytes at `path` of the app `kind` carried in `group` as it runs
    /// as `asker`, the person of `people` whose first key it is ([`runs_as`]);
    /// none when those files have nothing there.
    async fn runs_at(&self, group: &str, app: &AppName, path: &str, asker: Who<'_>) -> Held {
        let read = self.layout(group).map_err(|e| traced(group, UNREAD, &e))?;
        let files = runs_as(&read, asker, app.as_str());
        let at = (SyncPath::parse(path).ok()).and_then(|p| files?.get(&p).copied());
        let Some(at) = at else { return Ok(None) };
        let hash = Hash::from(*at.as_bytes());
        let holders: Vec<_> = self.holders(group).into_iter().flatten().collect();
        let fetched = self.hold(hash, &holders).await;
        let held = self.store.get_bytes(hash).await;
        // Not held yet only when the store says it lacks the file; else it failed to read it.
        let lacks = held.is_err() && matches!(self.store.has(hash).await, Ok(false));
        tracing::debug!(target: "folder", path, %hash, lacks, ?fetched, why = ?held.as_ref().err(), "carried: the file as the app runs as the person, from the store, or why not");
        held.map(Some)
            .map_err(|_| if lacks { NOT_YET } else { UNSTORED })
    }

    /// The app `kind`'s route in `group`, for the key `key` of `people`:
    /// its own acts read, or one written as that key's person (the module
    /// docs).
    async fn acts(&self, app: (&AppName, &str), asker: (&People, &str), req: Request) -> Given {
        let ((app, group), (people, key)) = (app, asker);
        let (prefix, rail, person) = (format!("{app}/"), &self.host.rail, &people[key].person);
        let admission = rail.admission(group).ok();
        let read = admission.as_ref().and_then(crate::rail::ops_json);
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
            let register = (admission.as_ref().map(|a| a.membership())).filter(|m| !m.unsettled);
            let spans = register.map(|m| m.spans());
            tracing::debug!(target: "folder", group, %app, spans = ?spans.as_ref().map(Vec::len), "carried: each person's time in the group, or none, their standing unsettled");
            // The page the group's https links print on, as `link` prints
            // them, so Give names it in its install line (give-page-host).
            let page = (self.host.printed_page(group)).map(|p| p.map(|p| p.as_str().to_owned()));
            tracing::debug!(target: "folder", group, %app, ?page, "carried: the page the group's links print on, none, or why it cannot be read");
            let page = page.map_or_else(|why| json!({ "unread": why }), |p| json!(p));
            let read = json!({ "you": person, "you_key": people[key].person_key, "group": group, "page": page, "people": all, "spans": spans, "ops": ops });
            return Ok(Json(read).into_response());
        }
        // A write is its person's own device's (capability-signed): this
        // computer signs only its own person's, and a phone signs its own
        // and carries it (`/carry`).
        let elsewhere = key != self.host.actor;
        tracing::debug!(target: "folder", group, %app, %person, elsewhere, "carried: a write as the person, refused from any device but this computer");
        if elsewhere {
            let why = format!("Open {app} from your home page to write in it.");
            return Ok(LendRefusal::closed(why));
        }
        let body = (to_bytes(req.into_body(), WRITE_MOST).await).map_err(|_| TOO_LARGE)?;
        let act = app.act(&body).ok_or(NOT_AN_ACT)?;
        if (act.get("corrects")).is_some_and(|id| !of_kind(ops, app.as_str(), id)) {
            return Err(NOT_ITS_OWN);
        }
        let answer = crate::rail::append_act(rail, group, act, Some((app, key, 1))).await;
        tracing::debug!(target: "folder", group, %app, %person, status = answer.status().as_u16(), "carried: an act written as the person");
        if answer.status().is_success() {
            self.nudge.notify_one();
        }
        Ok(answer)
    }
}

/// Whether the act `id` among `ops`, a group's acts as the wire spells
/// them, is of the app `kind`'s own part, `<kind>/…`: what an app's
/// correction may void, at a carried app's route and in a lent app's
/// answer alike (capability-signed).
pub(crate) fn of_kind(ops: &[Value], kind: &str, id: &Value) -> bool {
    let prefix = format!("{kind}/");
    let own = |o: &Value| (o["payload"]["kind"].as_str()).is_some_and(|k| k.starts_with(&prefix));
    ops.iter().any(|o| &o["id"] == id && own(o))
}

/// How many entries an app may write as one person at once, and, since one
/// refills every `3600 / WRITES_AN_HOUR` seconds, in an hour: this number's
/// one home. Ten times the fastest use on record, 20 in a minute by a
/// driven browser settling up in Money
/// (`demo/runs/v1ui/2026-10-15-helper/data/summary.json`). A person's
/// burst never reaches it; a loop's stops here, since every member keeps
/// each entry for good (svrn-docs `rings/current/EXTENSION_POINT.md` §7, G1).
pub(crate) const WRITES_AN_HOUR: i64 = 200;

/// How many seconds before `more` entries of the app `app` from the key
/// `key` pass [`WRITES_AN_HOUR`], none if they do now, so one answer's acts
/// fit whole or none is written: a refill, read
/// from the record so a restart forgets nothing, walking its person's
/// entries of that app in time order as GCRA does in integers. An entry
/// is one under `<app>/` or, since each is kept by every member for good, a
/// correction that only voids one of those ([`of_kind`] on what it voids).
/// A stamp ahead of this computer's clock counts as now: `stamp` looks only
/// at `on_behalf_of` and `/carry` only at the signature and standing
/// (rail.rs), so a phone's fast clock reaches this walk, and must not lock
/// its person out.
pub(crate) fn spent(read: &Admission, app: &str, key: &str, more: usize) -> i64 {
    let first = read.membership().person_of(key).map_or(key, |(f, _)| f);
    // An unreadable clock is said at warn (`unix_secs`) and lets the write
    // through, as the window did: read as far past every stamp, not as 0.
    let now = crate::door::unix_now().unwrap_or(i64::MAX / 2);
    let all = crate::rail::ops_json(read).unwrap_or_default();
    let ops = all.as_array().map(Vec::as_slice).unwrap_or_default();
    let own = |o: &&Value| of_kind(std::slice::from_ref(*o), app, &o["id"]);
    let voids = |o: &&Value| o.get("corrects").is_some_and(|c| of_kind(ops, app, c));
    let entry = |o: &&Value| o["person_key"] == first && (own(o) || voids(o));
    let entries = ops.iter().filter(entry);
    let stamps = entries.filter_map(|o| o["ts_unix"].as_i64());
    let wait = wait_after(stamps.chain(std::iter::repeat_n(now, more - 1)), now);
    tracing::debug!(target: "folder", app, key, more, wait, "carried: this app's entries as this person, and the seconds before it may write as many more");
    wait
}

/// The seconds before an entry after `stamps`, in time order, is in budget
/// at `now`. Entries a program wrote with the token are the person's and
/// pass no budget, so they can overdraw it; an empty budget is as far as it
/// goes, or a bulk import would hold the app's page back for days.
pub(crate) fn wait_after(stamps: impl Iterator<Item = i64>, now: i64) -> i64 {
    let each = 3600 / WRITES_AN_HOUR;
    let empty_at = |ts: i64| ts + WRITES_AN_HOUR * each;
    let next = |due: i64, ts: i64| (due.max(ts.min(now)) + each).min(empty_at(ts.min(now)));
    let due = stamps.fold(0, next);
    (due - now - (WRITES_AN_HOUR - 1) * each).max(0)
}

/// The person told [`WRITES_AN_HOUR`] is spent, in words a developer can
/// act on, and `Retry-After`, since agents and fetch wrappers read that.
pub(crate) fn budget_refused(app: &str, wait: i64) -> Response {
    let words = format!("{app} is writing faster than an app may here, about {WRITES_AN_HOUR} entries an hour, because every member keeps each one for good. It can write again in {wait} seconds.");
    let refused = LendRefusal::Limit.say(StatusCode::TOO_MANY_REQUESTS, words);
    ([(header::RETRY_AFTER, wait.to_string())], refused).into_response()
}

/// A line a device signed, taken at [`crate::rail::carry`], past its app's
/// budget: a phone's own lines reach the record only there. A line this
/// copy already counts is not new, so never over.
pub(crate) fn line_spent(read: &Admission, line: &Op<SignedOp>) -> Option<Response> {
    let body = serde_json::to_value(line).ok()?;
    let part = |v: &Value| Some(v["kind"].as_str()?.split_once('/')?.0.to_string());
    let of_corrected = || {
        let id = body.get("corrects")?;
        let ops = crate::rail::ops_json(read)?;
        part(&ops.as_array()?.iter().find(|o| &o["id"] == id)?["payload"])
    };
    let stated = part(&body["payload"]).or_else(|| part(&body["replacement"]));
    let app = stated.or_else(of_corrected)?;
    let wait =
        (!read.membership().counts(&line.id)).then(|| spent(read, &app, line.stands_as(), 1));
    (wait.filter(|w| *w > 0)).map(|w| budget_refused(&app, w))
}

/// [`Append`] under the group's write lock, an app's budget counted there
/// so writes in flight at once cannot pass it between count and commit:
/// the app, the key, and how many of one answer's acts are still to be
/// written, this one and those after it ([`spent`]'s `more`).
pub(crate) struct Budgeted<'a>(
    pub(crate) Append,
    pub(crate) &'a AppName,
    pub(crate) &'a str,
    pub(crate) usize,
);

impl Command for Budgeted<'_> {
    type Refusal = Result<AppendRefusal, i64>;
    const READS: Reads = Reads::All;

    fn decide(&self, before: &Admission) -> Result<Vec<RailAct>, Self::Refusal> {
        let wait = spent(before, self.1.as_str(), self.2, self.3);
        (wait == 0).then_some(()).ok_or(Err(wait))?;
        self.0.decide(before).map_err(Ok)
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
