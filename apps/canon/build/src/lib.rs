//! canon-core, JSON in and JSON out, for a page that has only JavaScript.
//!
//! House rules is a carried app, and a carried page gets no
//! `'wasm-unsafe-eval'` (scripts/trust_floor.txt; svrn-docs
//! rings/current/WHAT_RUNS_AS_YOU.md §6). So this crate is built to wasm and
//! then through binaryen's `wasm2js` to plain JavaScript, by
//! `apps/canon/build.sh` alone, never by the svrngs workspace. Nothing here
//! decides a house's rules: every answer is canon-core's (CANON_IN_SVRNGS §3),
//! and the replay is canon-cli's own `replay.rs`, compiled unchanged.
//!
//! The boundary is three exports, every value an i32, so wasm2js has no i64
//! to split at the edge: `canon_input(len)` gives the place to write a
//! request's UTF-8 bytes, `canon_call()` answers it and returns the answer's
//! length, `canon_output()` gives where the answer is. The glue is
//! `call.mjs`.

use std::cell::RefCell;

use canon_core::{Act, ActKind, Log, Scope, Seat};
use serde::Deserialize;
use serde_json::{json, Value};

/// canon-cli's scenario runner, the one `canon replay` runs and canon's
/// governance bar (`crates/canon-cli/tests/governance_bar.rs`) scores, read
/// from the archive of the pinned commit `build.sh` unpacks. Its `run` and
/// renderers are the CLI's and are never called here.
#[cfg(feature = "replay")]
#[allow(dead_code, clippy::all)]
#[path = "../../../../target/canon/src/crates/canon-cli/src/replay.rs"]
mod replay;

/// What `replay.rs` names in the rest of canon-cli, for its `run` alone.
/// Reaching any of them means the wrapper called the CLI's path, which it
/// never does, so each says so and stops.
#[cfg(feature = "replay")]
mod cmds {
    pub fn fail(_: impl std::fmt::Display) -> i32 {
        unreachable!("canon-cli's command line: the wrapper never runs it")
    }
    pub fn flag<'a>(_: &'a [String], _: &str) -> Option<&'a str> {
        unreachable!("canon-cli's command line: the wrapper never runs it")
    }
    pub fn has(_: &[String], _: &str) -> bool {
        unreachable!("canon-cli's command line: the wrapper never runs it")
    }
    pub fn positionals(_: &[String]) -> Vec<&str> {
        unreachable!("canon-cli's command line: the wrapper never runs it")
    }
    pub fn dir() -> Result<std::path::PathBuf, String> {
        unreachable!("canon-cli's command line: the wrapper never runs it")
    }
}
#[cfg(feature = "replay")]
mod store {
    pub const FILE: &str = "acts.jsonl";
    pub fn now() -> i64 {
        unreachable!("canon-cli's command line: the wrapper never runs it")
    }
    pub fn ignore_local(_: &std::path::Path) {
        unreachable!("canon-cli's command line: the wrapper never runs it")
    }
}
#[cfg(feature = "replay")]
mod wrap {
    pub fn hang(_: &str, _: &str) -> String {
        unreachable!("canon-cli's command line: the wrapper never runs it")
    }
}
#[cfg(feature = "replay")]
mod govern {
    pub(crate) fn rule_from(_: &[String], _: &str) -> Result<canon_core::Rule, String> {
        unreachable!("canon-cli's command line: the wrapper never runs it")
    }
    pub(crate) fn ratify_from(_: &str) -> Result<canon_core::Ratify, String> {
        unreachable!("canon-cli's command line: the wrapper never runs it")
    }
}

