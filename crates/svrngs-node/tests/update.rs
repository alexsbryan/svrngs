//! The home page’s update (PLAN §1, the update row): the page’s
//! `SHA256SUMS`, once found signed by a release key no earlier than the
//! last signature accepted (DESIGN §7), is the one decider of its release,
//! `release` in the data dir the one record of what this computer
//! installed and when its signature was made, and only a tarball that
//! matches the page’s sum and unpacks to a regular file replaces the
//! binary. Each refusal keeps the running bytes. The page is no socket:
//! its files are read by a `Get` (update.rs `check_by`, `apply_at`), its
//! release signed by a test's own key, and the product's own `fetch` is
//! asked only where no request can go.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::collections::HashMap;
use std::path::Path;

use ring::digest;
use ring::signature::{Ed25519KeyPair, KeyPair};
use sha2::{Digest, Sha256};
use svrngs_node::invite::ORIGIN_FILE;
use svrngs_node::update::signed::{verify, Refusal, RELEASE_KEYS};
use svrngs_node::update::{
    apply_at, check_by, fetch, packaged as packaged_words, Check, Get, Outcome, UpdateError,
    RELEASE_FILE,
};

/// The tarball’s name, as `install.sh` and `scripts/build-release.sh`
/// spell it for this computer.
fn tarball_name() -> String {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    format!("svrngs-{os}-{}.tar.gz", std::env::consts::ARCH)
}

/// `bytes` in a tarball named `name`, as the release’s is.
fn tarball(dir: &Path, name: &str, bytes: &[u8]) -> Vec<u8> {
    let src = tempfile::tempdir().unwrap();
    std::fs::write(src.path().join("svrngs"), bytes).unwrap();
    let at = dir.join(name);
    let made = std::process::Command::new("tar")
        .arg("-czf")
        .arg(&at)
        .arg("-C")
        .arg(src.path())
        .arg("svrngs")
        .status()
        .unwrap();
    assert!(made.success());
    std::fs::read(at).unwrap()
}

/// A tarball whose `svrngs` is a symlink, never a file this computer runs.
fn symlink_tarball(dir: &Path, name: &str) -> Vec<u8> {
    let src = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink("/etc/passwd", src.path().join("svrngs")).unwrap();
    let at = dir.join(name);
    let made = std::process::Command::new("tar")
        .arg("-czf")
        .arg(&at)
        .arg("-C")
        .arg(src.path())
        .arg("svrngs")
        .status()
        .unwrap();
    assert!(made.success());
    std::fs::read(at).unwrap()
}

/// `SHA256SUMS`’s line for `name`, as the release writes it.
fn sums(name: &str, tarball: &[u8]) -> Vec<u8> {
    format!("{}  {name}\n", hex::encode(Sha256::digest(tarball))).into_bytes()
}

fn exe(dir: &Path) -> std::path::PathBuf {
    dir.join("svrngs")
}

/// When the tests' releases are signed, unless one says otherwise.
const SIGNED_AT: u32 = 1_800_000_000;

/// The key the tests' pages sign their releases with, never the release
/// key, and the keys their updates are held to: that one alone.
fn page_key() -> Ed25519KeyPair {
    test_key(9)
}

fn keys() -> [[u8; 32]; 1] {
    [point(&page_key())]
}

/// A page's `SHA256SUMS`, `sums`, and its signature by [`page_key`] made at
/// `created`.
fn signed_page(sums: Vec<u8>, created: u32) -> HashMap<String, Vec<u8>> {
    let sig = Packet::at(created).sign(&sums, &page_key());
    HashMap::from([
        ("get/SHA256SUMS".to_string(), sums),
        ("get/SHA256SUMS.sig".to_string(), sig),
    ])
}

/// A page's release of `tarball` as `name`: its sums, their signature made
/// at `created`, and the tarball.
fn release(name: &str, tarball: Vec<u8>, created: u32) -> HashMap<String, Vec<u8>> {
    let mut files = signed_page(sums(name, &tarball), created);
    files.insert(format!("get/{name}"), tarball);
    files
}

#[tokio::test]
async fn an_update_already_running_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let held = std::fs::File::create(dir.path().join("update.lock")).unwrap();
    held.try_lock().unwrap();
    std::fs::write(exe(dir.path()), b"the old svrngs").unwrap();

    assert!(matches!(
        apply_at(dir.path(), &exe(dir.path()), fetch, &keys()).await,
        Err(UpdateError::Busy)
    ));
}

