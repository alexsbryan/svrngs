//! A computer gets svrngs from the group's link (DESIGN §7; v1-onboarding
//! (b)): `curl -fsSL <page>install.sh | sh -s -- '<link>'`, the line the
//! group's page gives, piped from a page host on 127.0.0.1 that serves
//! `web/install.sh` and, under `get/`, this build's `svrngs` as this
//! computer's tarball beside its `SHA256SUMS`, as `scripts/build-release.sh`
//! lays them out. Each run has a `HOME` of its own, and `launchctl` and
//! `open` are stubs on its `PATH` — the first starting and stopping
//! `svrngs` as launchd would, the second keeping what it was asked to
//! open — so nothing of this computer's own login or browser is touched.
//! Sam's computer is reached through a relay on 127.0.0.1 that the link
//! names. Refused: a tarball that does not match its sum, by name, and a
//! platform svrngs is not built for; run twice, it joins once. With
//! `--relay` (relay-verb (c)) it keeps `svrngs relay` running beside
//! svrngs on macOS, and on Linux prints the system unit, installing none;
//! a relay's agent is recorded by the stub, never run, since it would take
//! this computer's ports 80 and 443. Run where `$SHELL` names zsh or bash,
//! it puts `~/.local/bin` on that shell's PATH, once. Where the claim is
//! install.sh's own, the line runs with no listener: a `curl` stub answers
//! the page's files, and svrngs is a stand-in that starts no node ([`line`]).

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
mod common;

#[path = "common/made.rs"]
mod made;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;

use svrngs::group::Name;
use svrngs_node::invite::ORIGIN_FILE;
use svrngs_node::link::Origin;
use svrngs_node::update::RELEASE_FILE;
use svrngs_node::RailAct;
// Only the macOS tests' (a_svrngs_started_by_hand_…): unused, and so
// clippy's error, where they are not built.
#[cfg(target_os = "macos")]
use tokio::io::AsyncBufReadExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// This computer's tarball, as install.sh names it from `uname`.
fn tarball() -> String {
    tarball_of("svrngs")
}

// Rehearsals can hold the downloaded payload to the same immutable product
// artifact as their browser world, instead of Cargo's shared profile pathname.
// Otherwise svrngs is as_main.rs's, this binary meeting at a relay on
// 127.0.0.1, so nothing install.sh starts reaches n0: a program named
// `svrngs`, made once for the binary, which the tarball carries.
fn binary() -> PathBuf {
    static MADE: std::sync::OnceLock<(tempfile::TempDir, PathBuf)> = std::sync::OnceLock::new();
    std::env::var_os("SCENES_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let (_, svrngs) = MADE.get_or_init(|| {
                let dir = tempfile::tempdir().unwrap();
                let svrngs = as_main::installed(dir.path());
                (dir, svrngs)
            });
            svrngs.clone()
        })
}

#[path = "common/as_main.rs"]
mod as_main;

/// svrngs itself, as install.sh places and starts it (common/as_main.rs).
#[test]
#[ignore = "run by install.sh's svrngs, in a process of its own, as svrngs"]
fn svrngs_here() {
    as_main::main()
}

/// `program`'s tarball for this computer: svrngs's, or Give's program's.
fn tarball_of(program: &str) -> String {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    format!("{program}-{os}-{}.tar.gz", std::env::consts::ARCH)
}

/// A page host: `site`'s files at their paths on 127.0.0.1, 404 else.
async fn host(site: PathBuf) -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut sock, _)) = listener.accept().await {
            let site = site.clone();
            tokio::spawn(async move {
                let mut head = vec![0u8; 8192];
                let n = sock.read(&mut head).await.unwrap_or(0);
                let head = String::from_utf8_lossy(&head[..n]);
                let path = head.split(' ').nth(1).unwrap_or("/");
                let path = path
                    .split(['?', '#'])
                    .next()
                    .unwrap()
                    .trim_start_matches('/');
                let file = (!path.contains("..")).then(|| site.join(path));
                let reply = match file.and_then(|f| std::fs::read(f).ok()) {
                    Some(body) => {
                        let mut r = format!(
                            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                            body.len()
                        )
                        .into_bytes();
                        r.extend(body);
                        r
                    }
                    None => {
                        b"HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
                            .to_vec()
                    }
                };
                let _ = sock.write_all(&reply).await;
            });
        }
    });
    port
}

/// The site: install.sh at its root, and under get/ this build's svrngs
/// with a stand-in for its keeper, plus a stand-in for Give, beside
/// `sums`, or their true SHA256SUMS. The stand-in is a script that says it
/// ran: install.sh only fetches, checks, places and keeps it, and Give's
/// own tests run the program itself (apps/give/tests/).
fn site(sums: Option<&str>) -> tempfile::TempDir {
    site_of(None, sums)
}

/// [`site`], its svrngs `stand_in`, a script by that name, where one is
/// given, else this build's.
fn site_of(stand_in: Option<&str>, sums: Option<&str>) -> tempfile::TempDir {
    let site = tempfile::tempdir().unwrap();
    std::fs::copy(
        harness::repo().join("web/install.sh"),
        site.path().join("install.sh"),
    )
    .unwrap();
    let get = site.path().join("get");
    std::fs::create_dir(&get).unwrap();
    std::fs::copy(site.path().join("install.sh"), get.join("install.sh")).unwrap();
    for name in ["SHA256SUMS.sig", "svrngs-release-key.asc"] {
        std::fs::write(get.join(name), "fixture").unwrap();
    }
    let bin = match stand_in {
        None => binary(),
        Some(script) => {
            let bin = site.path().join("svrngs");
            std::fs::write(&bin, script).unwrap();
            let exec = std::os::unix::fs::PermissionsExt::from_mode(0o755);
            std::fs::set_permissions(&bin, exec).unwrap();
            bin
        }
    };
    std::fs::write(site.path().join("svrngs-keeper"), "#!/bin/sh\nexit 0\n").unwrap();
    let made = std::process::Command::new("tar")
        .arg("-czf")
        .arg(get.join(tarball()))
        .arg("-C")
        .arg(bin.parent().unwrap())
        .arg("svrngs")
        .arg("-C")
        .arg(site.path())
        .arg("svrngs-keeper")
        .status()
        .unwrap();
    assert!(made.success());
    let stand_in = site.path().join("give");
    std::fs::write(&stand_in, "#!/bin/sh\necho \"give $*\"\n").unwrap();
    let exec = std::os::unix::fs::PermissionsExt::from_mode(0o755);
    std::fs::set_permissions(&stand_in, exec).unwrap();
    let made = std::process::Command::new("tar")
        .arg("-czf")
        .arg(get.join(tarball_of("give")))
        .arg("-C")
        .arg(site.path())
        .arg("give")
        .status()
        .unwrap();
    assert!(made.success());
    // Summed as install.sh sums (web/install.sh:92): sha256sum, else
    // shasum, which a Mac has and a Linux computer may not (Fedora's is
    // perl-Digest-SHA's). Both write `<sum>  <file>`.
    let tarballs = [tarball(), tarball_of("give"), "install.sh".into()];
    let summed = std::process::Command::new("sha256sum")
        .args(&tarballs)
        .current_dir(&get)
        .output()
        .or_else(|_| {
            std::process::Command::new("shasum")
                .args(["-a", "256"])
                .args(&tarballs)
                .current_dir(&get)
                .output()
        })
        .unwrap();
    assert!(summed.status.success(), "the tarballs were not summed");
    let truth = String::from_utf8(summed.stdout).unwrap();
    let published = sums.map_or_else(
        || truth.clone(),
        |lines| format!("{lines}{}  install.sh\n", sum_of(&get.join("install.sh"))),
    );
    std::fs::write(get.join("SHA256SUMS"), published).unwrap();
    site
}

/// The stubs: `launchctl` runs the svrngs its agent names as launchd
/// would, and stops it as launchd does: `bootout` returns at once, and until
/// the service is gone `print` still finds it and `bootstrap` fails with 5
/// (5 of 5 on real launchd, 2026-11-06). `open` keeps the address; `uname`, when `uname`
/// is given, names another platform; and `sh` is dash where this computer
/// has it, Debian's `/bin/sh`, which holds install.sh to POSIX.
fn stubs(home: &Path, uname: Option<&str>) -> PathBuf {
    let stubs = home.join("stubs");
    std::fs::create_dir_all(&stubs).unwrap();
    let launchctl = r#"#!/bin/sh
echo "launchctl $*" >>"$HOME/stubs/said"
case "$1" in
  bootstrap)
    if [ -e "$HOME/stubs/leaving" ]; then echo "Bootstrap failed: 5: Input/output error" >&2; exit 5; fi
    grep -q '<string>relay</string>\|<string>keep</string>' "$3" && exit 0
    run=$(sed -n 's:.*<array><string>\(.*\)</string></array>.*:\1:p' "$3")
    dir=$(sed -n 's:.*<key>SVRNGS_DIR</key><string>\(.*\)</string>.*:\1:p' "$3")
    SVRNGS_DIR="$dir" "$run" >>"$dir/svrngs.log" 2>&1 &
    echo $! >"$HOME/stubs/pid" ;;
  bootout)
    case "$2" in */sh.svrnme.give) exit 0 ;; esac
    [ -f "$HOME/stubs/pid" ] || exit 3
    pid=$(cat "$HOME/stubs/pid"); rm "$HOME/stubs/pid"; kill "$pid"
    touch "$HOME/stubs/leaving"
    (while kill -0 "$pid" 2>/dev/null; do sleep 0.1; done; sleep 0.3; rm -f "$HOME/stubs/leaving") >/dev/null 2>&1 & ;;
  print) [ -e "$HOME/stubs/leaving" ] ;;
  kickstart) [ -f "$HOME/stubs/pid" ] ;;
