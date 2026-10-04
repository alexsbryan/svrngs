//! svrngs-engine: the fold engine, waiting (DESIGN §9) until an app needs its
//! state offline on a device that is not its host. It was built for E0, the
//! design at tag `e0-bot` (C4), where an app was a module of pure exports —
//! `init`, `reduce`, `propose`, `say`, `due` — reached only through
//! [`Engine`]: JSON in and out, the five laws and the sixth, and the budgets.
//!
//! [`QuickJs`] is the first implementation: QuickJS embedded in the host,
//! with what would let two copies disagree removed before any app code runs.
//! The wasm build under wasmtime, with metering inside its hashed bytes and
//! fold segments, is the second, and waits for strangers' code. The engine
//! never logs: it is pure, and returns outcomes for its caller to trace.

mod quickjs;

pub use quickjs::{Limits, QuickJs};
use serde_json::Value;

/// One app's exports, run on JSON.
pub trait Engine {
    /// Fold `acts`, in the order given, onto `seed` — or onto the app's
    /// `init()` when there is none.
    fn fold_segment(&self, seed: Option<&Value>, acts: &[Value]) -> Result<Segment, EngineError>;
    /// A chat input as the act to write, or an answer that writes nothing.
    fn propose(&self, state: &Value, input: &Value) -> Result<Value, EngineError>;
    /// A text view of the state.
    fn say(&self, state: &Value, view: &Value) -> Result<Value, EngineError>;
    /// What is due at `now`, in Unix seconds.
    fn due(&self, state: &Value, now: u64) -> Result<Value, EngineError>;
}

/// A fold's outcome. A segment that breaches halts whole: the fold never
/// skips an act, so there is no state to report beside the breach.
#[derive(Debug, Clone, PartialEq)]
pub enum Segment {
    Complete(Value),
    Incomplete(Breach),
}

/// Why an app's code stopped before it finished. Each is named for the
/// developer, because each is theirs to fix.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Breach {
    #[error("ran out of fuel: it took more steps than one call is given")]
    Fuel,
    #[error("ran out of memory")]
    Memory,
    #[error("ran out of stack: recursion too deep")]
    Stack,
    #[error("threw: {0}")]
    Threw(String),
    /// What it returned does not survive a JSON round trip unchanged — `-0`,
    /// `NaN`, `Infinity`, `undefined`, a `Map`, a `Set`, a function.
    #[error("returned what is not exactly JSON: {0}")]
    NotJson(String),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EngineError {
    #[error("the app could not be loaded: {0}")]
    Load(String),
    #[error("the app exports no function `{0}`")]
    Missing(&'static str),
    #[error("`{call}` {breach}")]
    Breach { call: &'static str, breach: Breach },
    #[error("the engine could not start: {0}")]
    Engine(String),
}

#[cfg(test)]
mod tests;