/// The page this computer was installed from, kept as `install.sh` keeps
/// it, at an address no test asks.
const PAGE: &str = "http://page.invalid/";

/// A page with no socket under it: `files` at their paths under [`PAGE`],
/// any other answered as [`fetch`] answers a page with no such file.
fn page(files: &HashMap<String, Vec<u8>>) -> impl Get + '_ {
    async move |url: &str, _max: usize| {
        let path = url.strip_prefix(PAGE).unwrap_or(url);
        let file = files.get(path).cloned();
        file.ok_or(UpdateError::Missing)
    }
}

/// `dir`'s computer, installed from [`PAGE`], running `the old svrngs`.
fn installed(dir: &Path) {
    std::fs::write(dir.join(ORIGIN_FILE), PAGE).unwrap();
    std::fs::write(exe(dir), b"the old svrngs").unwrap();
}

/// **A page's release is checked and its verified bytes replace the
/// binary, with no socket**: the page's `get/SHA256SUMS`, its signature and
/// tarball read by a [`Get`] (update.rs `check_by`, `apply_at`); Available,
/// Updated, the new bytes in place, the release file the tarball's hash and
/// the signature's time, then Current. Planted: the release file not
/// written after the rename (`check` then says Available); the tarball's
/// hash compared to the installed record, not the page's sum (Mismatch).
#[tokio::test]
async fn a_page_s_release_replaces_the_binary_read_with_no_socket() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"the new svrngs");
    let files = release(&name, tarball.clone(), SIGNED_AT);
    installed(dir.path());

    assert_eq!(
        check_by(dir.path(), page(&files), &keys()).await.unwrap(),
        Check::Available
    );
    let applied = apply_at(dir.path(), &exe(dir.path()), page(&files), &keys()).await;
    assert_eq!(applied.unwrap(), Outcome::Updated);
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the new svrngs");
    assert_eq!(
        std::fs::read_to_string(dir.path().join(RELEASE_FILE)).unwrap(),
        format!("{}\n{SIGNED_AT}\n", hex::encode(Sha256::digest(&tarball)))
    );
    assert_eq!(
        check_by(dir.path(), page(&files), &keys()).await.unwrap(),
        Check::Current
    );
}

/// **A tarball that is not the page's is refused, with no socket**: the
/// page's sum one digit off; Mismatch, the binary stays, no release file.
/// Planted: the hash check skipped (update.rs `apply_at`), the bytes put
/// in place; the sum's first digit not compared.
#[tokio::test]
async fn a_tarball_not_the_page_s_is_refused_read_with_no_socket() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"the new svrngs");
    let mut sums = sums(&name, &tarball);
    sums[0] = if sums[0] == b'0' { b'1' } else { b'0' };
    let mut files = signed_page(sums, SIGNED_AT);
    files.insert(format!("get/{name}"), tarball);
    installed(dir.path());

    let applied = apply_at(dir.path(), &exe(dir.path()), page(&files), &keys()).await;
    assert!(matches!(applied, Err(UpdateError::Mismatch)), "{applied:?}");
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the old svrngs");
    assert!(!dir.path().join(RELEASE_FILE).exists());
}

/// **A tarball whose svrngs is a symlink is refused, with no socket**:
/// Refused, the binary stays. Planted: the regular-file check skipped
/// (update.rs `apply_at`); the link followed (`metadata` for
/// `symlink_metadata`), its target put in place.
#[tokio::test]
async fn a_tarball_whose_svrngs_is_a_symlink_is_refused_read_with_no_socket() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = symlink_tarball(dir.path(), &name);
    let files = release(&name, tarball, SIGNED_AT);
    installed(dir.path());

    let applied = apply_at(dir.path(), &exe(dir.path()), page(&files), &keys()).await;
    assert!(
        matches!(applied, Err(UpdateError::Refused(_))),
        "{applied:?}"
    );
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the old svrngs");
}

/// **A page with no line for this computer is not judged current, with no
/// socket**: NotListed. Planted: any tarball's line taken (update.rs
/// `wanted`); no line read as a sum of zeros.
#[tokio::test]
async fn a_page_with_no_line_for_this_computer_is_not_current_read_with_no_socket() {
    let dir = tempfile::tempdir().unwrap();
    let other =
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  svrngs-other.tar.gz\n";
    let files = signed_page(other.to_vec(), SIGNED_AT);
    installed(dir.path());
    let checked = check_by(dir.path(), page(&files), &keys()).await;
    assert!(
        matches!(checked, Err(UpdateError::NotListed)),
        "{checked:?}"
    );
}

