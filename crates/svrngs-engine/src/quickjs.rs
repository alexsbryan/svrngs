//! [`QuickJs`]: QuickJS embedded in the host, the first [`Engine`].
//!
//! Every call gets a fresh runtime, so nothing an app leaves in a module
//! global reaches the next call (the sixth law, made structural rather than
//! trusted). The context is built without `Date`, `WeakRef`,
//! `FinalizationRegistry` and typed arrays — QuickJS has no `Intl` — and
//! `driver.js` removes `Math.random` and the locale methods before the app's
//! module is evaluated.
//!
//! Fuel is counted in QuickJS's own interrupt polls, one every 10,000
//! operations (`JS_INTERRUPT_COUNTER_INIT`, rquickjs-sys quickjs.c:556), so
//! a budget is a count of steps, not of time, and the interrupt it raises is
//! uncatchable (quickjs.c:8645). Memory is budgeted by the allocator, which
//! remembers refusing, so an out-of-memory is a breach even when the app
//! catches the exception QuickJS raises for it (DESIGN C4). A stack overflow
//! is still catchable here; the wasm engine's trap is E3's (D7 E3.3).

use std::cell::Cell;
use std::rc::Rc;

use rquickjs::allocator::{Allocator, RustAllocator};
use rquickjs::context::intrinsic::{Eval, Json, MapSet, Promise, RegExp, RegExpCompiler};
use rquickjs::{CaughtError, Context, Ctx, Function, Module, Runtime};
use serde_json::Value;

use crate::{Breach, Engine, EngineError, Segment};

const DRIVER: &str = include_str!("driver.js");
const EXPORTS: [&str; 5] = ["init", "reduce", "propose", "say", "due"];
/// QuickJS's own words for a stack overflow (quickjs.c:8567).
const STACK_OVERFLOW: &str = "Maximum call stack size exceeded";

/// What one call may spend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Bytes QuickJS may allocate.
    pub memory: usize,
    /// Bytes of stack QuickJS may use. It sits below the host thread's own
    /// stack (2 MiB for a spawned thread), so QuickJS's limit is the one that
    /// binds.
    pub stack: usize,
    /// Interrupt polls, of 10,000 operations each.
    pub fuel: u64,
}

impl Limits {
    /// E0's budgets, for the one first-party app on the host. The manifest
    /// names them once the engine is pinned by an SDK version (DESIGN D0a).
    pub const E0: Limits = Limits {
        memory: 64 << 20,
        stack: 512 << 10,
        fuel: 20_000,
    };
}

/// One app's module, run by QuickJS.
pub struct QuickJs {
    source: String,
    limits: Limits,
}

impl QuickJs {
    /// Take an app's module source. It is evaluated once here, so a bundle
    /// that does not load is refused now rather than on its first call.
    pub fn new(source: impl Into<String>, limits: Limits) -> Result<Self, EngineError> {
        let engine = Self {
            source: source.into(),
            limits,
        };
        engine
            .run(None, None, String::new())?
            .map_err(|breach| EngineError::Breach { call: "load", breach })?;
        Ok(engine)
    }

    /// Evaluate the driver and the app in a fresh runtime, then call the
    /// driver's `call` — or stop after loading when `call` is `None`.
    fn run(
        &self,
        call: Option<&'static str>,
        a: Option<String>,
        b: String,
    ) -> Result<Result<Value, Breach>, EngineError> {
        let full = Rc::new(Cell::new(false));
        let memory = Budgeted {
            used: 0,
            cap: self.limits.memory,
            refused: full.clone(),
        };
        let rt = Runtime::new_with_alloc(memory).map_err(|e| EngineError::Engine(e.to_string()))?;
        rt.set_max_stack_size(self.limits.stack);
        let spent = Rc::new(Cell::new(false));
        let (flag, budget, mut polls) = (spent.clone(), self.limits.fuel, 0u64);
        rt.set_interrupt_handler(Some(Box::new(move || {
            polls += 1;
            flag.set(polls > budget);
            polls > budget
        })));
        let ctx = Context::custom::<(Eval, RegExpCompiler, RegExp, Json, MapSet, Promise)>(&rt)
            .map_err(|e| EngineError::Engine(e.to_string()))?;
        ctx.with(|ctx| {
            let driver: Function = ctx.eval(DRIVER).map_err(|e| EngineError::Engine(caught(&ctx, e)))?;
            let load = |e| {
                let why = caught(&ctx, e);
                EngineError::Load(if full.get() { Breach::Memory.to_string() } else { why })
            };
            let (module, done) = Module::declare(ctx.clone(), "app.js", self.source.as_str())
                .and_then(|m| m.eval())
                .map_err(load)?;
            done.finish::<()>().map_err(load)?;
            let app = module.namespace().map_err(load)?;
            let Some(call) = call else { return Ok(Ok(Value::Null)) };
            let result = driver.call::<_, String>((app, call, a, b)).map_err(|e| caught(&ctx, e));
            match result {
                _ if spent.get() => Ok(Err(Breach::Fuel)),
                _ if full.get() => Ok(Err(Breach::Memory)),
                Ok(envelope) => opened(call, &envelope),
                Err(why) if why.contains(STACK_OVERFLOW) => Ok(Err(Breach::Stack)),
                Err(why) => Ok(Err(Breach::Threw(why))),
            }
        })
    }