esac
"#;
    let open = "#!/bin/sh\necho \"$*\" >>\"$HOME/stubs/opened\"\n";
    // As systemd --user would: its unit's ExecStart run on `enable --now`,
    // stopped on `stop`, active while it runs (start-group (d)).
    let systemctl = r#"#!/bin/sh
echo "systemctl $*" >>"$HOME/stubs/said"
unit="$HOME/.config/systemd/user/svrngs.service"
case "$*" in
  "--user enable --now svrngs.service")
    run=$(sed -n 's/^ExecStart=//p' "$unit")
    dir=$(sed -n 's/^Environment=SVRNGS_DIR=//p' "$unit")
    SVRNGS_DIR="$dir" $run >>"$dir/svrngs.log" 2>&1 &
    echo $! >"$HOME/stubs/pid" ;;
  "--user is-active --quiet svrngs.service") [ -f "$HOME/stubs/pid" ] ;;
  "--user restart svrngs.service") [ -f "$HOME/stubs/pid" ] ;;
  "--user stop svrngs.service")
    pid=$(cat "$HOME/stubs/pid"); rm "$HOME/stubs/pid"; kill "$pid"
    while kill -0 "$pid" 2>/dev/null; do sleep 0.1; done ;;
esac
"#;
    let mut files = vec![
        ("launchctl", launchctl.to_string()),
        ("open", open.into()),
        ("xdg-open", open.into()),
        ("systemctl", systemctl.into()),
    ];
    if let Some(u) = uname {
        let (s, m) = u.split_once(' ').unwrap();
        files.push((
            "uname",
            format!("#!/bin/sh\ncase \"$1\" in -m) echo {m} ;; *) echo {s} ;; esac\n"),
        ));
    }
    for (name, body) in files {
        let f = stubs.join(name);
        std::fs::write(&f, body).unwrap();
        let exec = std::os::unix::fs::PermissionsExt::from_mode(0o755);
        std::fs::set_permissions(&f, exec).unwrap();
    }
    if Path::new("/bin/dash").exists() && !stubs.join("sh").exists() {
        std::os::unix::fs::symlink("/bin/dash", stubs.join("sh")).unwrap();
    }
    stubs
}

/// The page's line, as a person pastes it, run in `home`.
async fn paste(port: u16, link: &str, home: &Path, uname: Option<&str>) -> Output {
    paste_more(port, link, "", home, uname).await
}

/// The line with `more` after the link.
async fn paste_more(port: u16, link: &str, more: &str, home: &Path, uname: Option<&str>) -> Output {
    let line = format!("curl -fsSL http://127.0.0.1:{port}/install.sh | sh -s -- '{link}'{more}");
    let stubs = stubs(home, uname);
    publisher_key(&stubs, true);
    executable(
        &stubs.join("curl"),
        &format!(
            r#"#!/bin/sh
n=$#
while [ "$n" -gt 0 ]; do
  case "$1" in
    --proto|--proto-redir) shift 2; n=$((n - 2)); continue ;;
    {PUBLISHER}*) set -- "$@" "http://127.0.0.1:{port}/get/${{1#{PUBLISHER}}}" ;;
    *) set -- "$@" "$1" ;;
  esac
  shift; n=$((n - 1))