/// **A page that does not answer is not judged current, with no socket**:
/// [`fetch`] asked where no request can go says it cannot judge, by the
/// same branch a refused connection takes (update.rs `fetch`, `send`'s
/// error), and `check` over a page that cannot be asked says that, never
/// Current. Planted: `send`'s error read as an empty answer; `check`
/// reading a page it cannot ask as Current.
#[tokio::test]
async fn a_page_that_does_not_answer_is_not_current_with_no_socket() {
    let said = fetch("http://[not-an-address/get/SHA256SUMS", 1024).await;
    assert!(
        matches!(&said, Err(UpdateError::CouldNotJudge(why)) if why.starts_with("cannot reach the page svrngs came from")),
        "{said:?}"
    );
    let dir = tempfile::tempdir().unwrap();
    installed(dir.path());
    let unanswered = async |url: &str, max: usize| fetch(&format!("http://[{url}"), max).await;
    let checked = check_by(dir.path(), unanswered, &keys()).await;
    assert!(
        matches!(checked, Err(UpdateError::CouldNotJudge(_))),
        "{checked:?}"
    );
}

/// `dir/bin/svrngs`, `the old svrngs`, beside a package's `updated-by`
/// saying `said` (update.rs `UPDATED_BY`), installed from [`PAGE`].
fn packaged(dir: &Path, said: &str) -> std::path::PathBuf {
    std::fs::write(dir.join(ORIGIN_FILE), PAGE).unwrap();
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::create_dir_all(dir.join("lib/svrngs")).unwrap();
    std::fs::write(dir.join("lib/svrngs/updated-by"), said).unwrap();
    let exe = dir.join("bin/svrngs");
    std::fs::write(&exe, b"the old svrngs").unwrap();
    exe
}

/// **A svrngs a package updates never replaces itself, and says the
/// package's words** (update.rs `packaged`, `apply_at`): a page offering a
/// new release is not asked, the binary stays, no release file is kept;
/// an `updated-by` that says nothing readable is said as svrngs's own
/// words. Planted: `updated-by` not looked for (Updated, the bytes
/// replaced); its first line not read (the default said for the
/// package's).
#[tokio::test]
async fn a_svrngs_a_package_updates_never_replaces_itself() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"the new svrngs");
    let files = release(&name, tarball, SIGNED_AT);
    let exe = packaged(dir.path(), "\n  Updated with pacman.  \nignored\n");

    let applied = apply_at(dir.path(), &exe, page(&files), &keys()).await;
    assert_eq!(
        applied.unwrap(),
        Outcome::Packaged("Updated with pacman.".into())
    );
    assert_eq!(std::fs::read(&exe).unwrap(), b"the old svrngs");
    assert!(!dir.path().join(RELEASE_FILE).exists());

    std::fs::write(dir.path().join("lib/svrngs/updated-by"), "\n\n").unwrap();
    assert_eq!(
        packaged_words(&exe).as_deref(),
        Some("svrngs updates with this computer's system updates.")
    );
}

/// **A svrngs that cannot write beside itself is refused before any
/// download** (update.rs `apply_at`): its folder read-only, Refused
/// naming why, the tarball never asked of the page, the binary kept.
/// Planted: the write asked only after the download (the tarball asked).
#[tokio::test]
async fn a_svrngs_that_cannot_write_beside_itself_is_refused_before_the_download() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"the new svrngs");
    let files = release(&name, tarball, SIGNED_AT);
    installed(dir.path());
    let held = dir.path().join("held");
    std::fs::create_dir(&held).unwrap();
    let exe = held.join("svrngs");
    std::fs::write(&exe, b"the old svrngs").unwrap();
    std::fs::set_permissions(&held, std::fs::Permissions::from_mode(0o555)).unwrap();

    let asked = std::sync::Mutex::new(Vec::new());
    let watched = async |url: &str, max: usize| {
        asked.lock().unwrap().push(url.to_string());
        page(&files)(url, max).await
    };
    let applied = apply_at(dir.path(), &exe, watched, &keys()).await;
    std::fs::set_permissions(&held, std::fs::Permissions::from_mode(0o755)).unwrap();

    assert!(
        matches!(&applied, Err(UpdateError::Refused(why)) if why.contains("whoever installed svrngs where it is updates it")),
        "{applied:?}"
    );
    assert!(!asked.lock().unwrap().iter().any(|u| u.ends_with(&name)));
    assert_eq!(std::fs::read(&exe).unwrap(), b"the old svrngs");
}

