//! The runtime: one card, kept on one computer, in rootless podman. It
//! starts the app, says whether it is healthy, snapshots its state into a
//! directory, restores it from one, and stops it. It knows no group and
//! names no svrngs type (GIVE.md §7a, decision 1), so it changes when podman
//! does and is tested without a group.
//!
//! An app is one pod named for it, its door published on this computer's
//! loopback alone; each service is a container in that pod, and each named
//! volume `<name>-<volume>`. Services reach each other by name, as under
//! compose, through the pod's hosts file.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::card::{Ask, Card, HealthTest, Service};
use crate::podman::{Podman, PodmanError};

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error(transparent)]
    Podman(#[from] PodmanError),
    #[error("{0} is not a name give runs an app under: lowercase letters, digits and -, at most 40")]
    Name(String),
    #[error("Podman here runs as root; give keeps apps only in rootless Podman")]
    Rootful,
    #[error("{service} stopped, exit {code}. Its last lines:\n{logs}")]
    Exited { service: String, code: i32, logs: String },
    #[error("{service} was not healthy within {after:?}. Its last lines:\n{logs}")]
    Unhealthy {
        service: String,
        after: Duration,
        logs: String,
    },
    #[error("{path}: {why}")]
    File { path: PathBuf, why: String },
}

/// The name an app runs under: its pod, and the prefix of its containers
/// and volumes. Tests use `svrngs-test-*`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(String);

impl Name {
    pub fn new(name: &str) -> Result<Name, RunError> {
        let ok = (1..=40).contains(&name.len())
            && name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            && !name.starts_with('-');
        if ok {
            Ok(Name(name.to_string()))
        } else {
            Err(RunError::Name(name.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The yes of whoever runs the app to each of its card's asks: the
/// environment those yeses set. A game's terms bind the person who runs it,
/// so the keeper's program builds this only from that person's answer.
#[derive(Debug, Clone)]
pub struct Agreed(Vec<(String, String)>);

impl Agreed {
    pub fn to(asks: &[Ask]) -> Agreed {
        Agreed(asks.iter().flat_map(|a| a.sets.iter().cloned()).collect())
    }
}

/// How a snapshot's copy was made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Copy {
    /// The filesystem shares the blocks: milliseconds, whatever the size.
    Reflink,
    /// The filesystem has no reflink (ext4), so the app stays paused as
    /// long as the copy takes.
    Plain,
}

/// A snapshot on disk: `<dir>/<volume>.tar` for each state volume, and
/// `<dir>/<volume>.root`, its own folder's owner and mode.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub dir: PathBuf,
    pub copies: Vec<(String, Copy)>,
    /// How long the app was paused.
    pub paused: Duration,
}

/// What an app is doing now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Health {
    Healthy,
    /// Running, and its healthcheck has not passed yet.
    Starting {
        service: String,
    },
    Exited {
        service: String,
        code: i32,
    },
}

/// One card, kept under one name.
pub struct App<'a> {
    podman: &'a Podman,
    card: &'a Card,
    name: Name,
}

impl<'a> App<'a> {
    pub fn new(podman: &'a Podman, card: &'a Card, name: Name) -> App<'a> {
        App { podman, card, name }
    }

    fn container(&self, service: &Service) -> String {
        format!("{}-{}", self.name.0, service.name)
    }

    fn volume(&self, volume: &str) -> String {
        format!("{}-{volume}", self.name.0)
    }

    /// Each image, by its digest, unless this computer has it already.
    pub fn pull(&self) -> Result<(), RunError> {
        for s in &self.card.services {
            let image = s.image.as_str();
            if self.podman.answers(&["image", "exists", image])? {
                tracing::debug!(image, "already here");
            } else {
                self.podman.run(&["pull", "--quiet", image])?;
                tracing::debug!(image, "pulled");
            }
        }
        Ok(())
    }

