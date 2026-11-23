//! Linux takeover transaction. The journal precedes every change to the
//! staged unit. Only an explicitly confirmed transaction can signal pids;
//! replay observes the port and never calls the stop path.

use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, anyhow, bail, ensure};
use serde::{Deserialize, Serialize};
use svrngs_keeper::{
    host,
    kept::{Boot, Places},
    runtime::Name,
    systemd::{SystemdUser, UnitState},
    unit::unit_name,
};

use crate::facts::{Draft, Recipe};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Stage {
    Prepared,
    Staged,
    Stopping,
    Stopped,
    Started,
    Verified,
    Rollback,
}

#[derive(Serialize, Deserialize)]
struct Record {
    boot: String,
    token: String,
    enable_linger: bool,
    stage: Stage,
    recipe: Recipe,
}

pub enum Outcome {
    Verified,
    Unchanged(String),
    Restored(String),
}

enum Recovered {
    Kept,
    Original,
    Restored,
    Interrupted,
}

pub fn boot_id() -> anyhow::Result<String> {
    let boot = fs::read_to_string("/proc/sys/kernel/random/boot_id").context("read boot id")?;
    ensure!(!boot.trim().is_empty(), "the kernel did not give a boot id");
    Ok(boot.trim().into())
}

fn read(path: &Path) -> anyhow::Result<Option<Record>> {
    match fs::read(path) {
        Ok(json) => Ok(Some(
            serde_json::from_slice(&json).with_context(|| format!("read {}", path.display()))?,
        )),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("read {}", path.display())),
    }
}

fn sync_dir(dir: &Path) -> io::Result<()> {
    File::open(dir)?.sync_all()
}

fn mkdir(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        return Ok(());
    }
    let Some(parent) = path.parent() else {
        return Err(io::ErrorKind::NotFound.into());
    };
    mkdir(parent)?;
    match fs::create_dir(path) {
        Ok(()) => sync_dir(parent),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists && path.is_dir() => sync_dir(parent),
        Err(e) => Err(e),
    }
}

fn atomic(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    (|| -> io::Result<()> {
        let dir = path.parent().expect("keeper file has a directory");
        mkdir(dir)?;
        let tmp = path.with_extension("keeper-tmp");
        unlink(&tmp)?;
        let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&tmp, path)?;
        sync_dir(dir)
    })()
    .with_context(|| format!("store {} durably", path.display()))
}

