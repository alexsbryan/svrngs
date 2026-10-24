//! The people of a scene: each a `svrngs` of this build, run as the
//! demos' drivers run one (demo/runs/c/dry-2026-10-12-rerun/run.py,
//! `env`), with a `HOME` and so a data dir of its own and nothing else of
//! this computer's environment but its `PATH`; the scene's group made by
//! `svrngs new --relay` at the relay on 127.0.0.1 and joined by `svrngs
//! join` at the link the founder's home page makes, so every act goes
//! through the real door, relay and record. What each person sees is read
//! in a browser, through `drive` (tests/browser/drive.rs).

use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use iroh_relay::server::Server;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};

use crate::harness::{relay, relay_url, repo, site, Engine};

/// The traces each person's svrngs keeps in `svrngs.log` in its `HOME`:
/// the door's, the record's and the folder's decisions, at debug.
const TRACED: &str =
    "info,door=debug,origin=debug,folder=debug,home=debug,removal=debug,svrngs_node::ring_sync=debug,transport=warn";

/// A wait in a scene that ran out on a host past the load bar: could not
/// be judged, so the scene ends there, never a pass ([`judged`]).
pub struct Unjudged(String);

/// A wait that ran out, `why`, judged as films' seeks are, against the one
/// load bar (load.rs `past_busy`), read now: on a host past it the scene
/// is could-not-judge, never a pass; under it the scene fails. Every other
/// assertion fails plainly under any load, and no limit is lengthened.
pub async fn late(why: impl std::fmt::Display) -> ! {
    let load = crate::load::load().await;
    match crate::load::past_busy(&load, crate::load::cores()) {
        Some(busy) => std::panic::panic_any(Unjudged(format!("{why}; {busy}"))),
        None => panic!("{why}; {load}"),
    }
}

/// `scene` played, its place `place` under `target/scenes/`: a wait that
/// ran out past the load bar ([`late`]) writes could-not-judge.txt there,
/// which the gate's "scenes" step reads as never a pass (scripts/check.sh);
/// any other failure fails the test as it came.
pub async fn judged(place: String, scene: impl std::future::Future<Output = ()> + Send + 'static) {
    let Err(failed) = tokio::spawn(scene).await else {
        return;
    };
    match failed.into_panic().downcast::<Unjudged>() {
        Ok(unjudged) => {
            eprintln!("could-not-judge: {}", unjudged.0);
            let to = repo().join("target/scenes").join(place);
            std::fs::write(to.join("could-not-judge.txt"), unjudged.0).unwrap();
        }
        Err(other) => std::panic::resume_unwind(other),
    }
}

/// A scene's place: its relay on 127.0.0.1 and its people's homes, under
/// `target/scenes/` (scratch stays under `target/`), kept after a failure
/// so each person's `svrngs.log` can be read.
pub struct Scene {
    pub relay: Server,
    dir: PathBuf,
}

