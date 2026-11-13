//! The keeper program, `give keep` (give-app-keep (a); GIVE.md §7): it
//! keeps an app only while the group's record names this computer for it.
//! It lends the app's door (`svrngs share`) from a healthy start, writes a
//! snapshot into the group's folder at each interval and at a clean stop,
//! each naming its card version, and unlends and stops the app when the
//! record names another computer.
//!
//! Each computer's [`Keeper`] is driven one round at a time at a `now` the
//! test sets, over what answers as podman, svrngs's command and its record
//! do (tests/fake): nothing runs, listens or waits, and a crash is the
//! keeper dropped and another started over the same files. What `plan`
//! decides is tests/plan.rs's; here is what the keeper does with it: the
//! copies it writes and says, what it lends and stops, and the state it
//! keeps. That podman answers as the fake does is tests/check.rs's one
//! podman test; that give keeps an app in real podman, beside a real
//! svrngs, is the give scenes' (crates/svrngs/tests/scenes/give*.rs).

mod fake;

use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;

use fake::Who;
use give::card::Card;
use give::keep::{EVERY, KEEP, Keeper, ROUND};
use give::lend::Svrngs;
use give::podman::Podman;
use give::record::Record;
use give::shelf::Shelf;
use serde_json::{Value, json};

const STANDIN: &str = include_str!("standin/compose.yaml");

const CY: Who = ("a1", "cy", "ka");
const BO: Who = ("b1", "bo", "kb");
/// Cy's other computer, which keeps nothing: a stop asked from it is Cy's.
const CYS_OTHER: Who = ("a2", "cy", "ka");

/// When each test begins, in seconds.
const T0: u64 = 1_800_000_000;

/// The test's folders, with both stand-ins' cards, `standin` and `other`,
/// each a kind of its own.
fn root(test: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("give-keeper-{test}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for kind in ["standin", "other"] {
        std::fs::create_dir_all(root.join("cards").join(kind)).unwrap();
        std::fs::write(root.join("cards").join(kind).join("compose.yaml"), STANDIN).unwrap();
    }
    root
}

/// One computer that keeps: its podman, its svrngs's command, its folder of
/// the group, give's own files, and the keeper program running there.
struct Computer {
    who: Who,
    record: Arc<fake::Record>,
    podman: Arc<fake::Podman>,
    lends: Arc<fake::Lends>,
    folder: PathBuf,
    dir: PathBuf,
    cards: PathBuf,
    keeper: Keeper,
}

impl Computer {
    fn new(record: &Arc<fake::Record>, who: Who, root: &Path) -> Computer {
        let (folder, dir) = (root.join(who.0).join("Elm"), root.join(who.0).join("give"));
        std::fs::create_dir_all(&folder).unwrap();
        let (podman, lends) = (Arc::new(fake::Podman::default()), Arc::new(fake::Lends::default()));
        let cards = root.join("cards");
        let keeper = program(record, who, &folder, &dir, &cards, &podman, &lends);
        Computer {
            who,
            record: record.clone(),
            podman,
            lends,
            folder,
            dir,
            cards,
            keeper,
        }
    }

    /// The keeper program started again over the same files, its state in
    /// memory gone: as after a crash, or a restart.
    fn restarted(&mut self) {
        self.keeper = program(
            &self.record,
            self.who,
            &self.folder,
            &self.dir,
            &self.cards,
            &self.podman,
            &self.lends,
        );
    }

    /// One round at `now`.
    fn round(&mut self, now: u64) {
        assert_eq!(self.keeper.round(now).unwrap(), self.who.0);
    }

    /// `n` rounds, `every` seconds apart, from `now` on.
    fn rounds(&mut self, now: &mut u64, n: usize, every: u64) {
        for _ in 0..n {
            *now += every;
            self.round(*now);
        }
    }

    fn pod(&self) -> String {
        format!("{}-standin", self.who.0)
    }

    fn volume(&self) -> String {
        format!("{}-data", self.pod())
    }

    fn runs(&self) -> bool {
        self.podman.pods().contains(&self.pod())
    }

    /// What the stand-in's `starts` holds here now.
    fn starts(&self) -> String {
        self.podman.file(&self.volume(), "starts").unwrap_or_default()
    }

    /// A podman command run here, as a person would.
    fn podman(&self, args: &[&str]) {
        Podman::by(self.podman.clone()).run(args).unwrap();
    }

