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
//! it puts `~/.local/bin` on that shell's PATH, once.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
mod common;

use std::path::{Path, PathBuf};
use std::process::Output;
use std::time::Duration;

use svrngs::cli;
use svrngs::group::Name;
use svrngs_node::invite::ORIGIN_FILE;
// Only the macOS tests' (the_line_run_twice_…): unused, and so clippy's
// error, where they are not built.
#[cfg(target_os = "macos")]
use svrngs_node::link::Origin;
#[cfg(target_os = "macos")]
use svrngs_node::RailAct;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// This computer's tarball, as install.sh names it from `uname`.
fn tarball() -> String {
    tarball_of("svrngs")
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
/// and a stand-in for Give's program, each as this computer's tarball, beside
/// `sums`, or their true SHA256SUMS. The stand-in is a script that says it
/// ran: install.sh only fetches, checks, places and keeps it, and Give's
/// own tests run the program itself (apps/give/tests/).
fn site(sums: Option<&str>) -> tempfile::TempDir {
    let site = tempfile::tempdir().unwrap();
    std::fs::copy(
        harness::repo().join("web/install.sh"),
        site.path().join("install.sh"),
    )
    .unwrap();
    let get = site.path().join("get");
    std::fs::create_dir(&get).unwrap();
    let bin = Path::new(env!("CARGO_BIN_EXE_svrngs"));
    let made = std::process::Command::new("tar")
        .arg("-czf")
        .arg(get.join(tarball()))
        .arg("-C")
        .arg(bin.parent().unwrap())
        .arg("svrngs")
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
    let summed = std::process::Command::new("shasum")
        .args(["-a", "256", &tarball(), &tarball_of("give")])
        .current_dir(&get)
        .output()
        .unwrap();
    let truth = String::from_utf8(summed.stdout).unwrap();
    std::fs::write(get.join("SHA256SUMS"), sums.unwrap_or(&truth)).unwrap();
    site
}

/// The stubs: `launchctl` runs the svrngs its agent names as launchd
/// would, and stops it; `open` keeps the address; `uname`, when `uname`
/// is given, names another platform; and `sh` is dash where this computer
/// has it, Debian's `/bin/sh`, which holds install.sh to POSIX.
fn stubs(home: &Path, uname: Option<&str>) -> PathBuf {
    let stubs = home.join("stubs");
    std::fs::create_dir_all(&stubs).unwrap();
    let launchctl = r#"#!/bin/sh
echo "launchctl $*" >>"$HOME/stubs/said"
case "$1" in
  bootstrap)
    grep -q '<string>relay</string>\|<string>keep</string>' "$3" && exit 0
    run=$(sed -n 's:.*<array><string>\(.*\)</string></array>.*:\1:p' "$3")
    dir=$(sed -n 's:.*<key>SVRNGS_DIR</key><string>\(.*\)</string>.*:\1:p' "$3")
    SVRNGS_DIR="$dir" "$run" >>"$dir/svrngs.log" 2>&1 &
    echo $! >"$HOME/stubs/pid" ;;
  bootout)
    case "$2" in */sh.svrnme.give) exit 0 ;; esac
    [ -f "$HOME/stubs/pid" ] || exit 3
    pid=$(cat "$HOME/stubs/pid"); rm "$HOME/stubs/pid"; kill "$pid"
    while kill -0 "$pid" 2>/dev/null; do sleep 0.1; done ;;
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
    paste_in(port, link, more, home, uname, None).await
}

