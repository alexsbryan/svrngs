//! The handler runner (DESIGN C4, C5; D7 E0.3): the host's half of an app.
//! It folds an app's acts through the [`Engine`], asks the app's `propose`
//! what a chat input should write, its `say` what to post, and its `due`
//! what to remind people of — and parses each answer into a type before
//! anything is written or posted. The bridge never sees app code; it sees
//! [`Command`]s, [`Reply`]s and [`Reminder`]s.
//!
//! The journal actor owns the runner and calls it with its [`Journal`], so a
//! fold reads the same acts a write is decided against.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use svrngs_core::{ActId, AppId, Body, Journal, Payload, Speaker, fold};
use svrngs_engine::{Breach, Engine, EngineError, Limits, QuickJs, Segment};
use tracing::{debug, error, warn};

/// The apps this host runs, as (manifest, module): built in, until installing
/// arrives with the SDK (DESIGN D7 E3.1). Their code stays in `apps/`.
const BUILT_IN: &[(&str, &str)] = &[(
    include_str!("../../../apps/expenses/manifest.json"),
    include_str!("../../../apps/expenses/expenses.js"),
)];

/// What a command option holds. A chat adapter renders each its own way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind {
    Text,
    /// Someone in the chat, given to the app as their speaker id.
    Person,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandOption {
    pub name: String,
    pub kind: OptionKind,
    pub description: String,
    pub required: bool,
}

/// A chat command an app declares: the one place it is declared (DESIGN D0a).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub options: Vec<CommandOption>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub name: AppId,
    pub version: String,
    pub commands: Vec<Command>,
}

/// What a person did in the chat that an app answers.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Input {
    Command {
        command: String,
        #[serde(default)]
        options: Map<String, Value>,
    },
    /// A tap on a button a [`Reply`] carried, by the button's id.
    Button { button: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Button {
    pub id: String,
    pub label: String,
}

/// What the bridge posts in answer to an input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reply {
    pub text: String,
    pub buttons: Vec<Button>,
    /// Shown only to the person who asked.
    pub private: bool,
}

/// What `propose` answered: an op to write for the speaker, or words that
/// write nothing. Never both.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Proposal {
    Write { op: Payload },
    Answer { text: String, private: bool },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct View {
    text: String,
    #[serde(default)]
    buttons: Vec<Button>,
}

/// One item `due` returned: whom it names, what to say, and the op whose
/// act stops it being due once it is posted.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DueItem {
    pub id: String,
    pub person: Speaker,
    pub text: String,
    pub fulfil: Payload,
}

/// A reminder for the bridge to post, mentioning the person it names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reminder {
    pub person: Speaker,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RunnerError {
    #[error("the built-in app {0} does not load: {1}")]
    BuiltIn(usize, String),
    #[error("no app answers {0}")]
    NoApp(String),
    #[error("{app}: {error}")]
    Engine { app: String, error: EngineError },
    #[error("{app} cannot add up its record: {breach}")]
    Incomplete { app: String, breach: Breach },
    #[error("{app}'s {call} answered what this host cannot read: {why}")]
    Answer {
        app: String,
        call: &'static str,
        why: String,
    },
}

struct App {
    manifest: Manifest,
    engine: QuickJs,
}

impl App {
    fn name(&self) -> String {
        self.manifest.name.as_str().to_string()
    }

    fn engine_error(&self, error: EngineError) -> RunnerError {
        RunnerError::Engine {
            app: self.name(),
            error,
        }
    }

    fn parse<T: for<'de> Deserialize<'de>>(&self, call: &'static str, v: Value) -> Result<T, RunnerError> {
        serde_json::from_value(v).map_err(|e| RunnerError::Answer {
            app: self.name(),
            call,
            why: e.to_string(),
        })
    }
}

pub struct Runner {
    apps: Vec<App>,
}

impl Runner {
    pub fn built_in() -> Result<Self, RunnerError> {
        let mut apps = Vec::new();
        for (i, (manifest, module)) in BUILT_IN.iter().enumerate() {
            let manifest: Manifest =
                serde_json::from_str(manifest).map_err(|e| RunnerError::BuiltIn(i, e.to_string()))?;
            let engine = QuickJs::new(*module, Limits::E0).map_err(|e| RunnerError::BuiltIn(i, e.to_string()))?;
            apps.push(App { manifest, engine });
        }
        Ok(Self { apps })
    }

    /// Every command every app declares, for the bridge to register.
    pub fn commands(&self) -> Vec<&Command> {
        self.apps.iter().flat_map(|a| &a.manifest.commands).collect()
    }

