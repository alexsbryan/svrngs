//! What give reaches outside its process, answered here as each answers it,
//! from state the test holds and sets: podman's command line ([`Podman`]),
//! svrngs's command ([`Lends`]) and svrngs's record API ([`Record`], one
//! [`Computer`] of it per keeper). Nothing is run, bound or waited on, so a
//! test of give's decisions repeats on any computer, podman or none.
//!
//! [`Podman`] answers the commands give runs as podman 6.0.1 answers them,
//! and keeps the stand-in card (tests/standin) as it runs: each start adds
//! a line to its state's `starts`, and one whose `starts` is a folder exits.
//! That the real podman answers so is tests/check.rs's one podman test
//! (`the_stand_in_keeps_its_state_across_a_restore`), the contained test of
//! this seam; a command give runs that this does not know fails the test
//! that ran it, never answers yes.

#![allow(dead_code, reason = "each test target uses its own part of these")]
#![allow(
    clippy::disallowed_types,
    reason = "a test reads and sets what the code under test reaches through `&self`"
)]

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use give::lend::LendError;
use serde_json::{Value, json};

/// A volume: its own folder's `uid:gid:mode`, and each path in it, a
/// file's bytes or, none, a folder.
#[derive(Debug, Clone)]
struct Volume {
    root: String,
    files: BTreeMap<String, Option<Vec<u8>>>,
}

