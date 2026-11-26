//! What a person gives Keep it running, checked before anything is written
//! (src/given.rs): each refusal by the plain words it says, against real
//! files in a scratch folder. No systemd and no launchd here.

use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use svrngs_keeper::given::{self, Account, Given, Refusal};

fn mode(path: &Path, mode: u32) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

/// A scratch folder only this account can change, with a program `serve`
/// in `bin/` and a folder `app/` to start it in.
struct Scratch {
    _dir: tempfile::TempDir,
    root: PathBuf,
    account: Account,
}

impl Scratch {
    fn new() -> Scratch {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        for sub in ["bin", "app"] {
            std::fs::create_dir(root.join(sub)).unwrap();
            mode(&root.join(sub), 0o755);
        }
        std::fs::write(root.join("bin/serve"), "#!/bin/sh\n").unwrap();
        mode(&root.join("bin/serve"), 0o755);
        let uid = std::fs::metadata(&root).unwrap().uid();
        Scratch {
            _dir: dir,
            root,
            account: Account { uid, own_group: None },
        }
    }

    fn path(&self) -> String {
        self.root.join("bin").display().to_string()
    }

    fn check(&self, words: &[&str], folder: &Path, synced: &[PathBuf]) -> Result<Given, Refusal> {
        let words = words.iter().map(|w| w.to_string()).collect();
        given::check(words, folder, synced, &self.path(), &self.account)
    }
}

#[test]
fn the_page_s_line_is_split_as_a_shell_would_and_tilde_is_home() {
    let home = Path::new("/home/ada");
    let (words, folder) = given::from_line("serve --model '~/a b.gguf' ~/x ~", "~/app", home, 8102).unwrap();
    assert_eq!(
        words,
        ["serve", "--model", "/home/ada/a b.gguf", "/home/ada/x", "/home/ada"]
    );
    assert_eq!(folder, Path::new("/home/ada/app"));
    let (_, folder) = given::from_line("serve", " ~ ", home, 8102).unwrap();
    assert_eq!(folder, home);
}

#[test]
fn what_only_a_shell_understands_is_refused_and_points_to_the_terminal() {
    let home = Path::new("/home/ada");
    let line = |l: &str| given::from_line(l, "/srv", home, 8102).unwrap_err();
    assert_eq!(line("serve 'open"), Refusal::Quotes);
    assert_eq!(line("  "), Refusal::Empty);
    assert_eq!(line("FOO=1 serve"), Refusal::Setting("FOO=1".into(), 8102));
    // An argument shaped like a setting is an argument.
    assert!(given::from_line("serve FOO=1", "/srv", home, 8102).is_ok());
    for (l, word) in [
        ("serve --home $HOME", "$HOME"),
        ("serve | tee log", "|"),
        ("cd app && serve", "&&"),
        ("cd app; serve", "app;"),
        ("serve > log", ">"),
        ("serve >log", ">log"),
        ("serve `pwd`", "`pwd`"),
    ] {
        let refused = line(l);
        assert_eq!(refused, Refusal::Shell(word.into(), 8102), "{l}");
        assert!(
            refused
                .to_string()
                .contains("put svrngs keep 8102 -- in front of your command"),
            "{refused}"
        );
    }
}

#[test]
fn a_program_found_in_path_is_kept_by_its_whole_path_under_its_own_word() {
    let s = Scratch::new();
    let given = s.check(&["serve", "--port", "8102"], &s.root.join("app"), &[]).unwrap();
    assert_eq!(
        given,
        Given {
            program: s.root.join("bin/serve"),
            argv0: "serve".into(),
            args: vec!["--port".into(), "8102".into()],
            folder: s.root.join("app"),
        }
    );
    assert_eq!(given.shown(), format!("{}/bin/serve --port 8102", s.root.display()));
    // A word with a `/` is found from the folder, as a shell finds it.
    let given = s.check(&["../bin/serve"], &s.root.join("app"), &[]).unwrap();
    assert_eq!(given.argv0, "../bin/serve");
    assert!(given.program.is_absolute());
}

#[test]
fn the_folder_is_whole_there_and_a_folder() {
    let s = Scratch::new();
    let refused = s.check(&["serve"], Path::new("app"), &[]).unwrap_err();
    assert_eq!(refused, Refusal::Relative("app".into()));
    let gone = s.root.join("gone");
    assert_eq!(
        s.check(&["serve"], &gone, &[]).unwrap_err(),
        Refusal::Folder(gone.clone(), "isn't there")
    );
    let file = s.root.join("bin/serve");
    assert_eq!(
        s.check(&["serve"], &file, &[]).unwrap_err(),
        Refusal::Folder(file, "isn't a folder")
    );
}