    /// This computer's copies of `app` under `under` (its own rolling
    /// ones, none), oldest first.
    fn copies(&self, app: &str, under: Option<&str>) -> Vec<PathBuf> {
        let mine = self.folder.join("give").join(app).join(self.who.0);
        let at = under.map_or(mine.clone(), |u| mine.join(u));
        let mut out: Vec<PathBuf> = std::fs::read_dir(at)
            .map(|d| d.flatten().map(|e| e.path()).collect())
            .unwrap_or_default();
        out.retain(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            !name.starts_with('.') && name != "had" && name != "stopped"
        });
        out.sort();
        out
    }

    fn snapshots(&self) -> Vec<PathBuf> {
        self.copies("standin", None)
    }

    fn had(&self) -> Vec<PathBuf> {
        self.copies("standin", Some("had"))
    }

    fn stopped(&self) -> Vec<PathBuf> {
        self.copies("standin", Some("stopped"))
    }

    /// The port its newest `share` of the stand-in lends.
    fn lent(&self) -> u16 {
        let shared = self.lends.shares().pop().expect("the stand-in lent");
        shared.split(' ').nth(1).unwrap().parse().unwrap()
    }
}

/// A keeper program for `who`, over its fakes, both stand-ins on its shelf.
fn program(
    record: &Arc<fake::Record>,
    who: Who,
    folder: &Path,
    dir: &Path,
    cards: &Path,
    podman: &Arc<fake::Podman>,
    lends: &Arc<fake::Lends>,
) -> Keeper {
    let answers = fake::Computer {
        record: record.clone(),
        who,
        folder: folder.to_path_buf(),
    };
    let shelf = Shelf::with(&[cards.join("standin"), cards.join("other")]).unwrap();
    let record = Record::by("the test's record", Arc::new(answers));
    let mut k = Keeper::new(record, shelf, format!("{}-", who.0), dir.to_path_buf(), EVERY);
    k.podman = Podman::by(podman.clone());
    k.svrngs = Svrngs::by(lends.clone());
    (k.port, k.stamp) = (fake::port, fake::stamp);
    k
}

fn standin() -> Card {
    Card::parse(STANDIN).unwrap()
}

