//! The keeper program: `give keep`, on a computer that keeps apps (GIVE.md
//! §7, §7a). Each round it reads give's acts in every group this computer
//! holds, folds them ([`give_fold`], the page's fold too), and keeps the
//! apps the fold names this computer for, and only those:
//!
//! - it lends an app's door (`svrngs share`) only while the fold names this
//!   computer, from a healthy start;
//! - it writes a snapshot into the group's folder,
//!   `<folder>/give/<app>/<this computer's key>/<time>/`, at each
//!   [`EVERY`] and at a clean stop, each naming its card version;
//! - it unlends and stops an app the fold names another computer for, with
//!   a last snapshot;
//! - on a yes to an offer of an app it keeps, it unlends it, takes a last
//!   snapshot, writes the move naming that snapshot and its sum, and stops
//!   it (GIVE.md §3 scene 3, §7 "Moving");
//! - it starts an app moved to it only from that snapshot, once its copy
//!   in the group's folder here sums to what the move names.
//!
//! It says `ready` in each group once, so the fold counts this computer
//! among those that can keep. What to do is [`plan`], pure over the folds
//! and what runs here; [`Keeper`] does it, and reads the clock.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use give_fold::{Keeping, Kept};
use serde_json::json;

use crate::card::Card;
use crate::lend::Svrngs;
use crate::podman::Podman;
use crate::record::{Group, Record};
use crate::runtime::{Agreed, App, Name};
use crate::shelf::Shelf;

/// How often a kept app's state is snapshotted into the group's folder:
/// [`give_fold::EVERY`], whose doc says why.
pub const EVERY: Duration = Duration::from_secs(give_fold::EVERY);

/// How many of its own snapshots of an app this computer leaves in the
/// folder: [`give_fold::KEEP`], whose doc says why.
pub use give_fold::KEEP;

/// How long a start that failed waits before the same act is tried again.
pub const RETRY: Duration = Duration::from_secs(60);

/// How long a round waits for the next.
pub const ROUND: Duration = Duration::from_secs(2);

/// An app this computer keeps now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Running {
    pub group: String,
    /// The act it keeps it by.
    pub act: String,
    /// Where its door listens on this computer's loopback.
    pub at: u16,
    /// When its last snapshot was taken, in seconds.
    pub snapshot: u64,
}

/// One thing to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// Say in `group` that this computer can keep.
    Ready {
        group: String,
    },
    Start {
        group: String,
        app: String,
        keeping: Keeping,
    },
    Snapshot {
        app: String,
    },
    /// Unlend it, snapshot it, write its move to the computer whose person
    /// said yes, and stop it.
    Move {
        app: String,
        keeping: Keeping,
    },
    /// Unlend it, snapshot it and stop it.
    Stop {
        app: String,
        why: String,
    },
    /// A keeping this computer will not take up, and why.
    Refuse {
        group: String,
        app: String,
        why: String,
    },
}