#[test]
fn a_program_that_is_not_found_or_does_not_run_is_refused() {
    let s = Scratch::new();
    let app = s.root.join("app");
    assert_eq!(
        s.check(&["nowhere"], &app, &[]).unwrap_err(),
        Refusal::NotFound("nowhere".into())
    );
    std::fs::write(app.join("notes.txt"), "").unwrap();
    mode(&app.join("notes.txt"), 0o644);
    assert_eq!(
        s.check(&["./notes.txt"], &app, &[]).unwrap_err(),
        Refusal::NotProgram(app.join("./notes.txt"), "isn't marked to run (chmod +x)")
    );
    assert_eq!(
        s.check(&["./gone"], &app, &[]).unwrap_err(),
        Refusal::NotProgram(app.join("./gone"), "isn't there")
    );
    assert_eq!(
        s.check(&["../bin"], &app, &[]).unwrap_err(),
        Refusal::NotProgram(app.join("../bin"), "isn't a file")
    );
}

#[test]
fn a_folder_or_program_inside_a_synced_folder_is_refused() {
    let s = Scratch::new();
    let synced = s.root.join("svrngs/house");
    std::fs::create_dir_all(&synced).unwrap();
    mode(&s.root.join("svrngs"), 0o755);
    mode(&synced, 0o755);
    let refused = s.check(&["serve"], &synced, std::slice::from_ref(&synced)).unwrap_err();
    assert_eq!(refused, Refusal::Synced(synced.clone(), synced.clone()));
    assert!(refused.to_string().contains("a folder your group syncs"), "{refused}");
    // A link from outside into it is followed and refused there.
    std::fs::write(synced.join("server.py"), "").unwrap();
    mode(&synced.join("server.py"), 0o755);
    symlink(synced.join("server.py"), s.root.join("bin/linked")).unwrap();
    let refused = s.check(&["linked"], &s.root.join("app"), std::slice::from_ref(&synced));
    assert_eq!(
        refused.unwrap_err(),
        Refusal::Synced(synced.join("server.py"), synced.clone())
    );
    // Outside it, the same link's program is kept by the link's path.
    let kept = s.check(&["linked"], &s.root.join("app"), &[]).unwrap();
    assert_eq!(kept.program, s.root.join("bin/linked"));
}

#[test]
fn what_another_account_can_change_is_refused() {
    let s = Scratch::new();
    let app = s.root.join("app");
    mode(&app, 0o777);
    assert_eq!(
        s.check(&["serve"], &app, &[]).unwrap_err(),
        Refusal::Writable(app.clone(), "anyone can write to it")
    );
    // Sticky is enough only for root's own folders, as /tmp is.
    mode(&app, 0o1777);
    assert_eq!(
        s.check(&["serve"], &app, &[]).unwrap_err(),
        Refusal::Writable(app.clone(), "anyone can write to it")
    );
    mode(&app, 0o775);
    let refused = s.check(&["serve"], &app, &[]).unwrap_err();
    assert_eq!(refused, Refusal::Writable(app.clone(), "its group can write to it"));
    assert!(
        refused
            .to_string()
            .contains(&format!("Run chmod go-w {}", app.display())),
        "{refused}"
    );
    // Unless the group is this account's own.
    let gid = std::fs::metadata(&app).unwrap().gid();
    let own = Account {
        own_group: Some(gid),
        ..s.account
    };
    let words = vec!["serve".to_string()];
    assert!(given::check(words, &app, &[], &s.path(), &own).is_ok());
    mode(&app, 0o755);
    // The program's own folder, and the program.
    mode(&s.root.join("bin/serve"), 0o757);
    assert_eq!(
        s.check(&["serve"], &app, &[]).unwrap_err(),
        Refusal::Writable(s.root.join("bin/serve"), "anyone can write to it")
    );
    mode(&s.root.join("bin/serve"), 0o755);
    mode(&s.root.join("bin"), 0o775);
    assert_eq!(
        s.check(&["serve"], &app, &[]).unwrap_err(),
        Refusal::Writable(s.root.join("bin"), "its group can write to it")
    );
    mode(&s.root.join("bin"), 0o755);
    // A link's own folder, though what it points to is sound: whoever can
    // write there can point it elsewhere.
    let links = s.root.join("links");
    std::fs::create_dir(&links).unwrap();
    symlink(s.root.join("bin/serve"), links.join("serve")).unwrap();
    mode(&links, 0o775);
    assert_eq!(
        s.check(&["../links/serve"], &app, &[]).unwrap_err(),
        Refusal::Writable(links.clone(), "its group can write to it")
    );
    mode(&links, 0o755);
    assert!(s.check(&["../links/serve"], &app, &[]).is_ok());
    // Owned by an account that is neither this one nor root.
    let other = Account {
        uid: s.account.uid + 1,
        own_group: None,
    };
    if s.account.uid != 0 {
        let words = vec!["serve".to_string()];
        let refused = given::check(words, &app, &[], &s.path(), &other).unwrap_err();
        assert!(matches!(refused, Refusal::Owner(_)), "{refused:?}");
    }
    assert!(s.check(&["serve"], &app, &[]).is_ok());
}

