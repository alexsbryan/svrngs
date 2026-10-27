//! The keeper program, `give keep` (give-app-keep (a); GIVE.md §7): it
//! keeps an app only while the group's record names this computer for it.
//! It lends the app's door (`svrngs share`) from a healthy start, writes a
//! snapshot into the group's folder at each interval and at a clean stop,
//! each naming its card version, and unlends and stops the app when the
//! record names another computer.
//!
//! svrngs is stood in for at its surface alone, as give reaches it: a record
//! API on loopback that the test owns, asked with a token, and a `svrngs`
//! command on the PATH that writes down what it was asked. The app is the
//! stand-in card, kept in podman. Without podman each test is never-ran,
//! with its reason. Pods and volumes are named `svrngs-test-*` and removed
//! by the test that made them.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use give::card::Card;
use give::podman::Podman;
use give::runtime::{App, Name};
use serde_json::{Value, json};

const STANDIN: &str = include_str!("standin/compose.yaml");
const TOKEN: &str = "a-token-for-the-test";

fn podman_here() -> Podman {
    let podman = Podman::default();
    if let Err(why) = podman.run(&["info", "--format", "{{.Host.Security.Rootless}}"]) {
        panic!("never-ran: podman does not answer on this computer, so no app can be kept here ({why})");
    }
    podman
}

/// The record API svrngs serves on loopback, as give reads it, for each
/// of `computers` by its token: its key, its person and their key, and its
/// folder of the one group, `g1`. One thread owns the record; `/test/act`
/// writes an act as another computer would have, for the test.
struct Svrngs {
    port: u16,
    stop: mpsc::Sender<()>,
}

fn answer(mut s: &TcpStream, status: u16, body: &Value) {
    let body = body.to_string();
    let _ = write!(
        s,
        "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.len()
    );
}

type Computer = (&'static str, (&'static str, &'static str, &'static str), PathBuf);

fn record(computers: Vec<Computer>) -> Svrngs {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    listener.set_nonblocking(true).unwrap();
    let (stop, stopped) = mpsc::channel();
    std::thread::spawn(move || {
        let mut ops: Vec<Value> = Vec::new();
        while stopped.try_recv().is_err() {
            let Ok((s, _)) = listener.accept() else {
                std::thread::sleep(Duration::from_millis(20));
                continue;
            };
            s.set_nonblocking(false).unwrap();
            let mut r = BufReader::new(&s);
            let (mut line, mut headers) = (String::new(), BTreeMap::new());
            r.read_line(&mut line).unwrap();
            loop {
                let mut h = String::new();
                r.read_line(&mut h).unwrap();
                let Some((k, v)) = h.trim_end().split_once(": ") else {
                    break;
                };
                headers.insert(k.to_lowercase(), v.to_string());
            }
            let len = headers.get("content-length").map_or(0, |l| l.parse().unwrap());
            let mut body = vec![0; len];
            r.read_exact(&mut body).unwrap();
            let body: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
            let path = line.split(' ').nth(1).unwrap_or_default().to_string();
            let shown = headers.get("authorization").cloned().unwrap_or_default();
            let by = computers.iter().find(|(token, ..)| shown == format!("Bearer {token}"));
            let Some((_, (actor, person, key), folder)) =
                by.or(computers.first().filter(|_| path.starts_with("/test/")))
            else {
                answer(&s, 401, &json!({ "error": "no token" }));
                continue;
            };
            let id = format!("op{}", ops.len() + 1);
            let ts = ops.len() as i64 + 1;
            match (line.split(' ').next().unwrap_or_default(), path.as_str()) {
                ("GET", "/v1/groups") => answer(
                    &s,
                    200,
                    &json!({ "key": actor, "groups": [{ "id": "g1", "name": "Elm", "folders": [folder] }] }),
                ),
                ("GET", "/v1/rail/log?namespace=g1") => answer(&s, 200, &json!({ "ops": ops })),
                ("POST", "/v1/rail/append?namespace=g1") => {
                    let op = json!({ "id": id, "actor": actor, "person": person, "person_key": key, "ts_unix": ts, "voided": false, "payload": body["payload"] });
                    ops.push(op);
                    answer(&s, 200, &json!({ "id": id }));
                }
                ("POST", "/test/act") => {
                    let mut op = body;
                    op["id"] = id.clone().into();
                    op["ts_unix"] = ts.into();
                    op["voided"] = false.into();
                    ops.push(op);
                    answer(&s, 200, &json!({ "id": id }));
                }
                _ => answer(&s, 404, &json!({ "error": path })),
            }
        }
    });
    Svrngs { port, stop }
}