    /// Start the app with its door on `127.0.0.1:<at>`. Its volumes are made
    /// if they are not here, and kept as they are if they are.
    pub fn start(&self, at: u16, agreed: &Agreed) -> Result<(), RunError> {
        let (name, door) = (self.name.as_str(), self.card.door);
        for v in &self.card.volumes {
            let v = self.volume(v);
            if !self.podman.answers(&["volume", "exists", &v])? {
                self.podman.run(&["volume", "create", &v])?;
            }
        }
        let publish = format!("127.0.0.1:{at}:{}/{}", door.port, door.protocol.transport().as_str());
        let mut pod = vec![
            "pod".to_string(),
            "create".into(),
            "--name".into(),
            name.into(),
            "--publish".into(),
            publish,
        ];
        for s in &self.card.services {
            pod.extend(["--add-host".into(), format!("{}:127.0.0.1", s.name)]);
        }
        self.podman.run(&pod.iter().map(String::as_str).collect::<Vec<_>>())?;
        for s in &self.card.services {
            let args = self.run_args(s, agreed);
            self.podman.run(&args.iter().map(String::as_str).collect::<Vec<_>>())?;
            tracing::debug!(service = s.name, app = name, "started");
        }
        Ok(())
    }

    fn run_args(&self, s: &Service, agreed: &Agreed) -> Vec<String> {
        let mut a: Vec<String> = [
            "run",
            "--detach",
            "--pod",
            self.name.as_str(),
            "--name",
            &self.container(s),
        ]
        .map(String::from)
        .into();
        for (k, v) in s.environment.iter().chain(&agreed.0) {
            a.extend(["--env".into(), format!("{k}={v}")]);
        }
        for m in &s.mounts {
            let source = m
                .volume
                .as_ref()
                .map(|v| format!("{}:", self.volume(v)))
                .unwrap_or_default();
            a.extend([
                "--volume".into(),
                format!("{source}{}{}", m.target, if m.read_only { ":ro" } else { "" }),
            ]);
        }
        if let Some(h) = &s.health {
            let test = match &h.test {
                HealthTest::Exec(words) => serde_json::to_string(words).expect("a list of strings is JSON"),
                HealthTest::Shell(line) => line.clone(),
            };
            a.extend(["--health-cmd".into(), test]);
            for (flag, d) in [
                ("--health-interval", h.interval),
                ("--health-timeout", h.timeout),
                ("--health-start-period", h.start_period),
            ] {
                a.extend([flag.into(), format!("{}ms", d.as_millis())]);
            }
            a.extend(["--health-retries".into(), h.retries.to_string()]);
        }
        if let Some(bytes) = s.memory {
            a.extend(["--memory".into(), bytes.to_string()]);
        }
        if let Some(u) = &s.user {
            a.extend(["--user".into(), u.clone()]);
        }
        if let Some(w) = &s.working_dir {
            a.extend(["--workdir".into(), w.clone()]);
        }
        if let Some(e) = &s.entrypoint {
            a.extend([
                "--entrypoint".into(),
                serde_json::to_string(e).expect("a list of strings is JSON"),
            ]);
        }
        a.push(s.image.as_str().into());
        a.extend(s.command.iter().flatten().cloned());
        a
    }

    /// Whether each service is running and, where it has a healthcheck,
    /// passing it now. A service with no healthcheck, in its card or its
    /// image, is healthy while it runs.
    pub fn health(&self) -> Result<Health, RunError> {
        for s in &self.card.services {
            let c = self.container(s);
            let state = self.podman.run(&[
                "container",
                "inspect",
                "--format",
                "{{.State.Status}} {{.State.ExitCode}}",
                &c,
            ])?;
            let (status, code) = state.split_once(' ').unwrap_or((&state, "0"));
            if matches!(status, "exited" | "stopped" | "dead") {
                tracing::debug!(service = s.name, status, code, "not running");
                return Ok(Health::Exited {
                    service: s.name.clone(),
                    code: code.parse().unwrap_or(-1),
                });
            }
            if status != "running" {
                tracing::debug!(service = s.name, status, "not running yet");
                return Ok(Health::Starting {
                    service: s.name.clone(),
                });
            }
            if self
                .podman
                .run(&["container", "inspect", "--format", "{{json .Config.Healthcheck}}", &c])?
                == "null"
            {
                tracing::debug!(service = s.name, "no healthcheck, so running is healthy");
                continue;
            }
            if !self.podman.answers(&["healthcheck", "run", &c])? {
                tracing::debug!(service = s.name, "its healthcheck has not passed");
                return Ok(Health::Starting {
                    service: s.name.clone(),
                });
            }
        }
        Ok(Health::Healthy)
    }

