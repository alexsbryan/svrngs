//! A card runs: the stand-in pulled by digest, started rootless, healthy by
//! its own healthcheck, snapshotted, stopped, restored and healthy again,
//! its state the same across the restore. These need podman; where it does
//! not answer, each is never-ran with its reason, a failure of the gate and
//! never a pass. Every pod and volume they make is named `svrngs-test-*` and
//! removed by the test that made it.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Command;

use give::card::Card;
use give::podman::Podman;
use give::runtime::{Agreed, App, Name};

const STANDIN: &str = include_str!("standin/compose.yaml");

fn podman_here() -> Podman {
    let podman = Podman::default();
    if let Err(why) = podman.run(&["info", "--format", "{{.Host.Security.Rootless}}"]) {
        panic!("never-ran: podman does not answer on this computer, so no card can run here ({why})");
    }
    podman
}

fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("give-{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn name(test: &str) -> Name {
    Name::new(&format!("svrngs-test-{test}-{}", std::process::id())).unwrap()
}

/// Removes what a test made, however it ends.
struct Removed<'a>(&'a App<'a>);

impl Drop for Removed<'_> {
    fn drop(&mut self) {
        self.0.remove().expect("the test's pod and volumes removed");
    }
}

fn get(port: u16, path: &str) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    write!(s, "GET {path} HTTP/1.0\r\n\r\n").unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out.split_once("\r\n\r\n")
        .map(|(_, body)| body.to_string())
        .unwrap_or_default()
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[test]
fn the_stand_in_keeps_its_state_across_a_restore() {
    let podman = podman_here();
    let card = Card::parse(STANDIN).unwrap();
    let app = App::new(&podman, &card, name("keep"));
    let _removed = Removed(&app);
    let (dir, at) = (scratch("keep"), free_port());
    app.pull().unwrap();
    app.start(at, &Agreed::to(&card.asks)).unwrap();
    app.wait_healthy().unwrap();
    let first = get(at, "/starts");
    assert_eq!(first.lines().count(), 1, "{first}");

    let snapshot = app.snapshot(&dir).unwrap();
    assert!(snapshot.dir.join("data.tar").is_file());
    let named = std::fs::read_to_string(snapshot.dir.join("card")).unwrap();
    assert_eq!(named, card.version, "the snapshot names the card version that wrote it");
    let left: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(left.len(), 1, "only the whole snapshot, no partial one: {left:?}");
    println!("snapshot: {:?}, paused {:?}", snapshot.copies, snapshot.paused);

    app.stop().unwrap();
    assert!(
        !podman.answers(&["pod", "exists", name("keep").as_str()]).unwrap(),
        "stopped is gone"
    );
    app.restore(&snapshot.dir).unwrap();
    assert_eq!(app.differs(&snapshot.dir, &dir).unwrap(), Vec::<String>::new());
    app.start(at, &Agreed::to(&card.asks)).unwrap();
    app.wait_healthy().unwrap();
    let again = get(at, "/starts");
    assert_eq!(
        again.lines().count(),
        2,
        "the first start's line kept, the second's added: {again}"
    );
    assert!(again.starts_with(&first), "{first} then {again}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A restore that does not bring back what was snapshotted is caught by the
/// check, not by the app's health: the state here is changed after the
/// snapshot, as a broken restore would leave it.
#[test]
fn a_state_unlike_its_snapshot_is_told() {
    let podman = podman_here();
    let card = Card::parse(STANDIN).unwrap();
    let app = App::new(&podman, &card, name("differs"));
    let _removed = Removed(&app);
    let (dir, at) = (scratch("differs"), free_port());
    app.pull().unwrap();
    app.start(at, &Agreed::to(&card.asks)).unwrap();
    app.wait_healthy().unwrap();
    let snapshot = app.snapshot(&dir).unwrap();
    app.stop().unwrap();
    app.start(at, &Agreed::to(&card.asks)).unwrap();
    app.wait_healthy().unwrap();
    app.stop().unwrap();
    assert_eq!(app.differs(&snapshot.dir, &dir).unwrap(), ["data/starts changed"]);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A copy is started only by the card version that wrote it (GIVE.md §4):
/// a snapshot naming another version is refused before any volume is
/// touched, so the state the app has stays as it is.
#[test]
fn a_snapshot_of_another_card_version_is_refused() {
    let podman = podman_here();
    let card = Card::parse(STANDIN).unwrap();
    let app = App::new(&podman, &card, name("version"));
    let _removed = Removed(&app);
    let (dir, at) = (scratch("version"), free_port());
    app.pull().unwrap();
    app.start(at, &Agreed::to(&card.asks)).unwrap();
    app.wait_healthy().unwrap();
    let snapshot = app.snapshot(&dir).unwrap();
    app.stop().unwrap();
    std::fs::write(snapshot.dir.join("card"), "another").unwrap();
    let refused = app.restore(&snapshot.dir).unwrap_err().to_string();
    assert!(
        refused.contains("another version of its card wrote it") && refused.contains(&card.version),
        "{refused}"
    );
    assert_eq!(
        app.differs(&snapshot.dir, &dir).unwrap(),
        Vec::<String>::new(),
        "the state untouched"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

fn give(args: &[&str]) -> (bool, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_give"))
        .args(args)
        .env("TMPDIR", env!("CARGO_TARGET_TMPDIR"))
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), said)
}

fn gone(podman: &Podman, name: &Name) {
    assert!(
        !podman.answers(&["pod", "exists", name.as_str()]).unwrap(),
        "{} left a pod",
        name.as_str()
    );
    let volumes = podman
        .run(&[
            "volume",
            "ls",
            "--quiet",
            "--filter",
            &format!("name={}", name.as_str()),
        ])
        .unwrap();
    assert_eq!(volumes, "", "{} left volumes", name.as_str());
}

#[test]
fn give_check_passes_the_stand_in_and_leaves_nothing() {
    let podman = podman_here();
    let n = name("check");
    let card = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/standin");
    let (ok, said) = give(&["check", "--name", n.as_str(), card]);
    println!("{said}");
    assert!(ok, "{said}");
    for step in [
        "pull by digest",
        "healthy",
        "snapshot",
        "stop",
        "restore: the state is the snapshot's",
        "healthy again",
    ] {
        assert!(said.contains(&format!("passed  {step}")), "{step}: {said}");
    }
    assert!(said.contains("Stand-in can go on the shelf."), "{said}");
    gone(&podman, &n);
}

#[test]
fn give_check_names_the_step_that_failed_and_leaves_nothing() {
    let podman = podman_here();
    let n = name("unhealthy");
    let dir = scratch("unhealthy-card");
    std::fs::create_dir_all(&dir).unwrap();
    let never = STANDIN.replace(
        "[\"CMD\", \"wget\", \"-q\", \"-O\", \"/dev/null\", \"http://web:8080/starts\"]",
        "[\"CMD\", \"false\"]",
    );
    std::fs::write(dir.join("compose.yaml"), never.replace("retries: 10", "retries: 2")).unwrap();
    let (ok, said) = give(&["check", "--name", n.as_str(), dir.to_str().unwrap()]);
    assert!(!ok, "{said}");
    assert!(said.contains("passed  start"), "{said}");
    assert!(said.contains("failed  healthy: web was not healthy within"), "{said}");
    gone(&podman, &n);
    let _ = std::fs::remove_dir_all(&dir);
}

/// A card with asks is not started until whoever runs it says yes; podman
/// is never reached, so this one runs anywhere.
#[test]
fn give_check_asks_before_it_starts_anything() {
    let dir = scratch("asks-card");
    std::fs::create_dir_all(&dir).unwrap();
    let asks = "  asks:\n    - say: Its makers ask whoever runs it to agree to their terms.\n      link: https://example.org/terms\n      sets: { AGREED: \"TRUE\" }\n";
    std::fs::write(dir.join("compose.yaml"), format!("{STANDIN}{asks}")).unwrap();
    let (ok, said) = give(&["check", "--name", "svrngs-test-asks", dir.to_str().unwrap()]);
    assert!(!ok, "{said}");
    assert!(
        said.contains(
            "failed  its asks: Its makers ask whoever runs it to agree to their terms. https://example.org/terms"
        ),
        "{said}"
    );
    assert!(said.contains("--agree"), "{said}");
    assert!(!said.contains("passed  start"), "{said}");
    let _ = std::fs::remove_dir_all(&dir);
}