impl Scene {
    /// The scene `name` in `engine`, its place emptied first.
    pub async fn new(name: &str, engine: Engine) -> Scene {
        let dir = repo().join(format!("target/scenes/{name}-{}", engine.as_str()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Scene {
            relay: relay().await,
            dir,
        }
    }

    /// The scene not played, and why, in its place, where the gate's
    /// "scenes" step reads it as never-ran (scripts/check.sh).
    pub fn never_ran(&self, why: &str) {
        eprintln!("never ran: {why}");
        std::fs::write(self.dir.join("never-ran.txt"), why).unwrap();
    }

    /// A bar the scene could not judge, and why, in its place, where the
    /// gate's "scenes" step reads it as could-not-judge, never a pass, when
    /// nothing else in the scene failed (scripts/check.sh).
    pub fn could_not_judge(&self, why: &str) {
        eprintln!("could-not-judge: {why}");
        std::fs::write(self.dir.join("could-not-judge.txt"), why).unwrap();
    }

    /// Where its scratch goes, under its place.
    pub fn scratch(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    /// `name`'s computer makes `group`, as `svrngs new <group> --as <name>
    /// --relay <the scene's>` does, and runs on as that command does.
    pub async fn founds(&self, name: &str, group: &str) -> (Person, String) {
        let at = relay_url(&self.relay);
        let args = ["new", group, "--as", name, "--relay", &at];
        let person = self.start(name, &args).await;
        let id = person.group_named(group).await;
        (person, id)
    }

    /// `name`'s computer joins by `link`, as `svrngs join <link> --as
    /// <name>` does, and runs on; returned once it stands in `group`.
    pub async fn joins(&self, name: &str, link: &str, group: &str) -> Person {
        let person = self.start(name, &["join", link, "--as", name]).await;
        person.group_named(group).await;
        person
    }

    /// `who`'s svrngs stopped, then `svrngs <args>` there: a second group
    /// made, as `svrngs new` is refused while svrngs runs.
    pub async fn again(&self, who: Person, args: &[&str]) -> Person {
        let name = who.name.clone();
        who.stop().await;
        self.start(&name, args).await
    }

    /// `svrngs <args>` in `name`'s home with no svrngs running there, as a
    /// terminal runs one that exits (`join --no-run`): what it printed.
    pub async fn once(&self, name: &str, args: &[&str]) -> String {
        let home = self.dir.join(name.to_lowercase());
        std::fs::create_dir_all(&home).unwrap();
        let run = svrngs(&home, name, args).stdin(Stdio::null()).output();
        let out = tokio::time::timeout(Duration::from_secs(90), run);
        let out = out.await.expect("the command ends within 90 s").unwrap();
        let said = String::from_utf8_lossy(&out.stdout).to_string();
        assert!(
            out.status.success(),
            "{name} `svrngs {args:?}` failed: {said}{}",
            String::from_utf8_lossy(&out.stderr)
        );
        said
    }

    /// The page, built from this commit (harness.rs `site`) and served on
    /// `localhost` by web/tests/serve.mjs, as the Events run served it
    /// (demo/runs/events/2026-10-14/run.py): where a phone joins and opens
    /// what its group lends. Its address, and the server, killed on drop.
    pub async fn page(&self) -> (String, Child) {
        let mut served = Command::new("node")
            .arg(repo().join("web/tests/serve.mjs"))
            .arg(site())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .expect("node runs");
        let mut lines = tokio::io::BufReader::new(served.stdout.take().unwrap()).lines();
        let port = lines
            .next_line()
            .await
            .unwrap()
            .expect("serve.mjs prints its port");
        let port = port.strip_prefix("port ").expect("`port <n>`").to_string();
        tokio::spawn(async move { while let Ok(Some(_)) = lines.next_line().await {} });
        (format!("http://localhost:{port}/"), served)
    }

    /// `svrngs <args>` in `who`'s home beside the svrngs running there,
    /// `typed` on its stdin, as a terminal there runs it: what it printed,
    /// once it exits.
    pub async fn command(&self, who: &Person, args: &[&str], typed: &str) -> String {
        let mut run = svrngs(&who.home, &who.name, args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = run.stdin.take().unwrap();
        stdin.write_all(typed.as_bytes()).await.unwrap();
        drop(stdin);
        let out = tokio::time::timeout(Duration::from_secs(60), run.wait_with_output());
        let out = out.await.expect("the command ends within 60 s").unwrap();
        let said = String::from_utf8_lossy(&out.stdout).to_string();
        assert!(
            out.status.success(),
            "{} `svrngs {args:?}` failed: {said}{}",
            who.name,
            String::from_utf8_lossy(&out.stderr)
        );
        said
    }

    /// `svrngs <args>` in `name`'s home, returned once its API answers.
    pub async fn start(&self, name: &str, args: &[&str]) -> Person {
        let home = self.dir.join(name.to_lowercase());
        std::fs::create_dir_all(&home).unwrap();
        // Appended to, so a person started again keeps what it traced.
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(home.join("svrngs.log"))
            .unwrap();
        let child = svrngs(&home, name, args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(log)
            .spawn()
            .expect("this build's svrngs runs");
        let mut child = child;
        // What it prints, drained so it never blocks on a full pipe.
        let out = child.stdout.take().unwrap();
        tokio::spawn(async move {
            let mut lines = tokio::io::BufReader::new(out).lines();
            while let Ok(Some(_)) = lines.next_line().await {}
        });
        let api = tokio::time::timeout(Duration::from_secs(60), async {
            loop {
                let written = std::fs::read_to_string(home.join(".svrngs/api")).unwrap_or_default();
                let written = written
                    .trim()
                    .trim_start_matches("http://")
                    .trim_end_matches('/');
                if let Ok(api) = written.parse::<SocketAddr>() {
                    if tokio::net::TcpStream::connect(api).await.is_ok() {
                        break api;
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await;
        let Ok(api) = api else {
            let log = home.join("svrngs.log");
            late(format!(
                "{name}'s svrngs did not answer within 60 s; its log is {}",
                log.display()
            ))
            .await
        };
        let page = format!("svrngs.localhost:{}", api.port());
        let http = reqwest::Client::builder()
            .no_proxy()
            .resolve("svrngs.localhost", api)
            .build()
            .unwrap();
        Person {
            name: name.into(),
            home,
            running: child,
            api,
            page,
            http,
        }
    }
}

/// `svrngs <args>`, this build's, in `home`, with nothing else of this
/// computer's environment but its `PATH`.
fn svrngs(home: &std::path::Path, name: &str, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_svrngs"));
    c.args(args)
        .env_clear()
        .env("HOME", home)
        .env("USER", name.to_lowercase())
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("RUST_LOG", TRACED)
        .env("NO_COLOR", "1")
        .kill_on_drop(true);
    c
}

/// One person's computer, its svrngs running until this is dropped or
/// [`Person::shut`].
pub struct Person {
    pub name: String,
    pub home: PathBuf,
    /// Its svrngs, killed when this is dropped.
    running: Child,
    api: SocketAddr,
    page: String,
    http: reqwest::Client,
}

impl Person {
    /// The home page's address, as a browser opens it.
    pub fn home_page(&self) -> String {
        format!("http://{}/", self.page)
    }

    /// Its svrngs stopped, and gone once this returns.
    pub async fn stop(mut self) {
        let _ = self.running.kill().await;
    }

    /// Where its traces are.
    pub fn log(&self) -> PathBuf {
        self.home.join("svrngs.log")
    }

    /// `GET <path>` of its record API, as a program on this computer asks.
    pub async fn get(&self, path: &str) -> Value {
        let got = self.http.get(format!("http://{}{path}", self.api)).send();
        let got = got.await.unwrap().bytes().await.unwrap();
        serde_json::from_slice(&got).unwrap_or(Value::Null)
    }

    /// `POST <path>` of `body` to its record API, as a program on this
    /// computer writes: the status and the answer.
    pub async fn post(&self, path: &str, body: &Value) -> (u16, Value) {
        let sent = self.http.post(format!("http://{}{path}", self.api));
        let sent = sent
            .header("content-type", "application/json")
            .body(body.to_string());
        let got = sent.send().await.unwrap();
        let status = got.status().as_u16();
        let got = got.bytes().await.unwrap();
        (status, serde_json::from_slice(&got).unwrap_or(Value::Null))
    }

    /// `POST <path>` of `body` on its home page, as the page's own script
    /// sends it, from its exact origin (home.rs `only_itself`).
    pub async fn on_home_page(&self, path: &str, body: &Value) -> Value {
        let sent = self.http.post(format!("http://{}{path}", self.page));
        let sent = sent
            .header("origin", format!("http://{}", self.page))
            .header("sec-fetch-site", "same-origin")
            .header("content-type", "application/json")
            .body(body.to_string());
        let got = sent.send().await.unwrap();
        let status = got.status();
        let got = got.bytes().await.unwrap();
        assert!(
            status.is_success(),
            "{}'s home page answered {path} {status}: {}",
            self.name,
            String::from_utf8_lossy(&got)
        );
        serde_json::from_slice(&got).unwrap_or(Value::Null)
    }

    /// The link the home page makes to invite someone to `group`.
    pub async fn invites(&self, group: &str) -> String {
        let asked = json!({ "group": group, "device": false });
        self.on_home_page("/link", &asked).await["client"]
            .as_str()
            .expect("a link")
            .to_string()
    }

    /// The id of the group called `name` it stands in, within 60 s.
    pub async fn group_named(&self, name: &str) -> String {
        for _ in 0..300 {
            let groups = self.get("/v1/groups").await;
            let held = groups["groups"].as_array().into_iter().flatten();
            let held = held
                .into_iter()
                .find(|g| g["name"] == name && g["person"].is_string());
            if let Some(g) = held {
                return g["id"].as_str().unwrap().to_string();
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        let log = self.log();
        late(format!(
            "{} does not stand in {name} within 60 s; its log is {}",
            self.name,
            log.display()
        ))
        .await
    }

    /// Where its browser opens the lend `name` of `person` in `group`,
    /// once it lists it, within 60 s.
    pub async fn lent_by(&self, group: &str, person: &str, name: &str) -> String {
        for _ in 0..120 {
            let lends = self.get(&format!("/v1/lends?namespace={group}")).await;
            let listed = lends["lent"].as_array().into_iter().flatten();
            if let Some(l) = listed
                .into_iter()
                .find(|l| l["person"] == person && l["name"] == name)
            {
                return l["origin"].as_str().unwrap().to_string();
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        let log = self.log();
        late(format!(
            "{} does not list {person}'s {name} within 60 s; its log is {}",
            self.name,
            log.display()
        ))
        .await
    }

    /// Where its browser opens what the group carries or lends of `kind`
    /// in `group`, once it lists it, within 60 s.
    pub async fn opens(&self, group: &str, kind: &str) -> String {
        for _ in 0..120 {
            let lends = self.get(&format!("/v1/lends?namespace={group}")).await;
            let listed = lends["lent"].as_array().into_iter().flatten();
            if let Some(l) = listed.into_iter().find(|l| l["kind"] == kind) {
                return l["origin"].as_str().unwrap().to_string();
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        let log = self.log();
        late(format!(
            "{} does not list {kind} within 60 s; its log is {}",
            self.name,
            log.display()
        ))
        .await
    }
}

/// The JSON line the browser half `script` printed, run in `engine`.
pub async fn sees(engine: Engine, script: &str, args: Value) -> Value {
    crate::drive::drive(engine, script, args, None).await
}

/// Each person's browser, kept for the whole scene so what a phone keeps
/// stays: the recorded Events run's driver,
/// demo/runs/events/2026-10-14/people.mjs, one JSON command a line.
pub struct Browsers {
    run: Child,
    said: tokio::io::Lines<tokio::io::BufReader<tokio::process::ChildStdout>>,
}

impl Browsers {
    /// people.mjs started, and each of `people` given a browser: a
    /// computer's Chromium, a phone's WebKit as an iPhone, or every one in
    /// `engine` when the scene is played in WebKit.
    pub async fn open(engine: Engine, people: &[(&str, Engine)]) -> Browsers {
        Browsers::driven("demo/runs/events/2026-10-14/people.mjs", engine, people).await
    }

    /// As [`Browsers::open`], with the recorded run's driver at `driver`
    /// in place of Events': House rules' (demo/runs/canon/2026-10-17-house/
    /// people.mjs) presses `steps` where Events' presses one button.
    pub async fn driven(driver: &str, engine: Engine, people: &[(&str, Engine)]) -> Browsers {
        // The doc run's people.mjs imports Playwright by `PLAYWRIGHT`, the
        // others by their path to web/tests/page.mjs.
        let playwright = repo().join("web/node_modules/playwright/index.mjs");
        let mut run = Command::new("node")
            .arg(repo().join(driver))
            .env("PLAYWRIGHT", playwright)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .expect("node runs");
        let mut said = tokio::io::BufReader::new(run.stdout.take().unwrap()).lines();
        let ready = said.next_line().await.unwrap().unwrap_or_default();
        assert!(ready.contains("ready"), "people.mjs did not start: {ready}");
        let mut browsers = Browsers { run, said };
        for (who, theirs) in people {
            let engine = match engine {
                Engine::Webkit => Engine::Webkit,
                Engine::Chromium => *theirs,
            };
            browsers
                .tell(json!({ "do": "open", "who": who, "engine": engine.as_str() }))
                .await;
        }
        browsers
    }

    /// One command, and its answer; an error the driver caught fails the
    /// scene, its words and the page's console with it.
    pub async fn tell(&mut self, command: Value) -> Value {
        let stdin = self.run.stdin.as_mut().unwrap();
        stdin
            .write_all(format!("{command}\n").as_bytes())
            .await
            .unwrap();
        stdin.flush().await.unwrap();
        let line = self
            .said
            .next_line()
            .await
            .unwrap()
            .expect("people.mjs answers");
        let got: Value = serde_json::from_str(&line).unwrap();
        assert!(got.get("error").is_none(), "{command}: {got:#}");
        got
    }
}
