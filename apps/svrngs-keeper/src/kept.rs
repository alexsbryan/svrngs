//! What `svrngs-keeper` does: keep a native card in a systemd user unit, and
//! list, stop, start and remove what it keeps (ralph/next/keeper/STATE.md,
//! "The user flow"). It supervises nothing: the OS starts the unit at boot
//! and again after a crash (`unit.rs`), and this only writes the unit and
//! asks systemd.
//!
//! A kept card lives in `<data>/svrngs-keeper/<name>/compose.yaml`, with the
//! card it replaced beside it as `previous.yaml`, for a later "go back".
//! Removing a card stops its unit and deletes the unit file; the card and
//! the program's folder stay.

use std::path::{Path, PathBuf};

use crate::card::{Card, Refused};
use crate::runtime::{Name, RunError};
use crate::systemd::{SystemdUser, UserUnit};
use crate::unit::{native, unit_name};

#[derive(Debug, thiserror::Error)]
pub enum KeptError {
    #[error("{0}")]
    Card(#[from] Refused),
    #[error(
        "{0} runs services from images, and svrngs-keeper keeps only a native card today (x-svrngs.native); give runs a card of images"
    )]
    NotNative(String),
    #[error("{0}; name it with --name")]
    Name(#[from] RunError),
    #[error("svrngs-keeper keeps no card named {0} here (svrngs-keeper list shows them)")]
    NotKept(String),
    #[error("{path}: {why}")]
    File { path: PathBuf, why: String },
    #[error("{0}")]
    Systemd(String),
}

fn file(path: &Path) -> impl FnOnce(std::io::Error) -> KeptError + '_ {
    move |e| KeptError::File {
        path: path.to_path_buf(),
        why: e.to_string(),
    }
}

/// Where kept cards and their units live.
#[derive(Debug, Clone)]
pub struct Places {
    /// `$XDG_DATA_HOME/svrngs-keeper`, or `~/.local/share/svrngs-keeper`.
    pub data: PathBuf,
    /// `$XDG_CONFIG_HOME/systemd/user`, or `~/.config/systemd/user`: where
    /// the user's manager reads units.
    pub units: PathBuf,
}

impl Places {
    /// From `HOME`, `XDG_DATA_HOME` and `XDG_CONFIG_HOME` as the caller read
    /// them; an empty XDG value is unset, as the XDG spec has it.
    pub fn of(home: &Path, data_home: Option<&str>, config_home: Option<&str>) -> Places {
        let or = |v: Option<&str>, default: &str| {
            v.filter(|v| !v.is_empty())
                .map_or_else(|| home.join(default), PathBuf::from)
        };
        Places {
            data: or(data_home, ".local/share").join("svrngs-keeper"),
            units: or(config_home, ".config").join("systemd/user"),
        }
    }

    /// The folder a card kept under `name` lives in.
    pub fn card_dir(&self, name: &Name) -> PathBuf {
        self.data.join(name.as_str())
    }

    pub fn unit_path(&self, name: &Name) -> PathBuf {
        self.units.join(unit_name(name))
    }
}

/// What `keep` did with the card it was given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stored {
    /// Kept here for the first time.
    New,
    /// The same card was kept already: nothing was written or restarted.
    Same,
    /// It replaced another card, now `previous.yaml`, and the unit started
    /// again from the new one if its unit changed.
    Changed { previous: PathBuf, restarted: bool },
}

/// A card `keep` kept.
#[derive(Debug, Clone)]
pub struct Kept {
    pub name: Name,
    pub unit: String,
    pub port: u16,
    pub stored: Stored,
}

/// The name a card is kept under when none is given: its title, lowercased,
/// each run of anything but a letter or digit one `-`.
pub fn name_of(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            out.push(c);
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

/// The question `keep` asks before it lets this account run with nobody
/// logged in (ralph/next/keeper/STATE.md, "The user flow").
pub const LINGER_ASK: &str =
    "To start it before you log in, your account needs to run when nobody is logged in. Turn that on?";

/// When a kept card will start, as the answer to [`LINGER_ASK`] left it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Boot {
    /// The account lingers already, so nothing was asked.
    Lingers,
    /// Asked, and yes: `keep` turns linger on (`SystemdUser::ensure_linger`).
    TurnOn,
    /// Asked and no, or no terminal to ask: it starts at login. `unread` is
    /// why whether the account lingers could not be read, if it could not.
    Login { unread: Option<String> },
}