/// The line pasted in a terminal whose login shell is `shell`, as `$SHELL`
/// names it; with none, a terminal that names none.
async fn paste_in(
    port: u16,
    link: &str,
    more: &str,
    home: &Path,
    uname: Option<&str>,
    shell: Option<&str>,
) -> Output {
    let line = format!("curl -fsSL http://127.0.0.1:{port}/install.sh | sh -s -- '{link}'{more}");
    let path = format!(
        "{}:/usr/bin:/bin:/usr/sbin:/sbin",
        stubs(home, uname).display()
    );
    let mut sh = tokio::process::Command::new("/bin/sh");
    sh.args(["-c", &line])
        .env_clear()
        .env("HOME", home)
        .env("PATH", path)
        .env("USER", "Alex");
    if let Some(shell) = shell {
        sh.env("SHELL", shell);
    }
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

#[tokio::test(flavor = "multi_thread")]
async fn a_tarball_that_does_not_match_its_sum_is_refused_by_name() {
    let wrong = format!("{}  {}\n", "0".repeat(64), tarball());
    let site = site(Some(&wrong));
    let port = host(site.path().to_path_buf()).await;
    let home = tempfile::tempdir().unwrap();
    let out = paste(
        port,
        &format!("http://127.0.0.1:{port}/#x"),
        home.path(),
        None,
    )
    .await;
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

#[tokio::test(flavor = "multi_thread")]
async fn a_platform_svrngs_is_not_built_for_is_refused() {
    let site = site(None);
    let port = host(site.path().to_path_buf()).await;
    let home = tempfile::tempdir().unwrap();
    let link = format!("http://127.0.0.1:{port}/#x");
    let out = paste(port, &link, home.path(), Some("FreeBSD riscv64")).await;
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{err}");
    assert!(
        err.contains("svrngs runs on Mac and Linux computers, Intel or ARM; this one is FreeBSD on riscv64, so nothing was installed."),
        "{err}"
    );
    assert!(!home.path().join(".local/bin/svrngs").exists());
}

#[cfg(target_os = "macos")]
#[tokio::test(flavor = "multi_thread")]
async fn the_line_run_twice_installs_runs_at_login_opens_home_and_joins_once() {
    let site = site(None);
    let port = host(site.path().to_path_buf()).await;
    let (_relay, ours, _dir, sam) = common::sams_computer(1).await;
    let page = Origin::parse(&format!("http://127.0.0.1:{port}/")).unwrap();
    let (seed, printed) = cli::new_group(
        &sam,
        Name::parse("house").unwrap(),
        Name::parse("Sam").unwrap(),
        Some(&page),
        None,
        &ours,
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
    let mut summed = vec!["-a".to_string(), "256".into()];
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
    }
}

/// **Give's program, as an option of the line** (give-install (a)): on a
/// clean data dir the line without `--give` installs svrngs alone, and
/// Give's page's line, `curl -fsSL <page>install.sh | sh -s -- '<page>'
/// --give`, then gets Give's program from the same page's `get/`, checked
/// against the same SHA256SUMS, puts it beside svrngs and keeps `give
/// keep` running at login beside it — a LaunchAgent on macOS, a systemd
/// --user unit on Linux, each recorded by the stand-in, never run, since
/// it would drive this computer's Podman. The agent finds svrngs and
/// Podman on its own PATH, which launchd and systemd do not give it, and
/// is waited for as it stops, since it copies each app it keeps to the
/// group as it stops. Run again with it, it updates Give's program and
/// keeps one agent. Planted, watched failing: install.sh as 60565068 left
/// it, which took `--give` for nothing and installed no program.
#[tokio::test(flavor = "multi_thread")]
async fn the_line_without_then_with_give_installs_give_s_program_beside_svrngs() {
    let site = site(None);
    as_linux(site.path());
    let port = host(site.path().to_path_buf()).await;
    let page = format!("http://127.0.0.1:{port}/");
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
        let without = paste(port, &page, home.path(), uname).await;
        let said = String::from_utf8_lossy(&without.stdout);
        assert!(without.status.success(), "{uname:?}: {said}");
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
            let with = paste_more(port, &page, " --give", home.path(), uname).await;
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

/// **Give's program checked as svrngs is** (give-install (a)): a give
/// tarball that does not match its sum is refused by name, and nothing is
/// installed, svrngs included, though svrngs's own tarball matched.
/// Planted, watched failing: install.sh as 60565068 left it, which fetched
/// no program and so refused nothing.
#[tokio::test(flavor = "multi_thread")]
async fn a_give_tarball_that_does_not_match_its_sum_is_refused_and_nothing_installed() {
    let site = site(None);
    let sums = std::fs::read_to_string(site.path().join("get/SHA256SUMS")).unwrap();
    let give = tarball_of("give");
    let wrong: String = (sums.lines())
        .map(|l| match l.ends_with(&give) {
            true => format!("{}  {give}\n", "0".repeat(64)),
            false => format!("{l}\n"),
        })
        .collect();
    std::fs::write(site.path().join("get/SHA256SUMS"), wrong).unwrap();
    let port = host(site.path().to_path_buf()).await;
    let home = tempfile::tempdir().unwrap();
    let page = format!("http://127.0.0.1:{port}/");
    let out = paste_more(port, &page, " --give", home.path(), None).await;
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

/// What svrngs answers the open `url` redeemed in a browser: its status,
/// and whether it set the session.
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
    let ask = format!("GET /{path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    sock.write_all(ask.as_bytes()).await.unwrap();
    let mut reply = String::new();
    sock.read_to_string(&mut reply).await.unwrap();
    let set = reply
        .to_ascii_lowercase()
        .contains("\r\nset-cookie: svrngs=");
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
        (303, true),
        "install.sh's open sets the session"
    );
    assert_ne!(redeemed(&first).await.0, 303, "a replayed open is refused");
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
        (303, true),
        "the opener's open sets the session"
    );
}

/// **svrngs on the login shell's PATH** (the room's report, 3: a person
/// could not run `svrngs` by name after install.sh). A founder's line, run
/// twice in a terminal that does not look in `~/.local/bin`: for zsh it
/// adds that directory to `~/.zshrc`, for bash on Linux to `~/.bashrc`
/// and on macOS to `~/.bash_profile`, as rustup and uv add theirs; it says
/// what it changed and that a new terminal window picks it up, and the
/// second run adds no second line and says it is there already. What the
/// profile held before stays. Planted, watched failing: install.sh as
/// e01c070a left it, which only printed where svrngs is.
#[tokio::test(flavor = "multi_thread")]
async fn the_line_puts_svrngs_on_the_login_shell_s_path() {
    let site = site(None);
    as_linux(site.path());
    let port = host(site.path().to_path_buf()).await;
    let page = format!("http://127.0.0.1:{port}/");
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
        let line = format!("export PATH=\"{}:$PATH\"", bin.display());
        let first = paste_in(port, &page, "", home.path(), uname, Some(shell)).await;
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
        let again = paste_in(port, &page, "", home.path(), uname, Some(shell)).await;
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
        assert_eq!(held.matches(&line).count(), 1, "{shell} {uname:?}: {held}");
    }
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
    let mut new = tokio::process::Command::new(env!("CARGO_BIN_EXE_svrngs"))
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
    let groups: serde_json::Value = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .get(format!("{}/v1/groups", api.trim()))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
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
    let (_relay, ours, _dir, sam) = common::sams_computer(2).await;
    let (_, printed) = cli::new_group(
        &sam,
        Name::parse("trip").unwrap(),
        Name::parse("Sam").unwrap(),
        None,
        None,
        &ours,
    )
    .await
    .expect("Sam makes trip");

    let home = tempfile::tempdir().unwrap();
    let svrngs = |args: &[&str]| {
        let mut c = tokio::process::Command::new(env!("CARGO_BIN_EXE_svrngs"));
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
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let groups = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let api = std::fs::read_to_string(home.path().join(".svrngs/api")).unwrap_or_default();
            let token =
                std::fs::read_to_string(home.path().join(".svrngs/token")).unwrap_or_default();
            let asked = http
                .get(format!("{}/v1/groups", api.trim()))
                .bearer_auth(token);
            if let Ok(answer) = asked.send().await {
                break answer.json::<serde_json::Value>().await.unwrap();
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

#[cfg(target_os = "macos")]
#[tokio::test(flavor = "multi_thread")]
async fn a_relay_is_kept_running_beside_svrngs_on_a_mac() {
    let site = site(None);
    let port = host(site.path().to_path_buf()).await;
    let home = tempfile::tempdir().unwrap();
    let page = format!("http://127.0.0.1:{port}/");
    let more = " --relay relay.example.org --open";
    let out = paste_more(port, &page, more, home.path(), None).await;
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

#[tokio::test(flavor = "multi_thread")]
async fn a_relay_on_linux_prints_its_system_unit_and_installs_none() {
    let site = site(None);
    // The Linux path is read, never run, here.
    as_linux(site.path());
    let port = host(site.path().to_path_buf()).await;
    let home = tempfile::tempdir().unwrap();
    let page = format!("http://127.0.0.1:{port}/");
    let linux = Some("Linux x86_64");
    let out = paste_more(
        port,
        &page,
        " --relay relay.example.org",
        home.path(),
        linux,
    )
    .await;
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

    let wrong = paste_more(
        port,
        &page,
        " --relay relay.example.org --as x",
        home.path(),
        linux,
    )
    .await;
    let err = String::from_utf8_lossy(&wrong.stderr);
    assert_eq!(wrong.status.code(), Some(1), "{err}");
    assert!(
        err.contains("a relay takes --open and nothing else, not --as."),
        "{err}"
    );
}