    /// The app that answers `input`, and `input` as that app reads it.
    fn route(&self, input: &Input, by: &Speaker, people: &Value) -> Result<(&App, Value), RunnerError> {
        let found = match input {
            Input::Command { command, options } => self
                .apps
                .iter()
                .find(|a| a.manifest.commands.iter().any(|c| &c.name == command))
                .map(|a| (a, json!({"command": command, "options": options}))),
            // A button's id is `<app>:<the id the app gave it>`.
            Input::Button { button } => button.split_once(':').and_then(|(app, id)| {
                self.apps
                    .iter()
                    .find(|a| a.manifest.name.as_str() == app)
                    .map(|a| (a, json!({"button": id})))
            }),
        };
        let Some((app, mut input)) = found else {
            warn!(?input, "runner:no_app");
            return Err(RunnerError::NoApp(format!("{input:?}")));
        };
        input["by"] = json!(by.as_str());
        input["people"] = people.clone();
        debug!(app = app.manifest.name.as_str(), "runner:route");
        Ok((app, input))
    }

    fn app(&self, name: &AppId) -> Result<&App, RunnerError> {
        self.apps
            .iter()
            .find(|a| &a.manifest.name == name)
            .ok_or_else(|| RunnerError::NoApp(name.as_str().to_string()))
    }

    /// `app`'s state: its acts, unvoided, in the record's order, folded.
    fn state(&self, app: &App, journal: &Journal) -> Result<Value, RunnerError> {
        let acts = acts_of(&app.manifest.name, journal);
        match app.engine.fold_segment(None, &acts).map_err(|e| app.engine_error(e))? {
            Segment::Complete(state) => {
                debug!(app = app.manifest.name.as_str(), acts = acts.len(), "runner:fold");
                Ok(state)
            }
            Segment::Incomplete(breach) => {
                error!(app = app.manifest.name.as_str(), acts = acts.len(), %breach, "runner:fold_incomplete");
                Err(RunnerError::Incomplete {
                    app: app.name(),
                    breach,
                })
            }
        }
    }

    /// What `input` from `by` should do, and which app answered it.
    pub fn propose(&self, journal: &Journal, input: &Input, by: &Speaker) -> Result<(AppId, Proposal), RunnerError> {
        let (app, input) = self.route(input, by, &people(journal))?;
        let state = self.state(app, journal)?;
        let answer = app.engine.propose(&state, &input).map_err(|e| app.engine_error(e))?;
        let proposal: Proposal = app.parse("propose", answer)?;
        debug!(
            app = app.manifest.name.as_str(),
            writes = matches!(proposal, Proposal::Write { .. }),
            "runner:propose"
        );
        Ok((app.manifest.name.clone(), proposal))
    }

    /// `app`'s view of its state, about the act `about` when given; each
    /// button's id gains the app's name, so a tap finds its way back.
    pub fn say(&self, journal: &Journal, app: &AppId, about: Option<&ActId>) -> Result<Reply, RunnerError> {
        let app = self.app(app)?;
        let state = self.state(app, journal)?;
        let view = json!({"people": people(journal), "act": about});
        let view: View = app.parse("say", app.engine.say(&state, &view).map_err(|e| app.engine_error(e))?)?;
        let buttons = view
            .buttons
            .into_iter()
            .map(|b| Button {
                id: format!("{}:{}", app.manifest.name.as_str(), b.id),
                label: b.label,
            })
            .collect();
        Ok(Reply {
            text: view.text,
            buttons,
            private: false,
        })
    }

    /// Every app's items due at `now`.
    pub fn due(&self, journal: &Journal, now: u64) -> Result<Vec<(AppId, DueItem)>, RunnerError> {
        let mut out = Vec::new();
        for app in &self.apps {
            let state = self.state(app, journal)?;
            let items: Vec<DueItem> =
                app.parse("due", app.engine.due(&state, now).map_err(|e| app.engine_error(e))?)?;
            debug!(app = app.manifest.name.as_str(), items = items.len(), "runner:due");
            out.extend(items.into_iter().map(|i| (app.manifest.name.clone(), i)));
        }
        Ok(out)
    }
}

/// The people the chat knows, by speaker, for an app to name them.
fn people(journal: &Journal) -> Value {
    journal
        .guests()
        .present()
        .map(|(s, e)| (s.as_str().to_string(), json!(e.name.as_str())))
        .collect::<Map<_, _>>()
        .into()
}

/// `app`'s acts as `reduce` takes them: `{id, ts, by, op}`, unvoided, in the
/// record's order. A correction's replacement stands where the correction is.
fn acts_of(app: &AppId, journal: &Journal) -> Vec<Value> {
    fold(journal)
        .into_iter()
        .filter(|f| !f.voided)
        .filter_map(|f| {
            let op = match f.act.body() {
                Body::Record(r) if &r.app == app => &r.op,
                Body::Correct(c) if &c.app == app => c.replacement.as_ref()?,
                _ => return None,
            };
            Some(json!({
                "id": f.act.id(),
                "ts": f.act.ts(),
                "by": f.speaker.map(|s| s.speaker.as_str()),
                "op": op,
            }))
        })
        .collect()
}