impl Boot {
    /// What linger reads now, without asking: for the line after a keep.
    pub fn read(lingers: Result<bool, String>) -> Boot {
        match lingers {
            Ok(true) => Boot::Lingers,
            Ok(false) => Boot::Login { unread: None },
            Err(why) => Boot::Login { unread: Some(why) },
        }
    }

    /// The plan's "It starts …", ending in a comma for what follows.
    pub fn starts(&self) -> &'static str {
        match self {
            Boot::Lingers | Boot::TurnOn => "It starts when this computer starts, before anyone logs in,",
            Boot::Login { unread: None } => "It starts when you log in,",
            Boot::Login { unread: Some(_) } => "It starts when you log in at least,",
        }
    }
}

/// Ask [`LINGER_ASK`] only while this account doesn't linger. `ask` is the
/// terminal: `None` when there is none to ask in, and a yes or a no.
pub fn boot(systemd: &SystemdUser, ask: impl FnOnce(&str) -> Option<bool>) -> Boot {
    let unread = match systemd.lingers() {
        Ok(true) => {
            tracing::debug!("lingers already: not asked");
            return Boot::Lingers;
        }
        Ok(false) => None,
        Err(why) => {
            tracing::debug!(why, "whether it lingers was not read, so it is asked");
            Some(why)
        }
    };
    match ask(LINGER_ASK) {
        Some(true) => {
            tracing::debug!("linger: yes");
            Boot::TurnOn
        }
        answer => {
            tracing::debug!(terminal = answer.is_some(), "linger: no, so it starts at login");
            Boot::Login { unread }
        }
    }
}

/// Keep the card `text` under `name` (or its title's): store it, write its
/// unit, and have systemd enable and start it, then turn linger on if
/// `boot` says to. `boot` is asked only once the card is read, so a card
/// refused asks nothing. The same card again resets nothing.
pub fn keep(
    systemd: &SystemdUser,
    places: &Places,
    text: &str,
    name: Option<&str>,
    boot: impl FnOnce() -> Boot,
) -> Result<Kept, KeptError> {
    let card = Card::parse(text)?;
    let name = Name::new(&name.map_or_else(|| name_of(&card.title), str::to_string))?;
    keep_card(systemd, places, &card, name, boot)
}

pub fn keep_card(
    systemd: &SystemdUser,
    places: &Places,
    card: &Card,
    name: Name,
    boot: impl FnOnce() -> Boot,
) -> Result<Kept, KeptError> {
    let Some(program) = &card.native else {
        tracing::debug!(title = card.title, "refused: not a native card");
        return Err(KeptError::NotNative(card.title.clone()));
    };
    let content = native(&name, card, program);
    install(systemd, places, card, name, boot, &content)
}

pub fn install(
    systemd: &SystemdUser,
    places: &Places,
    card: &Card,
    name: Name,
    boot: impl FnOnce() -> Boot,
    content: &str,
) -> Result<Kept, KeptError> {
    let boot = boot();
    let dir = places.card_dir(&name);
    let (path, previous) = (dir.join("compose.yaml"), dir.join("previous.yaml"));
    let before = std::fs::read_to_string(&path).ok();
    let text = &card.text;
    let stored = match &before {
        None => Stored::New,
        Some(b) if b == text => Stored::Same,
        Some(_) => Stored::Changed {
            previous: previous.clone(),
            restarted: false,
        },
    };
    tracing::debug!(name = name.as_str(), ?stored, "card read");
    if stored != Stored::Same {
        std::fs::create_dir_all(&dir).map_err(file(&dir))?;
        if let Some(before) = &before {
            std::fs::write(&previous, before).map_err(file(&previous))?;
        }
        std::fs::write(&path, text).map_err(file(&path))?;
    }
    let (unit, unit_path) = (unit_name(&name), places.unit_path(&name));
    let unit_before = std::fs::read_to_string(&unit_path).ok();
    let user_unit = UserUnit {
        name: &unit,
        path: &unit_path,
        content,
        prefix: "svrngs-keeper",
        what: "programs it keeps",
    };
    systemd.install(&user_unit, true).map_err(KeptError::Systemd)?;
    if boot == Boot::TurnOn {
        systemd.ensure_linger(user_unit.prefix, user_unit.what);
    }
    // `enable --now` leaves a running unit as it is, so a changed unit runs
    // only once it starts again. The unit names its card's version, so any
    // new card changes it.
    let restart = unit_before.is_some_and(|b| b != content);
    tracing::debug!(unit, restart, "unit installed");
    if restart {
        systemd.ctl(&["--user", "restart", &unit]).map_err(KeptError::Systemd)?;
    }
    let stored = match stored {
        Stored::Changed { previous, .. } => Stored::Changed {
            previous,
            restarted: restart,
        },
        s => s,
    };
    Ok(Kept {
        name,
        unit,
        port: card.door.port,
        stored,
    })
}

