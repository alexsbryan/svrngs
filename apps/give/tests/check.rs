//! A card runs: the stand-in pulled by digest, started rootless, healthy by
//! its own healthcheck, snapshotted, stopped, restored and healthy again,
//! its state the same across the restore, and an app left paused stopped
//! and removed. `give check`'s steps, and the runtime's decisions under
//! them, are judged over tests/fake's podman, so they run anywhere.
//!
//! One test needs podman, the contained test of give's seam with it
//! (docs/CHECKS.md, seam D): that real podman answers the commands give
//! runs as tests/fake answers them. Where podman does not answer, it is
//! never-ran with its reason, a failure of the gate and never a pass. Every
//! pod and volume it makes is named `svrngs-test-*` and removed by it.

mod fake;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use give::card::{Card, Transport};
use give::check::{Step, check, line_failed, line_passed, line_shelf};
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

/// Real podman keeps the stand-in as give asks and as tests/fake answers:
/// started, healthy, snapshotted whole (its card version named), stopped
/// and gone, restored the same, started again from the state it had. Left
/// paused by a keeper killed in a snapshot's pause, whole (the pod
/// "Paused") or half, killed inside `pod pause` itself (the pod
/// "Degraded"), podman's `pod rm` refuses it ("cannot remove container …
/// as it is paused", podman 6.0.1), and give still stops it, and removes
/// it.
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

    let (pod, web) = (name("keep"), format!("{}-web", name("keep").as_str()));
    for (pause, end) in [
        ("pod", "stops"),
        ("web", "stops"),
        ("pod", "is removed"),
        ("web", "is removed"),
    ] {
        if !podman.answers(&["pod", "exists", pod.as_str()]).unwrap() {
            app.start(free_port(), &Agreed::to(&card.asks)).unwrap();
            app.wait_healthy().unwrap();
        }
        if pause == "pod" {
            podman.run(&["pod", "pause", pod.as_str()]).unwrap();
        } else {
            podman.run(&["pause", &web]).unwrap();
        }
        let refused = podman.run(&["pod", "rm", pod.as_str()]);
        let state = podman.run(&["pod", "inspect", "--format", "{{.State}}", pod.as_str()]);
        let paused = if pause == "pod" { "Paused" } else { "Degraded" };
        assert_eq!(state.unwrap(), paused, "the plant: paused by {pause}");
        assert!(
            refused.is_err(),
            "podman refuses the app paused by {pause}, as tests/fake does"
        );
        let ended = if end == "stops" { app.stop() } else { app.remove() };
        ended.unwrap_or_else(|e| panic!("an app paused by {pause} {end}: {e}"));
    }
    gone(&podman, &pod);
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

/// A port for the check's door: nothing listens, as nothing runs.
fn no_port(_: Transport) -> std::io::Result<u16> {
    Ok(40000)
}

/// `give check` of the card in `dir` over `podman`, in process: what it
/// says, a line each, as main.rs says them, and how it ended.
fn checked(podman: &Arc<fake::Podman>, dir: &Path, test: &str) -> (Result<Card, String>, Vec<String>) {
    let mut said = Vec::new();
    let mut passed = |step: Step, what: String| said.push(line_passed(step, &what));
    let podman = Podman::by(podman.clone());
    let name = Name::new(&format!("svrngs-test-{test}")).unwrap();
    let ended = check(&podman, dir, name, &scratch(test), false, no_port, &mut passed);
    match ended {
        Ok(card) => {
            said.push(line_shelf(&card));
            (Ok(card), said)
        }
        Err(failed) => {
            said.push(line_failed(&failed));
            (Err(line_failed(&failed)), said)
        }
    }
}

/// The check passes the stand-in step by step, saying each, and leaves no
/// pod and no volume behind: over a podman that answers as podman does
/// (tests/fake), so it runs anywhere.
#[test]
fn a_check_passes_the_stand_in_and_leaves_nothing() {
    let podman = Arc::new(fake::Podman::default());
    let card = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/standin"));
    let (ended, said) = checked(&podman, card, "check");
    assert!(ended.is_ok(), "{said:#?}");
    for step in [
        "pull by digest",
        "healthy",
        "snapshot",
        "stop",
        "restore: the state is the snapshot's",
        "healthy again",
    ] {
        let line = format!("passed  {step}");
        assert!(said.iter().any(|l| l.starts_with(&line)), "{step}: {said:#?}");
    }
    assert_eq!(said.last().unwrap(), "Stand-in can go on the shelf.", "{said:#?}");
    assert_eq!(podman.pods(), Vec::<String>::new(), "no pod left");
    assert_eq!(podman.volumes(), Vec::<String>::new(), "no volume left");
}

/// A card that never becomes healthy fails at `healthy`, after `start`
/// passed, and the check removes what it made. Its retries are none, so
/// its patience is spent at the first answer that it is not healthy yet,
/// with no wait (runtime.rs `wait_healthy`).
#[test]
fn a_check_names_the_step_that_failed_and_leaves_nothing() {
    let podman = Arc::new(fake::Podman::default());
    podman.unhealthy();
    let dir = scratch("unhealthy-card-fake");
    std::fs::create_dir_all(&dir).unwrap();
    let never = STANDIN.replace(
        "[\"CMD\", \"wget\", \"-q\", \"-O\", \"/dev/null\", \"http://web:8080/starts\"]",
        "[\"CMD\", \"false\"]",
    );
    std::fs::write(dir.join("compose.yaml"), never.replace("retries: 10", "retries: 0")).unwrap();
    let (ended, said) = checked(&podman, &dir, "unhealthy");
    let failed = ended.unwrap_err();
    assert!(said.iter().any(|l| l.starts_with("passed  start")), "{said:#?}");
    assert!(
        failed.starts_with("failed  healthy: web was not healthy within"),
        "{failed}"
    );
    assert_eq!(podman.pods(), Vec::<String>::new(), "no pod left");
    assert_eq!(podman.volumes(), Vec::<String>::new(), "no volume left");
    let _ = std::fs::remove_dir_all(&dir);
}