/// What this computer does now: `folds` being each group's fold, `running`
/// what it keeps now, `failed` each app's last start or move that failed
/// (its act or its offer's, and when), and `now` the time in seconds. Stops come first, so an app
/// that changes keeping here stops before it starts again.
pub fn plan(
    me: &str,
    folds: &[(String, Kept)],
    running: &BTreeMap<String, Running>,
    failed: &BTreeMap<String, (String, u64)>,
    now: u64,
    every: Duration,
) -> Vec<Step> {
    let (mut steps, mut starts) = (Vec::new(), Vec::new());
    let mut wanted: BTreeMap<&str, (&str, &Keeping)> = BTreeMap::new();
    for (group, kept) in folds {
        if !kept.can_keep.contains_key(me) {
            steps.push(Step::Ready { group: group.clone() });
        }
        for (app, keeping) in kept.apps.iter().filter(|(_, k)| k.keeper == me) {
            // svrngs lends a port to every group its person is in, under one
            // name, so this computer keeps one app of a kind at a time.
            if let Some((other, _)) = wanted.get(app.as_str()) {
                let why = format!("this computer keeps {app} for {other} already");
                steps.push(Step::Refuse {
                    group: group.clone(),
                    app: app.clone(),
                    why,
                });
                continue;
            }
            wanted.insert(app, (group, keeping));
        }
    }
    let waits = |app: &str, act: &str| {
        (failed.get(app)).is_some_and(|(a, at)| a == act && now.saturating_sub(*at) < RETRY.as_secs())
    };
    for (app, r) in running {
        match wanted.get(app.as_str()) {
            Some((g, k)) if *g == r.group && k.act == r.act => {
                let yes = (k.offer.as_ref()).filter(|o| o.answer == give_fold::Answer::Yes);
                if let Some(yes) = yes {
                    if !waits(app, &yes.act) {
                        let keeping = (*k).clone();
                        steps.push(Step::Move {
                            app: app.clone(),
                            keeping,
                        });
                    }
                } else if now.saturating_sub(r.snapshot) >= every.as_secs() {
                    steps.push(Step::Snapshot { app: app.clone() });
                }
            }
            Some(_) => steps.push(Step::Stop {
                app: app.clone(),
                why: "it is kept here by another act now, so it starts again from that".into(),
            }),
            None => steps.push(Step::Stop {
                app: app.clone(),
                why: "the group's record names another computer to keep it, or none".into(),
            }),
        }
    }
    for (app, (group, k)) in wanted {
        let same = running.get(app).is_some_and(|r| r.group == group && r.act == k.act);
        if !same && !waits(app, &k.act) {
            starts.push(Step::Start {
                group: group.into(),
                app: app.into(),
                keeping: k.clone(),
            });
        }
    }
    steps.extend(starts);
    steps
}