/// A kept card, by name: its unit must be installed.
fn kept(places: &Places, name: &str) -> Result<(Name, String), KeptError> {
    let name = Name::new(name).map_err(|_| KeptError::NotKept(name.to_string()))?;
    if !places.unit_path(&name).exists() {
        tracing::debug!(name = name.as_str(), "refused: no unit kept under this name");
        return Err(KeptError::NotKept(name.as_str().to_string()));
    }
    let unit = unit_name(&name);
    Ok((name, unit))
}

/// Stop it until it is started again: `disable --now`, so a boot does not
/// start it either.
pub fn stop(systemd: &SystemdUser, places: &Places, name: &str) -> Result<Name, KeptError> {
    let (name, unit) = kept(places, name)?;
    systemd
        .ctl(&["--user", "disable", "--now", &unit])
        .map_err(KeptError::Systemd)?;
    Ok(name)
}

/// Start it, and at each boot again: `enable --now`.
pub fn start(systemd: &SystemdUser, places: &Places, name: &str) -> Result<Name, KeptError> {
    let (name, unit) = kept(places, name)?;
    let active = systemd.show(&unit, &["ActiveState"]).map_err(KeptError::Systemd)?;
    if active.trim() == "ActiveState=failed" {
        tracing::debug!(unit, "kept:start_reset_failed");
        systemd
            .ctl(&["--user", "reset-failed", &unit])
            .map_err(KeptError::Systemd)?;
    } else {
        tracing::debug!(unit, state = active.trim(), "kept:start_without_reset");
    }
    systemd
        .ctl(&["--user", "enable", "--now", &unit])
        .map_err(KeptError::Systemd)?;
    Ok(name)
}

/// Restart a running unit. Only an explicit `start` undoes a stop.
pub fn restart(systemd: &SystemdUser, places: &Places, name: &str) -> Result<Name, KeptError> {
    let (name, unit) = kept(places, name)?;
    systemd
        .ctl(&["--user", "try-restart", &unit])
        .map_err(KeptError::Systemd)?;
    Ok(name)
}

/// Run it here no longer: its unit is stopped and deleted. Its card stays,
/// in the folder this returns, and so does the program's own folder.
pub fn remove(systemd: &SystemdUser, places: &Places, name: &str) -> Result<PathBuf, KeptError> {
    let (name, unit) = kept(places, name)?;
    systemd
        .uninstall(&unit, &places.unit_path(&name))
        .map_err(KeptError::Systemd)?;
    Ok(places.card_dir(&name))
}

/// Whether a kept card's unit runs, as systemd answered.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Running {
    /// `active`, since the time systemd gave (`HH:MM`), if it gave one.
    Since(Option<String>),
    /// `activating`, or `reloading`: on its way up.
    Starting,
    /// `inactive` or `deactivating`: stopped, as asked.
    Stopped,
    /// `failed`, with systemd's `Result` (`exit-code`, `start-limit-hit`, …).
    Failed(String),
}

/// When a kept card's unit starts.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Starts {
    /// Enabled, and the user's manager lingers: at boot, before login.
    AtBoot,
    /// Enabled, without linger: when its user logs in.
    AtLogin,
    /// Enabled, and whether it lingers could not be read (why).
    Enabled(String),
    /// Not enabled: only when someone starts it.
    ByHand,
}

/// A kept card's unit, as `systemctl --user show` reads it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Status {
    pub running: Running,
    pub starts: Starts,
    pub restarts: u32,
}

/// What `list` shows of one card in the data folder.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Row {
    Kept {
        name: String,
        port: u16,
        status: Status,
    },
    /// Its card stays and its unit was removed.
    Removed {
        name: String,
        dir: PathBuf,
    },
    /// Its card or its state could not be read, and why: no guess is shown.
    Unread {
        name: String,
        why: String,
    },
}

/// The properties `status` reads.
pub const SHOWN: [&str; 5] = ["LoadState", "ActiveState", "Result", "NRestarts", "UnitFileState"];