/// The snapshot dir `rel` of `from`'s folder, copied into `to`'s as the
/// group's folder carries it: `whole`, or with its state's tar cut after
/// its first file, as a copy still arriving. Podman imports a tar cut at a
/// file's end as a whole one, exit 0, the later files missing (podman 6.0.1
/// on a Mac, 2026-10-26), so only the move's sum tells it from the copy the
/// move names.
fn carry(from: &Computer, to: &Computer, rel: &str, whole: bool) {
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

/// A file of the stand-in's state, as a copy holds it, if it holds it.
fn file_in(copy: &Path, name: &str) -> Option<String> {
    let mut tar = tar::Archive::new(std::fs::File::open(copy.join("data.tar")).unwrap());
    for e in tar.entries().unwrap() {
        let mut e = e.unwrap();
        if e.path().unwrap().file_name().is_some_and(|n| n == name) {
            let mut out = String::new();
            e.read_to_string(&mut out).unwrap();
            return Some(out);
        }
    }
    None
}

/// The stand-in's `starts`, as a copy holds it.
fn starts_in(copy: &Path) -> String {
    file_in(copy, "starts").unwrap_or_else(|| panic!("{} holds no starts", copy.display()))
}

/// `copy`, written again beside itself as a copy that does not start: its
/// state's `starts` a folder, so the stand-in's `date >> /data/starts`
/// fails and it exits. Its folder, relative to the group's.
fn broken(c: &Computer, copy: &Path) -> String {
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
    dir.strip_prefix(&c.folder).unwrap().to_string_lossy().to_string()
}

fn rel(c: &Computer, dir: &Path) -> String {
    dir.strip_prefix(&c.folder).unwrap().to_string_lossy().to_string()
}

/// The keeper says it can keep, keeps the app the record names it for and
/// not the one it names another computer for, copies its state to the
/// group each interval, each copy naming its card version, and unlends and
/// stops it, with a last copy of the state it had, kept apart under
/// `had/`, once another computer takes it over.
#[test]
fn a_keeper_keeps_only_what_the_record_names_it_for() {
    let (root, card) = (root("names"), standin());
    let record = Arc::new(fake::Record::default());
    let mut cy = Computer::new(&record, CY, &root);
    let mut now = T0;
    cy.round(now);
    assert_eq!(record.of("ready", "a1").len(), 1, "it says this computer can keep");

    // Bo's computer starts `other`: the record names Bo's, so cy's keeper
    // neither runs nor lends it.
    record.act(BO, json!({ "kind": "ready" }));
    record.act(
        BO,
        json!({ "kind": "started", "app": "other", "card": card.version, "agreed": true }),
    );
    // Cy's page starts `standin` on cy's computer.
    let started = record.act(
        CY,
        json!({ "kind": "started", "app": "standin", "card": card.version, "agreed": true }),
    );
    cy.rounds(&mut now, 1, ROUND.as_secs());
    let port = cy.lent();
    assert_eq!(
        cy.lends.asked(),
        [format!("share {port} --name standin --program standin --group g1")],
        "an HTTP door is lent as a page"
    );
    assert_eq!(
        cy.podman.published(&cy.pod()),
        Some(format!("127.0.0.1:{port}:8080/tcp")),
        "the port lent is the app's door, on loopback alone"
    );
    assert_eq!(cy.starts().lines().count(), 1, "started once: {}", cy.starts());

    // A copy each interval, naming its card version, each said.
    cy.rounds(&mut now, 1, EVERY.as_secs());
    let copy = cy.snapshots().pop().expect("a copy of standin in the group's folder");
    assert_eq!(std::fs::read_to_string(copy.join("card")).unwrap(), card.version);
    assert!(copy.join("data.tar").is_file());
    cy.rounds(&mut now, 1, EVERY.as_secs() - 1);
    assert_eq!(cy.snapshots().len(), 1, "none before the interval is up");
    cy.rounds(&mut now, KEEP, EVERY.as_secs());
    assert_eq!(record.of("copied", "a1").len(), KEEP + 1, "each copy said");
    assert_eq!(cy.snapshots().len(), KEEP, "older copies past {KEEP} removed");

    assert!(
        !cy.lends.asked().iter().any(|l| l.contains("--name other")),
        "other is kept by bo's computer, so cy's never lends it: {:?}",
        cy.lends.asked()
    );
    assert!(!cy.podman.pods().contains(&"a1-other".into()), "nor runs it");
    assert_eq!(record.of("ready", "a1").len(), 1, "this computer says it can keep once");

    // Bo's computer takes standin over: cy's unlends it, copies it once
    // more as it stops, and stops it.
    let newest = cy.snapshots().pop().unwrap();
    let (from, sum) = (rel(&cy, &newest), give::runtime::sum(&newest).unwrap());
    record.act(
        BO,
        json!({ "kind": "taken", "app": "standin", "card": card.version, "over": started, "from": from, "sum": sum, "agreed": true }),
    );
    cy.rounds(&mut now, 1, ROUND.as_secs());
    assert_eq!(cy.lends.asked().last().unwrap(), "unshare standin");
    assert!(!cy.runs(), "stopped");
    let last = cy.had().pop().expect("a last copy as it stopped, as it had it");
    assert_eq!(std::fs::read_to_string(last.join("card")).unwrap(), card.version);
    cy.rounds(&mut now, 3, ROUND.as_secs());
    assert_eq!(
        cy.lends.shares().len(),
        1,
        "nothing lent again once another computer keeps it: {:?}",
        cy.lends.asked()
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Asked to stop, the keeper stops cleanly: it unlends what it keeps, copies
/// its state to the group a last time, and stops it.
#[test]
fn a_clean_stop_copies_the_state_and_unlends() {
    let (root, card) = (root("clean"), standin());
    let record = Arc::new(fake::Record::default());
    let mut cy = Computer::new(&record, CY, &root);
    let mut now = T0;
    cy.round(now);
    record.act(
        CY,
        json!({ "kind": "started", "app": "standin", "card": card.version, "agreed": true }),
    );
    cy.rounds(&mut now, 1, ROUND.as_secs());
    cy.rounds(&mut now, 1, EVERY.as_secs());
    let before = cy.snapshots().pop().expect("a copy while it runs");

    cy.keeper.stop_all("a1");
    assert_eq!(cy.lends.asked().last().unwrap(), "unshare standin", "unlent");
    let last = cy.snapshots().pop().expect("a last copy as it stopped");
    assert!(last > before, "{last:?} after {before:?}");
    assert_eq!(std::fs::read_to_string(last.join("card")).unwrap(), card.version);
    assert_eq!(record.of("copied", "a1").len(), 2, "the last copy said");
    assert!(!cy.runs(), "stopped");
    let _ = std::fs::remove_dir_all(&root);
}

/// `give keep`, asked to stop by SIGTERM, stops cleanly and exits 0 (main.rs
/// `run_keep`): the signal's wiring, with nothing kept, so nothing is
/// reached. Its svrngs's API is at no HTTP address, so each round fails
/// before anything is dialled ("Invalid base URL").
#[test]
fn a_keeper_asked_to_stop_exits_0() {
    let root = root("sigterm");
    let data = root.join("svrngs");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("api"), "ftp://nowhere\n").unwrap();
    std::fs::write(data.join("token"), "a-token\n").unwrap();
    let mut keeper = Command::new(env!("CARGO_BIN_EXE_give"))
        .arg("keep")
        .env("SVRNGS_DIR", &data)
        .env("GIVE_DIR", root.join("give"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut out = BufReader::new(keeper.stdout.take().unwrap());
    let mut line = String::new();
    // Said once its handlers are set (main.rs), so the signal is its own.
    out.read_line(&mut line).unwrap();
    assert!(line.starts_with("Keeping what your groups give"), "{line}");
    let sent = Command::new("kill")
        .args(["-TERM", &keeper.id().to_string()])
        .status()
        .unwrap();
    assert!(sent.success());
    assert!(keeper.wait().unwrap().success(), "a keeper asked to stop exits 0");
    let mut rest = String::new();
    out.read_to_string(&mut rest).unwrap();
    assert!(rest.ends_with("Stopped.\n"), "{rest}");
    let _ = std::fs::remove_dir_all(&root);
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
    let (root, card) = (root("move"), standin());
    let record = Arc::new(fake::Record::default());
    let (mut cy, mut bo) = (Computer::new(&record, CY, &root), Computer::new(&record, BO, &root));
    let mut now = T0;
    cy.round(now);
    bo.round(now);
    assert!(!record.of("ready", "a1").is_empty() && !record.of("ready", "b1").is_empty());
    // Bo's computer is off while the app moves, and comes on with its copy
    // still arriving: the copy it meets first is torn.
    bo.keeper.stop_all("b1");

    let started = record.act(
        CY,
        json!({ "kind": "started", "app": "standin", "card": card.version, "agreed": true }),
    );
    cy.rounds(&mut now, 1, ROUND.as_secs());
    let shared = cy.lends.shares();
    let before = cy.starts();
    assert_eq!(before.lines().count(), 1, "{before}");
    // A second file in its state, as a world has many.
    cy.podman.write(&cy.volume(), "later", "later\n");

    // Bo says no: everything stays as it was.
    let offer = || {
        record.act(
            CY,
            json!({ "kind": "offered", "app": "standin", "over": started, "to": "b1" }),
        )
    };
    let o1 = offer();
    record.act(BO, json!({ "kind": "declined", "app": "standin", "offer": o1 }));
    cy.rounds(&mut now, 4, ROUND.as_secs());
    assert_eq!(
        cy.lends.asked(),
        shared,
        "a no: cy's computer still lends it, and only lends it"
    );
    assert!(bo.lends.asked().is_empty(), "a no: bo's computer lends nothing");
    assert!(record.of("moved", "a1").is_empty(), "a no: nothing moves");
    assert!(cy.runs() && cy.starts() == before, "a no: the app runs on, untouched");

    // Bo says yes.
    let o2 = offer();
    record.act(
        BO,
        json!({ "kind": "accepted", "app": "standin", "offer": o2, "agreed": true }),
    );
    cy.rounds(&mut now, 1, ROUND.as_secs());
    let moved = record.of("moved", "a1").pop().expect("cy's computer writes the move");
    assert_eq!(
        cy.lends.asked().last().unwrap(),
        "unshare standin",
        "unlent as it moved"
    );
    assert!(!cy.runs(), "cy's computer stops it, once moved");
    let from = moved["payload"]["from"].as_str().unwrap().to_string();
    let last = cy.snapshots().pop().unwrap();
    assert_eq!(cy.folder.join(&from), last, "the move names the last copy");
    assert_eq!(moved["payload"]["sum"], json!(give::runtime::sum(&last).unwrap()));
    assert_eq!(moved["payload"]["over"], json!(started));
    assert_eq!(moved["payload"]["card"], json!(card.version));

    // Bo's computer comes on with the copy torn: it waits, starting nothing.
    carry(&cy, &bo, &from, false);
    bo.restarted();
    bo.rounds(&mut now, 6, ROUND.as_secs());
    assert!(
        bo.lends.asked().is_empty(),
        "a torn copy starts nothing: {:?}",
        bo.lends.asked()
    );
    assert!(!bo.runs());
    carry(&cy, &bo, &from, true);
    bo.rounds(&mut now, 1, ROUND.as_secs());
    assert_eq!(bo.lends.shares().len(), 1, "bo's computer lends standin");
    let after = bo.starts();
    assert!(
        after.starts_with(&before) && after.lines().count() == 2,
        "the state after the move is the state before it, and one start more:\nbefore {before:?}\nafter  {after:?}"
    );
    assert_eq!(
        bo.podman.file(&bo.volume(), "later").as_deref(),
        Some("later\n"),
        "every file of it"
    );
    cy.rounds(&mut now, 2, ROUND.as_secs());
    assert_eq!(
        cy.lends.shares().len(),
        1,
        "cy's computer lends it no more: {:?}",
        cy.lends.asked()
    );
    let _ = std::fs::remove_dir_all(&root);
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
    let (root, card) = (root("gone"), standin());
    let record = Arc::new(fake::Record::default());
    let (mut cy, mut bo) = (Computer::new(&record, CY, &root), Computer::new(&record, BO, &root));
    let mut now = T0;
    cy.round(now);
    bo.round(now);
    let started = record.act(
        CY,
        json!({ "kind": "started", "app": "standin", "card": card.version, "agreed": true }),
    );
    cy.rounds(&mut now, 1, ROUND.as_secs());
    assert_eq!(cy.lends.shares().len(), 1, "cy's computer lends standin");
    // Three copies, each said in the record, the keeper's answer that it is there.
    cy.rounds(&mut now, 3, EVERY.as_secs());
    let copies = record.of("copied", "a1");
    assert_eq!(copies.len(), 3);
    for c in &copies {
        assert_eq!(c["payload"]["over"], json!(started), "{c}");
        assert_eq!(c["payload"]["card"], json!(card.version));
        let dir = cy.folder.join(c["payload"]["from"].as_str().unwrap());
        assert_eq!(
            c["payload"]["sum"].as_str().unwrap(),
            give::runtime::sum(&dir).unwrap(),
            "a copy's sum is its files'"
        );
    }

    // Cy's computer loses power: its keeper and its app stop at once.
    cy.podman(&["pod", "stop", "--time", "0", &cy.pod()]);
    let cy_starts = record.kept().next("standin").unwrap().from.clone();
    assert!(cy.folder.join(&cy_starts).is_dir());

    // Its newest copy is one that does not start.
    let bad = broken(&cy, &cy.folder.join(&cy_starts));
    let bad_sum = give::runtime::sum(&cy.folder.join(&bad)).unwrap();
    record.act(
        CY,
        json!({ "kind": "copied", "app": "standin", "card": card.version, "over": started, "from": bad, "sum": bad_sum }),
    );
    for c in cy.snapshots() {
        carry(&cy, &bo, &rel(&cy, &c), true);
    }

    // Bo keeps it there, from the newest copy: it does not start.
    let next = record.kept().next("standin").unwrap().clone();
    assert_eq!(next.from, bad, "the newest copy is the one offered");
    let t1 = record.act(
        BO,
        json!({ "kind": "taken", "app": "standin", "card": card.version, "over": started, "from": next.from, "sum": next.sum, "agreed": true }),
    );
    bo.rounds(&mut now, 1, ROUND.as_secs());
    let failed = record
        .of("failed", "b1")
        .pop()
        .expect("bo's computer says the copy did not start");
    assert_eq!(failed["payload"]["over"], json!(t1));
    assert!(bo.lends.asked().is_empty(), "a copy that did not start is not lent");
    bo.rounds(&mut now, 3, ROUND.as_secs());
    assert_eq!(
        record.of("failed", "b1").len(),
        1,
        "nor tried again: the person chooses"
    );

    // The copy before it is offered, and starts; then exits within a minute.
    let before = record.kept().next("standin").unwrap().clone();
    assert_eq!(before.from, cy_starts, "the copy before the one that did not start");
    let t2 = record.act(
        BO,
        json!({ "kind": "taken", "app": "standin", "card": card.version, "over": t1, "from": before.from, "sum": before.sum, "agreed": true }),
    );
    bo.rounds(&mut now, 1, ROUND.as_secs());
    assert_eq!(bo.lends.shares().len(), 1, "bo's computer lends standin");
    bo.podman.exit(&format!("{}-web", bo.pod()), 137);
    bo.rounds(&mut now, 1, ROUND.as_secs());
    let exited = (record.of("failed", "b1").into_iter())
        .find(|f| f["payload"]["over"] == json!(t2))
        .expect("bo's computer says it exited");
    assert_eq!(
        bo.lends.asked().last().unwrap(),
        "unshare standin",
        "an app that exited is unlent ({exited})"
    );

    // The one before that starts, and stays.
    let third = record.kept().next("standin").unwrap().clone();
    assert!(third.from < cy_starts, "{} before {cy_starts}", third.from);
    let t3 = record.act(
        BO,
        json!({ "kind": "taken", "app": "standin", "card": card.version, "over": t2, "from": third.from, "sum": third.sum, "agreed": true }),
    );
    bo.rounds(&mut now, 1, ROUND.as_secs());
    assert_eq!(bo.lends.shares().len(), 2, "bo's computer lends standin again");
    let bos = bo.starts();
    assert_eq!(record.kept().apps["standin"].act, t3);

    // Cy's computer comes back: it sees Bo's keeps it, lends nothing,
    // starts nothing, and keeps the state it had as a copy, apart.
    let shares = cy.lends.shares().len();
    cy.restarted();
    cy.rounds(&mut now, 1, ROUND.as_secs());
    let mine = (record.of("copied", "a1").into_iter())
        .find(|c| c["payload"]["from"].as_str().unwrap().contains("/had/"))
        .expect("cy's computer copies the state it had");
    assert_eq!(
        mine["payload"]["over"],
        json!(started),
        "the state as cy's keeping had it"
    );
    let cys_had = cy.had().pop().expect("in cy's folder");
    assert_eq!(cy.folder.join(mine["payload"]["from"].as_str().unwrap()), cys_had);
    let listed = (record.kept().had("standin").into_iter())
        .map(|c| c.act.clone())
        .collect::<Vec<_>>();
    assert_eq!(listed, [mine["id"].as_str().unwrap()], "listed as cy's computer had it");
    cy.rounds(&mut now, 3, ROUND.as_secs());
    assert_eq!(cy.lends.shares().len(), shares, "cy's computer lends nothing once back");
    assert!(!cy.runs(), "nor runs it");
    assert_eq!(bo.starts(), bos, "bo's state is untouched: nothing merged");

    // Starting cy's copy instead is a takeover from it: bo's computer
    // keeps its own later state as it had it, and starts from cy's.
    carry(&cy, &bo, &rel(&cy, &cys_had), true);
    let cys = record.kept().had("standin")[0].clone();
    record.act(
        BO,
        json!({ "kind": "taken", "app": "standin", "card": card.version, "over": t3, "from": cys.from, "sum": cys.sum, "agreed": true }),
    );
    bo.rounds(&mut now, 1, ROUND.as_secs());
    assert_eq!(bo.lends.shares().len(), 3, "bo's computer lends cy's copy");
    let (now_starts, cy_lines) = (bo.starts(), starts_in(&cys_had));
    assert!(
        now_starts.starts_with(&cy_lines) && now_starts.lines().count() == cy_lines.lines().count() + 1,
        "started from cy's copy, and one start more:\ncy's  {cy_lines:?}\nnow   {now_starts:?}"
    );
    assert_ne!(now_starts, bos, "not bo's state");
    assert!(!bo.had().is_empty(), "bo's later state kept as bo's had it");
    let listed = (record.kept().had("standin").into_iter())
        .map(|c| c.computer.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        listed,
        ["a1".to_string(), "b1".to_string()].into(),
        "both copies as each had it stand: nothing thrown away"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Stop running and keep a copy (docs/APP_ACCESS_AND_STOP.md B2, B3, B5,
/// B6; §5 "Final-copy ordering", "Stop means stopped", "Retention and
/// restart"). Cy's computer keeps the stand-in, and copies it while it
/// runs; then a last write lands, and Cy, from her other computer, asks it
/// stopped. Cy's computer says it began, unlends it, stops it, and keeps
/// its stopped state apart, whole, with its card's bytes, and says it
/// stopped naming that copy: the last write is in it. Nothing lends or
/// runs it after, through copies' pruning and the program's restart. Bo's
/// computer starts it again from that copy, waiting while the copy has not
/// reached it or is torn, and the state there is the stopped state, with
/// the one start Bo's gave it.
#[test]
fn a_stop_keeps_the_last_write_and_stays_stopped_until_started_again() {
    let (root, card) = (root("halt"), standin());
    let record = Arc::new(fake::Record::default());
    let (mut cy, mut bo) = (Computer::new(&record, CY, &root), Computer::new(&record, BO, &root));
    let mut now = T0;
    cy.round(now);
    bo.round(now);
    let stops = |actor| (record.of("ready", actor).last()).is_some_and(|r| r["payload"]["stops"] == true);
    assert!(stops("a1") && stops("b1"), "both computers say they can keep, and stop");
    let started = record.act(
        CY,
        json!({ "kind": "started", "app": "standin", "card": card.version, "agreed": true }),
    );
    cy.rounds(&mut now, 1, ROUND.as_secs());
    let shared = cy.lends.shares().pop().expect("cy's computer lends standin");
    cy.rounds(&mut now, 1, EVERY.as_secs());
    assert_eq!(record.of("copied", "a1").len(), 1, "a copy while it runs");
    // The last write, after the copy taken while it ran.
    cy.podman.write(&cy.volume(), "last", "last\n");

    let stop = record.act(
        CYS_OTHER,
        json!({ "kind": "stop_asked", "app": "standin", "over": started }),
    );
    cy.rounds(&mut now, 1, ROUND.as_secs());
    let stopped = record.of("stopped", "a1").pop().expect("cy's computer says it stopped");
    assert!(!cy.runs(), "stopped: no pod");
    let ops = record.ops();
    let at = |id: &Value| ops.iter().position(|o| o["id"] == *id).unwrap();
    let begun = record.of("stop_begun", "a1");
    assert_eq!(begun.len(), 1, "it said it began, once: {begun:?}");
    assert!(at(&begun[0]["id"]) < at(&stopped["id"]), "began before it stopped");
    assert_eq!(begun[0]["payload"]["stop"], json!(stop));
    assert_eq!(
        cy.lends.asked(),
        [shared.clone(), "unshare standin".to_string()],
        "unlent, and lent no more"
    );
    let p = &stopped["payload"];
    assert_eq!(
        (&p["over"], &p["stop"], &p["card"]),
        (&json!(started), &json!(stop), &json!(card.version))
    );
    let from = p["from"].as_str().unwrap().to_string();
    let copy = cy.folder.join(&from);
    assert_eq!(cy.stopped(), std::slice::from_ref(&copy), "kept apart, under stopped/");
    assert_eq!(
        p["sum"].as_str().unwrap(),
        give::runtime::sum(&copy).unwrap(),
        "whole: its sum is its files'"
    );
    assert_eq!(
        std::fs::read_to_string(copy.join("compose.yaml")).unwrap(),
        STANDIN,
        "with its card's bytes"
    );
    assert_eq!(
        file_in(&copy, "last").as_deref(),
        Some("last\n"),
        "the last write before the stop is in it"
    );
    assert_eq!(
        starts_in(&copy).lines().count(),
        1,
        "the app never started again on the way"
    );
    let state = record.kept().apps["standin"].state.clone();
    assert!(
        matches!(&state, give_fold::State::Stopped(_, c) if c.from == from),
        "{state:?}"
    );
    let copies = record.of("copied", "a1").len();

    // Intervals, copies' pruning and the program's restart leave it
    // stopped, and its copy.
    cy.rounds(&mut now, 2, EVERY.as_secs());
    cy.keeper.stop_all("a1");
    cy.restarted();
    cy.rounds(&mut now, 2, EVERY.as_secs());
    assert_eq!(
        cy.lends.asked(),
        [shared.clone(), "unshare standin".to_string()],
        "nothing lent again"
    );
    assert!(!cy.runs(), "nor started");
    assert_eq!(cy.stopped(), std::slice::from_ref(&copy), "its copy stands");
    assert_eq!(
        (record.of("stopped", "a1").len(), record.of("copied", "a1").len()),
        (1, copies),
        "nor copied again"
    );

    // Bo starts it again on Bo's computer, from that copy: missing, then
    // torn, it waits; whole, it starts from it.
    let restart = json!({ "kind": "taken", "app": "standin", "card": card.version, "over": started, "from": from, "sum": p["sum"], "agreed": true });
    let again = record.act(BO, restart);
    bo.rounds(&mut now, 2, ROUND.as_secs());
    assert!(
        bo.lends.asked().is_empty(),
        "missing bytes start nothing: {:?}",
        bo.lends.asked()
    );
    carry(&cy, &bo, &from, false);
    bo.rounds(&mut now, 2, ROUND.as_secs());
    assert!(
        bo.lends.asked().is_empty(),
        "torn bytes start nothing: {:?}",
        bo.lends.asked()
    );
    assert!(!bo.runs(), "never fresh");
    carry(&cy, &bo, &from, true);
    bo.rounds(&mut now, 1, ROUND.as_secs());
    assert_eq!(bo.lends.shares().len(), 1, "bo's computer lends standin");
    assert_eq!(
        bo.podman.file(&bo.volume(), "last").as_deref(),
        Some("last\n"),
        "started from the stopped state"
    );
    let starts = bo.starts();
    assert!(
        starts.starts_with(&starts_in(&copy)) && starts.lines().count() == 2,
        "{starts:?}"
    );
    assert_eq!(record.kept().apps["standin"].act, again);
    assert_eq!(
        cy.stopped(),
        std::slice::from_ref(&copy),
        "the stopped copy kept through the start again"
    );

    // It comes back to Cy's computer, on Cy's yes, and is copied there
    // past three times: the rolling copies' pruning leaves the stopped one.
    let o = record.act(
        BO,
        json!({ "kind": "offered", "app": "standin", "over": again, "to": "a1" }),
    );
    record.act(
        CY,
        json!({ "kind": "accepted", "app": "standin", "offer": o, "agreed": true }),
    );
    bo.rounds(&mut now, 1, ROUND.as_secs());
    let moved = record.of("moved", "b1").pop().expect("bo's computer moves it");
    let back = moved["payload"]["from"].as_str().unwrap().to_string();
    carry(&bo, &cy, &back, true);
    cy.rounds(&mut now, 1, ROUND.as_secs());
    cy.rounds(&mut now, KEEP + 1, EVERY.as_secs());
    let mine = (record.of("copied", "a1").into_iter()).filter(|c| c["payload"]["over"] == moved["id"]);
    assert_eq!(mine.count(), KEEP + 1, "copied on cy's computer, kept again");
    assert_eq!(
        cy.snapshots().len(),
        KEEP,
        "rolling copies pruned: {:?}",
        cy.snapshots()
    );
    assert_eq!(cy.stopped(), [copy], "the stopped copy stands through pruning");
    let _ = std::fs::remove_dir_all(&root);
}

/// A stop that fails says which step, and is no stop (B4; §5 "Failure
/// truth", "Recovery"). Each step is made to fail in turn, the program
/// crashing and started again between them: unlending (svrngs's command
/// fails), stopping (podman's `pod stop` fails), keeping the copy (the
/// folder can't take it) and saying it stopped (the record refuses it).
/// Each failure is said with its step, or, the record's own, traced here;
/// none is said stopped, the app's state is kept, and the app is never
/// started nor lent again. With every fault gone, the same stop finishes:
/// the copy made before the record's failure is the one said, and the app
/// never started again on the way.
#[test]
fn a_stop_that_fails_names_its_step_and_finishes_the_same_stop_after_a_crash() {
    let (root, card) = (root("halt-fail"), standin());
    let record = Arc::new(fake::Record::default());
    let mut cy = Computer::new(&record, CY, &root);
    let mut now = T0;
    cy.round(now);
    let started = record.act(
        CY,
        json!({ "kind": "started", "app": "standin", "card": card.version, "agreed": true }),
    );
    cy.rounds(&mut now, 1, ROUND.as_secs());
    let shared = cy.lends.shares().pop().expect("cy's computer lends standin");
    let failed = |step: &str| {
        (record.of("stop_failed", "a1").into_iter())
            .find(|f| f["payload"]["step"] == step)
            .unwrap_or_else(|| panic!("its {step} said failed"))
    };

    cy.lends.unshare_fails(Some("svrngs: the lend could not be withdrawn"));
    record.act(CY, json!({ "kind": "stop_asked", "app": "standin", "over": started }));
    cy.rounds(&mut now, 1, ROUND.as_secs());
    let unlend = failed("unlend");
    let why = unlend["payload"]["why"].as_str().unwrap();
    assert!(why.contains("could not be withdrawn"), "{unlend}");
    assert!(cy.runs(), "unlent nothing, so it may still run, and it does");
    cy.restarted();
    cy.lends.unshare_fails(None);

    cy.podman.fail("pod stop", "the pod could not be stopped");
    cy.rounds(&mut now, 1, ROUND.as_secs());
    let stop = failed("stop");
    assert!(
        stop["payload"]["why"]
            .as_str()
            .unwrap()
            .contains("could not be stopped"),
        "{stop}"
    );
    cy.restarted();
    cy.podman.heal();

    let into = cy.folder.join("give/standin/a1/stopped");
    std::fs::create_dir_all(into.parent().unwrap()).unwrap();
    std::fs::write(&into, "a file where the copy goes").unwrap();
    cy.rounds(&mut now, 1, ROUND.as_secs());
    failed("copy");
    assert!(!cy.runs(), "stopped before its copy failed");
    cy.restarted();
    std::fs::remove_file(&into).unwrap();

    record.refuse(&["give/stopped"]);
    now += ROUND.as_secs();
    let ((), traced) = fake::traced(|| cy.round(now));
    assert!(traced.contains("the group could not be told"), "{traced}");
    assert_eq!(cy.stopped().len(), 1, "a whole copy, and the record refusing its word");
    cy.restarted();
    let made = cy.stopped();
    assert!(record.of("stopped", "a1").is_empty(), "no false success at any step");
    assert!(cy.podman.volumes().contains(&cy.volume()), "its state is kept");

    record.refuse(&[]);
    cy.rounds(&mut now, 1, ROUND.as_secs());
    let stopped = record.of("stopped", "a1").pop().expect("the same stop finishes");
    let copy = cy.folder.join(stopped["payload"]["from"].as_str().unwrap());
    assert_eq!(
        made,
        std::slice::from_ref(&copy),
        "the copy made before the record's failure, said, not taken again"
    );
    assert_eq!(starts_in(&copy).lines().count(), 1, "never started again on the way");
    assert_eq!(cy.lends.shares(), [shared], "nor lent again");
    assert_eq!(record.of("stop_begun", "a1").len(), 1, "one stop, begun once");
    assert!(
        cy.podman.volumes().contains(&cy.volume()),
        "its state is kept once stopped"
    );
    let _ = std::fs::remove_dir_all(&root);
}