done
exec /usr/bin/curl "$@"
"#
        ),
    );
    let path = format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", stubs.display());
    let mut sh = tokio::process::Command::new("/bin/sh");
    sh.args(["-c", &line])
        .env_clear()
        .env("HOME", home)
        .env("PATH", path)
        .env("USER", "Alex");
    let out = sh.output();
    let out = tokio::time::timeout(Duration::from_secs(120), out)
        .await
        .expect("the line ends within two minutes")
        .unwrap();
    eprintln!(
        "--- the line, in {}:\n{}{}",
        home.display(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

/// svrngs as the stub launchd started it, stopped when the test ends.
struct Started(PathBuf);
impl Drop for Started {
    fn drop(&mut self) {
        if let Ok(pid) = std::fs::read_to_string(self.0.join("stubs/pid")) {
            let _ = std::process::Command::new("kill").arg(pid.trim()).status();
        }
    }
}

/// Planted failure: a refused join after the installer has stopped an
/// existing installation. Both service-manager paths must restart it.
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_join_restarts_the_existing_installation() {
    let site = site(None);
    as_linux(site.path());
    let port = host(site.path().to_path_buf()).await;
    let page = format!("http://127.0.0.1:{port}/");
    let mac = cfg!(target_os = "macos").then_some(None);
    for uname in mac.into_iter().chain([Some("Linux x86_64")]) {
        let home = tempfile::tempdir().unwrap();
        let _started = Started(home.path().to_path_buf());
        assert!(paste(port, &page, home.path(), uname)
            .await
            .status
            .success());
        let key = std::fs::read(home.path().join(".svrngs/identity")).unwrap();
        let refused = paste(port, &format!("{page}#not-a-link"), home.path(), uname).await;
        assert!(
            !refused.status.success(),
            "the planted invalid link is refused"
        );
        let launched = std::fs::read_to_string(home.path().join("stubs/said")).unwrap();
        let start = match uname {
            None => "launchctl bootstrap",
            Some(_) => "systemctl --user enable --now svrngs.service",
        };
        assert_eq!(
            launched.matches(start).count(),
            2,
            "restarted after refusal: {launched}"
        );
        assert!(
            home.path().join("stubs/pid").exists(),
            "no service left stopped"
        );
        assert_eq!(
            std::fs::read(home.path().join(".svrngs/identity")).unwrap(),
            key
        );
    }
}

#[cfg(target_os = "macos")]
/// A svrngs started by hand, which no launchd or systemd can stop, holds
/// the data a join needs; older than the join route (2020f28e), it also
/// cannot join for the line. The line stops it, and keeps svrngs at login in
/// its place. Watched failing: without the stop it ran on, unstopped.
#[tokio::test(flavor = "multi_thread")]
async fn a_svrngs_started_by_hand_is_stopped_by_the_line() {
    let site = site(None);
    let port = host(site.path().to_path_buf()).await;
    let page = format!("http://127.0.0.1:{port}/");
    let home = tempfile::tempdir().unwrap();
    let _started = Started(home.path().to_path_buf());
    assert!(paste(port, &page, home.path(), None).await.status.success());
    let kept = match cfg!(target_os = "macos") {
        true => "Library/LaunchAgents/sh.svrnme.svrngs.plist",
        false => ".config/systemd/user/svrngs.service",
    };
    let pid = std::fs::read_to_string(home.path().join("stubs/pid")).unwrap();
    let stopped = std::process::Command::new("kill").arg(pid.trim()).status();
    assert!(stopped.unwrap().success());
    let gone = format!(
        "while kill -0 {} 2>/dev/null; do sleep 0.1; done",
        pid.trim()
    );
    let waited = std::process::Command::new("/bin/sh")
        .args(["-c", &gone])
        .status();
    assert!(waited.unwrap().success());
    std::fs::remove_file(home.path().join("stubs/pid")).unwrap();
    std::fs::remove_file(home.path().join(kept)).unwrap();

    let mut by_hand = tokio::process::Command::new(home.path().join(".local/bin/svrngs"))
        .env_clear()
        .env("HOME", home.path())
        .env("USER", "Alex")
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut said = tokio::io::BufReader::new(by_hand.stdout.take().unwrap()).lines();
    let up = async {
        while let Some(line) = said.next_line().await.unwrap() {
            if line.contains("Leave this running") {
                return;
            }
        }
        panic!("svrngs ended before it ran");
    };
    tokio::time::timeout(Duration::from_secs(30), up)
        .await
        .expect("svrngs runs by hand");

    let line = paste(port, &page, home.path(), None).await;
    assert!(line.status.success());
    let out = String::from_utf8_lossy(&line.stdout);
    assert!(
        out.contains("Stopped the svrngs running here"),
        "the stop is said: {out}"
    );
    tokio::time::timeout(Duration::from_secs(10), by_hand.wait())
        .await
        .expect("the svrngs started by hand stopped")
        .unwrap();
    assert!(
        home.path().join("stubs/pid").exists(),
        "svrngs is kept at login in its place"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn native_arrival_retries_one_identity_and_opens_the_exact_group() {
    let site = site(None);
    let port = host(site.path().to_path_buf()).await;
    // A Sam of its own key: with the_line_run_twice_…'s, also 1, both
    // answered a relay their joiners shared, which handed one's ask to the
    // other, refused NoInvite (2 of 12 runs once both met at one relay on
    // 127.0.0.1, as_main.rs; the issuer's traces, redeem.rs `serve_ask`).
    let (_relay, _, _dir, sam) = common::sams_computer(3).await;
    let page = Origin::parse(&format!("http://127.0.0.1:{port}/")).unwrap();
    let (seed, printed) = made::new_group(
        &sam,
        Name::parse("house").unwrap(),
        Name::parse("Sam").unwrap(),
        Some(&page),
        &[],
    )
    .await
    .unwrap();
    let link = printed.https.unwrap();
    let home = tempfile::tempdir().unwrap();
    let _started = Started(home.path().to_path_buf());
    let destination = format!("/#group-{}", seed.id());
    let more = format!(" --at '{destination}' --as 'Bo O'\\''Brien'");
    let mut identity = None;
    for _ in 0..2 {
        let out = paste_more(port, &link, &more, home.path(), None).await;
        assert!(
            out.status.success(),
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let key = std::fs::read(home.path().join(".svrngs/identity")).unwrap();
        assert_eq!(identity.get_or_insert(key.clone()), &key);
        let opened = std::fs::read_to_string(home.path().join("stubs/opened")).unwrap();
        let url = opened.lines().last().unwrap();
        let url = reqwest::Url::parse(url).unwrap();
        let reply: serde_json::Value = reqwest::Client::new()
            .post(format!(
                "http://127.0.0.1:{}{}?{}",
                url.port().unwrap(),
                url.path(),
                url.query().unwrap()
            ))
            .header("Host", format!("svrngs.localhost:{}", url.port().unwrap()))
            .header("Origin", url.origin().ascii_serialization())
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(
            reply["at"], destination,
            "the bootstrap redeems to the group, not home"
        );
        assert_eq!(
            std::fs::read_to_string(home.path().join(".svrngs/origin")).unwrap(),
            page.as_str()
        );
    }
    let file = sam
        .rail()
        .group_dir(&seed.id())
        .unwrap()
        .join("ring_oplog.jsonl");
    let admits = std::fs::read_to_string(file).unwrap().lines()
        .filter_map(|line| serde_json::from_str::<svrngs_node::Op<svrngs_node::SignedOp>>(line).ok())
        .filter(|op| matches!(&op.kind.act, RailAct::Admit { person, .. } if person.as_str() == "Bo O'Brien"))
        .count();
    assert_eq!(admits, 1);
    sam.endpoint().close().await;
}

#[cfg(target_os = "macos")]
#[tokio::test(flavor = "multi_thread")]
async fn the_line_run_twice_installs_runs_at_login_opens_home_and_joins_once() {
    let site = site(None);
    let port = host(site.path().to_path_buf()).await;
    let (_relay, _, _dir, sam) = common::sams_computer(1).await;
    let page = Origin::parse(&format!("http://127.0.0.1:{port}/")).unwrap();
    let (seed, printed) = made::new_group(
        &sam,
        Name::parse("house").unwrap(),
        Name::parse("Sam").unwrap(),
        Some(&page),
        &[],
    )
    .await
    .expect("Sam makes house");
    let link = printed.https.expect("a link on the page's address");
    let home = tempfile::tempdir().unwrap();
    let _started = Started(home.path().to_path_buf());

    let first = paste(port, &link, home.path(), None).await;
    let said = String::from_utf8_lossy(&first.stdout);
    assert!(
        first.status.success(),
        "{said}{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let bin = home.path().join(".local/bin");
    for line in [
        format!(
            "Checked: {} is the one the release lists, unchanged on its way here.",
            tarball()
        ),
        format!("Installed svrngs in {}.", bin.display()),
        format!(
            "To run svrngs yourself, type {}/svrngs: your terminal does not look in {}.",
            bin.display(),
            bin.display()
        ),
        "You're in house, as Alex.".to_string(),
        "svrngs runs now, and whenever you log in".to_string(),
    ] {
        assert!(said.contains(&line), "not said: {line}\n{said}");
    }
    let agent = home
        .path()
        .join("Library/LaunchAgents/sh.svrnme.svrngs.plist");
    let plist = std::fs::read_to_string(&agent).expect("a LaunchAgent written");
    assert!(plist.contains(&format!("<string>{}/svrngs</string>", bin.display())));
    let api = std::fs::read_to_string(home.path().join(".svrngs/api")).unwrap();
    let port_home = api.trim().rsplit(':').next().unwrap().to_string();
    let opened = std::fs::read_to_string(home.path().join("stubs/opened")).unwrap();
    let address = format!("http://svrngs.localhost:{port_home}/");
    assert!(
        opened.starts_with(&format!("{address}svrngs-open?code=")),
        "{opened}"
    );
    // As the command and the home page say it, and the same the second time
    // (start-address (b)).
    let address = format!(
        "Your home page, the same address each time svrngs starts: http://svrngs.localhost:{port_home}/"
    );
    assert!(said.contains(&address), "not said: {address}\n{said}");

    let again = paste(port, &link, home.path(), None).await;
    let said = String::from_utf8_lossy(&again.stdout);
    assert!(
        again.status.success(),
        "{said}{}",
        String::from_utf8_lossy(&again.stderr)
    );
    for line in [
        "Stopped svrngs, to update it.".to_string(),
        format!("Updated svrngs in {}.", bin.display()),
        "You're in house already, on this computer.".to_string(),
        address,
    ] {
        assert!(said.contains(&line), "not said again: {line}\n{said}");
    }
    let launched = std::fs::read_to_string(home.path().join("stubs/said")).unwrap();
    assert_eq!(
        launched.matches("launchctl bootstrap").count(),
        2,
        "{launched}"
    );

    let id = seed.id();
    let file = sam.rail().group_dir(&id).unwrap().join("ring_oplog.jsonl");
    let admits = std::fs::read_to_string(file)
        .unwrap()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<svrngs_node::Op<svrngs_node::SignedOp>>(l).unwrap())
        .filter(
            |op| matches!(&op.kind.act, RailAct::Admit { person, .. } if person.as_str() == "Alex"),
        )
        .count();
    assert_eq!(admits, 1, "run twice, the line joined {admits} times");
    sam.endpoint().close().await;
}

/// This build's binary under Linux's name too, beside this computer's, for
/// a run whose `uname` says Linux: the Linux path is run with it here.
fn as_linux(site: &Path) {
    // On Linux x86_64 this computer's tarball is that one, summed by `site`.
    if tarball() == "svrngs-linux-x86_64.tar.gz" {
        return;
    }
    let get = site.join("get");
    let mut summed = vec!["-a".to_string(), "256".into(), "install.sh".into()];
    for program in ["svrngs", "give"] {
        let linux = format!("{program}-linux-x86_64.tar.gz");
        std::fs::copy(get.join(tarball_of(program)), get.join(&linux)).unwrap();
        summed.extend([tarball_of(program), linux]);
    }
    let sums = std::process::Command::new("shasum")
        .args(&summed)
        .current_dir(&get)
        .output()
        .unwrap();
    std::fs::write(get.join("SHA256SUMS"), sums.stdout).unwrap();
}

/// **A founder's one line** (start-group (d)): someone with no link yet
/// pastes the line the group's page gives with no link, the page's address
/// in the link's place, `curl -fsSL <page>install.sh | sh -s -- '<page>'`.
/// It installs svrngs, joins nothing, keeps it running at login — on
/// macOS by the stand-in for launchctl, on Linux by the one for systemctl,
/// each starting it as the system would — keeps the page svrngs came from
/// in the data dir, where a group made on the home page keeps it, and
/// names and opens the home page, where "Make a group" is, with a session,
/// and puts the opener in place ([`the_opener_opens_the_home_page`]). Planted,
/// watched failing: install.sh as a136f60 left it, joining the page's
/// address as a link and refusing.
#[tokio::test(flavor = "multi_thread")]
async fn a_founder_s_line_installs_keeps_svrngs_at_login_and_opens_the_home_page() {
    let site = site(None);
    as_linux(site.path());
    let port = host(site.path().to_path_buf()).await;
    let page = format!("http://127.0.0.1:{port}/");
    let mac = cfg!(target_os = "macos").then_some(None);
    for uname in mac.into_iter().chain([Some("Linux x86_64")]) {
        let home = tempfile::tempdir().unwrap();
        let _started = Started(home.path().to_path_buf());
        let out = paste(port, &page, home.path(), uname).await;
        let said = String::from_utf8_lossy(&out.stdout);
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "{uname:?}: {said}{err}");
        let bin = home.path().join(".local/bin");
        assert!(
            said.contains(&format!("Installed svrngs in {}.", bin.display())),
            "{said}"
        );
        assert!(
            !said.contains("You're in"),
            "a founder joins nothing: {said}"
        );
        assert!(
            said.contains("svrngs runs now, and whenever you log in"),
            "{said}"
        );
        let launched = std::fs::read_to_string(home.path().join("stubs/said")).unwrap();
        let kept = match uname {
            None => "launchctl bootstrap",
            Some(_) => "systemctl --user enable --now svrngs.service",
        };
        assert_eq!(launched.matches(kept).count(), 1, "{launched}");
        let data = home.path().join(".svrngs");
        let api = std::fs::read_to_string(data.join("api")).unwrap();
        let address = format!(
            "http://svrngs.localhost:{}/",
            api.trim().rsplit(':').next().unwrap()
        );
        let named = format!("Your home page, the same address each time svrngs starts: {address}");
        assert!(said.contains(&named), "{said}");
        the_opener_opens_the_home_page(home.path(), uname.is_some(), &address, &said).await;
        // Where the node reads it for a group made on the home page:
        // install.sh spells the file in shell, so it is pinned here.
        let origin = std::fs::read_to_string(data.join(ORIGIN_FILE)).unwrap();
        assert_eq!(origin, page, "the page svrngs came from, kept");
        // What the home page’s update reads: the hash install.sh checked.
        let sums = std::fs::read_to_string(site.path().join("get/SHA256SUMS")).unwrap();
        let name = tarball();
        let want = sums
            .lines()
            .find(|line| line.ends_with(&name))
            .and_then(|line| line.split_whitespace().next())
            .unwrap();
        let release = std::fs::read_to_string(data.join(RELEASE_FILE)).unwrap();
        assert_eq!(release, want, "what this computer installed, kept");
    }
}

/// What svrngs answers the open `url` redeemed as a browser's bootstrap
/// redeems it, by a POST from its own origin (home/session.js): its status,
/// and whether it gave the origin its key (session.rs `redeem`).
async fn redeemed(url: &str) -> (u16, bool) {
    let (host, path) = url
        .strip_prefix("http://")
        .unwrap()
        .split_once('/')
        .unwrap();
    let port = host.rsplit(':').next().unwrap();
    let mut sock = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    let ask = format!("POST /{path} HTTP/1.1\r\nHost: {host}\r\nOrigin: http://{host}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    sock.write_all(ask.as_bytes()).await.unwrap();
    let mut reply = String::new();
    sock.read_to_string(&mut reply).await.unwrap();
    let set = reply.contains("\"key\":");
    (reply[9..12].parse().unwrap(), set)
}

/// **install.sh opens the home page with a session, and puts in place an
/// opener that does again** (give-session): what it opened is an open's
/// address, whose code, redeemed, sets the home page's session once. The
/// opener — an app in `~/Applications` on macOS, a `.desktop` entry in the
/// applications menu on Linux — runs `svrngs open`, which opens another,
/// and the line says where it is in the person's words. Run with the
/// stubs on `PATH`, as install.sh ran, so no browser of this computer's is
/// opened. Red before the opener: no app, no entry, and the home page
/// opened at its bare address, which a browser with no session is refused.
async fn the_opener_opens_the_home_page(home: &Path, linux: bool, address: &str, said: &str) {
    let opened = || std::fs::read_to_string(home.join("stubs/opened")).unwrap();
    let open = |n: usize| {
        opened()
            .lines()
            .nth(n)
            .map(String::from)
            .unwrap_or_default()
    };
    let first = open(0);
    assert!(
        first.starts_with(&format!("{address}svrngs-open?code=")),
        "{}",
        opened()
    );
    assert_eq!(
        redeemed(&first).await,
        (200, true),
        "install.sh's open sets the session"
    );
    assert_ne!(redeemed(&first).await.0, 200, "a replayed open is refused");
    let (run, where_) = if linux {
        let entry = home.join(".local/share/applications/svrngs.desktop");
        let entry = std::fs::read_to_string(&entry).expect("a .desktop entry");
        let exec = entry.lines().find_map(|l| l.strip_prefix("Exec="));
        (exec.expect("an Exec line").to_string(), "applications menu")
    } else {
        let app = home.join("Applications/svrngs.app/Contents/MacOS/svrngs");
        (format!("'{}'", app.display()), "Applications")
    };
    let words = format!("To open it again, open svrngs from your {where_}.");
    assert!(said.contains(&words), "not said: {words}\n{said}");
    let path = format!(
        "{}:/usr/bin:/bin:/usr/sbin:/sbin",
        home.join("stubs").display()
    );
    let ran = tokio::process::Command::new("/bin/sh")
        .args(["-c", &run])
        .env_clear()
        .env("HOME", home)
        .env("PATH", path)
        .output()
        .await
        .unwrap();
    assert!(ran.status.success(), "the opener ran: {ran:?}");
    let second = open(1);
    assert!(
        second.starts_with(&format!("{address}svrngs-open?code=")),
        "{}",
        opened()
    );
    assert_eq!(
        redeemed(&second).await,
        (200, true),
        "the opener's open sets the session"
    );
}

/// The founder has the group's folder from `new` (bash-first-minute (d);
/// Demo C's dry run, finding 7): `svrngs new trip`, this build's, in a
/// `HOME` of its own, says where the folder is and makes it at
/// `~/svrngs/trip`, where `join` puts everyone's; and `/v1/groups` — what
/// the home page says it from — names the place, so a photo put there is
/// shared with no `share --folder`. Watched failing: `new` before it
/// placed the folder.
#[tokio::test(flavor = "multi_thread")]
async fn new_places_the_groups_folder() {
    use tokio::io::AsyncBufReadExt;
    let home = tempfile::tempdir().unwrap();
    let mut new = tokio::process::Command::new(binary())
        .args(["new", "trip", "--as", "Sam"])
        .env_clear()
        .env("HOME", home.path())
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut out = tokio::io::BufReader::new(new.stdout.take().unwrap()).lines();
    let mut said = String::new();
    let printed = tokio::time::timeout(std::time::Duration::from_secs(60), async {
        while let Ok(Some(line)) = out.next_line().await {
            said.push_str(&line);
            said.push('\n');
            if line.starts_with("Leave this running") {
                return true;
            }
        }
        false
    })
    .await;
    eprintln!("{said}");
    assert_eq!(printed, Ok(true), "new printed all it says: {said}");
    let place = home.path().join("svrngs/trip");
    assert!(
        said.contains(&format!(
            "trip's folder is {}: what anyone in trip puts there reaches this computer",
            place.display()
        )),
        "{said}"
    );
    assert!(place.is_dir(), "{} not made", place.display());
    let api = std::fs::read_to_string(home.path().join(".svrngs/api")).unwrap();
    let token = std::fs::read_to_string(home.path().join(".svrngs/token")).unwrap();
    let (_, groups) = svrngs_node::api::asked(&api, "/v1/groups", &token, None)
        .await
        .unwrap();
    let groups: serde_json::Value = serde_json::from_slice(&groups).unwrap();
    assert_eq!(
        groups["groups"][0]["folders"],
        serde_json::json!([place]),
        "{groups:#}"
    );
    new.kill().await.unwrap();
}

/// Joining puts the group's folder at `~/svrngs/<group>` (v1-onboarding
/// (d)), kept the same as everyone's: `svrngs join --no-run`, this build's,
/// in a `HOME` of its own, says where and makes it; svrngs then run there
/// as login runs it keeps it, and `/v1/groups` — what the home page says
/// it from — names the place. A file of Sam's reaching it is measured in
/// the onboarding run (demo/runs/onboarding/): a member is dialled by key
/// through n0's lookup, which no test here reaches. Watched failing: join
/// before it placed the folder.
#[tokio::test(flavor = "multi_thread")]
async fn joining_places_the_groups_folder() {
    let (_relay, _, _dir, sam) = common::sams_computer(2).await;
    let (_, printed) = made::new_group(
        &sam,
        Name::parse("trip").unwrap(),
        Name::parse("Sam").unwrap(),
        None,
        &[],
    )
    .await
    .expect("Sam makes trip");

    let home = tempfile::tempdir().unwrap();
    let svrngs = |args: &[&str]| {
        let mut c = tokio::process::Command::new(binary());
        c.args(args)
            .env_clear()
            .env("HOME", home.path())
            .env("USER", "Alex")
            .kill_on_drop(true);
        c
    };
    let joined = svrngs(&["join", &printed.client, "--no-run"])
        .output()
        .await
        .unwrap();
    let said = String::from_utf8_lossy(&joined.stdout);
    eprintln!("{said}{}", String::from_utf8_lossy(&joined.stderr));
    assert!(joined.status.success());
    let place = home.path().join("svrngs/trip");
    assert!(
        said.contains(&format!(
            "trip's folder is {}: what anyone in trip puts there reaches this computer",
            place.display()
        )),
        "{said}"
    );
    assert!(place.is_dir(), "{} not made", place.display());

    let mut running = svrngs(&[]).spawn().unwrap();
    let groups = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let api = std::fs::read_to_string(home.path().join(".svrngs/api")).unwrap_or_default();
            let token =
                std::fs::read_to_string(home.path().join(".svrngs/token")).unwrap_or_default();
            let asked = svrngs_node::api::asked(&api, "/v1/groups", &token, None).await;
            if let Ok((_, answer)) = asked {
                break serde_json::from_slice::<serde_json::Value>(&answer).unwrap();
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("svrngs answers at the API it wrote");
    eprintln!("/v1/groups: {groups}");
    assert_eq!(
        groups["groups"][0]["folders"],
        serde_json::json!([place]),
        "{groups:#}"
    );
    running.kill().await.unwrap();
    sam.endpoint().close().await;
}

// install.sh's own claims, over the page's files with no listener: it
// fetches only by `curl -fsSL <url> [-o <file>]` (web/install.sh:90, 98),
// so a `curl` stub answers from the site as the page host would; and where
// the claim is install.sh's, svrngs is a stand-in that runs no node. A
// child process alone: no socket, relay or QUIC (docs/CHECKS.md, X).

/// The page these lines name. Never resolved: the `curl` stub answers it.
const PAGE: &str = "http://127.0.0.1:4000/";
const PUBLISHER: &str = "https://github.com/alexsbryan/svrngs/releases/latest/download/";

fn executable(path: &Path, text: &str) {
    std::fs::write(path, text).unwrap();
    std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
}

fn publisher_key(stubs: &Path, accepted: bool) {
    let fingerprint = if accepted {
        "46B4D67C5E6621093BB1AB2B057BF9082AA03CFB"
    } else {
        "0000000000000000000000000000000000000000"
    };
    executable(&stubs.join("gpg"), &format!("#!/bin/sh\ncase \" $* \" in *' --verify '*) echo '[GNUPG:] VALIDSIG {fingerprint}' ;; esac\n"));
}

/// svrngs's stand-in: install.sh places, keeps and opens it. Run with
/// nothing, as launchd and systemd run it, it waits to be stopped; `open`
/// writes the api file svrngs writes at its start and says what svrngs
/// says; anything else is refused.
const STAND_IN: &str = r#"#!/bin/sh
case "${1:-}" in
  "") exec sleep 600 ;;
  open) printf 'http://127.0.0.1:4000' >"$SVRNGS_DIR/api" && echo "Your home page is open in your browser." ;;
  *) echo "svrngs: the stand-in does not $1" >&2; exit 1 ;;
esac
"#;

/// The site of [`site`], its svrngs [`STAND_IN`].
fn stand_in_site(sums: Option<&str>) -> tempfile::TempDir {
    site_of(Some(STAND_IN), sums)
}

/// The page's line, as a person pastes it, run in `home` with the stubs
/// and a `curl` that answers [`PAGE`]'s urls from `site`: the file at the
/// url's path, to `-o` or said, each url kept in `stubs/fetched`; 22, as
/// `-f` fails on a 404, for a file the site has not, and 6, a host not
/// found, for any other host.
fn line(
    site: &Path,
    link: &str,
    more: &str,
    home: &Path,
    uname: Option<&str>,
    shell: Option<&str>,
) -> Output {
    let stubs = stubs(home, uname);
    let curl = format!(
        r#"#!/bin/sh
url="" out=""
while [ $# -gt 0 ]; do
  case "$1" in -o) out="$2"; shift 2 ;; -*) shift ;; *) url="$1"; shift ;; esac
done
echo "$url" >>"$HOME/stubs/fetched"
case "$url" in
    {PAGE}*) file="{0}/${{url#{PAGE}}}" ;;
    {PUBLISHER}*) file="{0}/get/${{url#{PUBLISHER}}}" ;;
    *) exit 6 ;;