/// One question, by `call`. `acts` is the wire form, one act per line, as
/// canon writes it; `now` is the clock the fold reads, the last act's when
/// absent, as `canon` folds.
#[derive(Deserialize)]
#[serde(tag = "call", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    /// The whole derived state: every rule, proposal and its status.
    Derive { acts: String, now: Option<i64> },
    /// How a scope makes its rules now (`None` is the whole canon).
    Rule {
        acts: String,
        scope: Option<String>,
        now: Option<i64>,
    },
    /// Where a person stands toward governing a scope at `now`.
    Seat {
        acts: String,
        actor: String,
        scope: Option<String>,
        now: Option<i64>,
    },
    /// An act minted by canon, its id content-addressed, in the wire form.
    Act {
        kind: Value,
        ts_unix: i64,
        actor: String,
    },
    /// `canon replay <dir> --json`: a scenario over a seed, step by step.
    #[cfg(feature = "replay")]
    Replay { seed: String, scenario: String },
}

/// Answer one request: `{"ok": …}` or `{"error": "…"}`, never a default.
pub fn respond(request: &[u8]) -> String {
    let answer = serde_json::from_slice::<Request>(request)
        .map_err(|e| format!("not a request canon reads: {e}"))
        .and_then(answer);
    match answer {
        Ok(v) => json!({ "ok": v }),
        Err(e) => json!({ "error": e }),
    }
    .to_string()
}

fn answer(r: Request) -> Result<Value, String> {
    let fold = |acts: &str, now: Option<i64>| {
        let log = Log::parse(acts).map_err(|e| e.to_string())?;
        Ok::<_, String>(match now {
            Some(t) => log.derive_at(t),
            None => log.derive(),
        })
    };
    let scope = |s: Option<String>| match s {
        None => Ok(None),
        Some(s) => Scope::new(&s)
            .map(Some)
            .ok_or_else(|| format!("`{s}` is not a scope")),
    };
    match r {
        Request::Derive { acts, now } => to_json(&fold(&acts, now)?),
        Request::Rule {
            acts,
            scope: s,
            now,
        } => {
            let canon = fold(&acts, now)?;
            let s = scope(s)?;
            let at = now.unwrap_or(i64::MAX);
            let rule = canon.ratification_for_at(s.as_ref(), at);
            Ok(json!({
                "rule": to_json(rule)?,
                "name": rule.name(),
                "prose": rule.prose(),
                "governs": to_json(&canon.governs_at(s.as_ref(), at))?,
            }))
        }
        Request::Seat {
            acts,
            actor,
            scope: s,
            now,
        } => {
            let canon = fold(&acts, now)?;
            let seat = canon.seat(&actor, scope(s)?.as_ref(), now.unwrap_or(i64::MAX));
            Ok(json!(match seat {
                Seat::Held => "held",
                Seat::Open => "open",
                Seat::Lacking => "lacking",
            }))
        }
        Request::Act {
            kind,
            ts_unix,
            actor,
        } => {
            let kind: ActKind = serde_json::from_value(kind).map_err(|e| e.to_string())?;
            let line = Log::from_acts(vec![Act::new(kind, ts_unix, actor)]).render();
            Ok(json!(line))
        }
        #[cfg(feature = "replay")]
        Request::Replay { seed, scenario } => {
            let run = replay::run_scenario(&seed, &scenario, None, None)?;
            // As `canon replay --json` collects it (replay.rs `run`).
            let out: serde_json::Map<String, Value> = run
                .steps
                .iter()
                .map(|s| (s.name.clone(), s.result.clone()))
                .collect();
            Ok(Value::Object(out))
        }
    }
}

fn to_json(v: &impl serde::Serialize) -> Result<Value, String> {
    serde_json::to_value(v).map_err(|e| e.to_string())
}

thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static OUTPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Room for a request of `len` bytes; the page writes it there.
#[no_mangle]
pub extern "C" fn canon_input(len: usize) -> *mut u8 {
    INPUT.with_borrow_mut(|b| {
        *b = vec![0; len];
        b.as_mut_ptr()
    })
}

/// Answer the request written at `canon_input`; the answer's length.
#[no_mangle]
pub extern "C" fn canon_call() -> usize {
    let out = INPUT.with_borrow(|b| respond(b));
    OUTPUT.with_borrow_mut(|o| {
        *o = out.into_bytes();
        o.len()
    })
}

/// Where the last answer is.
#[no_mangle]
pub extern "C" fn canon_output() -> *const u8 {
    OUTPUT.with_borrow(|o| o.as_ptr())
}