/// What a page asks of the computer it updates, as `apply_at` asks it,
/// kept so a test can say the tarball was never fetched.
fn asked_of<'a>(
    files: &'a HashMap<String, Vec<u8>>,
    asked: &'a std::sync::Mutex<Vec<String>>,
) -> impl Get + 'a {
    async move |url: &str, max: usize| {
        asked.lock().unwrap().push(url.to_string());
        page(files)(url, max).await
    }
}

/// **A page serving no signature beside its sums offers nothing, and says
/// why** (update.rs `latest`): `check` and `apply` refuse Unsigned in a
/// person's words, naming the page, the file and the fix; judged, so the
/// home page draws it where the offer would be (home.rs `update_check`,
/// `held`); the tarball never asked, the binary and release file kept.
/// Planted: the signature's 404 passed on as the page's (Missing, "the page
/// answered 404 Not Found").
#[tokio::test]
async fn an_unsigned_page_offers_nothing_and_says_why() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"the new svrngs");
    let mut files = release(&name, tarball, SIGNED_AT);
    files.remove("get/SHA256SUMS.sig");
    installed(dir.path());

    let checked = check_by(dir.path(), page(&files), &keys()).await;
    let Err(refused) = checked else {
        panic!("{checked:?}")
    };
    assert!(
        matches!(&refused, UpdateError::Unsigned(at) if at == "page.invalid"),
        "{refused:?}"
    );
    assert!(refused.judged());
    assert_eq!(
        refused.to_string(),
        "No update: page.invalid offers a new svrngs without the svrngs release key’s signature (get/SHA256SUMS.sig), so it can’t be told from anyone’s. Whoever runs that page can add the signature from the release."
    );
    let asked = std::sync::Mutex::new(Vec::new());
    let applied = apply_at(
        dir.path(),
        &exe(dir.path()),
        asked_of(&files, &asked),
        &keys(),
    )
    .await;
    assert!(
        matches!(applied, Err(UpdateError::Unsigned(_))),
        "{applied:?}"
    );
    assert!(!asked.lock().unwrap().iter().any(|u| u.ends_with(&name)));
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the old svrngs");
    assert!(!dir.path().join(RELEASE_FILE).exists());
}

/// **A release signed by another key offers nothing, and says why**: a
/// page's sums signed by a key the update is not held to, BadSignature
/// with `verify`'s Key; sums changed after they were signed, Digest; both
/// judged, the binary kept. Planted: any key accepted in update/signed.rs
/// `verify` (Updated, the other key's tarball in place).
#[tokio::test]
async fn a_release_signed_by_another_key_offers_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"the new svrngs");
    let mut files = release(&name, tarball, SIGNED_AT);
    let theirs = Packet::at(SIGNED_AT).sign(&files["get/SHA256SUMS"], &test_key(5));
    files.insert("get/SHA256SUMS.sig".to_string(), theirs);
    installed(dir.path());

    let applied = apply_at(dir.path(), &exe(dir.path()), page(&files), &keys()).await;
    let Err(refused) = applied else {
        panic!("{applied:?}")
    };
    assert!(
        matches!(&refused, UpdateError::BadSignature(at, Refusal::Key) if at == "page.invalid"),
        "{refused:?}"
    );
    assert!(refused.judged());
    assert!(refused.to_string().starts_with("No update: page.invalid offers a new svrngs whose signature (get/SHA256SUMS.sig) is not the svrngs release key’s (no release key made it)"), "{refused}");
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the old svrngs");

    let mut files = release(&name, files[&format!("get/{name}")].clone(), SIGNED_AT);
    files.get_mut("get/SHA256SUMS").unwrap().push(b'\n');
    let checked = check_by(dir.path(), page(&files), &keys()).await;
    assert!(
        matches!(checked, Err(UpdateError::BadSignature(_, Refusal::Digest))),
        "{checked:?}"
    );
}

/// `dir`'s computer as an update left it: `release` keeping a hash not the
/// page's and the time of the signature it was accepted under.
fn updated_at(dir: &Path, created: u32) {
    installed(dir);
    let kept = format!("{}\n{created}\n", "a".repeat(64));
    std::fs::write(dir.join(RELEASE_FILE), kept).unwrap();
}

