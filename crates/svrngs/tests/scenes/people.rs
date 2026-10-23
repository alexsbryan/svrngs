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
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};

use crate::harness::{relay, relay_url, repo, Engine};

/// The traces each person's svrngs keeps in `svrngs.log` in its `HOME`:
/// the door's, the record's and the folder's decisions, at debug.
const TRACED: &str =
    "info,door=debug,origin=debug,folder=debug,home=debug,removal=debug,transport=warn";

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

    /// `svrngs <args>` in `name`'s home, returned once its API answers.
    async fn start(&self, name: &str, args: &[&str]) -> Person {
        let home = self.dir.join(name.to_lowercase());
        std::fs::create_dir_all(&home).unwrap();
        let log = std::fs::File::create(home.join("svrngs.log")).unwrap();
        let child = Command::new(env!("CARGO_BIN_EXE_svrngs"))
            .args(args)
            .env_clear()
            .env("HOME", &home)
            .env("USER", name.to_lowercase())
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("RUST_LOG", TRACED)
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(log)
            .kill_on_drop(true)
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
        .await
        .unwrap_or_else(|_| {
            panic!(
                "{name}'s svrngs never answered; its log is {}",
                home.join("svrngs.log").display()
            )
        });
        let page = format!("svrngs.localhost:{}", api.port());
        let http = reqwest::Client::builder()
            .no_proxy()
            .resolve("svrngs.localhost", api)
            .build()
            .unwrap();
        Person {
            name: name.into(),
            home,
            _running: child,
            api,
            page,
            http,
        }
    }
}

/// One person's computer, its svrngs running until this is dropped or
/// [`Person::shut`].
pub struct Person {
    pub name: String,
    pub home: PathBuf,
    /// Its svrngs, killed when this is dropped.
    _running: Child,
    api: SocketAddr,
    page: String,
    http: reqwest::Client,
}

impl Person {
    /// The home page's address, as a browser opens it.
    pub fn home_page(&self) -> String {
        format!("http://{}/", self.page)
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
    async fn group_named(&self, name: &str) -> String {
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
        panic!(
            "{} never stands in {name}; its log is {}",
            self.name,
            self.log().display()
        )
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
        panic!(
            "{} never lists {kind}; its log is {}",
            self.name,
            self.log().display()
        )
    }
}

/// The JSON line the browser half `script` printed, run in `engine`.
pub async fn sees(engine: Engine, script: &str, args: Value) -> Value {
    crate::drive::drive(engine, script, args, None).await
}
