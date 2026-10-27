//! On a Mac, podman's machine is made and started when it is absent or
//! stopped, sized for what the cards declare, with nothing typed. The plan is
//! read from `podman machine list`; its commands are run against a podman
//! that records them, since making a machine on this host would stop the one
//! that runs here. The machine give makes has Rosetta on; a card that needs
//! it, on a machine with it off, is refused before anything starts.

use give::machine::{Machine, PODMANS_OWN, Plan, ensure_machine, needs_rosetta, plan, read};
use give::podman::Podman;
use give::shelf::Shelf;

const GB: u64 = 1 << 30;

fn machine(running: bool, memory: u64) -> Machine {
    Machine {
        name: "podman-machine-default".into(),
        default: true,
        running,
        memory,
    }
}

#[test]
fn the_plan_follows_what_podman_has() {
    let name = "podman-machine-default".to_string();
    assert_eq!(plan(&[], 0), Plan::Init { memory: PODMANS_OWN });
    assert_eq!(
        plan(&[], 3 * GB),
        Plan::Init {
            memory: PODMANS_OWN + 3 * GB
        }
    );
    assert_eq!(plan(&[machine(true, 24 * GB)], 3 * GB), Plan::Ready);
    assert_eq!(
        plan(&[machine(false, 24 * GB)], 3 * GB),
        Plan::Start { name: name.clone() }
    );
    assert_eq!(
        plan(&[machine(false, 2 * GB)], 3 * GB),
        Plan::Grow {
            name,
            memory: PODMANS_OWN + 3 * GB
        }
    );
    // Growing a running machine would stop what runs in it: the person's.
    assert_eq!(
        plan(&[machine(true, 2 * GB)], 3 * GB),
        Plan::TooSmall {
            has: 2 * GB,
            needs: PODMANS_OWN + 3 * GB
        }
    );
    let other = Machine {
        name: "other".into(),
        default: false,
        running: false,
        memory: GB,
    };
    assert_eq!(
        plan(&[other, machine(true, 8 * GB)], 0),
        Plan::Ready,
        "the default machine is the one podman uses"
    );
}

#[test]
fn podmans_list_is_read_as_it_prints_it() {
    // `podman machine list --format json`, podman 6.0.1 on this Mac, trimmed.
    let json = r#"[{"Name": "podman-machine-default", "Default": true, "Running": true, "VMType": "applehv",
        "CPUs": 8, "Memory": "25769803776", "DiskSize": "128849018880"}]"#;
    assert_eq!(read(json).unwrap(), [machine(true, 24 * GB)]);
    assert!(read("{}").is_err());
}

/// A podman that answers `machine list` with `list` and records every
/// command it is given, and what `CONTAINERS_CONF_OVERRIDE` gives it to read.
fn recording(test: &str, list: &str) -> (Podman, std::path::PathBuf) {
    recording_with(test, list, "[]")
}

/// [`recording`], answering `machine inspect` with `inspect`.
fn recording_with(test: &str, list: &str, inspect: &str) -> (Podman, std::path::PathBuf) {
    let dir =
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("give-machine-{test}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (script, log) = (dir.join("podman"), dir.join("said"));
    let _ = std::fs::remove_file(&log);
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\necho \"$@\" >> '{log}'\n[ -n \"$CONTAINERS_CONF_OVERRIDE\" ] && cat \"$CONTAINERS_CONF_OVERRIDE\" >> '{log}'\n\
             [ \"$1 $2\" = 'machine list' ] && echo '{list}'\n[ \"$1 $2\" = 'machine inspect' ] && echo '{inspect}'\nexit 0\n",
            log = log.display()
        ),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    (Podman::at(script), log)
}