/// **A validly signed release older than the last one accepted is
/// refused**: signed a second before the kept time, `check` and `apply`
/// refuse Older in a person's words, judged, the binary and the kept time
/// unchanged; one signed at the kept time itself is offered. Planted: the
/// floor not compared (Updated, the older release in place).
#[tokio::test]
async fn a_signed_release_older_than_the_last_accepted_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"an older svrngs");
    let older = release(&name, tarball.clone(), SIGNED_AT - 1);
    updated_at(dir.path(), SIGNED_AT);

    let checked = check_by(dir.path(), page(&older), &keys()).await;
    assert!(
        matches!(&checked, Err(UpdateError::Older(at)) if at == "page.invalid"),
        "{checked:?}"
    );
    let applied = apply_at(dir.path(), &exe(dir.path()), page(&older), &keys()).await;
    let Err(refused) = applied else {
        panic!("{applied:?}")
    };
    assert!(refused.judged());
    assert_eq!(
        refused.to_string(),
        "No update: page.invalid offers a svrngs signed earlier than the one this computer last installed, and svrngs never goes back. Whoever runs that page can put the newest release there."
    );
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the old svrngs");
    let kept = std::fs::read_to_string(dir.path().join(RELEASE_FILE)).unwrap();
    assert!(kept.ends_with(&format!("\n{SIGNED_AT}\n")), "{kept}");

    let same = release(&name, tarball, SIGNED_AT);
    let checked = check_by(dir.path(), page(&same), &keys()).await;
    assert_eq!(checked.unwrap(), Check::Available);
}

/// **A newer signed release installs and keeps its time**: signed a second
/// after the kept time, Updated, its bytes in place, `release` its hash and
/// its signature's time; then Current. Planted: only the hash kept (the
/// time lost, the release file one line).
#[tokio::test]
async fn a_newer_signed_release_installs_and_keeps_its_time() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"the new svrngs");
    let newer = release(&name, tarball.clone(), SIGNED_AT + 1);
    updated_at(dir.path(), SIGNED_AT);

    let applied = apply_at(dir.path(), &exe(dir.path()), page(&newer), &keys()).await;
    assert_eq!(applied.unwrap(), Outcome::Updated);
    assert_eq!(std::fs::read(exe(dir.path())).unwrap(), b"the new svrngs");
    let hash = hex::encode(Sha256::digest(&tarball));
    assert_eq!(
        std::fs::read_to_string(dir.path().join(RELEASE_FILE)).unwrap(),
        format!("{hash}\n{}\n", SIGNED_AT + 1)
    );
    let checked = check_by(dir.path(), page(&newer), &keys()).await;
    assert_eq!(checked.unwrap(), Check::Current);
}

/// **`release` as `install.sh` writes it, one line, is still read**: its
/// hash the page's, Current, with no floor; a second line that is no time
/// cannot be judged, never read as none. Planted: the hash read as the
/// whole file (the hash then read as the time: CouldNotJudge, here and in
/// each test above that keeps a time); a second line that is no time read
/// as none (Current).
#[tokio::test]
async fn install_sh_s_one_line_release_is_still_read() {
    let dir = tempfile::tempdir().unwrap();
    let name = tarball_name();
    let tarball = tarball(dir.path(), &name, b"the new svrngs");
    let files = release(&name, tarball.clone(), SIGNED_AT);
    installed(dir.path());
    let hash = hex::encode(Sha256::digest(&tarball));
    std::fs::write(dir.path().join(RELEASE_FILE), &hash).unwrap();

    let checked = check_by(dir.path(), page(&files), &keys()).await;
    assert_eq!(checked.unwrap(), Check::Current);
    std::fs::write(dir.path().join(RELEASE_FILE), format!("{hash}\nsoon\n")).unwrap();
    let checked = check_by(dir.path(), page(&files), &keys()).await;
    assert!(
        matches!(checked, Err(UpdateError::CouldNotJudge(_))),
        "{checked:?}"
    );
}

// The signed-update check (update/signed.rs `verify`), against v1.11.2's
// own `SHA256SUMS` and signature, and against signatures a test makes with
// a key of its own.

/// v1.11.2's `SHA256SUMS` and `SHA256SUMS.sig`, as `gh release download
/// v1.11.2 -R alexsbryan/svrngs -p 'SHA256SUMS*'` gave them: public, and
/// signed by the release key.
const SUMS: &[u8] = include_bytes!("fixtures/v1.11.2/SHA256SUMS");
const SIG: &[u8] = include_bytes!("fixtures/v1.11.2/SHA256SUMS.sig");
/// When v1.11.2's signature was made: `gpg --list-packets`, "created".
const V1_11_2: u32 = 1_795_043_504;

