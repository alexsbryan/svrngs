// SPDX-License-Identifier: AGPL-3.0-or-later
use serde_json::Value;
use svrngs_node::{
    job::Keep,
    lend::{LendName, Raw},
    plan::*,
    share::Pid,
};

fn facts() -> (Nature, Terms) {
    (
        Nature {
            name: LendName::parse("Coder").unwrap(),
            port: 8101_u16.try_into().unwrap(),
            platform: Platform::Linux,
            raw: None,
            runs: Runs::ByHand,
            pid: Some(Pid {
                pid: 41,
                start: 197,
            }),
            binding: "observed".into(),
        },
        Terms {
            groups: [("house-id".into(), "House".into())].into(),
            limit: Some(2_u16.try_into().unwrap()),
            availability: Availability::Keep,
        },
    )
}

fn fold(events: &[Event], state: State) -> State {
    events.iter().cloned().fold(state, step)
}

fn observation(nature: Nature, terms: Terms, app: Return, door: Return) -> [Event; 4] {
    [
        Event::Nature(nature),
        Event::Starts { app, door },
        Event::Terms(terms),
        Event::Revision(13),
    ]
}

#[test]
fn p1_every_prefix_replays_and_material_changes_invalidate_the_token() {
    let (nature, terms) = facts();
    let mut events = observation(
        nature.clone(),
        terms.clone(),
        Return::Startup,
        Return::Startup,
    )
    .to_vec();
    events.extend([
        Event::Job(Job::Starting),
        Event::Done(Done::NotPublished {
            why: "disk full".into(),
        }),
    ]);
    let whole = fold(&events, State::default());
    assert_eq!(
        whole.report,
        Some(Report::Done(Done::NotPublished {
            why: "disk full".into()
        }))
    );
    let shown = plan(&whole).unwrap();
    for split in 0..=events.len() {
        let prefix = fold(&events[..split], State::default());
        assert_eq!(plan(&prefix).is_some(), split >= 4);
        let resumed = fold(&events[split..], prefix);
        assert_eq!(resumed, whole);
        assert_eq!(plan(&resumed).unwrap(), shown);
    }

    let mut changed = vec![
        ("revision", Event::Revision(14)),
        (
            "return",
            Event::Starts {
                app: Return::Startup,
                door: Return::SignIn,
            },
        ),
    ];
    for (label, changed_nature) in [
        (
            "pid start",
            Nature {
                pid: Some(Pid {
                    pid: 41,
                    start: 198,
                }),
                ..nature.clone()
            },
        ),
        (
            "deployment",
            Nature {
                runs: Runs::Transient,
                ..nature.clone()
            },
        ),
        (
            "target",
            Nature {
                port: 8102_u16.try_into().unwrap(),
                ..nature.clone()
            },
        ),
        (
            "binding",
            Nature {
                binding: "new-socket".into(),
                ..nature.clone()
            },
        ),
    ] {
        changed.push((label, Event::Nature(changed_nature)));
    }
    changed.extend([
        (
            "audience id",
            Event::Terms(Terms {
                groups: [("different-id".into(), "House".into())].into(),
                ..terms.clone()
            }),
        ),
        (
            "limit",
            Event::Terms(Terms {
                limit: None,
                ..terms.clone()
            }),
        ),
        (
            "choice",
            Event::Terms(Terms {
                availability: Availability::ThisRun,
                ..terms.clone()
            }),
        ),
    ]);
    for (label, event) in changed {
        let current = step(whole.clone(), event);
        assert_ne!(plan(&current).unwrap().token, shown.token, "{label}");
    }
    let service = |files: &str| {
        Event::Nature(Nature {
            runs: Runs::OwnService(Keep {
                unit: "coder.service".into(),
                files: files.into(),
            }),
            ..nature.clone()
        })
    };
    let before = step(whole.clone(), service("v1"));
    let after = step(whole.clone(), service("v2"));
    assert_ne!(plan(&before).unwrap().token, plan(&after).unwrap().token);

    let peers = [
        ("house-id".into(), "House".into()),
        ("studio-id".into(), "Studio".into()),
    ];
    let select = |groups| {
        step(
            whole.clone(),
            Event::Terms(Terms {
                groups,
                ..terms.clone()
            }),
        )
    };
    let first = select(peers.iter().cloned().collect());
    let second = select(peers.into_iter().rev().collect());
    assert_eq!(plan(&first), plan(&second));
    assert!(plan(&select(Default::default())).is_none());
}