esac
[ -f "$file" ] || exit 22
if [ -n "$out" ]; then cp "$file" "$out"; else cat "$file"; fi
"#,
        site.display()
    );
    std::fs::write(stubs.join("curl"), curl).unwrap();
    let exec = std::os::unix::fs::PermissionsExt::from_mode(0o755);
    std::fs::set_permissions(stubs.join("curl"), exec).unwrap();
    let line = format!("curl -fsSL {PAGE}install.sh | sh -s -- '{link}'{more}");
    let path = format!("{}:/usr/bin:/bin:/usr/sbin:/sbin", stubs.display());
    let mut sh = std::process::Command::new("/bin/sh");
    sh.args(["-c", &line])
        .env_clear()
        .env("HOME", home)
        .env("PATH", path)
        .env("USER", "Alex");
    if !stubs.join("gpg").exists() {
        publisher_key(&stubs, true);
    }
    if let Some(shell) = shell {
        sh.env("SHELL", shell);
    }
    let out = sh.output().unwrap();
    eprintln!(
        "--- the line, in {}:\n{}{}",
        home.display(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

#[test]
fn install_sh_refuses_a_tarball_off_its_sum_by_name() {
    let wrong = format!("{}  {}\n", "0".repeat(64), tarball());
    let site = stand_in_site(Some(&wrong));
    let home = tempfile::tempdir().unwrap();
    let out = line(
        site.path(),
        &format!("{PAGE}#x"),
        "",
        home.path(),
        None,
        None,
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{err}");
    assert!(
        err.contains(&format!(
            "{} is not the one the release lists, so it was refused",
            tarball()
        )),
        "{err}"
    );
    assert!(
        !home.path().join(".local/bin/svrngs").exists(),
        "installed anyway"
    );
}

/// Refused before any fetch: the line's own install.sh is the one url asked.
#[test]
fn install_sh_refuses_a_platform_svrngs_is_not_built_for_before_any_fetch() {
    let site = stand_in_site(None);
    let home = tempfile::tempdir().unwrap();
    let link = format!("{PAGE}#x");
    let out = line(
        site.path(),
        &link,
        "",
        home.path(),
        Some("FreeBSD riscv64"),
        None,
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{err}");
    assert!(
        err.contains("svrngs runs on Mac and Linux computers, Intel or ARM; this one is FreeBSD on riscv64, so nothing was installed."),
        "{err}"
    );
    assert!(!home.path().join(".local/bin/svrngs").exists());
    let fetched = std::fs::read_to_string(home.path().join("stubs/fetched")).unwrap();
    assert_eq!(fetched, format!("{PAGE}install.sh\n"));
}

/// **Give's program checked as svrngs is** (give-install (a)): a give
/// tarball off its sum is refused by name, and nothing is installed,
/// svrngs included, though svrngs's own tarball matched.
#[test]
fn install_sh_refuses_give_off_its_sum_and_installs_nothing() {
    let site = stand_in_site(None);
    let sums = std::fs::read_to_string(site.path().join("get/SHA256SUMS")).unwrap();
    let give = tarball_of("give");
    let wrong: String = (sums.lines())
        .map(|l| match l.ends_with(&give) {
            true => format!("{}  {give}\n", "0".repeat(64)),
            false => format!("{l}\n"),
        })
        .collect();
    std::fs::write(site.path().join("get/SHA256SUMS"), wrong).unwrap();
    let home = tempfile::tempdir().unwrap();
    let out = line(site.path(), PAGE, " --give", home.path(), None, None);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{err}");
    let refused = format!("{give} is not the one the release lists, so it was refused");
    assert!(err.contains(&refused), "{err}");
    let bin = home.path().join(".local/bin");
    assert!(!bin.join("give").exists(), "installed anyway");
    assert!(
        !bin.join("svrngs").exists(),
        "svrngs installed beside a refused program"
    );
}

/// **A refused open is said at once, not waited out**: where `svrngs open`
/// refuses (exit 1; here a stand-in for any refusal of svrngs's own), the
/// line stops with svrngs's own words, never retrying for
/// 30 s and then calling it a start that did not come. Watched failing on
/// install.sh before: "svrngs did not start within 30 seconds" after 30 tries.
#[test]
fn install_sh_says_a_refused_open_in_svrngs_s_words() {
    let opens = "open) printf 'http://127.0.0.1:4000' >\"$SVRNGS_DIR/api\" && echo \"Your home page is open in your browser.\" ;;";
    let refuses =
        "open) echo 'svrngs: your home page was not opened: the stand-in refuses' >&2; exit 1 ;;";
    assert!(STAND_IN.contains(opens));
    let site = site_of(Some(&STAND_IN.replace(opens, refuses)), None);
    let home = tempfile::tempdir().unwrap();
    let _started = Started(home.path().to_path_buf());
    let out = line(site.path(), PAGE, "", home.path(), None, None);
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{err}");
    let words = "svrngs: your home page was not opened: the stand-in refuses";
    assert!(err.contains(words), "not said: {words}\n{err}");
    assert!(!err.contains("did not start"), "{err}");
}

/// A tarball in `dir`: a release someone checked, as they give it to the line.
fn checked(dir: &Path, program: &str, members: &[(&str, &str)]) -> PathBuf {
    let made = dir.join(program);
    std::fs::create_dir_all(&made).unwrap();
    for (name, script) in members {
        std::fs::write(made.join(name), script).unwrap();
    }
    let tarball = dir.join(format!("checked {program}.tar.gz"));
    let tar = std::process::Command::new("tar")
        .arg("-czf")
        .arg(&tarball)
        .arg("-C")
        .arg(&made)
        .args(members.iter().map(|(name, _)| name))
        .status();
    assert!(tar.unwrap().success());
    tarball
}

/// `file`'s SHA-256, as install.sh keeps it in the release record.
fn sum_of(file: &Path) -> String {
    let bytes = std::fs::read(file).unwrap();
    hex::encode(ring::digest::digest(&ring::digest::SHA256, &bytes))
}

#[test]
fn default_install_takes_the_installer_and_archive_from_the_publisher_manifest() {
    let site = stand_in_site(None);
    let archive = tarball();
    for accepted in [false, true] {
        let home = tempfile::tempdir().unwrap();
        let _started = Started(home.path().to_path_buf());
        publisher_key(&stubs(home.path(), None), accepted);
        let out = line(site.path(), PAGE, "", home.path(), None, None);
        assert_eq!(out.status.success(), accepted, "{out:?}");
        if !accepted {
            assert!(String::from_utf8_lossy(&out.stderr)
                .contains("not signed by the svrngs release key"));
        }
        assert_eq!(
            home.path().join(".local/bin/svrngs-keeper").is_file(),
            accepted
        );
        let fetched = std::fs::read_to_string(home.path().join("stubs/fetched")).unwrap();
        assert!(
            fetched
                .lines()
                .skip(1)
                .all(|url| url.starts_with(PUBLISHER)),
            "{fetched}"
        );
        assert_eq!(fetched.contains(&format!("{PUBLISHER}{archive}")), accepted);
    }
}

/// **A release someone checked is installed as given, and the page only
/// joins** (svrn-docs MARKETPLACE_REVIEW_SVRNGS.md §1): the page serves
/// another svrngs and Give's program, with sums that match them, as a page
/// someone else runs could. With `--archive` and `--give-archive` beside
/// `--give`, each program installed is the given file's byte for byte, no
/// file under the page's `get/` is fetched, the release kept is the given
/// file's sum, and the page is still kept; `--archive` stands before the
/// link as after it. Without them, the page's and its sum, as before.
/// Watched failing on install.sh before `--archive`: the flag taken for
/// join's, and the page's svrngs installed.
#[test]
fn install_sh_installs_the_archive_it_is_given_and_never_the_page_s() {
    let site = site_of(Some(&format!("{STAND_IN}# the page's\n")), None);
    let given = tempfile::tempdir().unwrap();
    let ours = format!("{STAND_IN}# the release you checked\n");
    let keeper = "#!/bin/sh\necho 'keeper from checked archive'\n";
    let archive = checked(
        given.path(),
        "svrngs",
        &[("svrngs", &ours), ("svrngs-keeper", keeper)],
    );
    let give = "#!/bin/sh\necho \"checked give $*\"\n";
    let give_archive = checked(given.path(), "give", &[("give", give)]);
    let (a, g) = (archive.display(), give_archive.display());
    let after = format!(" --archive '{a}' --give --give-archive '{g}'");
    // Before the link, as a caller that checked the release writes it.
    let before = format!(" '{a}' '{PAGE}'");
    for (link, more, gave) in [(PAGE, after.as_str(), true), ("--archive", &before, false)] {
        let home = tempfile::tempdir().unwrap();
        let _started = Started(home.path().to_path_buf());
        let out = line(site.path(), link, more, home.path(), None, None);
        let said = String::from_utf8_lossy(&out.stdout);
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "{more}: {said}{err}");
        let bin = home.path().join(".local/bin");
        assert_eq!(std::fs::read_to_string(bin.join("svrngs")).unwrap(), ours);
        assert_eq!(
            std::fs::read_to_string(bin.join("svrngs-keeper")).unwrap(),
            keeper
        );
        let used = format!("Using svrngs from {a}, the file you gave, in place of the page's.");
        assert!(said.contains(&used), "not said: {used}\n{said}");
        if gave {
            assert_eq!(std::fs::read_to_string(bin.join("give")).unwrap(), give);
        }
        let fetched = std::fs::read_to_string(home.path().join("stubs/fetched")).unwrap();
        assert_eq!(
            fetched,
            format!("{PAGE}install.sh\n"),
            "the page's get/ asked"
        );
        let data = home.path().join(".svrngs");
        let release = std::fs::read_to_string(data.join(RELEASE_FILE)).unwrap();
        assert_eq!(
            release,
            sum_of(&archive),
            "the release kept is the file given"
        );
        let origin = std::fs::read_to_string(data.join(ORIGIN_FILE)).unwrap();
        assert_eq!(origin, PAGE, "the page is still kept, for links");
    }

    let home = tempfile::tempdir().unwrap();
    let _started = Started(home.path().to_path_buf());
    let out = line(site.path(), PAGE, "", home.path(), None, None);
    assert!(out.status.success(), "{out:?}");
    let installed = home.path().join(".local/bin/svrngs");
    assert!(std::fs::read_to_string(installed)
        .unwrap()
        .ends_with("# the page's\n"));
    let sums = std::fs::read_to_string(site.path().join("get/SHA256SUMS")).unwrap();
    let release = std::fs::read_to_string(home.path().join(".svrngs").join(RELEASE_FILE));
    assert!(
        sums.starts_with(&release.unwrap()),
        "the page's sum kept: {sums}"
    );
    let fetched = std::fs::read_to_string(home.path().join("stubs/fetched")).unwrap();
    assert!(
        fetched.contains(&format!("{PUBLISHER}SHA256SUMS")),
        "{fetched}"
    );
}

/// An archive the line cannot use is refused before anything is installed
/// or fetched from the page's `get/`, saying what to give instead: a file
/// that is not there, `--archive` with no file, a file holding no svrngs,
/// and `--give-archive` without `--give`, which alone installs Give's
/// program. Watched failing on install.sh before `--archive`.
#[test]
fn install_sh_refuses_an_archive_it_cannot_use_and_installs_nothing() {
    let site = stand_in_site(None);
    let given = tempfile::tempdir().unwrap();
    let junk = given.path().join("junk.tar.gz");
    std::fs::write(&junk, "not a tarball").unwrap();
    let give = checked(given.path(), "give", &[("give", "#!/bin/sh\n")]);
    let missing_keeper = checked(given.path(), "svrngs", &[("svrngs", STAND_IN)]);
    let gone = given.path().join("gone.tar.gz");
    let refusals = [
        (format!(" --archive '{}'", gone.display()), format!("{} is not a file you can read, so nothing was installed: give --archive the release you checked.", gone.display())),
        (" --archive".into(), "--archive wants the file of the release you checked, as svrngs-linux-x86_64.tar.gz is.".into()),
        (format!(" --archive '{}'", junk.display()), format!("{} must hold each of these programs: svrngs svrngs-keeper. Nothing was installed.", junk.display())),
        (format!(" --archive '{}'", missing_keeper.display()), format!("{} must hold each of these programs: svrngs svrngs-keeper. Nothing was installed.", missing_keeper.display())),
        (format!(" --give-archive '{}'", give.display()), "--give-archive is the file of Give's program, which only --give installs: add --give after the link.".into()),
    ];
    for (more, words) in refusals {
        let home = tempfile::tempdir().unwrap();
        let out = line(site.path(), PAGE, &more, home.path(), None, None);
        let err = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{more}: {err}");
        assert!(err.contains(&words), "{more}: not said: {words}\n{err}");
        assert!(
            !home.path().join(".local/bin/svrngs").exists(),
            "{more}: installed anyway"
        );
        let fetched = std::fs::read_to_string(home.path().join("stubs/fetched")).unwrap();
        assert!(
            !fetched.contains("get/"),
            "{more}: the page's get/ asked: {fetched}"
        );
    }
}

/// `--relay <name> --open` on macOS: a LaunchAgent of its own running
/// `svrngs relay <name> --open`, bootstrapped once, svrngs's own untouched.
#[cfg(target_os = "macos")]
#[test]
fn install_sh_keeps_a_relay_running_beside_svrngs_on_a_mac() {
    let site = stand_in_site(None);
    let home = tempfile::tempdir().unwrap();
    let more = " --relay relay.example.org --open";
    let out = line(site.path(), PAGE, more, home.path(), None, None);
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{said}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let bin = home.path().join(".local/bin");
    assert!(
        said.contains(&format!("Installed svrngs in {}.", bin.display())),
        "{said}"
    );
    assert!(
        said.contains("svrngs-relay runs now, and whenever you log in"),
        "{said}"
    );
    assert!(!said.contains("You're in"), "a relay joins nothing: {said}");
    let agents = home.path().join("Library/LaunchAgents");
    let plist = std::fs::read_to_string(agents.join("sh.svrnme.svrngs.relay.plist")).unwrap();
    let run = format!(
        "<array><string>{}/svrngs</string><string>relay</string><string>relay.example.org</string><string>--open</string></array>",
        bin.display()
    );
    assert!(plist.contains(&run), "{plist}");
    assert!(plist.contains("svrngs-relay.log"), "{plist}");
    assert!(
        !agents.join("sh.svrnme.svrngs.plist").exists(),
        "svrngs's own agent is not touched"
    );
    let launched = std::fs::read_to_string(home.path().join("stubs/said")).unwrap();
    assert_eq!(
        launched.matches("launchctl bootstrap").count(),
        1,
        "{launched}"
    );
}

/// `--relay` on Linux: the system unit printed, none installed, nothing
/// asked of systemd; `--as` beside it refused.
#[test]
fn install_sh_prints_a_relay_s_system_unit_on_linux_and_installs_none() {
    let site = stand_in_site(None);
    as_linux(site.path());
    let home = tempfile::tempdir().unwrap();
    let linux = Some("Linux x86_64");
    let more = " --relay relay.example.org";
    let out = line(site.path(), PAGE, more, home.path(), linux, None);
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{said}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let bin = home.path().join(".local/bin");
    for line in [
        "As root, put this in /etc/systemd/system/svrngs-relay.service, then run systemctl enable --now svrngs-relay:".to_string(),
        format!("ExecStart={}/svrngs relay relay.example.org", bin.display()),
        "AmbientCapabilities=CAP_NET_BIND_SERVICE".to_string(),
        "WantedBy=multi-user.target".to_string(),
    ] {
        assert!(said.contains(&line), "not said: {line}\n{said}");
    }
    assert!(
        !home.path().join("stubs/said").exists(),
        "nothing asked of systemd"
    );
    assert!(
        !home.path().join(".config/systemd").exists(),
        "no unit installed"
    );

    let more = " --relay relay.example.org --as x";
    let wrong = line(site.path(), PAGE, more, home.path(), linux, None);
    let err = String::from_utf8_lossy(&wrong.stderr);
    assert_eq!(wrong.status.code(), Some(1), "{err}");
    assert!(
        err.contains("a relay takes --open and nothing else, not --as."),
        "{err}"
    );
}

/// **svrngs on the login shell's PATH** (the room's report, 3): install.sh's
/// profile step, for zsh `~/.zshrc`, for bash `~/.bashrc` on Linux and
/// `~/.bash_profile` on macOS: one PATH line, said once, then said to be
/// there; what the profile held, kept.
#[test]
fn install_sh_puts_svrngs_on_the_login_shell_s_path_once() {
    let site = stand_in_site(None);
    as_linux(site.path());
    let mac = cfg!(target_os = "macos");
    let mut runs = vec![("/bin/bash", Some("Linux x86_64"), ".bashrc")];
    if mac {
        runs.push(("/bin/zsh", None, ".zshrc"));
        runs.push(("/bin/bash", None, ".bash_profile"));
    } else {
        runs.push(("/usr/bin/zsh", Some("Linux x86_64"), ".zshrc"));
    }
    for (shell, uname, file) in runs {
        let home = tempfile::tempdir().unwrap();
        let _started = Started(home.path().to_path_buf());
        let profile = home.path().join(file);
        std::fs::write(&profile, "alias ll='ls -l'\n").unwrap();
        let bin = home.path().join(".local/bin");
        let export = format!("export PATH=\"{}:$PATH\"", bin.display());
        let first = line(site.path(), PAGE, "", home.path(), uname, Some(shell));
        let said = String::from_utf8_lossy(&first.stdout);
        assert!(first.status.success(), "{shell} {uname:?}: {said}");
        let added = format!(
            "Added {} to where your terminal looks for programs, in {}. Open a new terminal window, and svrngs runs there by name.",
            bin.display(),
            profile.display()
        );
        assert!(
            said.contains(&added),
            "{shell} {uname:?}: not said: {added}\n{said}"
        );
        let again = line(site.path(), PAGE, "", home.path(), uname, Some(shell));
        let said = String::from_utf8_lossy(&again.stdout);
        assert!(again.status.success(), "{shell} {uname:?}: {said}");
        let there = format!(
            "{} already adds {} to where your terminal looks for programs. Open a new terminal window, and svrngs runs there by name.",
            profile.display(),
            bin.display()
        );
        assert!(
            said.contains(&there),
            "{shell} {uname:?}: not said: {there}\n{said}"
        );
        let held = std::fs::read_to_string(&profile).unwrap();
        assert!(held.starts_with("alias ll='ls -l'\n"), "{held}");
        assert_eq!(
            held.matches(&export).count(),
            1,
            "{shell} {uname:?}: {held}"
        );
    }
}

/// **Give's program, as an option of the line** (give-install (a)),
/// install.sh's claim: Give's program only when asked, checked, placed
/// and kept at login beside svrngs, its agent or unit finding svrngs and
/// Podman and waited for as it stops; run again, updated, one agent.
#[test]
fn install_sh_installs_and_keeps_give_beside_svrngs_only_when_asked() {
    let site = stand_in_site(None);
    as_linux(site.path());
    let mac = cfg!(target_os = "macos").then_some(None);
    for uname in mac.into_iter().chain([Some("Linux x86_64")]) {
        let home = tempfile::tempdir().unwrap();
        let _started = Started(home.path().to_path_buf());
        let bin = home.path().join(".local/bin");
        let agent = match uname {
            None => home
                .path()
                .join("Library/LaunchAgents/sh.svrnme.give.plist"),
            Some(_) => home.path().join(".config/systemd/user/give.service"),
        };
        let without = line(site.path(), PAGE, "", home.path(), uname, None);
        let said = String::from_utf8_lossy(&without.stdout);
        assert!(without.status.success(), "{uname:?}: {said}");
        assert!(bin.join("svrngs-keeper").is_file(), "{uname:?}: {said}");
        assert!(
            !bin.join("give").exists(),
            "{uname:?}: Give's program, unasked"
        );
        assert!(!agent.exists(), "{uname:?}: Give's program kept, unasked");

        let platform = match uname {
            None => tarball_of("give"),
            Some(_) => "give-linux-x86_64.tar.gz".into(),
        };
        for (run, done) in [(1, "Installed"), (2, "Updated")] {
            let with = line(site.path(), PAGE, " --give", home.path(), uname, None);
            let said = String::from_utf8_lossy(&with.stdout);
            let err = String::from_utf8_lossy(&with.stderr);
            assert!(with.status.success(), "{uname:?}, run {run}: {said}{err}");
            for line in [
                format!(
                    "Checked: {platform} is the one the release lists, unchanged on its way here."
                ),
                "Updated svrngs in".into(),
                format!("{done} give in {}.", bin.display()),
                "give runs now, and whenever you log in".into(),
            ] {
                assert!(
                    said.contains(&line),
                    "{uname:?}, run {run}, not said: {line}\n{said}"
                );
            }
            assert_eq!(
                said.contains("Stopped give, to update it."),
                run == 2,
                "{uname:?}, run {run}: {said}"
            );
            assert!(
                !said.contains("You're in"),
                "a page's address joins nothing: {said}"
            );
        }
        let placed = std::fs::read_to_string(bin.join("give")).unwrap();
        assert!(placed.contains("echo \"give $*\""), "{placed}");
        let kept = std::fs::read_to_string(&agent).unwrap();
        let path = format!("{}:/opt/podman/bin:", bin.display());
        let launched = std::fs::read_to_string(home.path().join("stubs/said")).unwrap();
        match uname {
            None => {
                let run = format!(
                    "<string>{}/give</string><string>keep</string>",
                    bin.display()
                );
                for line in [
                    run,
                    path,
                    "<key>ExitTimeOut</key><integer>0</integer>".into(),
                ] {
                    assert!(kept.contains(&line), "not in the agent: {line}\n{kept}");
                }
                let n = launched.matches("bootstrap gui/").count();
                assert_eq!(n, 3 + 2, "svrngs three times, give twice: {launched}");
            }
            Some(_) => {
                let run = format!("ExecStart={}/give keep", bin.display());
                for line in [
                    run,
                    format!("Environment=PATH={path}"),
                    "TimeoutStopSec=infinity".into(),
                ] {
                    assert!(kept.contains(&line), "not in the unit: {line}\n{kept}");
                }
                let n = launched.matches("--user enable --now give.service").count();
                assert_eq!(n, 2, "{launched}");
            }
        }
        let data = home.path().join(".svrngs");
        assert!(
            kept.contains(&data.display().to_string()),
            "the agent names svrngs's data dir: {kept}"
        );
    }
}

/// **`--uninstall`** where the link goes takes away what the line put
/// here — what runs at login, svrngs and Give's program, the opener — and
/// keeps the data dir and the profile's PATH line, each said, fetching
/// nothing past install.sh itself; anything after it is refused before
/// anything is taken, `--archive` included though it is taken out of the
/// arguments before the link is read, and run again it finds nothing left
/// to take.
/// Planted, watched failing: install.sh as 94413a92 left it, which refuses
/// `--uninstall` as a link that names no page.
#[test]
fn install_sh_uninstalls_what_it_put_here_and_keeps_the_data() {
    let site = stand_in_site(None);
    as_linux(site.path());
    let mac = cfg!(target_os = "macos").then_some(None);
    for uname in mac.into_iter().chain([Some("Linux x86_64")]) {
        let home = tempfile::tempdir().unwrap();
        let _started = Started(home.path().to_path_buf());
        let h = home.path();
        let zsh = Some("/bin/zsh");
        let installed = line(site.path(), PAGE, " --give", h, uname, zsh);
        let err = String::from_utf8_lossy(&installed.stderr);
        assert!(installed.status.success(), "{uname:?}: {err}");
        let bin = h.join(".local/bin");
        let (agents, opener) = match uname {
            None => (
                h.join("Library/LaunchAgents"),
                h.join("Applications/svrngs.app"),
            ),
            Some(_) => (
                h.join(".config/systemd/user"),
                h.join(".local/share/applications/svrngs.desktop"),
            ),
        };
        let kept: Vec<PathBuf> = match uname {
            None => vec![
                agents.join("sh.svrnme.svrngs.plist"),
                agents.join("sh.svrnme.give.plist"),
            ],
            Some(_) => vec![agents.join("svrngs.service"), agents.join("give.service")],
        };
        let placed: Vec<PathBuf> = (kept.into_iter())
            .chain([
                bin.join("svrngs"),
                bin.join("svrngs-keeper"),
                bin.join("give"),
                opener.clone(),
            ])
            .collect();
        for p in &placed {
            assert!(p.exists(), "{uname:?}: not installed: {}", p.display());
        }

        for (more, words) in [
            (" --give", "--uninstall takes nothing after it, not --give."),
            (
                " --archive svrngs-linux-x86_64.tar.gz",
                "--uninstall installs nothing, so it takes no --archive.",
            ),
        ] {
            let more = line(site.path(), "--uninstall", more, h, uname, zsh);
            let err = String::from_utf8_lossy(&more.stderr);
            assert_eq!(more.status.code(), Some(1), "{uname:?}: {err}");
            assert!(err.contains(words), "{err}");
            for p in &placed {
                assert!(p.exists(), "{uname:?}: taken by a refusal: {}", p.display());
            }
        }

        let fetched = h.join("stubs/fetched");
        let before = std::fs::read_to_string(&fetched).unwrap();
        let out = line(site.path(), "--uninstall", "", h, uname, zsh);
        let said = String::from_utf8_lossy(&out.stdout);
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "{uname:?}: {said}{err}");
        let stop = "To stop an app, open your home page and choose Stop before uninstalling.";
        let (while_installed, after_remove) = said.split_once("Stopped svrngs,").unwrap();
        assert!(while_installed.contains(stop), "{said}");
        assert!(!after_remove.contains("choose Stop"), "{said}");
        assert!(!said.contains("svrngs-keeper"), "{said}");
        for p in &placed {
            assert!(!p.exists(), "{uname:?}: left: {}", p.display());
        }
        assert!(
            !h.join("stubs/pid").exists(),
            "{uname:?}: svrngs still runs"
        );
        let data = h.join(".svrngs");
        assert!(data.join(ORIGIN_FILE).exists(), "{uname:?}: data taken");
        for words in [
            "Stopped svrngs, and took it off what runs when you log in".to_string(),
            "Stopped give, and took it off what runs when you log in".into(),
            format!("Removed svrngs from {}.", bin.display()),
            format!("Removed give from {}.", bin.display()),
            "Uninstalling svrngs does not stop your apps or remove their setup or files.".into(),
            "Native apps may still start.".into(),
            "Managed Linux containers need svrngs installed to start and stop.".into(),
            "If you've already uninstalled, reinstall svrngs to manage your apps again.".into(),
            format!("Removed the svrngs opener, {}.", opener.display()),
            format!(".zshrc still adds {}", bin.display()),
            format!("Kept {}: your key and your groups' copies.", data.display()),
            "svrngs is uninstalled.".into(),
        ] {
            assert!(
                said.contains(&words),
                "{uname:?}, not said: {words}\n{said}"
            );
        }
        let now = std::fs::read_to_string(&fetched).unwrap();
        assert_eq!(&now[before.len()..], format!("{PAGE}install.sh\n"));
        if uname.is_some() {
            let asked = std::fs::read_to_string(h.join("stubs/said")).unwrap();
            for unit in ["svrngs", "give"] {
                let disabled = format!("systemctl --user disable --quiet {unit}.service");
                assert!(asked.contains(&disabled), "{asked}");
            }
        }

        let again = line(site.path(), "--uninstall", "", h, uname, zsh);
        let said = String::from_utf8_lossy(&again.stdout);
        assert!(again.status.success(), "{uname:?}: {said}");
        assert!(
            !said.contains("Removed") && !said.contains("Stopped"),
            "{said}"
        );
        assert!(said.contains("svrngs is uninstalled."), "{said}");
        assert!(!said.contains("choose Stop"), "{said}");
        assert!(
            said.contains("reinstall svrngs to manage your apps again."),
            "{said}"
        );
    }
}

/// `svrngs restart` asks what keeps svrngs running to restart it (cli.rs
/// `restart`), and where nothing does, and no svrngs runs, starts this one
/// itself, apart from its terminal, and says once it answers as this build
/// that it runs; where what it restarted never answers, where to read why.
/// This build's svrngs, asked where install.sh kept the stand-in at login,
/// which starts no node, then where nothing does. Planted, watched
/// failing: 0ba18ef9 refused the second, "svrngs is not kept running at
/// login here, so nothing can restart it: stop it where it runs (Ctrl-C)".
#[test]
fn restart_asks_launchd_or_systemd_and_starts_svrngs_where_neither_keeps_it() {
    let site = stand_in_site(None);
    let home = tempfile::tempdir().unwrap();
    let _started = Started(home.path().to_path_buf());
    assert!(line(site.path(), PAGE, "", home.path(), None, None)
        .status
        .success());
    let program = tempfile::tempdir().unwrap();
    let svrngs = as_main::installed_at(program.path(), "http://relay.test/");
    let restart = || {
        std::process::Command::new(&svrngs)
            .arg("restart")
            .env_clear()
            .env("HOME", home.path())
            .env("PATH", home.path().join("stubs"))
            .output()
            .unwrap()
    };
    let running = format!("svrngs {} is running.\n", svrngs_node::api::build());
    // The stand-in install.sh kept answers nothing: said, with where why is.
    let kept = restart();
    let said = std::fs::read_to_string(home.path().join("stubs/said")).unwrap();
    let asked = match cfg!(target_os = "macos") {
        true => "launchctl kickstart -k gui/",
        false => "systemctl --user restart svrngs.service",
    };
    assert!(said.contains(asked), "{said}");
    let log = home.path().join(".svrngs/svrngs.log");
    let why = String::from_utf8_lossy(&kept.stderr);
    assert!(
        why.contains(&format!(
            "svrngs couldn't start here ({} says why)",
            log.display()
        )),
        "{why}"
    );

    let pid = std::fs::read_to_string(home.path().join("stubs/pid")).unwrap();
    let stopped = std::process::Command::new("kill").arg(pid.trim()).status();
    assert!(stopped.unwrap().success());
    std::fs::remove_file(home.path().join("stubs/pid")).unwrap();
    let dir = home.path().join(".svrngs");
    while svrngs::cli::claim(&dir).is_err() {
        std::thread::sleep(Duration::from_millis(50));
    }
    let started = restart();
    let holder = std::process::Command::new("lsof")
        .arg("-t")
        .arg(dir.join("lock"))
        .output()
        .unwrap();
    let holder = String::from_utf8_lossy(&holder.stdout).trim().to_string();
    let _ = std::process::Command::new("kill").arg(&holder).status();
    let told = String::from_utf8_lossy(&started.stdout);
    assert!(started.status.success(), "{started:?}");
    assert!(told.ends_with(&running), "{told}");
    assert!(!holder.is_empty(), "no svrngs holds the data dir");
}