    /// Wait until every service is healthy, for as long as compose would:
    /// its `start_period` and then `retries` intervals. Asked each second.
    #[expect(
        clippy::disallowed_methods,
        reason = "the runtime is podman's adapter: it waits on the clock"
    )]
    pub fn wait_healthy(&self) -> Result<(), RunError> {
        let patience = self
            .card
            .services
            .iter()
            .filter_map(|s| s.health.as_ref())
            .map(|h| h.start_period + h.interval * h.retries)
            .max()
            .unwrap_or_default();
        let began = Instant::now();
        loop {
            match self.health()? {
                Health::Healthy => return Ok(()),
                Health::Exited { service, code } => {
                    let logs = self.logs(&service);
                    return Err(RunError::Exited { service, code, logs });
                }
                Health::Starting { service } if began.elapsed() >= patience => {
                    let logs = self.logs(&service);
                    return Err(RunError::Unhealthy {
                        service,
                        after: patience,
                        logs,
                    });
                }
                Health::Starting { .. } => std::thread::sleep(Duration::from_secs(1)),
            }
        }
    }

    fn logs(&self, service: &str) -> String {
        let c = format!("{}-{service}", self.name.0);
        self.podman
            .run(&["logs", "--tail", "20", &c])
            .unwrap_or_else(|e| format!("(no logs: {e})"))
    }

    /// Snapshot the state volumes into a new directory under `into`: the app
    /// paused, each volume copied inside podman's storage (a reflink where
    /// the filesystem has one), the app unpaused, then each copy written out
    /// as `<volume>.tar`. The directory appears whole or not at all.
    #[expect(
        clippy::disallowed_methods,
        reason = "a snapshot is named for when it was taken, and times its pause"
    )]
    pub fn snapshot(&self, into: &Path) -> Result<Snapshot, RunError> {
        let shell = self.storage_shell()?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let partial = into.join(format!(".{stamp}.partial"));
        std::fs::create_dir_all(&partial).map_err(|e| file(&partial, e))?;
        let name = self.name.as_str();
        let paused = Instant::now();
        self.podman.run(&["pod", "pause", name])?;
        let copies: Result<Vec<_>, RunError> = self
            .card
            .state
            .iter()
            .map(|v| Ok((v.clone(), self.copy(&shell, v)?, self.root_of(&shell, &self.volume(v))?)))
            .collect();
        let unpaused = self.podman.run(&["pod", "unpause", name]);
        let paused = paused.elapsed();
        let copies = copies?;
        unpaused?;
        tracing::debug!(app = name, ?paused, ?copies, "paused for the copy");
        for (v, _, root) in &copies {
            let at = partial.join(format!("{v}.root"));
            std::fs::write(&at, root).map_err(|e| file(&at, e))?;
        }
        let copies = copies.into_iter().map(|(v, copy, _)| (v, copy)).collect();
        for v in &self.card.state {
            let tar = partial.join(format!("{v}.tar"));
            let copy = format!("{}-snapshot", self.volume(v));
            self.podman
                .run(&["volume", "export", "--output", &tar.to_string_lossy(), &copy])?;
            self.podman.run(&["volume", "rm", &copy])?;
        }
        let dir = into.join(stamp.to_string());
        std::fs::rename(&partial, &dir).map_err(|e| file(&dir, e))?;
        tracing::debug!(app = name, dir = %dir.display(), "snapshot written");
        Ok(Snapshot { dir, copies, paused })
    }

    /// How a command runs as podman does in its storage, where the volumes
    /// are: `podman unshare` here, or inside podman's machine on a Mac.
    fn storage_shell(&self) -> Result<Vec<&'static str>, RunError> {
        let info = self.podman.run(&[
            "info",
            "--format",
            "{{.Host.ServiceIsRemote}} {{.Host.Security.Rootless}}",
        ])?;
        match info.as_str() {
            "false true" => Ok(vec!["unshare"]),
            "true true" => Ok(vec!["machine", "ssh", "--", "podman", "unshare"]),
            _ => {
                tracing::debug!(info, "refused: podman runs as root");
                Err(RunError::Rootful)
            }
        }
    }

    fn mountpoint(&self, volume: &str) -> Result<String, RunError> {
        Ok(self
            .podman
            .run(&["volume", "inspect", "--format", "{{.Mountpoint}}", volume])?)
    }

    /// The owner and mode of a volume's own folder, `uid:gid:mode`, which
    /// `podman volume export` leaves out of its tar: an app that runs as a
    /// user of its own writes there only if they come back with the state.
    fn root_of(&self, shell: &[&str], volume: &str) -> Result<String, RunError> {
        let mount = self.mountpoint(volume)?;
        let args: Vec<&str> = shell
            .iter()
            .copied()
            .chain(["stat", "-c", "%u:%g:%a", &mount])
            .collect();
        Ok(self.podman.run(&args)?)
    }

    /// Copy one volume to `<volume>-snapshot`: a reflink if the filesystem
    /// takes one, else a plain copy into a fresh volume.
    fn copy(&self, shell: &[&str], volume: &str) -> Result<Copy, RunError> {
        let (from, to) = (self.volume(volume), format!("{}-snapshot", self.volume(volume)));
        let from = self
            .podman
            .run(&["volume", "inspect", "--format", "{{.Mountpoint}}", &from])?;
        for (copy, flag) in [(Copy::Reflink, "--reflink=always"), (Copy::Plain, "--reflink=never")] {
            if self.podman.answers(&["volume", "exists", &to])? {
                self.podman.run(&["volume", "rm", &to])?;
            }
            self.podman.run(&["volume", "create", &to])?;
            let into = self
                .podman
                .run(&["volume", "inspect", "--format", "{{.Mountpoint}}", &to])?;
            let (src, dst) = (format!("{from}/."), format!("{into}/"));
            let args: Vec<&str> = shell.iter().copied().chain(["cp", "-a", flag, &src, &dst]).collect();
            match (copy, self.podman.run(&args)) {
                (_, Ok(_)) => return Ok(copy),
                (Copy::Reflink, Err(why)) => tracing::debug!(volume, %why, "no reflink here, so a plain copy"),
                (Copy::Plain, Err(why)) => return Err(why.into()),
            }
        }
        unreachable!("the plain copy returns")
    }

    /// Stop the app and remove its pod and containers. Its volumes stay.
    pub fn stop(&self) -> Result<(), RunError> {
        let name = self.name.as_str();
        if !self.podman.answers(&["pod", "exists", name])? {
            tracing::debug!(app = name, "not running, so nothing to stop");
            return Ok(());
        }
        let grace = self
            .card
            .services
            .iter()
            .map(|s| s.stop_grace)
            .max()
            .unwrap_or_default();
        self.podman
            .run(&["pod", "stop", "--time", &grace.as_secs().to_string(), name])?;
        self.podman.run(&["pod", "rm", name])?;
        Ok(())
    }

    /// Replace each state volume with the snapshot's, its own folder's
    /// owner and mode included. The app is stopped.
    pub fn restore(&self, snapshot: &Path) -> Result<(), RunError> {
        let shell = self.storage_shell()?;
        for v in &self.card.state {
            let (tar, root) = (snapshot.join(format!("{v}.tar")), snapshot.join(format!("{v}.root")));
            let owner = std::fs::read_to_string(&root).map_err(|e| file(&root, e))?;
            let parts: Vec<&str> = owner.trim().split(':').collect();
            let [uid, gid, mode] = parts.as_slice() else {
                return Err(RunError::File {
                    path: root,
                    why: format!("{owner} is not uid:gid:mode"),
                });
            };
            if !parts
                .iter()
                .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
            {
                return Err(RunError::File {
                    path: root,
                    why: format!("{owner} is not uid:gid:mode"),
                });
            }
            if !tar.is_file() {
                return Err(RunError::File {
                    path: tar,
                    why: format!("the snapshot has no copy of {v}"),
                });
            }
            let volume = self.volume(v);
            if self.podman.answers(&["volume", "exists", &volume])? {
                self.podman.run(&["volume", "rm", &volume])?;
            }
            self.podman.run(&["volume", "create", &volume])?;
            self.podman
                .run(&["volume", "import", &volume, &tar.to_string_lossy()])?;
            let mount = self.mountpoint(&volume)?;
            let owner = format!("{uid}:{gid}");
            for change in [["chown", owner.as_str(), &mount], ["chmod", mode, &mount]] {
                self.podman
                    .run(&shell.iter().copied().chain(change).collect::<Vec<_>>())?;
            }
            tracing::debug!(volume, snapshot = %snapshot.display(), "restored");
        }
        Ok(())
    }

    /// Where the state volumes differ from a snapshot, file by file: what
    /// each holds, its owner, mode and bytes. Empty when they match.
    pub fn differs(&self, snapshot: &Path, scratch: &Path) -> Result<Vec<String>, RunError> {
        let (shell, mut out) = (self.storage_shell()?, Vec::new());
        for v in &self.card.state {
            let at = snapshot.join(format!("{v}.root"));
            let was = std::fs::read_to_string(&at).map_err(|e| file(&at, e))?;
            let is = self.root_of(&shell, &self.volume(v))?;
            if was.trim() != is {
                out.push(format!("{v}'s own folder was {} and is {is}", was.trim()));
            }
            let now = scratch.join(format!("{v}.now.tar"));
            self.podman
                .run(&["volume", "export", "--output", &now.to_string_lossy(), &self.volume(v)])?;
            let (was, is) = (contents(&snapshot.join(format!("{v}.tar")))?, contents(&now)?);
            let _ = std::fs::remove_file(&now);
            for path in was.keys().chain(is.keys()).collect::<std::collections::BTreeSet<_>>() {
                match (was.get(path), is.get(path)) {
                    (Some(a), Some(b)) if a == b => {}
                    (Some(_), Some(_)) => out.push(format!("{v}/{path} changed")),
                    (Some(_), None) => out.push(format!("{v}/{path} is missing")),
                    _ => out.push(format!("{v}/{path} was not in the snapshot")),
                }
            }
        }
        Ok(out)
    }

    /// Remove the app: its pod, containers and every volume, state included.
    pub fn remove(&self) -> Result<(), RunError> {
        let name = self.name.as_str();
        if self.podman.answers(&["pod", "exists", name])? {
            self.podman.run(&["pod", "rm", "--force", "--time", "0", name])?;
        }
        for v in &self.card.volumes {
            for v in [self.volume(v), format!("{}-snapshot", self.volume(v))] {
                if self.podman.answers(&["volume", "exists", &v])? {
                    self.podman.run(&["volume", "rm", "--force", &v])?;
                }
            }
        }
        Ok(())
    }
}