fn unlink(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => sync_dir(path.parent().expect("keeper file has a directory")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

fn lock(places: &Places, key: &str) -> anyhow::Result<File> {
    let dir = places.data.join("locks");
    mkdir(&dir)?;
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(key))?;
    file.try_lock()
        .with_context(|| format!("keeper lock {key} could not be taken"))?;
    Ok(file)
}

pub fn lock_port(places: &Places, port: u16) -> anyhow::Result<File> {
    lock(places, &format!("port-{port}"))
}

fn lock_name(places: &Places, name: &Name) -> anyhow::Result<File> {
    lock(places, &format!("name-{}", name.as_str()))
}

pub fn lock_card(places: &Places, name: &Name) -> anyhow::Result<File> {
    let guard = lock_name(places, name)?;
    ensure!(
        !places.takeover_path(name).try_exists()?,
        "{} has a takeover in progress",
        name.as_str()
    );
    Ok(guard)
}

fn pending(places: &Places) -> anyhow::Result<Vec<(Name, Record)>> {
    let dirs = match fs::read_dir(&places.data) {
        Ok(dirs) => dirs,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut pending = Vec::new();
    for dir in dirs {
        let dir = dir?;
        if dir.file_type()?.is_dir()
            && let Some(name) = dir.file_name().to_str().and_then(|n| Name::new(n).ok())
            && let Some(record) = read(&places.takeover_path(&name))?
        {
            pending.push((name, record));
        }
    }
    pending.sort_unstable_by(|(a, _), (b, _)| a.as_str().cmp(b.as_str()));
    Ok(pending)
}

pub fn lock_free_port(places: &Places, port: u16) -> anyhow::Result<File> {
    let guard = lock_port(places, port)?;
    ensure!(
        !pending(places)?.iter().any(|(_, r)| r.recipe.port == port),
        "port {port} has a takeover in progress"
    );
    Ok(guard)
}

pub fn reconcile(systemd: &SystemdUser, places: &Places) -> anyhow::Result<()> {
    for (name, record) in pending(places)? {
        let _port = lock_port(places, record.recipe.port)?;
        let _name = lock_name(places, &name)?;
        let job = Job::new(systemd, places, &name);
        if let Some(mut latest) = read(&job.journal)? {
            ensure!(
                latest.recipe.port == record.recipe.port,
                "{} moved to another port; try again",
                job.unit
            );
            job.recover(&mut latest, &boot_id()?, true)?;
        }
    }
    Ok(())
}

pub fn take(systemd: &SystemdUser, places: &Places, draft: &Draft, consent: Boot) -> anyhow::Result<Outcome> {
    let port = draft.recipe.port;
    let _port = lock_free_port(places, port)?;
    let _name = lock_card(places, &draft.name)?;
    let job = Job::new(systemd, places, &draft.name);
    ensure!(
        !job.unit_path.try_exists()? && systemd.state(&job.unit) == UnitState::NotInstalled,
        "{} already exists or could not be read; choose another name",
        job.unit
    );
    let boot = boot_id()?;
    let current = Draft::read(systemd, &boot, port, Some(draft.name.as_str()))
        .context("what runs there changed; review it again")?;
    ensure!(
        current.offer.token == draft.offer.token,
        "what runs there changed; review it again"
    );
    let mut record = Record {
        boot,
        token: current.offer.token,
        enable_linger: consent == Boot::TurnOn,
        stage: Stage::Prepared,
        recipe: current.recipe,
    };
    job.save(&mut record, Stage::Prepared)?;
    let taken = (|| {
        job.stage(&record)?;
        job.save(&mut record, Stage::Staged)?;
        job.save(&mut record, Stage::Stopping)?;
        host::stop(&record.recipe.chain, port).map_err(|e| anyhow!(e))?;
        job.save(&mut record, Stage::Stopped)?;
        ensure!(
            host::ss(port).map_err(|e| anyhow!(e))?.is_empty(),
            "another process is on port {port}"
        );
        job.start(&mut record)?;
        ensure!(
            job.await_answer(&record.recipe, &job.unit),
            "{} did not answer on port {port} within 30 s",
            job.unit
        );
        Ok::<(), anyhow::Error>(())
    })();
    if let Err(why) = taken {
        tracing::debug!(unit = job.unit, %why, "takeover:failed");
        return match job.recover(&mut record, &boot_id()?, false)? {
            Recovered::Original => Ok(Outcome::Unchanged(why.to_string())),
            Recovered::Restored => Ok(Outcome::Restored(why.to_string())),
            Recovered::Kept => Ok(Outcome::Verified),
            Recovered::Interrupted => Err(why),
        };
    }
    job.enable(&mut record)?;
    Ok(Outcome::Verified)
}

struct Job<'a> {
    systemd: &'a SystemdUser,
    unit: String,
    scope: String,
    unit_path: std::path::PathBuf,
    journal: std::path::PathBuf,
    interrupted: std::path::PathBuf,
}

impl<'a> Job<'a> {
    fn new(systemd: &'a SystemdUser, places: &Places, name: &Name) -> Self {
        Self {
            systemd,
            unit: unit_name(name),
            scope: format!("svrngs-keeper-{}-rollback.scope", name.as_str()),
            unit_path: places.unit_path(name),
            journal: places.takeover_path(name),
            interrupted: places.interrupted_path(name),
        }
    }

    fn ctl(&self, args: &[&str]) -> anyhow::Result<()> {
        self.systemd.ctl(args).map_err(|e| anyhow!(e))
    }

    fn save(&self, record: &mut Record, stage: Stage) -> anyhow::Result<()> {
        record.stage = stage;
        atomic(&self.journal, &serde_json::to_vec(record)?)?;
        let stage = serde_json::to_value(stage)?;
        let stage = stage.as_str().expect("stage has a wire name");
        tracing::debug!(unit = self.unit, stage, "takeover:stage");
        checkpoint(stage);
        Ok(())
    }

    // Check the file before driving the manager; recovery never removes a
    // job installed over the staged one, even if it has the same name.
    fn staged(&self, recipe: &Recipe) -> anyhow::Result<bool> {
        match fs::read_to_string(&self.unit_path) {
            Ok(text) if text == recipe.unit_text => Ok(true),
            Ok(_) => bail!("{} changed during takeover", self.unit_path.display()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    fn stage(&self, record: &Record) -> anyhow::Result<()> {
        if !self.staged(&record.recipe)? {
            atomic(&self.unit_path, record.recipe.unit_text.as_bytes())?;
        }
        self.ctl(&["--user", "daemon-reload"])
    }

    fn drop_unit(&self, recipe: &Recipe, remove: bool) -> anyhow::Result<()> {
        if self.staged(recipe)? {
            self.ctl(&["--user", "disable", "--now", &self.unit])?;
            if remove {
                unlink(&self.unit_path)?;
                self.ctl(&["--user", "daemon-reload"])?;
            }
        }
        Ok(())
    }

    fn start(&self, record: &mut Record) -> anyhow::Result<()> {
        self.ctl(&["--user", "start", &self.unit])?;
        self.save(record, Stage::Started)
    }

    fn owner(&self, sockets: &[svrngs_keeper::promote::Listen], unit: &str, recipe: &Recipe) -> bool {
        !sockets.is_empty()
            && self.systemd.show(unit, &["ControlGroup"]).is_ok_and(|show| {
                show.lines()
                    .find_map(|l| l.strip_prefix("ControlGroup="))
                    .is_some_and(|group| {
                        group.starts_with('/')
                            && host::owns(sockets, |pid| host::in_group(pid, group, &recipe.listener_exe))
                    })
            })
    }

    fn active(&self) -> bool {
        self.systemd.show(&self.unit, &["ActiveState"]).is_ok_and(|s| {
            matches!(
                s.trim(),
                "ActiveState=active" | "ActiveState=activating" | "ActiveState=reloading"
            )
        })
    }

    fn await_answer(&self, recipe: &Recipe, unit: &str) -> bool {
        let owns = || host::ss(recipe.port).is_ok_and(|sockets| self.owner(&sockets, unit, recipe));
        let mut verified = false;
        host::within(30, || {
            verified = owns() && host::answers(&recipe.dial, recipe.port, recipe.http) && owns();
            verified
                || self
                    .systemd
                    .show(unit, &["ActiveState"])
                    .is_ok_and(|s| s.trim() == "ActiveState=failed")
        });
        verified
    }

    fn enable(&self, record: &mut Record) -> anyhow::Result<()> {
        checkpoint("answered");
        self.ctl(&["--user", "enable", &self.unit])?;
        let dir = self.unit_path.parent().expect("unit has a directory");
        sync_dir(&dir.join("default.target.wants"))?;
        sync_dir(dir)?;
        checkpoint("enabled");
        self.save(record, Stage::Verified)?;
        self.finish_verified(record)
    }

    fn finish_verified(&self, record: &Record) -> anyhow::Result<()> {
        if record.enable_linger {
            self.systemd.ensure_linger("svrngs-keeper", "programs it keeps");
        }
        self.finish(record)
    }

    fn finish(&self, record: &Record) -> anyhow::Result<()> {
        let card = self.journal.with_file_name("compose.yaml");
        match fs::read(&card) {
            Ok(prior) if prior != record.recipe.card.as_bytes() => {
                atomic(&card.with_file_name("previous.yaml"), &prior)?
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        atomic(&card, record.recipe.card.as_bytes())?;
        self.clear()
    }

    fn clear(&self) -> anyhow::Result<()> {
        // A kill during a journal rewrite can leave the private temporary copy.
        unlink(&self.journal.with_extension("keeper-tmp"))?;
        unlink(&self.journal)?;
        Ok(())
    }

    fn recover(&self, record: &mut Record, boot: &str, interrupted: bool) -> anyhow::Result<Recovered> {
        if record.stage == Stage::Verified {
            tracing::debug!(unit = self.unit, "takeover:finish_verified");
            self.finish_verified(record)?;
            return Ok(Recovered::Kept);
        }
        let port = record.recipe.port;
        let sockets = host::ss(port).map_err(|e| anyhow!(e))?;
        let listener = record
            .recipe
            .chain
            .first()
            .context("takeover journal has no listener")?;
        let same_boot = record.boot == boot;
        let original = same_boot && host::started(listener.pid).map_err(|e| anyhow!(e))? == Some(listener.start);
        if original
            && host::owns(&sockets, |pid| {
                pid == listener.pid && host::started(pid) == Ok(Some(listener.start))
            })
        {
            tracing::debug!(port, "takeover:original_still_listens");
            self.drop_unit(&record.recipe, true)?;
            self.clear()?;
            return Ok(Recovered::Original);
        }
        let kept = self.staged(&record.recipe)? && self.owner(&sockets, &self.unit, &record.recipe);
        let restored =
            same_boot && record.stage == Stage::Rollback && self.owner(&sockets, &self.scope, &record.recipe);
        let occupied = !sockets.is_empty() && !kept && !restored;
        if same_boot && occupied {
            tracing::debug!(port, "takeover:port_taken_no_signals");
            bail!("another process is on port {port}; nothing was stopped and the takeover is pending");
        }
        if interrupted
            && record.stage != Stage::Rollback
            && (kept || (!occupied && self.active()))
            && self.await_answer(&record.recipe, &self.unit)
        {
            tracing::debug!(port, "takeover:recovered_kept");
            self.enable(record)?;
            return Ok(Recovered::Kept);
        }
        if same_boot {
            ensure!(
                !original,
                "the original pid still runs but is not listening on {port}; takeover is pending"
            );
            for parent in record.recipe.chain.iter().skip(1) {
                ensure!(
                    host::started(parent.pid).map_err(|e| anyhow!(e))? != Some(parent.start),
                    "an original parent still runs; takeover is pending"
                );
            }
            return self.rollback(record, restored).map(|()| Recovered::Restored);
        }
        tracing::debug!(port, "takeover:recover_after_boot");
        let interrupted_at = serde_json::to_string(&record.stage)?;
        self.stage(record)?;
        if !occupied && self.start(record).is_ok() && self.await_answer(&record.recipe, &self.unit) {
            self.enable(record)?;
            Ok(Recovered::Kept)
        } else {
            tracing::debug!(port, "takeover:interrupted_needs_start");
            self.drop_unit(&record.recipe, false)?;
            atomic(&self.interrupted, interrupted_at.trim_matches('"').as_bytes())?;
            self.finish(record)?;
            Ok(Recovered::Interrupted)
        }
    }

    fn rollback(&self, record: &mut Record, already_answers: bool) -> anyhow::Result<()> {
        self.save(record, Stage::Rollback)?;
        self.drop_unit(&record.recipe, true)?;
        if !already_answers
            && !self
                .systemd
                .show(&self.scope, &["ControlGroup"])
                .is_ok_and(|s| s.contains("ControlGroup=/"))
        {
            let log = self.journal.with_file_name("original.log");
            let out = OpenOptions::new().create(true).append(true).mode(0o600).open(log)?;
            Command::new("systemd-run")
                .args(["--user", "--scope", "--quiet", "--unit", &self.scope, "--"])
                .arg(std::env::current_exe()?)
                .arg("__rollback")
                .arg(&self.journal)
                .stdin(Stdio::null())
                .stdout(out.try_clone()?)
                .stderr(out)
                .spawn()?;
        }
        ensure!(
            self.await_answer(&record.recipe, &self.scope),
            "the original did not answer after rollback; see {}",
            self.journal.with_file_name("original.log").display()
        );
        tracing::debug!(unit = self.unit, "takeover:restored_original");
        checkpoint("restored");
        self.clear()
    }
}

pub fn exec_original(path: &Path) -> anyhow::Result<()> {
    let record = read(path)?.context("no takeover left to roll back")?;
    ensure!(
        record.stage == Stage::Rollback && record.boot == boot_id()?,
        "not rolling back this boot"
    );
    let port = record.recipe.port;
    ensure!(
        host::ss(port).map_err(|e| anyhow!(e))?.is_empty(),
        "another process is on port {port}"
    );
    Err(record.recipe.original.exec()).context("exec original in its rollback scope")
}

fn checkpoint(stage: &str) {
    // A debug build can be SIGKILLed at each real on-disk edge by tests/takeover.py.
    if cfg!(debug_assertions) && std::env::var("SVRNGS_KEEPER_TEST_PAUSE").as_deref() == Ok(stage) {
        rustix::process::kill_process(rustix::process::getpid(), rustix::process::Signal::STOP)
            .expect("pause the host test");
    }
}