/// Why a start did not happen.
#[derive(Debug, thiserror::Error)]
pub enum StartError {
    #[error("this computer has no card for {0}; give here is older or newer than the one that started it")]
    NoCard(String),
    #[error("{app} was started with version {wanted} of its card, and this computer has version {here}")]
    OtherVersion { app: String, wanted: String, here: String },
    #[error("whoever keeps {0} has not said yes to what its makers ask of whoever runs it")]
    NotAgreed(String),
    #[error("this computer keeps no folder of {0}, so it has nowhere to keep copies of the app")]
    NoFolder(String),
    /// Not a failure: the copy it moves from is still on its way here.
    #[error("{app}'s last copy, {from}, has not all reached this computer yet")]
    NotHereYet { app: String, from: String },
    #[error("{0}")]
    Runtime(#[from] crate::runtime::RunError),
    #[error("{0}")]
    Machine(#[from] crate::machine::MachineError),
    #[error("{0}")]
    Lend(#[from] crate::lend::LendError),
    #[error("{path}: {why}")]
    File { path: PathBuf, why: String },
}

/// The keeper program's state, and what it reaches through.
pub struct Keeper {
    pub record: Record,
    pub podman: Podman,
    pub svrngs: Svrngs,
    pub shelf: Shelf,
    /// What the app's pod and volumes are named by: `<prefix><app>`.
    pub prefix: String,
    /// give's own files on this computer: which act each app's volumes
    /// hold (`<dir>/<app>.act`).
    pub dir: PathBuf,
    pub every: Duration,
    pub running: BTreeMap<String, Running>,
    pub failed: BTreeMap<String, (String, u64)>,
    /// Each group's folder, as the last round read it.
    folders: BTreeMap<String, PathBuf>,
    /// The refusals already traced, by act, so a round says each once.
    told: std::collections::BTreeSet<String>,
}

impl Keeper {
    pub fn new(record: Record, shelf: Shelf, prefix: String, dir: PathBuf, every: Duration) -> Keeper {
        Keeper {
            record,
            podman: Podman::default(),
            svrngs: Svrngs::default(),
            shelf,
            prefix,
            dir,
            every,
            running: BTreeMap::new(),
            failed: BTreeMap::new(),
            folders: BTreeMap::new(),
            told: Default::default(),
        }
    }

    /// One round, at `now` in seconds: read, fold, plan, do. This
    /// computer's key, as svrngs says it.
    pub fn round(&mut self, now: u64) -> Result<String, crate::record::RecordError> {
        let groups = self.record.groups()?;
        let mut folds = Vec::new();
        for Group { id, folder, .. } in &groups.groups {
            let kept = give_fold::fold(&self.record.acts(id)?);
            for r in kept.refused.iter().filter(|r| self.told.insert(r.act.clone())) {
                tracing::debug!(group = id, act = r.act, refusal = r.code(), why = %r, "keep: the fold refused an act");
            }
            if let Some(folder) = folder {
                self.folders.insert(id.clone(), folder.clone());
            }
            folds.push((id.clone(), kept));
        }
        let steps = plan(&groups.key, &folds, &self.running, &self.failed, now, self.every);
        for step in steps {
            tracing::debug!(?step, "keep: a step");
            self.step(&groups.key, step, now);
        }
        Ok(groups.key)
    }

    fn step(&mut self, me: &str, step: Step, now: u64) {
        match step {
            Step::Ready { group } => match self.ready() {
                Ok(()) => match self.record.write(&group, json!({ "kind": "ready" })) {
                    Ok(()) => println!("This computer can keep apps for {group} now."),
                    Err(why) => eprintln!("give: could not say so in {group}: {why}"),
                },
                Err(why) => tracing::debug!(group, %why, "keep: not ready, so not said"),
            },
            Step::Refuse { group, app, why } => tracing::debug!(group, app, why, "keep: refused"),
            Step::Start { group, app, keeping } => match self.start(me, &group, &app, &keeping, now) {
                Ok(r) => {
                    println!("Keeping {app} for {group}, on 127.0.0.1:{}.", r.at);
                    self.failed.remove(&app);
                    self.running.insert(app, r);
                }
                Err(StartError::NotHereYet { app, from }) => {
                    tracing::debug!(
                        group,
                        app,
                        from,
                        act = keeping.act,
                        "keep: waits for the copy it moves from"
                    );
                    if self.told.insert(format!("{}#here", keeping.act)) {
                        println!("Waiting for {app}'s last copy to reach this computer.");
                    }
                }
                Err(why) => {
                    tracing::debug!(group, app, act = keeping.act, %why, "keep: refused or failed to start");
                    eprintln!("give: {app} did not start here: {why}");
                    self.failed.insert(app, (keeping.act, now));
                }
            },
            Step::Snapshot { app } => {
                if let Err(why) = self.snapshot(me, &app) {
                    eprintln!("give: {app} was not copied to the group: {why}");
                }
                if let Some(r) = self.running.get_mut(&app) {
                    r.snapshot = now;
                }
            }
            Step::Move { app, keeping } => self.hand(me, &app, &keeping, now),
            Step::Stop { app, why } => {
                tracing::debug!(app, why, "keep: stopping");
                self.stop(me, &app);
                println!("Stopped keeping {app}: {why}.");
            }
        }
    }

    /// Whether this computer can keep: podman answers, rootless.
    fn ready(&self) -> Result<(), crate::podman::PodmanError> {
        self.podman
            .run(&["info", "--format", "{{.Host.Security.Rootless}}"])
            .map(drop)
    }

    fn name(&self, app: &str) -> Result<Name, crate::runtime::RunError> {
        Name::new(&format!("{}{app}", self.prefix))
    }

    fn card(&self, app: &str) -> Result<&Card, StartError> {
        self.shelf.get(app).ok_or_else(|| StartError::NoCard(app.into()))
    }

    fn start(&self, me: &str, group: &str, app: &str, k: &Keeping, now: u64) -> Result<Running, StartError> {
        let card = self.card(app)?;
        if card.version != k.card {
            let (wanted, here) = (k.card.clone(), card.version.clone());
            return Err(StartError::OtherVersion {
                app: app.into(),
                wanted,
                here,
            });
        }
        if !card.asks.is_empty() && !k.agreed {
            return Err(StartError::NotAgreed(app.into()));
        }
        let folder = self
            .folders
            .get(group)
            .ok_or_else(|| StartError::NoFolder(group.into()))?;
        let held = self.dir.join(format!("{app}.act"));
        let holds = std::fs::read_to_string(&held).ok().is_some_and(|h| h == k.act);
        // A move names its snapshot's sum: start only from exactly that.
        if let (Some(from), Some(sum), false) = (&k.from, &k.sum, holds) {
            let here = crate::runtime::sum(&folder.join(from)).ok();
            tracing::debug!(app, from, sum, ?here, "keep: the copy it moves from, as it is here");
            if here.as_ref() != Some(sum) {
                let (app, from) = (app.into(), from.clone());
                return Err(StartError::NotHereYet { app, from });
            }
        }
        crate::machine::ensure(&self.podman, card.memory())?;
        let run = App::new(&self.podman, card, self.name(app)?);
        run.pull()?;
        run.stop()?;
        match (&k.from, holds) {
            // The volumes here are this keeping's already: a restart.
            (_, true) => tracing::debug!(app, act = k.act, "keep: the state here is this keeping's"),
            (Some(from), false) => run.restore(&folder.join(from))?,
            // A fresh start: whatever state is here is another keeping's,
            // copied to the group as it stopped.
            (None, false) => run.remove()?,
        }
        write(&held, &k.act)?;
        let at = free_port().map_err(|e| file(Path::new("127.0.0.1:0"), e))?;
        run.start(at, &Agreed::to(&card.asks))?;
        let lent = (run.wait_healthy().map_err(StartError::from))
            .and_then(|()| Ok(self.svrngs.share(at, app, card.door.protocol)?));
        if let Err(why) = lent {
            let _ = run.stop();
            return Err(why);
        }
        tracing::debug!(app, group, me, at, act = k.act, "keep: kept, and lent");
        Ok(Running {
            group: group.into(),
            act: k.act.clone(),
            at,
            snapshot: now,
        })
    }

    /// A snapshot of `app` into its group's folder, and this computer's
    /// older ones there past [`KEEP`] removed.
    fn snapshot(&self, me: &str, app: &str) -> Result<PathBuf, StartError> {
        let r = self.running.get(app).ok_or_else(|| StartError::NoCard(app.into()))?;
        let folder = self
            .folders
            .get(&r.group)
            .ok_or_else(|| StartError::NoFolder(r.group.clone()))?;
        let into = folder.join("give").join(app).join(me);
        let card = self.card(app)?;
        let snapshot = App::new(&self.podman, card, self.name(app)?).snapshot(&into)?;
        tracing::debug!(app, dir = %snapshot.dir.display(), paused = ?snapshot.paused, "keep: copied to the group");
        prune(&into, KEEP);
        Ok(snapshot.dir)
    }

    /// Move `app` on the yes to its offer: unlend it, take a last snapshot,
    /// write the move naming it and its sum, and stop it. A step that fails
    /// is said, the app lent again where it still runs, and the move tried
    /// again after [`RETRY`].
    fn hand(&mut self, me: &str, app: &str, k: &Keeping, now: u64) {
        let Some((yes, r)) = k.offer.clone().zip(self.running.get(app).cloned()) else {
            return tracing::debug!(app, "keep: no yes, or not kept here, so not moved");
        };
        let failed = |keeper: &mut Keeper, why: String| {
            tracing::debug!(
                app,
                offer = yes.act,
                why,
                "keep: the move failed, so it waits to be tried again"
            );
            eprintln!("give: {app} did not move to {}’s computer: {why}", yes.person);
            keeper.failed.insert(app.into(), (yes.act.clone(), now));
            let lent = (keeper.card(app)).map(|card| keeper.svrngs.share(r.at, app, card.door.protocol));
            if !matches!(lent, Ok(Ok(()))) {
                eprintln!("give: {app} could not be lent again here");
            }
        };
        if let Err(why) = self.svrngs.unshare(app) {
            return failed(self, format!("it could not be unlent: {why}"));
        }
        let last = self.snapshot(me, app).and_then(|dir| {
            let sum = crate::runtime::sum(&dir)?;
            let folder = self
                .folders
                .get(&r.group)
                .ok_or_else(|| StartError::NoFolder(r.group.clone()))?;
            let from = dir.strip_prefix(folder).unwrap_or(&dir).to_string_lossy().to_string();
            Ok((from, sum))
        });
        let (from, sum) = match last {
            Ok(last) => last,
            Err(why) => return failed(self, format!("its last copy was not taken: {why}")),
        };
        let moved = json!({ "kind": "moved", "app": app, "card": k.card, "over": k.act, "from": from, "sum": sum });
        if let Err(why) = self.record.write(&r.group, moved) {
            return failed(self, format!("the move was not written: {why}"));
        }
        tracing::debug!(app, to = yes.to, from, sum, "keep: moved, from its last copy");
        let stopped = self
            .card(app)
            .and_then(|card| Ok(App::new(&self.podman, card, self.name(app)?).stop()?));
        match stopped {
            Ok(()) => {
                self.running.remove(app);
                println!("Moved {app} to {}’s computer, from its last copy.", yes.person);
            }
            // Kept as running, so the next round stops it: the record names
            // another computer now.
            Err(why) => eprintln!("give: {app} moved, and could not be stopped here: {why}"),
        }
    }

    /// Unlend `app`, snapshot it, stop it: each step said if it fails, and
    /// the next still taken.
    pub fn stop(&mut self, me: &str, app: &str) {
        if let Err(why) = self.svrngs.unshare(app) {
            eprintln!("give: {app} could not be unlent: {why}");
        }
        if let Err(why) = self.snapshot(me, app) {
            eprintln!("give: {app} was not copied to the group as it stopped: {why}");
        }
        let stopped = self.card(app).and_then(|card| {
            let run = App::new(&self.podman, card, self.name(app)?);
            Ok(run.stop()?)
        });
        if let Err(why) = stopped {
            eprintln!("give: {app} could not be stopped: {why}");
        }
        self.running.remove(app);
    }

    /// Every app this computer keeps, stopped cleanly: a last snapshot each.
    pub fn stop_all(&mut self, me: &str) {
        let apps: Vec<_> = self.running.keys().cloned().collect();
        for app in apps {
            self.stop(me, &app);
            println!("Stopped keeping {app}, its last state copied to the group.");
        }
    }
}

/// The snapshots under `dir` past the newest `keep`, removed. A snapshot's
/// name is when it was taken, in milliseconds (runtime.rs `snapshot`).
fn prune(dir: &Path, keep: usize) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
    let mut taken: Vec<(u128, PathBuf)> = read
        .flatten()
        .filter_map(|e| Some((e.file_name().to_str()?.parse().ok()?, e.path())))
        .collect();
    taken.sort();
    let old = taken.len().saturating_sub(keep);
    for (_, path) in taken.into_iter().take(old) {
        let removed = std::fs::remove_dir_all(&path);
        tracing::debug!(path = %path.display(), ?removed, "keep: an older copy removed");
    }
}

/// A port free on this computer's loopback for TCP and UDP alike, since a
/// door may be either.
fn free_port() -> std::io::Result<u16> {
    loop {
        let port = std::net::TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
        if std::net::UdpSocket::bind(("127.0.0.1", port)).is_ok() {
            return Ok(port);
        }
    }
}

fn file(path: &Path, e: std::io::Error) -> StartError {
    StartError::File {
        path: path.to_path_buf(),
        why: e.to_string(),
    }
}

fn write(path: &Path, text: &str) -> Result<(), StartError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| file(dir, e))?;
    }
    std::fs::write(path, text).map_err(|e| file(path, e))
}