impl Svrngs {
    fn call(&self, method: &str, path: &str, body: &Value) -> Value {
        let mut s = TcpStream::connect(("127.0.0.1", self.port)).unwrap();
        let body = body.to_string();
        write!(
            s,
            "{method} {path} HTTP/1.1\r\nauthorization: Bearer {TOKEN}\r\ncontent-length: {}\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        serde_json::from_str(out.split_once("\r\n\r\n").unwrap().1).unwrap()
    }

    fn ops(&self) -> Vec<Value> {
        self.call("GET", "/v1/rail/log?namespace=g1", &Value::Null)["ops"]
            .as_array()
            .unwrap()
            .clone()
    }

    /// An act written as `actor`'s computer, `person`'s, with give's prefix.
    fn act(&self, (actor, person, key): (&str, &str, &str), mut payload: Value) -> String {
        payload["kind"] = format!("give/{}", payload["kind"].as_str().unwrap()).into();
        let op = json!({ "actor": actor, "person": person, "person_key": key, "payload": payload });
        self.call("POST", "/test/act", &op)["id"].as_str().unwrap().to_string()
    }
}

impl Drop for Svrngs {
    fn drop(&mut self) {
        let _ = self.stop.send(());
    }
}

const CY: (&str, &str, &str) = ("a1", "cy", "ka");
const BO: (&str, &str, &str) = ("b1", "bo", "kb");
/// Bo's computer's token, where a test runs a keeper there too.
const BOS: &str = "bo-s-token-for-the-test";

/// Each place a test keeps its files, under the target dir.
struct Places {
    root: PathBuf,
    data: PathBuf,
    give: PathBuf,
    folder: PathBuf,
    bin: PathBuf,
    asked: PathBuf,
    cards: PathBuf,
}

fn places(test: &str) -> Places {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("give-keep-{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let p = Places {
        data: root.join("svrngs"),
        give: root.join("give"),
        folder: root.join("Elm"),
        bin: root.join("bin"),
        asked: root.join("asked.txt"),
        cards: root.join("cards"),
        root,
    };
    for d in [&p.data, &p.give, &p.folder, &p.bin, &p.cards] {
        std::fs::create_dir_all(d).unwrap();
    }
    // svrngs's command, as give runs it: what it was asked, one line each.
    let svrngs = p.bin.join("svrngs");
    std::fs::write(&svrngs, format!("#!/bin/sh\necho \"$*\" >> '{}'\n", p.asked.display())).unwrap();
    std::fs::set_permissions(&svrngs, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    // Two stand-ins, each a kind of its own: `standin` and `other`.
    for kind in ["standin", "other"] {
        std::fs::create_dir_all(p.cards.join(kind)).unwrap();
        std::fs::write(p.cards.join(kind).join("compose.yaml"), STANDIN).unwrap();
    }
    p
}

/// Where `p`'s svrngs listens, and its token.
fn point(p: &Places, svrngs: &Svrngs, token: &str) {
    std::fs::write(p.data.join("api"), format!("http://127.0.0.1:{}\n", svrngs.port)).unwrap();
    std::fs::write(p.data.join("token"), format!("{token}\n")).unwrap();
}

fn prefix(test: &str) -> String {
    format!("svrngs-test-{test}-{}-", std::process::id())
}

/// `give keep`, with both stand-ins on its shelf, copying every 4 s.
fn keep(p: &Places, test: &str) -> Child {
    let path = format!("{}:{}", p.bin.display(), std::env::var("PATH").unwrap());
    Command::new(env!("CARGO_BIN_EXE_give"))
        .args(["keep", "--card"])
        .arg(p.cards.join("standin"))
        .arg("--card")
        .arg(p.cards.join("other"))
        .env("SVRNGS_DIR", &p.data)
        .env("GIVE_DIR", &p.give)
        .env("GIVE_TEST_PREFIX", prefix(test))
        .env("GIVE_TEST_EVERY", "4")
        .env("PATH", path)
        .env("RUST_LOG", "give=debug")
        .stdout(std::fs::File::create(p.root.join("keep.out")).unwrap())
        .stderr(std::fs::File::create(p.root.join("keep.err")).unwrap())
        .stdin(Stdio::null())
        .spawn()
        .unwrap()
}

/// Removes the stand-ins' pods and volumes, and the keeper, however a test
/// ends; `more`, each further keeper's, by its test name and root.
struct Removed<'a> {
    podman: &'a Podman,
    card: Card,
    test: String,
    keeper: Option<Child>,
    root: PathBuf,
    more: Vec<(String, Option<Child>, PathBuf)>,
}

impl Drop for Removed<'_> {
    fn drop(&mut self) {
        let first = (self.test.clone(), self.keeper.take(), self.root.clone());
        for (test, keeper, root) in [first].into_iter().chain(self.more.drain(..)) {
            if let Some(k) = keeper {
                stopped(k);
            }
            for kind in ["standin", "other"] {
                let name = Name::new(&format!("{}{kind}", prefix(&test))).unwrap();
                App::new(self.podman, &self.card, name)
                    .remove()
                    .expect("the test's pods and volumes removed");
            }
            if !std::thread::panicking() {
                continue;
            }
            for f in ["keep.out", "keep.err"] {
                let text = std::fs::read_to_string(root.join(f)).unwrap_or_default();
                eprintln!(
                    "--- {test}'s keeper's {f}:\n{}",
                    text.lines()
                        .rev()
                        .take(60)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            }
        }
    }
}

/// A keeper asked to stop, and killed only if it has not within a minute.
/// Killed at once, it can be inside a copy's `podman pod pause`, which
/// runs on as its own process and pauses the pod after `remove` has
/// unpaused it, and `pod rm` refuses a paused pod: "cannot remove
/// container ... as it is paused" (ralph/next/give/ctl/log/sessions/
/// give-app-scenes-1.await.log, the pod still paused after the run).
/// Asked, it finishes the copy, unpausing the pod, before it exits.
fn stopped(mut k: Child) {
    let _ = Command::new("kill").args(["-TERM", &k.id().to_string()]).status();
    for _ in 0..240 {
        if k.try_wait().is_ok_and(|s| s.is_some()) {
            return;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    let _ = k.kill();
    let _ = k.wait();
}

fn until<T>(what: &str, within: Duration, mut f: impl FnMut() -> Option<T>) -> T {
    let tries = within.as_millis() / 250;
    for _ in 0..tries {
        if let Some(t) = f() {
            return t;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    panic!("not within {within:?}: {what}");
}

fn asked(p: &Places) -> Vec<String> {
    std::fs::read_to_string(&p.asked)
        .unwrap_or_default()
        .lines()
        .map(String::from)
        .collect()
}

/// The snapshots this computer wrote of `app`, oldest first.
fn snapshots(p: &Places, app: &str) -> Vec<PathBuf> {
    let dir = p.folder.join("give").join(app).join("a1");
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|d| {
            d.flatten()
                .map(|e| e.path())
                .filter(|p| !p.file_name().unwrap().to_string_lossy().starts_with('.'))
                .filter(|p| p.file_name().unwrap() != "had")
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}

fn get(port: u16, path: &str) -> Option<String> {
    let mut s = TcpStream::connect(("127.0.0.1", port)).ok()?;
    write!(s, "GET {path} HTTP/1.0\r\n\r\n").ok()?;
    let mut out = String::new();
    s.read_to_string(&mut out).ok()?;
    out.split_once("\r\n\r\n").map(|(_, body)| body.to_string())
}

fn signal(child: &Child, signal: &str) {
    let ok = Command::new("kill")
        .args([signal, &child.id().to_string()])
        .status()
        .unwrap()
        .success();
    assert!(ok, "kill {signal} the keeper");
}

/// The keeper says it can keep, keeps the app the record names it for and
/// not the one it names another computer for, copies its state to the
/// group each interval, each copy naming its card version, and unlends and
/// stops it, with a last copy of the state it had, kept apart under
/// `had/`, once another computer takes it over.
#[test]
fn the_keeper_keeps_only_what_the_record_names_it_for() {
    let podman = podman_here();
    let card = Card::parse(STANDIN).unwrap();
    let p = places("names");
    let svrngs = record(vec![(TOKEN, CY, p.folder.clone())]);
    point(&p, &svrngs, TOKEN);
    let mut removed = Removed {
        podman: &podman,
        card: card.clone(),
        test: "names".into(),
        keeper: None,
        root: p.root.clone(),
        more: Vec::new(),
    };
    removed.keeper = Some(keep(&p, "names"));

    let readies = || {
        svrngs
            .ops()
            .into_iter()
            .filter(|o| o["payload"]["kind"] == "give/ready" && o["actor"] == "a1")
            .count()
    };
    until(
        "the keeper says this computer can keep",
        Duration::from_secs(30),
        || (readies() > 0).then_some(()),
    );

    // Bo's computer starts `other`: the record names Bo's, so cy's keeper
    // neither runs nor lends it.
    svrngs.act(BO, json!({ "kind": "ready" }));
    svrngs.act(
        BO,
        json!({ "kind": "started", "app": "other", "card": card.version, "agreed": true }),
    );
    // Cy's page starts `standin` on cy's computer.
    let started = svrngs.act(
        CY,
        json!({ "kind": "started", "app": "standin", "card": card.version, "agreed": true }),
    );

    let shared = until("the keeper lends standin", Duration::from_secs(180), || {
        asked(&p)
            .into_iter()
            .find(|l| l.starts_with("share ") && l.contains("--name standin"))
    });
    let port: u16 = shared.split(' ').nth(1).unwrap().parse().unwrap();
    assert_eq!(
        shared,
        format!("share {port} --name standin --kind standin"),
        "an HTTP door is lent as a page"
    );
    let starts = get(port, "/starts").expect("the lent port answers");
    assert_eq!(starts.lines().count(), 1, "{starts}");

    // A copy each interval, naming its card version.
    let copy = until(
        "a copy of standin in the group's folder",
        Duration::from_secs(60),
        || snapshots(&p, "standin").pop(),
    );
    assert_eq!(std::fs::read_to_string(copy.join("card")).unwrap(), card.version);
    assert!(copy.join("data.tar").is_file());
    until("a second copy, an interval on", Duration::from_secs(60), || {
        (snapshots(&p, "standin").len() >= 2).then_some(())
    });

    assert!(
        !asked(&p).iter().any(|l| l.contains("--name other")),
        "other is kept by bo's computer, so cy's never lends it: {:?}",
        asked(&p)
    );
    let other = format!("{}other", prefix("names"));
    assert!(!podman.answers(&["pod", "exists", &other]).unwrap(), "nor runs it");
    assert_eq!(readies(), 1, "this computer says it can keep once: {:#?}", svrngs.ops());

    // Bo's computer takes standin over: cy's unlends it, copies it once
    // more as it stops, and stops it.
    let newest = snapshots(&p, "standin").pop().unwrap();
    let from = newest.strip_prefix(&p.folder).unwrap().to_string_lossy().to_string();
    let sum = give::runtime::sum(&newest).unwrap();
    svrngs.act(BO, json!({ "kind": "taken", "app": "standin", "card": card.version, "over": started, "from": from, "sum": sum, "agreed": true }));
    until("the keeper unlends standin", Duration::from_secs(30), || {
        asked(&p).contains(&"unshare standin".to_string()).then_some(())
    });
    let standin = format!("{}standin", prefix("names"));
    until("the keeper stops standin", Duration::from_secs(60), || {
        (!podman.answers(&["pod", "exists", &standin]).unwrap()).then_some(())
    });
    let last = had(&p, "standin", "a1")
        .pop()
        .expect("a last copy as it stopped, as it had it");
    assert_eq!(std::fs::read_to_string(last.join("card")).unwrap(), card.version);
    assert!(snapshots(&p, "standin").len() <= 3, "older copies past three removed");
    std::thread::sleep(Duration::from_secs(5));
    assert_eq!(
        asked(&p).iter().filter(|l| l.starts_with("share ")).count(),
        1,
        "nothing lent again once another computer keeps it: {:?}",
        asked(&p)
    );

    let mut keeper = removed.keeper.take().unwrap();
    signal(&keeper, "-TERM");
    assert!(keeper.wait().unwrap().success(), "a keeper asked to stop exits 0");
    let _ = std::fs::remove_dir_all(&p.root);
}

/// Asked to stop, the keeper stops cleanly: it unlends what it keeps, copies
/// its state to the group a last time, stops it, and exits 0.
#[test]
fn a_clean_stop_copies_the_state_and_unlends() {
    let podman = podman_here();
    let card = Card::parse(STANDIN).unwrap();
    let p = places("clean");
    let svrngs = record(vec![(TOKEN, CY, p.folder.clone())]);
    point(&p, &svrngs, TOKEN);
    let mut removed = Removed {
        podman: &podman,
        card: card.clone(),
        test: "clean".into(),
        keeper: None,
        root: p.root.clone(),
        more: Vec::new(),
    };
    removed.keeper = Some(keep(&p, "clean"));
    let readies = || {
        svrngs
            .ops()
            .into_iter()
            .filter(|o| o["payload"]["kind"] == "give/ready" && o["actor"] == "a1")
            .count()
    };
    until(
        "the keeper says this computer can keep",
        Duration::from_secs(30),
        || (readies() > 0).then_some(()),
    );
    svrngs.act(
        CY,
        json!({ "kind": "started", "app": "standin", "card": card.version, "agreed": true }),
    );
    until("the keeper lends standin", Duration::from_secs(180), || {
        asked(&p)
            .into_iter()
            .find(|l| l.starts_with("share ") && l.contains("--name standin"))
    });
    let before = snapshots(&p, "standin").pop();

    let mut keeper = removed.keeper.take().unwrap();
    signal(&keeper, "-TERM");
    assert!(keeper.wait().unwrap().success(), "a keeper asked to stop exits 0");
    assert!(
        asked(&p).contains(&"unshare standin".to_string()),
        "unlent: {:?}",
        asked(&p)
    );
    let last = snapshots(&p, "standin").pop().expect("a last copy as it stopped");
    assert!(Some(&last) > before.as_ref(), "{last:?} after {before:?}");
    assert_eq!(std::fs::read_to_string(last.join("card")).unwrap(), card.version);
    let standin = format!("{}standin", prefix("clean"));
    assert!(!podman.answers(&["pod", "exists", &standin]).unwrap(), "stopped");
    let _ = std::fs::remove_dir_all(&p.root);
}

/// The snapshot dir `rel` of `from`'s folder, copied into `to`'s as the
/// group's folder carries it: `whole`, or with its state's tar cut after
/// its first file, as a copy still arriving. Podman imports a tar cut at a
/// file's end as a whole one, exit 0, the later files missing (podman 6.0.1
/// on a Mac, 2026-10-26), so only the move's sum tells it from the copy the
/// move names.
fn carry(from: &Places, to: &Places, rel: &str, whole: bool) {
    let (src, dst) = (from.folder.join(rel), to.folder.join(rel));
    std::fs::create_dir_all(&dst).unwrap();
    for e in std::fs::read_dir(&src).unwrap().flatten() {
        let bytes = std::fs::read(e.path()).unwrap();
        let torn = !whole && e.file_name().to_string_lossy().ends_with(".tar");
        let bytes = if torn {
            &bytes[..first_file_end(&bytes)]
        } else {
            &bytes[..]
        };
        std::fs::write(dst.join(e.file_name()), bytes).unwrap();
    }
}

/// Where a tar's first file ends: its header, then its bytes in 512s.
fn first_file_end(tar: &[u8]) -> usize {
    let mut archive = tar::Archive::new(tar);
    let first = archive.entries().unwrap().next().unwrap().unwrap();
    let size = first.header().size().unwrap() as usize;
    first.raw_file_position() as usize + size.div_ceil(512) * 512
}

/// Moving it to another computer (give-app-move; GIVE.md §3 scene 3): Cy
/// offers the stand-in to Bo's computer. Bo's no leaves everything as it
/// was: Cy's computer still keeps and lends it, Bo's starts nothing. On
/// Bo's yes, Cy's computer unlends it, takes a last snapshot, stops it and
/// writes the move naming that snapshot and its sum; Bo's computer starts
/// only from exactly that snapshot, waiting while its copy is still torn,
/// and the stand-in's state there is its state before the move, with the
/// one start Bo's computer gave it.
#[test]
fn a_move_starts_the_computer_offered_from_the_last_copy() {
    let podman = podman_here();
    let card = Card::parse(STANDIN).unwrap();
    let (a, b) = (places("move-a"), places("move-b"));
    let svrngs = record(vec![(TOKEN, CY, a.folder.clone()), (BOS, BO, b.folder.clone())]);
    point(&a, &svrngs, TOKEN);
    point(&b, &svrngs, BOS);
    let mut removed = Removed {
        podman: &podman,
        card: card.clone(),
        test: "move-a".into(),
        keeper: None,
        root: a.root.clone(),
        more: Vec::new(),
    };
    removed.keeper = Some(keep(&a, "move-a"));
    removed
        .more
        .push(("move-b".into(), Some(keep(&b, "move-b")), b.root.clone()));
    let of = |kind: &str, actor: &str| {
        (svrngs.ops().into_iter())
            .filter(|o| o["payload"]["kind"] == format!("give/{kind}") && o["actor"] == actor)
            .collect::<Vec<_>>()
    };
    until("both computers say they can keep", Duration::from_secs(30), || {
        (!of("ready", "a1").is_empty() && !of("ready", "b1").is_empty()).then_some(())
    });
    // Bo's computer is off while the app moves, and comes on with its copy
    // still arriving: the copy it meets first is torn.
    let bos = removed.more[0].1.take().unwrap();
    signal(&bos, "-TERM");
    assert!(
        bos.wait_with_output().unwrap().status.success(),
        "bo's keeper stops cleanly"
    );

    let started = svrngs.act(
        CY,
        json!({ "kind": "started", "app": "standin", "card": card.version, "agreed": true }),
    );
    let shared = until("cy's computer lends standin", Duration::from_secs(180), || {
        asked(&a).into_iter().find(|l| l.starts_with("share "))
    });
    let port: u16 = shared.split(' ').nth(1).unwrap().parse().unwrap();
    let before = get(port, "/starts").expect("the lent port answers");
    assert_eq!(before.lines().count(), 1, "{before}");
    // A second file in its state, as a world has many.
    let web = format!("{}standin-web", prefix("move-a"));
    podman
        .run(&["exec", &web, "sh", "-c", "echo later > /data/later"])
        .unwrap();

    // Bo says no: everything stays as it was.
    let offer = |svrngs: &Svrngs| {
        svrngs.act(
            CY,
            json!({ "kind": "offered", "app": "standin", "over": started, "to": "b1" }),
        )
    };
    let o1 = offer(&svrngs);
    svrngs.act(BO, json!({ "kind": "declined", "app": "standin", "offer": o1 }));
    std::thread::sleep(Duration::from_secs(8));
    assert_eq!(
        asked(&a),
        std::slice::from_ref(&shared),
        "a no: cy's computer still lends it, and only lends it"
    );
    assert!(
        asked(&b).is_empty(),
        "a no: bo's computer lends nothing: {:?}",
        asked(&b)
    );
    assert!(of("moved", "a1").is_empty(), "a no: nothing moves");
    assert_eq!(
        get(port, "/starts").as_deref(),
        Some(before.as_str()),
        "a no: the app runs on, untouched"
    );

    // Bo says yes.
    let o2 = offer(&svrngs);
    svrngs.act(
        BO,
        json!({ "kind": "accepted", "app": "standin", "offer": o2, "agreed": true }),
    );
    let moved = until("cy's computer writes the move", Duration::from_secs(120), || {
        of("moved", "a1").pop()
    });
    assert!(
        asked(&a).contains(&"unshare standin".to_string()),
        "unlent before it moved: {:?}",
        asked(&a)
    );
    let pod = |test: &str| format!("{}standin", prefix(test));
    until("cy's computer stops it, once moved", Duration::from_secs(60), || {
        (!podman.answers(&["pod", "exists", &pod("move-a")]).unwrap()).then_some(())
    });
    let from = moved["payload"]["from"].as_str().unwrap().to_string();
    let last = snapshots(&a, "standin").pop().unwrap();
    assert_eq!(a.folder.join(&from), last, "the move names the last copy");
    assert_eq!(moved["payload"]["over"], json!(started));
    assert_eq!(moved["payload"]["card"], json!(card.version));

    // Bo's computer comes on with the copy torn: it waits, starting nothing.
    carry(&a, &b, &from, false);
    removed.more[0].1 = Some(keep(&b, "move-b"));
    std::thread::sleep(Duration::from_secs(12));
    assert!(asked(&b).is_empty(), "a torn copy starts nothing: {:?}", asked(&b));
    assert!(!podman.answers(&["pod", "exists", &pod("move-b")]).unwrap());
    carry(&a, &b, &from, true);
    let shared = until("bo's computer lends standin", Duration::from_secs(180), || {
        asked(&b).into_iter().find(|l| l.starts_with("share "))
    });
    let port: u16 = shared.split(' ').nth(1).unwrap().parse().unwrap();
    let after = get(port, "/starts").expect("bo's lent port answers");
    assert!(
        after.starts_with(&before) && after.lines().count() == 2,
        "the state after the move is the state before it, and one start more:\nbefore {before:?}\nafter  {after:?}"
    );
    assert_eq!(get(port, "/later").as_deref(), Some("later\n"), "every file of it");
    std::thread::sleep(Duration::from_secs(3));
    assert_eq!(
        asked(&a).iter().filter(|l| l.starts_with("share ")).count(),
        1,
        "cy's computer lends it no more: {:?}",
        asked(&a)
    );
    let _ = std::fs::remove_dir_all(&a.root);
    let _ = std::fs::remove_dir_all(&b.root);
}

/// The copies of `app` that `actor`'s computer wrote as it had it after
/// another took the app over, oldest first.
fn had(p: &Places, app: &str, actor: &str) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(p.folder.join("give").join(app).join(actor).join("had"))
        .map(|d| d.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    out.retain(|p| !p.file_name().unwrap().to_string_lossy().starts_with('.'));
    out.sort();
    out
}

/// The stand-in's `starts`, as a copy holds it.
fn starts_in(copy: &Path) -> String {
    let mut tar = tar::Archive::new(std::fs::File::open(copy.join("data.tar")).unwrap());
    for e in tar.entries().unwrap() {
        let mut e = e.unwrap();
        if e.path().unwrap().file_name().is_some_and(|n| n == "starts") {
            let mut out = String::new();
            e.read_to_string(&mut out).unwrap();
            return out;
        }
    }
    panic!("{} holds no starts", copy.display());
}

/// `copy`, written again beside itself as a copy that does not start: its
/// state's `starts` a folder, so the stand-in's `date >> /data/starts`
/// fails and it exits. Its folder, relative to the group's.
fn broken(p: &Places, copy: &Path) -> String {
    let dir = copy.with_file_name("99999999999999");
    std::fs::create_dir_all(&dir).unwrap();
    for f in ["card", "data.root"] {
        std::fs::copy(copy.join(f), dir.join(f)).unwrap();
    }
    let mut tar = tar::Builder::new(std::fs::File::create(dir.join("data.tar")).unwrap());
    let mut h = tar::Header::new_gnu();
    h.set_entry_type(tar::EntryType::Directory);
    h.set_path("starts/").unwrap();
    h.set_mode(0o755);
    h.set_uid(1000);
    h.set_gid(1000);
    h.set_size(0);
    h.set_cksum();
    tar.append(&h, std::io::empty()).unwrap();
    tar.finish().unwrap();
    dir.strip_prefix(&p.folder).unwrap().to_string_lossy().to_string()
}

/// The keeper is gone, then back (give-app-takeover; GIVE.md §3 scenes 4
/// and 5). Cy's computer keeps the stand-in, copying it each interval, and
/// loses power. Bo's takes it over from Cy's newest copy, which does not
/// start: Bo's says so, and lends nothing. Bo's takes it from the copy
/// before; that one starts, and exits within a minute: Bo's says so again,
/// and unlends it. From the copy before that it starts and stays. Cy's
/// computer comes back: it lends nothing and starts nothing, and keeps the
/// state it had as a copy of its own, apart. Bo's state is untouched by it:
/// nothing is merged. Starting that copy instead is a takeover from it, and
/// Bo's own later state is kept as Bo's had it: nothing is thrown away.
#[test]
fn the_keeper_gone_then_back() {
    let podman = podman_here();
    let card = Card::parse(STANDIN).unwrap();
    let (a, b) = (places("gone-a"), places("gone-b"));
    let svrngs = record(vec![(TOKEN, CY, a.folder.clone()), (BOS, BO, b.folder.clone())]);
    point(&a, &svrngs, TOKEN);
    point(&b, &svrngs, BOS);
    let mut removed = Removed {
        podman: &podman,
        card: card.clone(),
        test: "gone-a".into(),
        keeper: None,
        root: a.root.clone(),
        more: Vec::new(),
    };
    removed.keeper = Some(keep(&a, "gone-a"));
    removed
        .more
        .push(("gone-b".into(), Some(keep(&b, "gone-b")), b.root.clone()));
    let of = |kind: &str, actor: &str| {
        (svrngs.ops().into_iter())
            .filter(|o| o["payload"]["kind"] == format!("give/{kind}") && o["actor"] == actor)
            .collect::<Vec<_>>()
    };
    let kept = || give_fold::fold(&give::record::gives(&svrngs.ops()));
    until("both computers say they can keep", Duration::from_secs(30), || {
        (!of("ready", "a1").is_empty() && !of("ready", "b1").is_empty()).then_some(())
    });
    let started = svrngs.act(
        CY,
        json!({ "kind": "started", "app": "standin", "card": card.version, "agreed": true }),
    );
    until("cy's computer lends standin", Duration::from_secs(180), || {
        asked(&a).into_iter().find(|l| l.starts_with("share "))
    });
    // Three copies, each said in the record, the keeper's answer that it is there.
    let copies = until("three copies of cy's, each said", Duration::from_secs(60), || {
        let said = of("copied", "a1");
        (said.len() >= 3).then_some(said)
    });
    for c in &copies {
        assert_eq!(c["payload"]["over"], json!(started), "{c}");
        assert_eq!(c["payload"]["card"], json!(card.version));
        let dir = a.folder.join(c["payload"]["from"].as_str().unwrap());
        assert_eq!(
            c["payload"]["sum"].as_str().unwrap(),
            give::runtime::sum(&dir).unwrap(),
            "a copy's sum is its files'"
        );
    }

    // Cy's computer loses power: its keeper and its app stop at once.
    let mut cys = removed.keeper.take().unwrap();
    signal(&cys, "-KILL");
    let _ = cys.wait();
    let pod = |test: &str| format!("{}standin", prefix(test));
    podman.run(&["pod", "stop", &pod("gone-a")]).unwrap();
    let cy_starts = {
        let newest = kept().next("standin").unwrap().from.clone();
        std::fs::metadata(a.folder.join(&newest)).unwrap();
        newest
    };

    // Its newest copy is one that does not start.
    let newest = a.folder.join(&cy_starts);
    let bad = broken(&a, &newest);
    let bad_sum = give::runtime::sum(&a.folder.join(&bad)).unwrap();
    svrngs.act(CY, json!({ "kind": "copied", "app": "standin", "card": card.version, "over": started, "from": bad, "sum": bad_sum }));
    for c in snapshots(&a, "standin") {
        let rel = c.strip_prefix(&a.folder).unwrap().to_string_lossy().to_string();
        carry(&a, &b, &rel, true);
    }

    // Bo keeps it there, from the newest copy: it does not start.
    let next = kept().next("standin").unwrap().clone();
    assert_eq!(next.from, bad, "the newest copy is the one offered");
    let t1 = svrngs.act(BO, json!({ "kind": "taken", "app": "standin", "card": card.version, "over": started, "from": next.from, "sum": next.sum, "agreed": true }));
    let failed = until(
        "bo's computer says the copy did not start",
        Duration::from_secs(180),
        || of("failed", "b1").pop(),
    );
    assert_eq!(failed["payload"]["over"], json!(t1));
    assert!(
        asked(&b).is_empty(),
        "a copy that did not start is not lent: {:?}",
        asked(&b)
    );
    std::thread::sleep(Duration::from_secs(5));
    assert_eq!(of("failed", "b1").len(), 1, "nor tried again: the person chooses");

    // The copy before it is offered, and starts; then exits within a minute.
    let before = kept().next("standin").unwrap().clone();
    assert_eq!(before.from, cy_starts, "the copy before the one that did not start");
    let t2 = svrngs.act(BO, json!({ "kind": "taken", "app": "standin", "card": card.version, "over": t1, "from": before.from, "sum": before.sum, "agreed": true }));
    until("bo's computer lends standin", Duration::from_secs(180), || {
        asked(&b).into_iter().find(|l| l.starts_with("share "))
    });
    podman.run(&["kill", &format!("{}-web", pod("gone-b"))]).unwrap();
    let exited = until("bo's computer says it exited", Duration::from_secs(60), || {
        of("failed", "b1")
            .into_iter()
            .find(|f| f["payload"]["over"] == json!(t2))
    });
    assert!(
        asked(&b).contains(&"unshare standin".to_string()),
        "an app that exited is unlent: {:?} ({exited})",
        asked(&b)
    );

    // The one before that starts, and stays.
    let third = kept().next("standin").unwrap().clone();
    assert!(third.from < cy_starts, "{} before {cy_starts}", third.from);
    let t3 = svrngs.act(BO, json!({ "kind": "taken", "app": "standin", "card": card.version, "over": t2, "from": third.from, "sum": third.sum, "agreed": true }));
    let shared = until("bo's computer lends standin again", Duration::from_secs(180), || {
        asked(&b).into_iter().filter(|l| l.starts_with("share ")).nth(1)
    });
    let port: u16 = shared.split(' ').nth(1).unwrap().parse().unwrap();
    let bos = until("bo's lent port answers", Duration::from_secs(30), || {
        get(port, "/starts")
    });
    assert_eq!(kept().apps["standin"].act, t3);

    // Cy's computer comes back: it sees Bo's keeps it, lends nothing,
    // starts nothing, and keeps the state it had as a copy, apart.
    let shares = asked(&a).iter().filter(|l| l.starts_with("share ")).count();
    removed.keeper = Some(keep(&a, "gone-a"));
    let mine = until(
        "cy's computer copies the state it had",
        Duration::from_secs(120),
        || {
            of("copied", "a1")
                .into_iter()
                .find(|c| c["payload"]["from"].as_str().unwrap().contains("/had/"))
        },
    );
    assert_eq!(
        mine["payload"]["over"],
        json!(started),
        "the state as cy's keeping had it"
    );
    let cys_had = had(&a, "standin", "a1").pop().expect("in cy's folder");
    assert_eq!(a.folder.join(mine["payload"]["from"].as_str().unwrap()), cys_had);
    let listed = kept()
        .had("standin")
        .into_iter()
        .map(|c| c.act.clone())
        .collect::<Vec<_>>();
    assert_eq!(listed, [mine["id"].as_str().unwrap()], "listed as cy's computer had it");
    std::thread::sleep(Duration::from_secs(5));
    assert_eq!(
        asked(&a).iter().filter(|l| l.starts_with("share ")).count(),
        shares,
        "cy's computer lends nothing once back: {:?}",
        asked(&a)
    );
    assert!(
        !podman.answers(&["pod", "exists", &pod("gone-a")]).unwrap(),
        "nor runs it"
    );
    assert_eq!(
        get(port, "/starts").as_deref(),
        Some(bos.as_str()),
        "bo's state is untouched: nothing merged"
    );

    // Starting cy's copy instead is a takeover from it: bo's computer
    // keeps its own later state as it had it, and starts from cy's.
    let rel = cys_had.strip_prefix(&a.folder).unwrap().to_string_lossy().to_string();
    carry(&a, &b, &rel, true);
    let cys = kept().had("standin")[0].clone();
    svrngs.act(BO, json!({ "kind": "taken", "app": "standin", "card": card.version, "over": t3, "from": cys.from, "sum": cys.sum, "agreed": true }));
    let shared = until("bo's computer lends cy's copy", Duration::from_secs(180), || {
        asked(&b).into_iter().filter(|l| l.starts_with("share ")).nth(2)
    });
    let port: u16 = shared.split(' ').nth(1).unwrap().parse().unwrap();
    let now = until("its port answers", Duration::from_secs(30), || get(port, "/starts"));
    let cy_lines = starts_in(&cys_had);
    assert!(
        now.starts_with(&cy_lines) && now.lines().count() == cy_lines.lines().count() + 1,
        "started from cy's copy, and one start more:\ncy's  {cy_lines:?}\nnow   {now:?}"
    );
    assert_ne!(now, bos, "not bo's state");
    assert!(
        !had(&b, "standin", "b1").is_empty(),
        "bo's later state kept as bo's had it"
    );
    // By computer, not by count: a copy bo's keeper began under t3 can be
    // said after the takeover lands, and folds as one more as bo's had it
    // (give-docs's gate, 2026-10-26: op15, begun 06:02:50.57, its pause
    // 3.1 s, said 06:02:54.0 after op14).
    let listed = (kept().had("standin").into_iter())
        .map(|c| c.computer.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        listed,
        ["a1".to_string(), "b1".to_string()].into(),
        "both copies as each had it stand: nothing thrown away"
    );
    let _ = std::fs::remove_dir_all(&a.root);
    let _ = std::fs::remove_dir_all(&b.root);
}
