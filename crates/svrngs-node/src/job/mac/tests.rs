use super::*;

#[test]
fn only_the_installed_agent_and_the_asking_daemon_match() {
    let path = Path::new("/home/Sam/Library/LaunchAgents/sh.svrnme.svrngs.plist");
    let good = format!(
        "gui/502/sh.svrnme.svrngs = {{\n\tpath = {}\n\tpid = 37\n\tproperties = keepalive | runatload\n}}",
        path.display()
    );
    assert!(registered(&good, path, Some(37)));
    assert!(registered(&good, path, None));
    assert!(!registered(&good, path, Some(38)));
    assert!(!registered(&good, Path::new("/tmp/copied.plist"), None));
    assert!(!registered(
        &good.replace(" | runatload", ""),
        path,
        Some(37)
    ));
}

/// The agent read for the lend, at this test's own process: its grant
/// names the label asked, so what was asked is seen.
#[cfg(target_os = "macos")]
fn this_agent(label: &str) -> Read {
    let me = std::process::id();
    let keep = Keep {
        unit: label.into(),
        files: "plist".into(),
    };
    let root = (me, crate::uid::start(me).expect("this process"));
    Ok(super::super::Now {
        keep,
        scope: Scope::Agent(root),
    })
}

/// **A Mac lend is kept through restarts only to the launch agent
/// svrngs-keeper runs it under, and only when every process approved is
/// that agent's** (`kept`, the one decider for Share and an approval): the
/// agent's root itself and a child of it are; the process that started
/// this test is not, and one of it beside the agent's grants nothing, nor
/// does none approved. A lender who chooses again after each restart, or a
/// UDP share, asks for no agent at all; no agent, or a unit's scope, grants
/// none. Planted, watched failing: `theirs` asking only that a process be
/// approved, never whether it descends from the agent's root (the
/// process beside the agent's, "one outside it", granted).
#[cfg(target_os = "macos")]
#[test]
fn a_mac_lend_is_kept_only_to_its_agent_and_only_for_the_agent_s_processes() {
    use crate::lend::{LendName, Program, Source};
    use std::collections::BTreeSet;
    let process = |pid: u32| (pid, crate::uid::start(pid).expect("a running process"));
    let me = process(std::process::id());
    let mut child = std::process::Command::new("/bin/sleep")
        .arg("30")
        .spawn()
        .unwrap();
    let under = process(child.id());
    let parent = process(std::os::unix::process::parent_id());
    let lend = |source| Lend {
        name: LendName::parse("notes").unwrap(),
        source,
        kind: Program::parse("llama-server").unwrap(),
        terms: Default::default(),
    };
    let port = std::num::NonZeroU16::new(21000).unwrap();
    let (tcp, udp) = (lend(Source::Port(port)), lend(Source::Raw(Raw::Udp, port)));
    let approved = |held: &[(u32, u64)]| Approved {
        boot: None,
        sockets: BTreeSet::new(),
        cgroups: BTreeSet::new(),
        processes: held.iter().copied().collect(),
        at: std::net::Ipv4Addr::LOCALHOST.into(),
        held: false,
    };
    let granted = kept(this_agent, &tcp, &approved(&[me, under]), true);
    assert_eq!(
        granted.map(|k| k.unit).as_deref(),
        Some("svrngs-keeper-notes")
    );
    assert!(
        kept(this_agent, &tcp, &approved(&[me]), true).is_some(),
        "the agent's own"
    );
    assert_eq!(
        kept(this_agent, &tcp, &approved(&[me, parent]), true),
        None,
        "one outside it"
    );
    assert_eq!(
        kept(this_agent, &tcp, &approved(&[parent]), true),
        None,
        "outside it"
    );
    assert_eq!(
        kept(this_agent, &tcp, &approved(&[]), true),
        None,
        "none approved"
    );
    let unasked: fn(&str) -> Read = |label| panic!("{label} asked for");
    assert_eq!(
        kept(unasked, &tcp, &approved(&[me]), false),
        None,
        "asked again after each restart"
    );
    assert_eq!(
        kept(unasked, &udp, &approved(&[me]), true),
        None,
        "a UDP share"
    );
    let gone: fn(&str) -> Read = |_| Err("it isn't running");
    assert_eq!(kept(gone, &tcp, &approved(&[me]), true), None, "no agent");
    let unit: fn(&str) -> Read = |label| {
        let keep = Keep {
            unit: label.into(),
            files: "unit".into(),
        };
        Ok(super::super::Now {
            keep,
            scope: Scope::Cgroups(BTreeSet::new()),
        })
    };
    assert_eq!(
        kept(unit, &tcp, &approved(&[me]), true),
        None,
        "a unit's scope"
    );
    child.kill().unwrap();
    child.wait().unwrap();
}