/// SIG with the byte at `at` changed, after checking it is `was` there
/// (`gpg --list-packets` and `xxd` on SIG: a header of two bytes; the body's
/// algorithm at 4 and hash at 5; the creation time at 33 to 36, in the
/// hashed area; the digest's first two bytes at 77; R from 81).
fn sig_changed(at: usize, was: u8, to: u8) -> Vec<u8> {
    assert_eq!(SIG[at], was, "SIG's byte {at}");
    let mut sig = SIG.to_vec();
    sig[at] = to;
    sig
}

/// `body` as one signature packet, in the old format the release key's is.
fn framed(body: Vec<u8>) -> Vec<u8> {
    [vec![0x88, u8::try_from(body.len()).unwrap()], body].concat()
}

/// SIG with one more hashed subpacket, its counts said again.
fn sig_with_subpacket(kind: u8, said: &[u8]) -> Vec<u8> {
    let end = 8 + usize::from(u16::from_be_bytes([SIG[6], SIG[7]]));
    let mut body = SIG[2..end].to_vec();
    body.extend([said.len() as u8 + 1, kind]);
    body.extend(said);
    let hashed = u16::try_from(body.len() - 6).unwrap();
    body[4..6].copy_from_slice(&hashed.to_be_bytes());
    body.extend(&SIG[end..]);
    framed(body)
}

/// A key of the test's own, never the release key.
fn test_key(seed: u8) -> Ed25519KeyPair {
    Ed25519KeyPair::from_seed_unchecked(&[seed; 32]).unwrap()
}

fn point(key: &Ed25519KeyPair) -> [u8; 32] {
    key.public_key().as_ref().try_into().unwrap()
}

/// A signature packet made by hand, shaped as the release key's: its
/// fields, and its hashed subpackets as (type, body).
struct Packet {
    version: u8,
    kind: u8,
    algorithm: u8,
    hash: u8,
    hashed: Vec<(u8, Vec<u8>)>,
}

impl Packet {
    /// The release key's shape, made at `created`.
    fn at(created: u32) -> Self {
        Self {
            version: 4,
            kind: 0x00,
            algorithm: 22,
            hash: 10,
            hashed: vec![(33, vec![4; 21]), (2, created.to_be_bytes().to_vec())],
        }
    }

    /// The packet over `sums`, by `key`: its digest SHA-256 where it says
    /// 8, else SHA-512, whatever it says.
    fn sign(&self, sums: &[u8], key: &Ed25519KeyPair) -> Vec<u8> {
        let mut body = vec![self.version, self.kind, self.algorithm, self.hash];
        let subpackets: Vec<u8> = self
            .hashed
            .iter()
            .flat_map(|(kind, said)| [vec![said.len() as u8 + 1, *kind], said.clone()].concat())
            .collect();
        body.extend(u16::try_from(subpackets.len()).unwrap().to_be_bytes());
        body.extend(subpackets);
        let algorithm = match self.hash {
            8 => &digest::SHA256,
            _ => &digest::SHA512,
        };
        let mut digest = digest::Context::new(algorithm);
        digest.update(sums);
        digest.update(&body);
        digest.update(&[4, 0xFF]);
        digest.update(&u32::try_from(body.len()).unwrap().to_be_bytes());
        let digest = digest.finish();
        let made = key.sign(digest.as_ref());
        // Unhashed: the issuer's key ID, as gpg puts it, read by no one.
        body.extend([0, 10, 9, 16, 0, 0, 0, 0, 0, 0, 0, 0]);
        body.extend(&digest.as_ref()[..2]);
        for half in made.as_ref().chunks(32) {
            let first = half.iter().position(|&b| b != 0).unwrap();
            let bits = (32 - first) * 8 - half[first].leading_zeros() as usize;
            body.extend(u16::try_from(bits).unwrap().to_be_bytes());
            body.extend(&half[first..]);
        }
        framed(body)
    }
}

/// **v1.11.2's own sums verify against the release key**, as its signature
/// was made (old format, a length of one byte), and the same packet in the
/// old format's two-byte length and the new format's. Planted: the trailer's
/// length counted without the six bytes before the subpackets (Digest, and
/// every test key's signature refused).
#[test]
fn v1_11_2_s_sums_verify_against_the_release_key() {
    let signed = verify(SUMS, SIG, RELEASE_KEYS).unwrap();
    assert_eq!(signed.created(), V1_11_2);
    let body = &SIG[2..];
    let two = [&[0x89, 0][..], &[SIG[1]], body].concat();
    let new = [&[0xC2, SIG[1]][..], body].concat();
    assert_eq!(verify(SUMS, &two, RELEASE_KEYS), Ok(signed));
    assert_eq!(verify(SUMS, &new, RELEASE_KEYS), Ok(signed));
}