/// A volume's tar, read as path → kind, owner, mode and a hash of the bytes.
/// Times are left out: a restored directory is touched as it is made.
fn contents(tar: &Path) -> Result<BTreeMap<String, String>, RunError> {
    let f = std::fs::File::open(tar).map_err(|e| file(tar, e))?;
    let mut archive = tar::Archive::new(f);
    let mut out = BTreeMap::new();
    for entry in archive.entries().map_err(|e| file(tar, e))? {
        let mut entry = entry.map_err(|e| file(tar, e))?;
        let h = entry.header().clone();
        let path = entry
            .path()
            .map_err(|e| file(tar, e))?
            .to_string_lossy()
            .trim_start_matches("./")
            .to_string();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(|e| file(tar, e))?;
        let link = h
            .link_name()
            .ok()
            .flatten()
            .map(|l| l.to_string_lossy().into_owned())
            .unwrap_or_default();
        let seen = format!(
            "{:?} {} {} {:o} {link} {}",
            h.entry_type(),
            h.uid().unwrap_or(0),
            h.gid().unwrap_or(0),
            h.mode().unwrap_or(0),
            blake3::hash(&bytes)
        );
        if !path.is_empty() {
            out.insert(path, seen);
        }
    }
    Ok(out)
}

fn file(path: &Path, e: std::io::Error) -> RunError {
    RunError::File {
        path: path.to_path_buf(),
        why: e.to_string(),
    }
}
