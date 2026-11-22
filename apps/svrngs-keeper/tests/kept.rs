//! `svrngs-keeper keep`, `list`, `stop`, `start` and `remove`, against a
//! stand-in `systemctl` and `loginctl` that record every argv, as host-kit's
//! own tests drive its install (src/systemd/tests.rs). The real units are
//! proved on a host by `tests/host.sh`.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use svrngs_keeper::kept::{self, KeptError, Places, Row, Running, Starts, Status, Stored};
use svrngs_keeper::systemd::SystemdUser;

const CODER: &str = r#"x-svrngs:
  title: Coder
  about: llama-server with qwen2.5-7b
  door: { port: 8101, protocol: http }
  state: []
  native:
    command: [/usr/bin/llama-server, -m, ~/models/q.gguf, --alias, "50% \"$HOME\" \\n", --port, 8101]
    folder: ~/models
    environment: [HIP_VISIBLE_DEVICES]
"#;

/// The unit `CODER` renders to, every word quoted and escaped.
const CODER_UNIT: &str = r#"[Unit]
Description=coder, kept by svrngs-keeper
StartLimitIntervalSec=60
StartLimitBurst=4

[Service]
WorkingDirectory=%h/models
ExecStart="/usr/bin/llama-server" "-m" "%h/models/q.gguf" "--alias" "50%% \"$$HOME\" \\n" "--port" "8101"
PassEnvironment=HIP_VISIBLE_DEVICES
Restart=on-failure
RestartSec=2
# A program that dies of the SIGTERM a stop sends may exit 143: a stop.
SuccessExitStatus=143

[Install]
WantedBy=default.target
"#;

struct Here {
    _dir: tempfile::TempDir,
    root: PathBuf,
    systemd: SystemdUser,
    places: Places,
}