#[test]
fn p2_host_product_matches_the_written_plans_and_words() {
    use Return::*;
    let (base, terms) = facts();
    let cases: Value = serde_json::from_str(include_str!("../src/plan/cases.json")).unwrap();
    let returns = [Startup, AfterUnlock, SignIn, ThisRun, Unknown];
    let runs = [
        Runs::ByHand,
        Runs::Container {
            pod: "coder".into(),
        },
        Runs::Kept(Keep {
            unit: "coder".into(),
            files: "v1".into(),
        }),
        Runs::OwnService(Keep {
            unit: "coder.service".into(),
            files: "v1".into(),
        }),
        Runs::OtherAccount("other".into()),
        Runs::Transient,
    ];
    let mut checked = 0;
    for (host, platform) in [("linux", Platform::Linux), ("mac", Platform::Mac)] {
        for raw in [None, Some(Raw::Tcp), Some(Raw::Udp)] {
            for (kind, runs) in runs.iter().enumerate() {
                for (a, &app) in returns.iter().enumerate() {
                    for (d, &door) in returns.iter().enumerate() {
                        for availability in [Availability::Keep, Availability::ThisRun] {
                            let nature = Nature {
                                platform,
                                raw,
                                runs: runs.clone(),
                                ..base.clone()
                            };
                            let terms = Terms {
                                availability,
                                ..terms.clone()
                            };
                            let state =
                                fold(&observation(nature, terms, app, door), State::default());
                            let plan = plan(&state).unwrap();
                            let deployment = cases["deployments"][kind].as_str().unwrap();
                            let point = cases["returns"][a][d].as_str().unwrap();
                            let (method, arrival, why) =
                                match (availability, raw, deployment, point) {
                                    (Availability::ThisRun, _, _, _) => ("run", "run", None),
                                    (_, Some(Raw::Udp), _, _) => ("run", "run", Some("udp")),
                                    (_, _, "reuse", "boot" | "unlock" | "login") => {
                                        ("reuse", if host == "mac" { "login" } else { point }, None)
                                    }
                                    (_, _, "reuse", reason) => (
                                        "run",
                                        "run",
                                        Some(if reason == "run" {
                                            if a >= d {
                                                "app"
                                            } else {
                                                "door"
                                            }
                                        } else {
                                            reason
                                        }),
                                    ),
                                    (_, _, reason, _) => ("run", "run", Some(reason)),
                                };
                            let context = (host, raw, kind, app, door, availability);
                            assert_eq!(
                                plan.when,
                                match arrival {
                                    "boot" => Startup,
                                    "unlock" => AfterUnlock,
                                    "login" => SignIn,
                                    "run" => ThisRun,
                                    _ => unreachable!(),
                                },
                                "{context:?}"
                            );
                            let words = plan.words();
                            assert_eq!(words.promise, cases["promise"][arrival], "{context:?}");
                            assert_eq!(words.impact, cases["impact"][method], "{context:?}");
                            assert_eq!(words.button, cases["button"][method], "{context:?}");
                            assert_eq!(words.reason, why.map(|key| format!("svrngs can’t verify that sharing will come back by itself with this setup: {}.", cases["reason"][key].as_str().unwrap())), "{context:?}");
                            match (method, &plan.method) {
                                ("reuse", Method::Reuse(_)) | ("run", Method::ThisRun) => (),
                                _ => panic!("wrong method: {context:?}"),
                            }
                            assert_eq!(plan.if_rev, 13);
                            checked += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(checked, 1800);
    println!(
        "P2: {checked} host combinations, including the rehearsal hosts and the passphrase disk"
    );
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "the local job's own listener, not a share dial"
)]
fn w2_failed_publication_leaves_the_listener_running_and_review_reuses_it() {
    use svrngs_node::lend::{self, Audience, Credentials, Lend};
    let dir = tempfile::tempdir().unwrap();
    let (mut nature, terms) = facts();
    let observe = |nature: Nature| {
        let rev = lend::revision(dir.path(), &nature.name).unwrap();
        let mut observed = observation(nature, terms.clone(), Return::Startup, Return::Startup);
        observed[3] = Event::Revision(rev);
        fold(&observed, State::default())
    };
    // Stand in for an already-running service with a scratch listener.
    let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
    let addr = listener.local_addr().unwrap();
    nature.port = addr.port().try_into().unwrap();
    let template = Lend::carried("coder", "house").unwrap();
    let keep = Keep {
        unit: "coder.service".into(),
        files: "verified".into(),
    };
    nature.runs = Runs::Kept(keep.clone());
    let before = observe(nature.clone());
    let first = plan(&before).unwrap();
    assert!(matches!(first.method, Method::Reuse(_)));
    assert!(
        !dir.path().join("locks").exists(),
        "preview creates no files"
    );
    let approved = eudoxia_community::binding::Approved {
        at: addr.ip(),
        boot: Some(9),
        sockets: [23].into(),
        cgroups: [42].into(),
        processes: [].into(),
        held: false,
    };
    let granted = (approved.clone(), Some(keep.clone()));

    // Obstruct the declaration writer, after the revision can be reserved.
    let partial = lend::lends_dir(dir.path()).join(format!(".{}.json.partial", nature.name));
    std::fs::create_dir_all(&partial).unwrap();
    let execute = |token: &str| {
        run::confirm(
            token,
            || (observe(nature.clone()), ()),
            |(), ready| {
                run::publish(
                    dir.path(),
                    ready,
                    template.clone(),
                    granted.clone(),
                    Credentials::Keep,
                )
            },
        )
    };
    let run::Confirmation::Finished(result, ()) = execute(&first.token) else {
        panic!("initial confirmation did not finish");
    };
    assert!(lend::read(dir.path(), &nature.name).unwrap().is_none());
    assert!(lend::all(dir.path()).unwrap().is_empty());
    let words = plan(&result)
        .unwrap()
        .report_words(result.report.as_ref().unwrap())
        .unwrap();
    assert!(
        words
            .text
            .starts_with("Running here; not shared. The share couldn’t be saved:")
            && words.text.contains(".json.partial"),
        "{words:?}"
    );
    assert_eq!(words.button.as_deref(), Some("Review sharing"));
    println!("{}", words.text);

    std::fs::remove_dir(&partial).unwrap();
    let reviewed = observe(nature.clone());
    let retry = plan(&reviewed).unwrap();
    assert!(matches!(retry.method, Method::Reuse(_)));
    let run::Confirmation::Finished(result, ()) = execute(&retry.token) else {
        panic!("retry did not finish");
    };
    let words = plan(&result)
        .unwrap()
        .report_words(result.report.as_ref().unwrap())
        .unwrap();
    assert!(words.text.starts_with("Shared with House."), "{words:?}");
    println!("{}", words.text);
    assert_eq!(
        lend::revision(dir.path(), &nature.name).unwrap(),
        retry.if_rev + 1
    );
    let saved = lend::read(dir.path(), &nature.name).unwrap().unwrap();
    assert_eq!(saved.port(), Some(nature.port));
    assert_eq!(saved.terms.approved, Some(approved));
    assert_eq!(saved.terms.keep, Some(keep));
    assert_eq!(
        saved.terms.audience,
        Audience::Selected(vec!["house-id".into()])
    );
    assert_eq!(saved.terms.limit, terms.limit);
    // The original listener still accepts a connection after both attempts.
    let _client =
        std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(1)).unwrap();
    listener.set_nonblocking(true).unwrap();
    assert!(listener.accept().is_ok());
}

#[test]
fn p3_changed_revision_requests_review_before_publishing() {
    let (nature, terms) = facts();
    let reviewed = observation(nature, terms, Return::Startup, Return::Startup);
    let original = fold(&reviewed, State::default());
    let current = step(original.clone(), Event::Revision(14));
    let result = run::confirm(
        &plan(&original).unwrap().token,
        || (current.clone(), ()),
        |(), _| {
            panic!("a stale review called an effect owner");
        },
    );
    let run::Confirmation::ReviewAgain(review, ()) = result else {
        panic!("a stale confirmation did not request another review");
    };
    assert_eq!(plan(&review).unwrap(), plan(&current).unwrap());
}