#[test]
fn an_account_s_own_group_is_named_as_it_is_with_no_one_else_in_it() {
    let passwd = "root:x:0:0::/root:/bin/sh\nada:x:1000:1000::/home/ada:/bin/sh\nbo:x:1001:100::/home/bo:/bin/sh\n";
    let group = "root:x:0:\nada:x:1000:\nusers:x:100:bo,ada\nwheel:x:10:ada\n";
    assert_eq!(given::own_group(passwd, group, 1000), Some(1000));
    assert_eq!(given::own_group(passwd, group, 1001), None, "users is everyone's");
    assert_eq!(given::own_group(passwd, "ada:x:1000:bo\n", 1000), None, "bo is in it");
    assert_eq!(given::own_group(passwd, group, 4242), None, "not listed");
}

fn keeper(args: &[&str]) -> std::process::Command {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_svrngs-keeper"));
    command.args(["run", "test-given"]).args(args);
    command
}

fn free_port() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port().to_string()
}

/// The page's refusals reach it as words to show, not a failure to read.
#[test]
fn the_page_is_told_why_in_its_own_answer() {
    let scratch = tempfile::tempdir().unwrap();
    let port = free_port();
    let got = keeper(&["--port", &port, "--json", "--folder", "~", "--command", "FOO=1 serve"])
        .env("XDG_DATA_HOME", scratch.path())
        .output()
        .unwrap();
    assert!(got.status.success(), "{}", String::from_utf8_lossy(&got.stderr));
    let said: serde_json::Value = serde_json::from_slice(&got.stdout).unwrap();
    assert_eq!(said["kind"], "refused");
    assert_eq!(
        said["why"],
        Refusal::Setting("FOO=1".into(), port.parse().unwrap()).to_string()
    );
    assert_eq!(scratch.path().read_dir().unwrap().count(), 0, "nothing written");
}

/// The terminal route shows what will run and where, and keeps nothing
/// without Enter: not with no terminal, and not when something else is typed.
#[test]
fn the_terminal_route_keeps_nothing_unconfirmed() {
    let scratch = tempfile::tempdir().unwrap();
    let port = free_port();
    let args = ["--port", &port, "--", "/bin/echo", "a b"];
    let got = keeper(&args)
        .env("XDG_DATA_HOME", scratch.path())
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    let said = String::from_utf8_lossy(&got.stderr);
    assert!(
        !got.status.success() && said.contains("there is no terminal here, so nothing was kept"),
        "{said}"
    );
    // In a terminal (`script` gives one), a typed `n` is not Enter.
    let bin = env!("CARGO_BIN_EXE_svrngs-keeper");
    let mut words = vec![bin, "run", "test-given"];
    words.extend(args);
    let mut script = std::process::Command::new("script");
    if cfg!(target_os = "macos") {
        script.args(["-q", "/dev/null"]).args(&words);
    } else {
        script.args(["-qec", &shlex::try_join(words.iter().copied()).unwrap(), "/dev/null"]);
    }
    let mut child = script
        .env("XDG_DATA_HOME", scratch.path())
        .current_dir(scratch.path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    std::io::Write::write_all(child.stdin.as_mut().unwrap(), b"n\n").unwrap();
    let got = child.wait_with_output().unwrap();
    let said = String::from_utf8_lossy(&got.stdout);
    assert!(
        said.contains("svrngs will keep this running as test-given, answering on port"),
        "{said}"
    );
    assert!(said.contains("    /bin/echo 'a b'\r\n"), "{said}");
    let folder = scratch.path().canonicalize().unwrap();
    assert!(said.contains(&format!("    in {}\r\n", folder.display())), "{said}");
    assert!(said.contains("Nothing was kept."), "{said}");
    assert_eq!(std::fs::read_dir(scratch.path()).unwrap().count(), 0, "nothing written");
}