/// **The sums with one byte changed are refused**: Digest, the signature's
/// first two bytes of digest not these sums'. Planted: the digest's prefix
/// not compared (Key, never Digest).
#[test]
fn the_sums_one_byte_changed_are_refused() {
    let mut sums = SUMS.to_vec();
    sums[0] ^= 1;
    assert_eq!(verify(&sums, SIG, RELEASE_KEYS), Err(Refusal::Digest));
}

/// **The signature one byte changed is refused**: in R (Key), in the hashed
/// area's creation time (Digest), in the digest's prefix (Digest). Planted:
/// Ed25519 not asked (R changed, Ok); the digest's prefix not compared
/// (the prefix changed, Ok; the time changed, Key).
#[test]
fn the_signature_one_byte_changed_is_refused() {
    let r = sig_changed(81, 0x86, 0x87);
    assert_eq!(verify(SUMS, &r, RELEASE_KEYS), Err(Refusal::Key));
    let time = sig_changed(36, 0xb0, 0xb1);
    assert_eq!(verify(SUMS, &time, RELEASE_KEYS), Err(Refusal::Digest));
    let prefix = sig_changed(77, 0x4e, 0x4f);
    assert_eq!(verify(SUMS, &prefix, RELEASE_KEYS), Err(Refusal::Digest));
}

/// **Bytes after the signature are refused**: one byte, and a second
/// packet. Planted: what follows the packet not read (Ok).
#[test]
fn bytes_after_the_signature_are_refused() {
    let one = [SIG, &[0]].concat();
    assert_eq!(verify(SUMS, &one, RELEASE_KEYS), Err(Refusal::Trailing));
    let twice = [SIG, SIG].concat();
    assert_eq!(verify(SUMS, &twice, RELEASE_KEYS), Err(Refusal::Trailing));
}

/// **A critical subpacket svrngs does not read is refused**, in v1.11.2's
/// signature and in one a test key signed whole; the same subpacket not
/// critical is skipped. Planted: the critical bit not read (v1.11.2's,
/// Digest).
#[test]
fn a_critical_subpacket_svrngs_does_not_read_is_refused() {
    let real = sig_with_subpacket(0x80 | 99, b"!");
    assert_eq!(
        verify(SUMS, &real, RELEASE_KEYS),
        Err(Refusal::Critical(99))
    );
    let key = test_key(7);
    let mut packet = Packet::at(V1_11_2);
    packet.hashed.push((0x80 | 99, b"!".to_vec()));
    let critical = packet.sign(SUMS, &key);
    assert_eq!(
        verify(SUMS, &critical, &[point(&key)]),
        Err(Refusal::Critical(99))
    );
    packet.hashed.last_mut().unwrap().0 = 99;
    let skipped = packet.sign(SUMS, &key);
    assert!(verify(SUMS, &skipped, &[point(&key)]).is_ok());
}

/// **Another hash, key algorithm, version or kind of signature is
/// refused**: v1.11.2's with its hash said as SHA-1 or its algorithm as
/// RSA; a test key's, validly signed but saying RSA, SHA-1, version 3 or a
/// text signature. SHA-256 by the key verifies. Planted: the algorithm not
/// read (v1.11.2's RSA, Digest); any hash read as SHA-512 (v1.11.2's
/// SHA-1, Digest).
#[test]
fn another_hash_key_algorithm_version_or_kind_is_refused() {
    let sha1 = sig_changed(5, 10, 2);
    assert_eq!(verify(SUMS, &sha1, RELEASE_KEYS), Err(Refusal::Shape));
    let rsa = sig_changed(4, 22, 1);
    assert_eq!(verify(SUMS, &rsa, RELEASE_KEYS), Err(Refusal::Shape));
    let key = test_key(7);
    let keys = [point(&key)];
    let shaped = |change: fn(&mut Packet)| {
        let mut packet = Packet::at(V1_11_2);
        change(&mut packet);
        verify(SUMS, &packet.sign(SUMS, &key), &keys)
    };
    assert_eq!(shaped(|p| p.algorithm = 1), Err(Refusal::Shape));
    assert_eq!(shaped(|p| p.hash = 2), Err(Refusal::Shape));
    assert_eq!(shaped(|p| p.version = 3), Err(Refusal::Shape));
    assert_eq!(shaped(|p| p.kind = 0x01), Err(Refusal::Shape));
    assert!(shaped(|p| p.hash = 8).is_ok());
}