/// The machine give makes has Rosetta on, which podman 5.6 and later leave
/// off: `init` reads it after the person's own containers.conf. Planted,
/// watched failing: `init` run without it.
#[test]
fn an_absent_machine_is_made_and_started_sized_for_the_card_with_rosetta_on() {
    let (podman, log) = recording("absent", "[]");
    ensure_machine(&podman, 3 * GB, true).unwrap();
    let said = std::fs::read_to_string(log).unwrap();
    assert_eq!(
        said,
        "machine list --format json\nmachine init --memory 5120 --now\n[machine]\nrosetta = true\n"
    );
}

/// A card that needs Rosetta, on a machine give did not make that has it
/// off, is refused before the machine or the app is started, in words a
/// person can act on; with it on, the machine starts as before. Planted,
/// watched failing: the check skipped, so the stopped machine was started.
#[test]
fn a_card_that_needs_rosetta_is_refused_where_the_machine_has_it_off() {
    let list = r#"[{"Name": "podman-machine-default", "Default": true, "Running": false, "Memory": "25769803776"}]"#;
    let (podman, log) = recording_with(
        "rosetta-off",
        list,
        r#"[{"Name": "podman-machine-default", "Rosetta": false}]"#,
    );
    let why = ensure_machine(&podman, GB, true).unwrap_err().to_string();
    assert_eq!(
        std::fs::read_to_string(log).unwrap(),
        "machine list --format json\nmachine inspect podman-machine-default\n"
    );
    assert!(why.contains("Turn on Rosetta in Podman Desktop’s settings"), "{why}");

    let (podman, log) = recording_with(
        "rosetta-on",
        list,
        r#"[{"Name": "podman-machine-default", "Rosetta": true}]"#,
    );
    ensure_machine(&podman, GB, true).unwrap();
    assert_eq!(
        std::fs::read_to_string(log).unwrap(),
        "machine list --format json\nmachine inspect podman-machine-default\nmachine start podman-machine-default\n"
    );

    // A card that does not need it never asks.
    let (podman, log) = recording_with("rosetta-unasked", list, "not read");
    ensure_machine(&podman, GB, false).unwrap();
    assert_eq!(
        std::fs::read_to_string(log).unwrap(),
        "machine list --format json\nmachine start podman-machine-default\n"
    );
}

/// Which card needs Rosetta where: Factorio's, which names the amd64 build,
/// on a Mac with Apple silicon alone; Minecraft's, which names none, nowhere.
#[test]
fn only_an_amd64_card_on_apple_silicon_needs_rosetta() {
    let shelf = Shelf::with(&[] as &[&str]).unwrap();
    let (factorio, minecraft) = (shelf.get("factorio").unwrap(), shelf.get("minecraft").unwrap());
    assert!(needs_rosetta(factorio, "macos", "aarch64"));
    assert!(!needs_rosetta(factorio, "macos", "x86_64"));
    assert!(!needs_rosetta(factorio, "linux", "aarch64"), "no machine there");
    assert!(!needs_rosetta(minecraft, "macos", "aarch64"));
}

#[test]
fn a_stopped_small_machine_is_grown_then_started() {
    let list = r#"[{"Name": "podman-machine-default", "Default": true, "Running": false, "Memory": "2147483648"}]"#;
    let (podman, log) = recording("small", list);
    ensure_machine(&podman, GB, false).unwrap();
    let said = std::fs::read_to_string(log).unwrap();
    assert_eq!(
        said,
        "machine list --format json\nmachine set --memory 3072 podman-machine-default\nmachine start podman-machine-default\n"
    );
}

#[test]
fn a_running_small_machine_is_left_running_and_the_person_told() {
    let list = r#"[{"Name": "podman-machine-default", "Default": true, "Running": true, "Memory": "2147483648"}]"#;
    let (podman, log) = recording("running", list);
    let why = ensure_machine(&podman, GB, false).unwrap_err().to_string();
    assert_eq!(std::fs::read_to_string(log).unwrap(), "machine list --format json\n");
    assert!(
        why.contains("gives Podman 2.0 GB, and keeping this needs 3.0 GB"),
        "{why}"
    );
}
