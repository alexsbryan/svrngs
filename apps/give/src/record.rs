//! svrngs's record, as give reaches it: the loopback API of the svrngs
//! running on this computer, asked with its token (CHARTER fence 4: the
//! record API, and nothing else of svrngs's). Where it listens is the `api`
//! file in svrngs's data dir and the token is the `token` file beside it,
//! the data dir being `$SVRNGS_DIR`, else `~/.svrngs`, as svrngs's own
//! command reads it (`svrngs --help`; apps/events/store.mjs reads the same
//! two files).
//!
//! Give's acts are `give/<kind>` in the record. [`gives`] takes them out of
//! the log with the prefix off, the shape the carried page reads at
//! `svrngs/acts`, so the fold reads the same acts from either.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde_json::{Value, json};

/// Give's part of the record, and of what a carried app called `give` reads.
pub const PREFIX: &str = "give/";

#[derive(Debug, thiserror::Error)]
pub enum RecordError {
    #[error("svrngs is not set up on this computer: {path} cannot be read ({why}). Install svrngs and run it first")]
    NotSetUp { path: PathBuf, why: String },
    #[error("svrngs on this computer did not answer at {api}: {why}. Is it running?")]
    Unanswered { api: String, why: String },
    #[error("svrngs on this computer refused {route}: {status} {said}")]
    Refused { route: String, status: u16, said: String },
    #[error("svrngs on this computer answered {route} with what give cannot read: {why}")]
    Unreadable { route: String, why: String },
}

/// A group this computer holds, as `/v1/groups` says it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// What every route names it by.
    pub id: String,
    pub name: String,
    /// The group's synced folder here, if it keeps one.
    pub folder: Option<PathBuf>,
}

/// This computer's key, and the groups it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Groups {
    pub key: String,
    pub groups: Vec<Group>,
}

/// svrngs's loopback API on this computer.
#[derive(Debug, Clone)]
pub struct Record {
    api: String,
    token: String,
}

/// svrngs's data dir, as its command reads it.
pub fn data_dir() -> Option<PathBuf> {
    let set = std::env::var_os("SVRNGS_DIR").filter(|d| !d.is_empty());
    set.map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".svrngs")))
}

fn read(path: &Path) -> Result<String, RecordError> {
    let text = std::fs::read_to_string(path).map_err(|e| RecordError::NotSetUp {
        path: path.to_path_buf(),
        why: e.to_string(),
    })?;
    Ok(text.trim().to_string())
}

impl Record {
    /// The record of the svrngs set up in `data_dir`.
    pub fn at(data_dir: &Path) -> Result<Record, RecordError> {
        let (api, token) = (read(&data_dir.join("api"))?, read(&data_dir.join("token"))?);
        tracing::debug!(api, data_dir = %data_dir.display(), "record: where svrngs listens, and its token read");
        Ok(Record { api, token })
    }

    fn call(&self, route: &str, body: Option<&Value>) -> Result<Value, RecordError> {
        let url = format!("{}{route}", self.api);
        let shown = format!("Bearer {}", self.token);
        let sent = match body {
            None => attohttpc::get(&url)
                .header("authorization", shown)
                .timeout(WITHIN)
                .send(),
            Some(body) => (attohttpc::post(&url).header("authorization", shown))
                .header("content-type", "application/json")
                .timeout(WITHIN)
                .text(body.to_string())
                .send(),
        };
        let unanswered = |why: String| RecordError::Unanswered {
            api: self.api.clone(),
            why,
        };
        let answer = sent.map_err(|e| unanswered(e.to_string()))?;
        let (status, text) = (
            answer.status().as_u16(),
            answer.text().map_err(|e| unanswered(e.to_string()))?,
        );
        let path = route.split('?').next().unwrap_or(route).to_string();
        if !(200..300).contains(&status) {
            tracing::debug!(route = path, status, said = text, "record: refused");
            return Err(RecordError::Refused {
                route: path,
                status,
                said: text,
            });
        }
        serde_json::from_str(&text).map_err(|e| RecordError::Unreadable {
            route: path,
            why: e.to_string(),
        })
    }

    /// This computer's key and the groups it holds.
    pub fn groups(&self) -> Result<Groups, RecordError> {
        let read = self.call("/v1/groups", None)?;
        let unreadable = |why: &str| RecordError::Unreadable {
            route: "/v1/groups".into(),
            why: why.into(),
        };
        let key = read["key"].as_str().ok_or_else(|| unreadable("no key"))?.to_string();
        let groups = (read["groups"].as_array().ok_or_else(|| unreadable("no groups"))?.iter())
            .filter_map(|g| {
                let folder = g["folders"]
                    .as_array()
                    .and_then(|f| f.first()?.as_str().map(PathBuf::from));
                let name = g["name"].as_str().unwrap_or_default().to_string();
                Some(Group {
                    id: g["id"].as_str()?.to_string(),
                    name,
                    folder,
                })
            })
            .collect();
        Ok(Groups { key, groups })
    }

    /// Give's acts in `group`, in the record's order, the prefix off.
    pub fn acts(&self, group: &str) -> Result<Vec<Value>, RecordError> {
        let read = self.call(&format!("/v1/rail/log?namespace={}", query(group)), None)?;
        let ops = read["ops"].as_array().ok_or_else(|| RecordError::Unreadable {
            route: "/v1/rail/log".into(),
            why: "no ops".into(),
        })?;
        Ok(gives(ops))
    }

    /// `payload`, whose `kind` is give's own without the prefix, written to
    /// `group`'s record as this computer's person.
    pub fn write(&self, group: &str, mut payload: Value) -> Result<(), RecordError> {
        let kind = payload["kind"].as_str().unwrap_or_default();
        payload["kind"] = format!("{PREFIX}{kind}").into();
        let route = format!("/v1/rail/append?namespace={}", query(group));
        self.call(&route, Some(&json!({ "op": "record", "payload": payload })))?;
        tracing::debug!(group, kind = %payload["kind"], "record: written");
        Ok(())
    }
}

/// How long svrngs has to answer one call on this computer.
const WITHIN: Duration = Duration::from_secs(10);

/// A group's id, as a query's value: ids are hex, so this only guards.
fn query(group: &str) -> String {
    group
        .bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Give's acts among a group's `ops`, as `/v1/rail/log` gives them, the
/// prefix taken off each act's kind, and each correction voiding one of
/// them: what svrngs's door gives a carried app called `give` at its
/// `svrngs/acts` (crates/svrngs-node/src/carried.rs, `acts`).
pub fn gives(ops: &[Value]) -> Vec<Value> {
    let own = |o: &Value| o["payload"]["kind"].as_str().is_some_and(|k| k.starts_with(PREFIX));
    let mine: std::collections::BTreeSet<_> = ops.iter().filter(|o| own(o)).filter_map(|o| o["id"].as_str()).collect();
    let voids_mine = |o: &Value| o["corrects"].as_str().is_some_and(|id| mine.contains(id));
    (ops.iter().filter(|o| own(o) || voids_mine(o)))
        .map(|o| {
            let mut o = o.clone();
            if let Some(kind) = o["payload"]["kind"].as_str().and_then(|k| k.strip_prefix(PREFIX)) {
                o["payload"]["kind"] = kind.to_string().into();
            }
            o
        })
        .collect()
}