    fn handler(&self, call: &'static str, state: &Value, arg: &Value) -> Result<Value, EngineError> {
        self.run(Some(call), Some(state.to_string()), arg.to_string())?
            .map_err(|breach| EngineError::Breach { call, breach })
    }
}

impl Engine for QuickJs {
    fn fold_segment(&self, seed: Option<&Value>, acts: &[Value]) -> Result<Segment, EngineError> {
        let acts = Value::Array(acts.to_vec()).to_string();
        Ok(match self.run(Some("fold"), seed.map(Value::to_string), acts)? {
            Ok(state) => Segment::Complete(state),
            Err(breach) => Segment::Incomplete(breach),
        })
    }

    fn propose(&self, state: &Value, input: &Value) -> Result<Value, EngineError> {
        self.handler("propose", state, input)
    }

    fn say(&self, state: &Value, view: &Value) -> Result<Value, EngineError> {
        self.handler("say", state, view)
    }

    fn due(&self, state: &Value, now: u64) -> Result<Value, EngineError> {
        self.handler("due", state, &Value::from(now))
    }
}

/// The exception behind `e`, as text, taken off the context.
fn caught(ctx: &Ctx<'_>, e: rquickjs::Error) -> String {
    CaughtError::from_error(ctx, e).to_string()
}

/// The driver's envelope: `{ok}`, `{inexact}` or `{missing}`.
fn opened(call: &'static str, envelope: &str) -> Result<Result<Value, Breach>, EngineError> {
    let mut v: Value = serde_json::from_str(envelope)
        .map_err(|e| EngineError::Engine(format!("the driver's answer to {call} is not JSON: {e}")))?;
    if let Some(ok) = v.get_mut("ok") {
        return Ok(Ok(ok.take()));
    }
    if let Some(why) = v.get("inexact").and_then(Value::as_str) {
        return Ok(Err(Breach::NotJson(why.to_string())));
    }
    let missing = v.get("missing").and_then(Value::as_str).unwrap_or_default();
    Err(EngineError::Missing(
        EXPORTS.into_iter().find(|e| *e == missing).unwrap_or(call),
    ))
}

/// QuickJS's allocator, refusing past `cap` bytes and remembering that it did.
struct Budgeted {
    used: usize,
    cap: usize,
    refused: Rc<Cell<bool>>,
}

impl Budgeted {
    fn fits(&mut self, freed: usize, wanted: usize) -> bool {
        let fits = (self.used - freed).checked_add(wanted).is_some_and(|n| n <= self.cap);
        if !fits {
            self.refused.set(true);
        }
        fits
    }
}

// SAFETY: every pointer comes from, and goes back to, `RustAllocator`; this
// only counts the bytes it reports and refuses before asking it.
unsafe impl Allocator for Budgeted {
    fn alloc(&mut self, size: usize) -> *mut u8 {
        if !self.fits(0, size) {
            return std::ptr::null_mut();
        }
        let p = RustAllocator.alloc(size);
        if !p.is_null() {
            self.used += unsafe { RustAllocator::usable_size(p) };
        }
        p
    }

    fn calloc(&mut self, count: usize, size: usize) -> *mut u8 {
        if !self.fits(0, count.saturating_mul(size)) {
            return std::ptr::null_mut();
        }
        let p = RustAllocator.calloc(count, size);
        if !p.is_null() {
            self.used += unsafe { RustAllocator::usable_size(p) };
        }
        p
    }

    unsafe fn dealloc(&mut self, ptr: *mut u8) {
        self.used -= unsafe { RustAllocator::usable_size(ptr) };
        unsafe { RustAllocator.dealloc(ptr) }
    }

    unsafe fn realloc(&mut self, ptr: *mut u8, new_size: usize) -> *mut u8 {
        // QuickJS reallocs only a live pointer to a non-zero size
        // (`js_realloc_rt`, quickjs.c:2048), as `RustAllocator` requires.
        let old = unsafe { RustAllocator::usable_size(ptr) };
        if !self.fits(old, new_size) {
            return std::ptr::null_mut();
        }
        let p = unsafe { RustAllocator.realloc(ptr, new_size) };
        if !p.is_null() {
            self.used = self.used - old + unsafe { RustAllocator::usable_size(p) };
        }
        p
    }

    unsafe fn usable_size(ptr: *mut u8) -> usize {
        unsafe { RustAllocator::usable_size(ptr) }
    }
}
