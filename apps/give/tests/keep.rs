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
            if let Some(mut k) = keeper {
                let _ = k.kill();
                let _ = k.wait();
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
/// stops it, with a last copy, once another computer takes it over.
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
    svrngs.act(BO, json!({ "kind": "taken", "app": "standin", "card": card.version, "over": started, "from": from, "agreed": true }));
    until("the keeper unlends standin", Duration::from_secs(30), || {
        asked(&p).contains(&"unshare standin".to_string()).then_some(())
    });
    let standin = format!("{}standin", prefix("names"));
    until("the keeper stops standin", Duration::from_secs(60), || {
        (!podman.answers(&["pod", "exists", &standin]).unwrap()).then_some(())
    });
    let last = snapshots(&p, "standin").pop().unwrap();
    assert!(last > newest, "a last copy as it stopped: {last:?} after {newest:?}");
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