/// **Another key's signature is refused against the release key**, and
/// verifies against its own, alone or as a second entry. Planted: Ed25519
/// not asked (Ok against the release key; R changed, Ok).
#[test]
fn another_key_s_signature_is_refused() {
    let key = test_key(7);
    let theirs = Packet::at(V1_11_2 + 1).sign(SUMS, &key);
    assert_eq!(verify(SUMS, &theirs, RELEASE_KEYS), Err(Refusal::Key));
    let signed = verify(SUMS, &theirs, &[point(&key)]).unwrap();
    assert_eq!(signed.created(), V1_11_2 + 1);
    let rotated = [RELEASE_KEYS[0], point(&key)];
    assert_eq!(verify(SUMS, &theirs, &rotated), Ok(signed));
}

/// **A signature saying no time it was made is refused**, and so is one
/// saying two. Planted: no time read as 0 (Ok).
#[test]
fn a_signature_with_no_time_or_two_is_refused() {
    let key = test_key(7);
    let mut packet = Packet::at(V1_11_2);
    packet.hashed.retain(|(kind, _)| *kind != 2);
    let timeless = packet.sign(SUMS, &key);
    assert_eq!(
        verify(SUMS, &timeless, &[point(&key)]),
        Err(Refusal::NoTime)
    );
    let mut packet = Packet::at(V1_11_2);
    packet.hashed.push((2, 1u32.to_be_bytes().to_vec()));
    let two = packet.sign(SUMS, &key);
    assert_eq!(verify(SUMS, &two, &[point(&key)]), Err(Refusal::Packet));
}

/// **Anything but one whole signature packet is refused**: a partial
/// length, a length left to the end, a public key's tag, nothing, and the
/// packet cut short. Planted: a partial length read as its part's
/// (Trailing).
#[test]
fn anything_but_one_whole_signature_packet_is_refused() {
    let body = &SIG[2..];
    let partial = [&[0xC2, 0xE7][..], body].concat();
    assert_eq!(verify(SUMS, &partial, RELEASE_KEYS), Err(Refusal::Packet));
    let to_the_end = [&[0x8B][..], body].concat();
    assert_eq!(
        verify(SUMS, &to_the_end, RELEASE_KEYS),
        Err(Refusal::Packet)
    );
    let key_tag = [&[0x98, SIG[1]][..], body].concat();
    assert_eq!(verify(SUMS, &key_tag, RELEASE_KEYS), Err(Refusal::Packet));
    assert_eq!(verify(SUMS, &[], RELEASE_KEYS), Err(Refusal::Packet));
    assert_eq!(
        verify(SUMS, &SIG[..100], RELEASE_KEYS),
        Err(Refusal::Packet)
    );
}

/// **The pinned key is `svrngs-release-key.asc`'s**: the file's armor, its
/// base64, then its first packet, a version 4 EdDSA public key on
/// ed25519 whose point is 0x40 and 32 bytes; the first of RELEASE_KEYS is
/// those 32. Planted: one byte of update/signed.rs `RELEASE_KEY` off
/// (this, and v1.11.2's sums refused, Key).
#[test]
fn the_pinned_key_is_svrngs_release_key_asc_s() {
    use base64::Engine;
    let asc = include_str!("../../../svrngs-release-key.asc");
    let armored: String = asc
        .lines()
        .skip_while(|line| !line.is_empty())
        .skip(1)
        .take_while(|line| !line.starts_with('=') && !line.starts_with("-----"))
        .collect();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(armored)
        .unwrap();
    assert_eq!(bytes[0], 0x98, "a public key packet, its length one byte");
    let key = &bytes[2..2 + usize::from(bytes[1])];
    assert_eq!((key[0], key[5]), (4, 22), "version 4, EdDSA");
    let ed25519 = [9, 0x2B, 6, 1, 4, 1, 0xDA, 0x47, 0x0F, 1];
    assert_eq!(key[6..16], ed25519, "on ed25519");
    assert_eq!(key[16..19], [1, 7, 0x40], "an MPI of 263 bits, 0x40 first");
    assert_eq!(RELEASE_KEYS[0], key[19..51]);
    assert_eq!(key.len(), 51);
}