/// The stand-in, started and healthy over a podman that answers as podman
/// does, and its snapshot.
fn kept<'a>(podman: &'a Podman, card: &'a Card, test: &str) -> (App<'a>, give::runtime::Snapshot, PathBuf) {
    let app = App::new(podman, card, name(test));
    let dir = scratch(test);
    app.pull().unwrap();
    app.start(40000, &Agreed::to(&card.asks)).unwrap();
    app.wait_healthy().unwrap();
    let snapshot = app.snapshot_at(&dir, fake::stamp).unwrap();
    (app, snapshot, dir)
}

/// What a restore brings back is compared with its snapshot file by file,
/// not judged by the app's health: the same state is none different; the
/// state changed after the snapshot, as a broken restore would leave it, is
/// told by its path.
#[test]
fn a_restore_unlike_its_snapshot_is_told() {
    let (fake, card) = (Arc::new(fake::Podman::default()), Card::parse(STANDIN).unwrap());
    let podman = Podman::by(fake.clone());
    let (app, snapshot, dir) = kept(&podman, &card, "differs-fake");
    app.stop().unwrap();
    app.restore(&snapshot.dir).unwrap();
    assert_eq!(app.differs(&snapshot.dir, &dir).unwrap(), Vec::<String>::new());
    app.start(40000, &Agreed::to(&card.asks)).unwrap();
    app.wait_healthy().unwrap();
    app.stop().unwrap();
    assert_eq!(app.differs(&snapshot.dir, &dir).unwrap(), ["data/starts changed"]);
    app.remove().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}

/// A copy is started only by the card version that wrote it (GIVE.md §4):
/// a snapshot naming another version is refused before podman is asked
/// anything, so the state the app has stays as it is.
#[test]
fn a_copy_another_card_version_wrote_is_refused() {
    let (fake, card) = (Arc::new(fake::Podman::default()), Card::parse(STANDIN).unwrap());
    let podman = Podman::by(fake.clone());
    let (app, snapshot, dir) = kept(&podman, &card, "version-fake");
    app.stop().unwrap();
    let named = std::fs::read_to_string(snapshot.dir.join("card")).unwrap();
    assert_eq!(named, card.version, "the snapshot names the card version that wrote it");
    std::fs::write(snapshot.dir.join("card"), "another").unwrap();
    let asked = fake.said();
    let refused = app.restore(&snapshot.dir).unwrap_err().to_string();
    assert!(
        refused.contains("another version of its card wrote it") && refused.contains(&card.version),
        "{refused}"
    );
    assert_eq!(fake.said(), asked, "podman asked nothing");
    assert_eq!(
        app.differs(&snapshot.dir, &dir).unwrap(),
        Vec::<String>::new(),
        "the state untouched"
    );
    app.remove().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}

/// An app a keeper left paused, whole (killed in a snapshot's pause) or
/// half (killed inside `pod pause`, the pod "Degraded"), is unpaused, then
/// stopped or removed: podman's `pod stop` leaves a paused container
/// paused and `pod rm` refuses it (tests/fake answers so, and
/// `the_stand_in_keeps_its_state_across_a_restore` holds podman to it).
#[test]
fn a_paused_app_is_unpaused_then_stopped_and_removed() {
    let (fake, card) = (Arc::new(fake::Podman::default()), Card::parse(STANDIN).unwrap());
    let podman = Podman::by(fake.clone());
    let app = App::new(&podman, &card, name("paused-fake"));
    let pod = name("paused-fake").as_str().to_string();
    let web = format!("{pod}-web");
    for (pause, end) in [
        ("pod", "stops"),
        ("pod", "is removed"),
        ("web", "stops"),
        ("web", "is removed"),
    ] {
        app.start(40000, &Agreed::to(&card.asks)).unwrap();
        match pause {
            "pod" => podman.run(&["pod", "pause", &pod]).unwrap(),
            _ => podman.run(&["pause", &web]).unwrap(),
        };
        let state = podman.run(&["pod", "inspect", "--format", "{{.State}}", &pod]).unwrap();
        assert_eq!(state, if pause == "pod" { "Paused" } else { "Degraded" });
        let asked = fake.said().len();
        let ended = if end == "stops" { app.stop() } else { app.remove() };
        ended.unwrap_or_else(|e| panic!("an app paused by {pause} {end}: {e}"));
        let said = &fake.said()[asked..];
        assert!(said.contains(&format!("unpause {web}\n")), "unpaused first: {said}");
        assert!(!podman.answers(&["pod", "exists", &pod]).unwrap(), "{end}: {said}");
    }
    app.remove().unwrap();
    assert_eq!(fake.pods(), Vec::<String>::new());
    assert_eq!(fake.volumes(), Vec::<String>::new());
}