impl Default for Volume {
    fn default() -> Self {
        Volume {
            root: "1000:1000:755".into(),
            files: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Run {
    Running,
    Paused,
    Exited(i32),
}

#[derive(Debug)]
struct Container {
    pod: String,
    run: Run,
    health: Option<String>,
}

#[derive(Debug)]
struct Pod {
    publish: String,
    /// Paused as a pod, its own container with its app's.
    paused: bool,
}

#[derive(Debug, Default)]
struct State {
    said: String,
    volumes: BTreeMap<String, Volume>,
    pods: BTreeMap<String, Pod>,
    containers: BTreeMap<String, Container>,
    /// Commands that fail, by how they begin, and what each says.
    failing: Vec<(String, String)>,
    /// Every healthcheck fails.
    unhealthy: bool,
    starts: u32,
    /// What `machine list` and `machine inspect` answer.
    list: String,
    inspect: String,
}

/// A podman that answers as podman does, from what it holds.
#[derive(Debug)]
pub struct Podman(Mutex<State>);

impl Default for Podman {
    fn default() -> Self {
        // A running machine, Rosetta on: what a Mac that keeps has.
        Podman::machines(
            r#"[{"Name": "m", "Default": true, "Running": true, "Memory": "25769803776"}]"#,
            r#"[{"Name": "m", "Rosetta": true}]"#,
        )
    }
}

fn mount(volume: &str) -> String {
    format!("/fake/{volume}/_data")
}

fn mounted(path: &str) -> Option<&str> {
    path.strip_prefix("/fake/")?.split("/_data").next()
}

impl Podman {
    /// One whose `machine list` and `machine inspect` answer `list` and
    /// `inspect`.
    pub fn machines(list: &str, inspect: &str) -> Podman {
        Podman(Mutex::new(State {
            list: list.into(),
            inspect: inspect.into(),
            ..State::default()
        }))
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.0.lock().unwrap()
    }

    /// Every command it was given, a line each, and after one given
    /// `CONTAINERS_CONF_OVERRIDE`, what that file held.
    pub fn said(&self) -> String {
        self.state().said.clone()
    }

    /// Each command that begins `with` fails from now, saying `why`.
    pub fn fail(&self, with: &str, why: &str) {
        self.state().failing.push((with.into(), why.into()));
    }

    /// No command fails from now.
    pub fn heal(&self) {
        self.state().failing.clear();
    }

    /// Every healthcheck fails from now.
    pub fn unhealthy(&self) {
        self.state().unhealthy = true;
    }

    /// `text` written to `path` in `volume`, as `podman exec` would.
    pub fn write(&self, volume: &str, path: &str, text: &str) {
        let mut s = self.state();
        let v = s.volumes.get_mut(volume).expect("the volume written to");
        v.files.insert(path.into(), Some(text.as_bytes().to_vec()));
    }

    /// What `path` in `volume` holds.
    pub fn file(&self, volume: &str, path: &str) -> Option<String> {
        let s = self.state();
        let bytes = s.volumes.get(volume)?.files.get(path)?.clone()?;
        Some(String::from_utf8(bytes).unwrap())
    }

    /// The container `name` ends, exit `code`, as `podman kill` ends it.
    pub fn exit(&self, name: &str, code: i32) {
        self.state().containers.get_mut(name).expect("the container killed").run = Run::Exited(code);
    }

    pub fn pods(&self) -> Vec<String> {
        self.state().pods.keys().cloned().collect()
    }

    pub fn volumes(&self) -> Vec<String> {
        self.state().volumes.keys().cloned().collect()
    }

    /// Where the pod `name` publishes its door, as `pod create` was told.
    pub fn published(&self, name: &str) -> Option<String> {
        self.state().pods.get(name).map(|p| p.publish.clone())
    }
}

impl give::podman::Runs for Podman {
    fn run(&self, args: &[&str], env: &[(OsString, OsString)]) -> Result<(i32, String, String), String> {
        let mut s = self.state();
        let line = args.join(" ");
        s.said.push_str(&format!("{line}\n"));
        if let Some((_, conf)) = env.iter().find(|(k, _)| k == "CONTAINERS_CONF_OVERRIDE") {
            let read = std::fs::read_to_string(conf).unwrap_or_default();
            s.said.push_str(&read);
        }
        if let Some((_, why)) = s.failing.iter().find(|(with, _)| line.starts_with(with.as_str())) {
            return Ok((125, String::new(), format!("Error: {why}")));
        }
        Ok(s.answer(args))
    }
}

type Answer = (i32, String, String);

fn ok(out: &str) -> Answer {
    (0, out.into(), String::new())
}

fn yes(is: bool) -> Answer {
    (if is { 0 } else { 1 }, String::new(), String::new())
}

fn refused(why: &str) -> Answer {
    (125, String::new(), format!("Error: {why}"))
}

impl State {
    fn answer(&mut self, args: &[&str]) -> Answer {
        match args {
            ["info", "--format", f] if f.contains("ServiceIsRemote") => ok("false true"),
            ["info", "--format", _] => ok("true"),
            ["machine", "list", ..] => ok(&self.list.clone()),
            ["machine", "inspect", _] => ok(&self.inspect.clone()),
            ["machine", "init" | "set" | "start", ..] => ok(""),
            ["image", "exists", _] => yes(true),
            ["image", "inspect", "--format", _, _] => ok("linux/arm64"),
            ["pull", ..] => ok(""),
            ["volume", "exists", v] => yes(self.volumes.contains_key(*v)),
            ["volume", "create", v] if self.volumes.contains_key(*v) => refused("volume already exists"),
            ["volume", "create", v] => {
                self.volumes.insert(v.to_string(), Volume::default());
                ok(v)
            }
            ["volume", "rm", v] | ["volume", "rm", "--force", v] => match self.volumes.remove(*v) {
                Some(_) => ok(v),
                None => refused("no such volume"),
            },
            ["volume", "inspect", "--format", "{{.Mountpoint}}", v] if self.volumes.contains_key(*v) => ok(&mount(v)),
            ["volume", "export", "--output", path, v] => self.export(v, Path::new(path)),
            ["volume", "import", v, path] => self.import(v, Path::new(path)),
            ["volume", "ls", "--quiet", "--filter", f] => {
                let named = f.strip_prefix("name=").unwrap_or(f);
                let found: Vec<_> = self.volumes.keys().filter(|v| v.contains(named)).cloned().collect();
                ok(&found.join("\n"))
            }
            ["pod", "exists", n] => yes(self.pods.contains_key(*n)),
            ["pod", "create", "--name", n, rest @ ..] => {
                let at = rest.iter().position(|a| *a == "--publish").map(|i| rest[i + 1]);
                let publish = at.unwrap_or_default().to_string();
                self.pods.insert(n.to_string(), Pod { publish, paused: false });
                ok(n)
            }
            ["pod", "pause", n] => self.pause(n, true),
            ["pod", "unpause", n] => self.pause(n, false),
            ["pod", "inspect", "--format", "{{.State}}", n] => {
                let Some(pod) = self.pods.get(*n) else {
                    return refused("no such pod");
                };
                let half = self.of(n).any(|(_, c)| c.run == Run::Paused);
                ok(match (pod.paused, half) {
                    (true, _) => "Paused",
                    (false, true) => "Degraded",
                    (false, false) => "Running",
                })
            }
            // A paused container is refused a stop, and its pod a remove
            // (tests/check.rs's podman test holds podman to it).
            ["pod", "stop", "--time", _, n] => {
                if self.of(n).any(|(_, c)| c.run == Run::Paused) {
                    return refused("container state improper");
                }
                for c in self.containers.values_mut().filter(|c| c.pod == *n) {
                    c.run = Run::Exited(0);
                }
                ok(n)
            }
            ["pod", "rm", n] | ["pod", "rm", "--force", n] => {
                if self.of(n).any(|(_, c)| c.run == Run::Paused) {
                    return refused("cannot remove container as it is paused");
                }
                if args.len() == 3 && self.of(n).any(|(_, c)| c.run == Run::Running) {
                    return refused("the pod has running containers");
                }
                self.containers.retain(|_, c| c.pod != *n);
                match self.pods.remove(*n) {
                    Some(_) => ok(n),
                    None => refused("no such pod"),
                }
            }
            [
                "ps",
                "--all",
                "--filter",
                pod,
                "--filter",
                "status=paused",
                "--format",
                "{{.Names}}",
            ] => {
                let pod = pod.strip_prefix("pod=").unwrap_or(pod);
                let paused: Vec<_> = (self.of(pod))
                    .filter(|(_, c)| c.run == Run::Paused)
                    .map(|(n, _)| n.clone())
                    .collect();
                ok(&paused.join("\n"))
            }
            ["pause", c] => self.set(c, Run::Paused),
            ["unpause", c] => self.set(c, Run::Running),
            ["run", "--detach", "--pod", pod, "--name", name, rest @ ..] => self.start(pod, name, rest),
            ["container", "inspect", "--format", f, c] => {
                let Some(container) = self.containers.get(*c) else {
                    return refused("no such container");
                };
                if f.contains("Healthcheck") {
                    return ok(container.health.as_deref().unwrap_or("null"));
                }
                ok(&match container.run {
                    Run::Running => "running 0".to_string(),
                    Run::Paused => "paused 0".to_string(),
                    Run::Exited(code) => format!("exited {code}"),
                })
            }
            ["healthcheck", "run", c] => {
                let running = self.containers.get(*c).is_some_and(|c| c.run == Run::Running);
                yes(running && !self.unhealthy)
            }
            ["logs", "--tail", _, _] => ok("the stand-in's last lines"),
            ["unshare", rest @ ..] => self.storage(rest),
            _ => refused(&format!("the test's podman does not answer `{}`", args.join(" "))),
        }
    }

    fn of<'a>(&'a self, pod: &'a str) -> impl Iterator<Item = (&'a String, &'a Container)> + 'a {
        self.containers.iter().filter(move |(_, c)| c.pod == pod)
    }

    fn set(&mut self, name: &str, run: Run) -> Answer {
        let Some(c) = self.containers.get_mut(name) else {
            return refused("no such container");
        };
        c.run = run;
        let pod = c.pod.clone();
        if run == Run::Running
            && let Some(p) = self.pods.get_mut(&pod)
        {
            p.paused = false;
        }
        ok(name)
    }

    fn pause(&mut self, pod: &str, paused: bool) -> Answer {
        let Some(p) = self.pods.get_mut(pod) else {
            return refused("no such pod");
        };
        p.paused = paused;
        let (from, to) = if paused {
            (Run::Running, Run::Paused)
        } else {
            (Run::Paused, Run::Running)
        };
        for c in self.containers.values_mut().filter(|c| c.pod == pod && c.run == from) {
            c.run = to;
        }
        ok(pod)
    }

    /// `podman run` of the stand-in: its state at `/data` gets a line more,
    /// or, where `starts` is a folder, `date >> /data/starts` fails and it
    /// exits.
    fn start(&mut self, pod: &str, name: &str, rest: &[&str]) -> Answer {
        if !self.pods.contains_key(pod) {
            return refused("no such pod");
        }
        let flag = |f: &str| rest.iter().position(|a| *a == f).map(|i| rest[i + 1]);
        let data = (rest.iter().enumerate())
            .filter(|(_, a)| **a == "--mount")
            .filter_map(|(i, _)| rest.get(i + 1))
            .find(|m| m.contains("destination=/data"))
            .and_then(|m| m.split(',').find_map(|p| p.strip_prefix("source=")));
        let health = flag("--health-cmd").map(|t| json!({ "Test": t }).to_string());
        let mut run = Run::Running;
        if let Some(v) = data {
            self.starts += 1;
            let volume = self.volumes.entry(v.to_string()).or_default();
            match volume.files.get_mut("starts") {
                Some(None) => run = Run::Exited(1),
                Some(Some(bytes)) => bytes.extend(format!("start {}\n", self.starts).bytes()),
                None => drop(
                    volume
                        .files
                        .insert("starts".into(), Some(format!("start {}\n", self.starts).into())),
                ),
            }
        }
        let container = Container {
            pod: pod.into(),
            run,
            health,
        };
        self.containers.insert(name.into(), container);
        ok(name)
    }

    /// What give runs in podman's storage, by a volume's mountpoint.
    fn storage(&mut self, args: &[&str]) -> Answer {
        match args {
            ["cp", "-a", _, from, to] => {
                let (Some(from), Some(to)) = (mounted(from), mounted(to)) else {
                    return refused("cp: not a volume");
                };
                let Some(files) = self.volumes.get(from).map(|v| v.files.clone()) else {
                    return refused("cp: no such volume");
                };
                self.volumes.entry(to.into()).or_default().files = files;
                ok("")
            }
            ["stat", "-c", "%u:%g:%a", m] => match mounted(m).and_then(|v| self.volumes.get(v)) {
                Some(v) => ok(&v.root),
                None => refused("stat: no such file"),
            },
            ["chown", owner, m] | ["chmod", owner, m] => {
                let Some(v) = mounted(m).and_then(|v| self.volumes.get_mut(v)) else {
                    return refused("no such file");
                };
                let mut parts: Vec<String> = v.root.split(':').map(String::from).collect();
                if args[0] == "chown" {
                    let (uid, gid) = owner.split_once(':').unwrap();
                    (parts[0], parts[1]) = (uid.into(), gid.into());
                } else {
                    parts[2] = owner.to_string();
                }
                v.root = parts.join(":");
                ok("")
            }
            _ => refused(&format!(
                "the test's podman does not answer `unshare {}`",
                args.join(" ")
            )),
        }
    }

    fn export(&self, volume: &str, to: &Path) -> Answer {
        let Some(v) = self.volumes.get(volume) else {
            return refused("no such volume");
        };
        let mut tar = tar::Builder::new(std::fs::File::create(to).unwrap());
        for (path, file) in &v.files {
            let mut h = tar::Header::new_gnu();
            let bytes = file.as_deref().unwrap_or_default();
            h.set_entry_type(match file {
                Some(_) => tar::EntryType::Regular,
                None => tar::EntryType::Directory,
            });
            h.set_mode(if file.is_some() { 0o644 } else { 0o755 });
            h.set_uid(1000);
            h.set_gid(1000);
            h.set_size(bytes.len() as u64);
            tar.append_data(&mut h, path, bytes).unwrap();
        }
        tar.finish().unwrap();
        ok("")
    }

    fn import(&mut self, volume: &str, from: &Path) -> Answer {
        let Ok(f) = std::fs::File::open(from) else {
            return refused("no such file");
        };
        let mut files = BTreeMap::new();
        for e in tar::Archive::new(f).entries().unwrap() {
            let Ok(mut e) = e else {
                return refused("the tar is not whole");
            };
            let path = e.path().unwrap().to_string_lossy().trim_end_matches('/').to_string();
            let file = if e.header().entry_type().is_dir() {
                None
            } else {
                let mut bytes = Vec::new();
                e.read_to_end(&mut bytes).unwrap();
                Some(bytes)
            };
            files.insert(path, file);
        }
        let Some(v) = self.volumes.get_mut(volume) else {
            return refused("no such volume");
        };
        v.files = files;
        ok("")
    }
}

/// svrngs's command: each line it was asked, and `unshare` failing when
/// told to, exit 2, as svrngs does when a lend can't be withdrawn.
#[derive(Debug, Default)]
pub struct Lends {
    asked: Mutex<Vec<String>>,
    unshare_fails: Mutex<Option<String>>,
}

impl Lends {
    pub fn asked(&self) -> Vec<String> {
        self.asked.lock().unwrap().clone()
    }

    pub fn shares(&self) -> Vec<String> {
        self.asked().into_iter().filter(|l| l.starts_with("share ")).collect()
    }

    /// `unshare` fails saying `why` from now, or, none, succeeds.
    pub fn unshare_fails(&self, why: Option<&str>) {
        *self.unshare_fails.lock().unwrap() = why.map(String::from);
    }
}

impl give::lend::Runs for Lends {
    fn run(&self, args: &[&str]) -> Result<(Option<i32>, String), LendError> {
        self.asked.lock().unwrap().push(args.join(" "));
        match (args.first(), self.unshare_fails.lock().unwrap().as_ref()) {
            (Some(&"unshare"), Some(why)) => Ok((Some(2), why.clone())),
            _ => Ok((Some(0), String::new())),
        }
    }
}

/// The record of the one group, `g1`, as svrngs's API on each member's
/// computer gives it: one log, written by any of them.
#[derive(Debug, Default)]
pub struct Record(Mutex<(Vec<Value>, Vec<String>)>);

/// A computer by its key, its person and their key.
pub type Who = (&'static str, &'static str, &'static str);

impl Record {
    pub fn ops(&self) -> Vec<Value> {
        self.0.lock().unwrap().0.clone()
    }

    /// The acts of give's `kind` written by `actor`'s computer.
    pub fn of(&self, kind: &str, actor: &str) -> Vec<Value> {
        (self.ops().into_iter())
            .filter(|o| o["payload"]["kind"] == format!("give/{kind}") && o["actor"] == actor)
            .collect()
    }

    /// An act written as `who`'s computer would, with give's prefix: its id.
    pub fn act(&self, (actor, person, key): Who, mut payload: Value) -> String {
        payload["kind"] = format!("give/{}", payload["kind"].as_str().unwrap()).into();
        let mut log = self.0.lock().unwrap();
        let id = format!("op{}", log.0.len() + 1);
        let ts = log.0.len() as i64 + 1;
        let op = json!({ "id": id, "actor": actor, "person": person, "person_key": key, "ts_unix": ts, "voided": false, "payload": payload });
        log.0.push(op);
        id
    }

    /// A write of each of `kinds` answered 500 from now, as a record that
    /// could not write it.
    pub fn refuse(&self, kinds: &[&str]) {
        self.0.lock().unwrap().1 = kinds.iter().map(|k| k.to_string()).collect();
    }

    /// Give's fold of the record now.
    pub fn kept(&self) -> give_fold::Kept {
        give_fold::fold(&give::record::gives(&self.ops()))
    }
}

/// The record's API on one computer: `who`'s, whose folder of `g1` is
/// `folder`.
#[derive(Debug)]
pub struct Computer {
    pub record: Arc<Record>,
    pub who: Who,
    pub folder: PathBuf,
}

impl give::record::Answers for Computer {
    fn call(&self, route: &str, body: Option<&Value>) -> Result<(u16, String), String> {
        let (actor, person, key) = self.who;
        let mut log = self.record.0.lock().unwrap();
        let answer = match (route, body) {
            ("/v1/groups", None) => {
                json!({ "key": actor, "groups": [{ "id": "g1", "name": "Elm", "folders": [self.folder] }] })
            }
            ("/v1/rail/log?namespace=g1", None) => json!({ "ops": log.0 }),
            ("/v1/rail/append?namespace=g1", Some(body)) => {
                if log.1.iter().any(|k| body["payload"]["kind"] == k.as_str()) {
                    return Ok((500, json!({ "error": "the record could not be written" }).to_string()));
                }
                let id = format!("op{}", log.0.len() + 1);
                let ts = log.0.len() as i64 + 1;
                let op = json!({ "id": id, "actor": actor, "person": person, "person_key": key, "ts_unix": ts, "voided": false, "payload": body["payload"] });
                log.0.push(op);
                json!({ "id": id })
            }
            _ => return Ok((404, json!({ "error": route }).to_string())),
        };
        Ok((200, answer.to_string()))
    }
}

/// A snapshot's name: one more each time, in milliseconds as the wall
/// clock's are, so each is new and later than the last.
pub fn stamp() -> u128 {
    static NEXT: AtomicU64 = AtomicU64::new(1_800_000_000_000);
    NEXT.fetch_add(1, Ordering::SeqCst) as u128
}

/// The port every test's app is said to listen on: nothing listens.
pub fn port() -> std::io::Result<u16> {
    Ok(40000)
}

/// What give traced at debug while `f` ran, on this thread.
pub fn traced<T>(f: impl FnOnce() -> T) -> (T, String) {
    #[derive(Clone)]
    struct Sink(Arc<Mutex<Vec<u8>>>);
    impl std::io::Write for Sink {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let into = Sink(Arc::default());
    let to = into.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .with_writer(move || to.clone())
        .finish();
    let out = tracing::subscriber::with_default(subscriber, f);
    let text = String::from_utf8_lossy(&into.0.lock().unwrap()).into_owned();
    (out, text)
}