/// Read `systemctl --user show -p` output for [`SHOWN`] and
/// `ActiveEnterTimestamp`, beside whether the manager lingers.
pub fn status(show: &str, lingers: Result<bool, String>) -> Result<Status, String> {
    let get = |key: &str| {
        show.lines()
            .find_map(|l| l.strip_prefix(key).and_then(|v| v.strip_prefix('=')))
            .map(str::trim)
    };
    if get("LoadState") != Some("loaded") {
        return Err(format!(
            "its unit is not loaded ({})",
            get("LoadState").unwrap_or("no LoadState")
        ));
    }
    let running = match get("ActiveState") {
        Some("active") => Running::Since(
            // `Sun 2026-11-22 09:14:03 PST`: its time, to the minute.
            get("ActiveEnterTimestamp")
                .and_then(|t| t.split_whitespace().nth(2))
                .and_then(|t| t.get(..5))
                .map(str::to_string),
        ),
        Some("activating" | "reloading") => Running::Starting,
        Some("inactive" | "deactivating") => Running::Stopped,
        Some("failed") => Running::Failed(get("Result").unwrap_or("").to_string()),
        other => return Err(format!("ActiveState {other:?} is not one svrngs-keeper reads")),
    };
    let starts = match get("UnitFileState") {
        Some("enabled" | "enabled-runtime" | "linked" | "linked-runtime" | "alias") => match lingers {
            Ok(true) => Starts::AtBoot,
            Ok(false) => Starts::AtLogin,
            Err(why) => Starts::Enabled(why),
        },
        Some(_) => Starts::ByHand,
        None => return Err("systemd gave no UnitFileState".into()),
    };
    let restarts = get("NRestarts")
        .and_then(|n| n.parse().ok())
        .ok_or("systemd gave no NRestarts")?;
    Ok(Status {
        running,
        starts,
        restarts,
    })
}

/// Every card kept here, in name order.
pub fn list(systemd: &SystemdUser, places: &Places) -> Result<Vec<Row>, KeptError> {
    let mut names = match std::fs::read_dir(&places.data) {
        Ok(entries) => entries
            .flatten()
            .filter(|e| e.path().join("compose.yaml").is_file())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(file(&places.data)(e)),
    };
    names.sort();
    let lingers = systemd.lingers();
    Ok(names
        .into_iter()
        .map(|name| row(systemd, places, name, &lingers))
        .collect())
}

fn row(systemd: &SystemdUser, places: &Places, name: String, lingers: &Result<bool, String>) -> Row {
    let unread = |why: String| {
        tracing::debug!(name, why, "could not read");
        Row::Unread {
            name: name.clone(),
            why,
        }
    };
    let Ok(n) = Name::new(&name) else {
        return unread("not a name svrngs-keeper keeps a card under".into());
    };
    let dir = places.card_dir(&n);
    let card = match std::fs::read_to_string(dir.join("compose.yaml")).map(|t| Card::parse(&t)) {
        Ok(Ok(card)) => card,
        Ok(Err(why)) => return unread(why.to_string()),
        Err(why) => return unread(why.to_string()),
    };
    if !places.unit_path(&n).exists() {
        return Row::Removed { name, dir };
    }
    let mut properties = SHOWN.to_vec();
    properties.push("ActiveEnterTimestamp");
    match systemd
        .show(&unit_name(&n), &properties)
        .and_then(|show| status(&show, lingers.clone()))
    {
        Ok(status) => Row::Kept {
            name,
            port: card.door.port,
            status,
        },
        Err(why) => unread(why),
    }
}

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Row::Kept { name, port, status } => {
                let running = match &status.running {
                    Running::Since(Some(at)) => format!("running since {at}"),
                    Running::Since(None) => "running".into(),
                    Running::Starting => "starting".into(),
                    Running::Stopped => "stopped".into(),
                    Running::Failed(r) if r == "start-limit-hit" => "keeps stopping, so it was left stopped".into(),
                    Running::Failed(r) => format!("stopped: it failed ({r})"),
                };
                let starts = match &status.starts {
                    Starts::AtBoot => "starts when this computer starts".into(),
                    Starts::AtLogin => "starts when you log in".into(),
                    Starts::Enabled(why) => {
                        format!("starts when you log in, or at boot if your account runs then ({why})")
                    }
                    Starts::ByHand => format!("starts only when you start it (svrngs-keeper start {name})"),
                };
                write!(
                    f,
                    "{name}   {running} · {starts} · {} restart{} · port {port}",
                    status.restarts,
                    if status.restarts == 1 { "" } else { "s" }
                )
            }
            Row::Removed { name, dir } => write!(
                f,
                "{name}   not run here · its card is in {} (svrngs-keeper keep {} runs it again)",
                dir.display(),
                dir.display()
            ),
            Row::Unread { name, why } => write!(f, "{name}   could not be read: {why}"),
        }
    }
}