impl Here {
    fn new() -> Here {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let log = root.join("argv.log");
        let systemctl = root.join("systemctl");
        std::fs::write(
            &systemctl,
            format!(
                "#!/bin/sh\necho \"$@\" >> {log}\n\
                 if [ \"$2\" = show ]; then cat {show}; fi\nexit 0\n",
                log = log.display(),
                show = root.join("show.txt").display()
            ),
        )
        .unwrap();
        let loginctl = root.join("loginctl");
        std::fs::write(&loginctl, "#!/bin/sh\necho Linger=yes\n").unwrap();
        for p in [&systemctl, &loginctl] {
            std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let places = Places::of(&root.join("home"), None, None);
        Here {
            _dir: dir,
            systemd: SystemdUser::at(systemctl, loginctl),
            root,
            places,
        }
    }

    /// Every argv systemctl was given since the last call.
    fn argv(&self) -> Vec<String> {
        let log = self.root.join("argv.log");
        let argv = std::fs::read_to_string(&log).unwrap_or_default();
        let _ = std::fs::remove_file(&log);
        argv.lines().map(str::to_string).collect()
    }

    fn unit(&self) -> PathBuf {
        self.places.units.join("svrngs-keeper-coder.service")
    }

    fn keep(&self, text: &str) -> Result<kept::Kept, KeptError> {
        kept::keep(&self.systemd, &self.places, text, None)
    }
}

fn body(unit: &Path) -> String {
    let text = std::fs::read_to_string(unit).unwrap();
    text.lines().skip(2).map(|l| format!("{l}\n")).collect()
}

#[test]
fn places_follow_xdg_and_default_under_home() {
    let p = Places::of(Path::new("/h"), None, Some(""));
    assert_eq!(p.data, Path::new("/h/.local/share/svrngs-keeper"));
    assert_eq!(p.units, Path::new("/h/.config/systemd/user"));
    let p = Places::of(Path::new("/h"), Some("/d"), Some("/c"));
    assert_eq!(
        (p.data.as_path(), p.units.as_path()),
        (Path::new("/d/svrngs-keeper"), Path::new("/c/systemd/user"))
    );
}

#[test]
fn a_name_is_the_title_made_plain() {
    assert_eq!(kept::name_of("Coder"), "coder");
    assert_eq!(kept::name_of("  Sam's model: 7B!"), "sam-s-model-7b");
}

#[test]
fn keep_stores_the_card_writes_its_unit_and_enables_it_now() {
    let here = Here::new();
    let kept = here.keep(CODER).unwrap();
    assert_eq!(
        (kept.name.as_str(), kept.unit.as_str(), kept.port, &kept.stored),
        ("coder", "svrngs-keeper-coder.service", 8101, &Stored::New)
    );
    assert_eq!(body(&here.unit()), CODER_UNIT);
    let head = std::fs::read_to_string(here.unit()).unwrap();
    let version = blake3::hash(CODER.as_bytes()).to_hex().to_string();
    assert!(
        head.starts_with(&format!(
            "# svrngs-keeper keeps coder from its card, version {version}.\n"
        )),
        "{head}"
    );
    assert_eq!(
        std::fs::read_to_string(here.places.data.join("coder/compose.yaml")).unwrap(),
        CODER
    );
    assert_eq!(
        here.argv(),
        [
            "--user daemon-reload",
            "--user enable --now svrngs-keeper-coder.service"
        ]
    );
}

#[test]
fn keep_twice_resets_nothing() {
    let here = Here::new();
    here.keep(CODER).unwrap();
    here.argv();
    let card = here.places.data.join("coder/compose.yaml");
    let written = std::fs::metadata(&card).unwrap().modified().unwrap();
    let again = here.keep(CODER).unwrap();
    assert_eq!(again.stored, Stored::Same);
    assert_eq!(
        here.argv(),
        [
            "--user daemon-reload",
            "--user enable --now svrngs-keeper-coder.service"
        ],
        "no restart: enable --now leaves a running unit as it is"
    );
    assert_eq!(std::fs::metadata(&card).unwrap().modified().unwrap(), written);
    assert!(!here.places.data.join("coder/previous.yaml").exists());
}

#[test]
fn a_new_card_keeps_the_one_before_and_starts_again_from_it() {
    let here = Here::new();
    here.keep(CODER).unwrap();
    here.argv();
    let changed = CODER.replace("--port, 8101", "--port, 8101, -c, 4096");
    let kept = here.keep(&changed).unwrap();
    let previous = here.places.data.join("coder/previous.yaml");
    assert_eq!(
        kept.stored,
        Stored::Changed {
            previous: previous.clone(),
            restarted: true
        }
    );
    assert_eq!(std::fs::read_to_string(&previous).unwrap(), CODER);
    assert!(body(&here.unit()).contains(r#""--port" "8101" "-c" "4096""#));
    assert_eq!(
        here.argv(),
        [
            "--user daemon-reload",
            "--user enable --now svrngs-keeper-coder.service",
            "--user restart svrngs-keeper-coder.service"
        ]
    );
    // Its words alone changed, and so did its version, which the unit names
    // (the grant that renews its lend binds to it): it starts again.
    let reworded = changed.replace("qwen2.5-7b", "qwen2.5-7b, longer");
    let kept = here.keep(&reworded).unwrap();
    assert_eq!(
        kept.stored,
        Stored::Changed {
            previous,
            restarted: true
        }
    );
}

#[test]
fn keep_refuses_a_card_of_images_and_writes_nothing() {
    let here = Here::new();
    let images = include_str!("../../give/cards/factorio/compose.yaml");
    match here.keep(images) {
        Err(KeptError::NotNative(title)) => assert_eq!(title, "Factorio"),
        other => panic!("expected NotNative, got {other:?}"),
    }
    assert!(!here.places.data.exists() && !here.places.units.exists());
    assert!(here.argv().is_empty());
}

#[test]
fn stop_and_start_last_past_a_boot() {
    let here = Here::new();
    here.keep(CODER).unwrap();
    here.argv();
    assert_eq!(
        kept::stop(&here.systemd, &here.places, "coder").unwrap().as_str(),
        "coder"
    );
    assert_eq!(here.argv(), ["--user disable --now svrngs-keeper-coder.service"]);
    kept::start(&here.systemd, &here.places, "coder").unwrap();
    assert_eq!(here.argv(), ["--user enable --now svrngs-keeper-coder.service"]);
}

#[test]
fn remove_deletes_the_unit_and_leaves_the_card() {
    let here = Here::new();
    here.keep(CODER).unwrap();
    here.argv();
    let dir = kept::remove(&here.systemd, &here.places, "coder").unwrap();
    assert_eq!(dir, here.places.data.join("coder"));
    assert!(!here.unit().exists());
    assert_eq!(std::fs::read_to_string(dir.join("compose.yaml")).unwrap(), CODER);
    assert_eq!(
        here.argv(),
        [
            "--user disable --now svrngs-keeper-coder.service",
            "--user daemon-reload"
        ]
    );
    std::fs::write(here.root.join("show.txt"), "").unwrap();
    let rows = kept::list(&here.systemd, &here.places).unwrap();
    assert!(
        matches!(&rows[..], [Row::Removed { name, .. }] if name == "coder"),
        "{rows:?}"
    );
}

#[test]
fn what_is_not_kept_is_refused_before_systemd_is_asked() {
    let here = Here::new();
    for name in ["coder", "../etc", "Coder"] {
        assert!(matches!(
            kept::stop(&here.systemd, &here.places, name),
            Err(KeptError::NotKept(_))
        ));
    }
    assert!(here.argv().is_empty());
}

#[test]
fn list_reads_each_kept_card_from_systemd() {
    let here = Here::new();
    here.keep(CODER).unwrap();
    std::fs::write(
        here.root.join("show.txt"),
        "LoadState=loaded\nActiveState=active\nResult=success\nNRestarts=1\nUnitFileState=enabled\n\
         ActiveEnterTimestamp=Sun 2026-11-22 09:14:03 PST\n",
    )
    .unwrap();
    here.argv();
    let rows = kept::list(&here.systemd, &here.places).unwrap();
    assert_eq!(
        rows,
        [Row::Kept {
            name: "coder".into(),
            port: 8101,
            status: Status {
                running: Running::Since(Some("09:14".into())),
                starts: Starts::AtBoot,
                restarts: 1
            }
        }]
    );
    assert_eq!(
        rows[0].to_string(),
        "coder   running since 09:14 · starts when this computer starts · 1 restart · port 8101"
    );
    assert_eq!(
        here.argv(),
        [
            "--user show svrngs-keeper-coder.service -p LoadState,ActiveState,Result,NRestarts,UnitFileState,ActiveEnterTimestamp"
        ]
    );
}

#[test]
fn status_reads_every_state_it_names_and_refuses_the_rest() {
    let show = |active: &str, file: &str| {
        format!("LoadState=loaded\nActiveState={active}\nResult=start-limit-hit\nNRestarts=4\nUnitFileState={file}\n")
    };
    let s = kept::status(&show("failed", "enabled"), Ok(false)).unwrap();
    assert_eq!(
        (s.running, s.starts, s.restarts),
        (Running::Failed("start-limit-hit".into()), Starts::AtLogin, 4)
    );
    let s = kept::status(&show("inactive", "disabled"), Ok(true)).unwrap();
    assert_eq!((s.running, s.starts), (Running::Stopped, Starts::ByHand));
    let s = kept::status(&show("activating", "enabled"), Err("no logind".into())).unwrap();
    assert_eq!(
        (s.running, s.starts),
        (Running::Starting, Starts::Enabled("no logind".into()))
    );
    // Never a guess: what systemd did not say is said, not defaulted.
    assert!(kept::status("LoadState=not-found\n", Ok(true)).is_err());
    assert!(kept::status(&show("maintenance", "enabled"), Ok(true)).is_err());
    assert!(kept::status(&show("active", "enabled").replace("NRestarts=4\n", ""), Ok(true)).is_err());
}
