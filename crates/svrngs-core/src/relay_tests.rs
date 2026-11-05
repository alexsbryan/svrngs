// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use alloc::string::ToString;

/// The normaliser, given / when / then: given a relay's URL as it was
/// written, when [`dotless`] reads it, then it is the URL WebKit reaches —
/// n0's four as iroh spells them (`iroh-1.0.2/src/defaults.rs:27-33`) and
/// as a computer of this build spells them, a relay of the operator's, a
/// port kept, and an address left alone. Watched failing with [`dotless`]
/// answering its input: every dotted row kept its dot.
#[test]
fn a_relays_url_loses_its_hosts_trailing_dot_and_nothing_else() {
    let rows = [
        (
            "n0's NA east, as iroh spells it",
            "https://use1-1.relay.n0.iroh.link./",
            "https://use1-1.relay.n0.iroh.link/",
        ),
        (
            "n0's NA west, as iroh spells it",
            "https://usw1-1.relay.n0.iroh.link./",
            "https://usw1-1.relay.n0.iroh.link/",
        ),
        (
            "n0's EU, as iroh spells it",
            "https://euc1-1.relay.n0.iroh.link./",
            "https://euc1-1.relay.n0.iroh.link/",
        ),
        (
            "n0's Asia-Pacific, as iroh spells it",
            "https://aps1-1.relay.n0.iroh.link./",
            "https://aps1-1.relay.n0.iroh.link/",
        ),
        (
            "n0's NA east, dotless already",
            "https://use1-1.relay.n0.iroh.link/",
            "https://use1-1.relay.n0.iroh.link/",
        ),
        (
            "a relay of one's own, dotted",
            "https://relay.example./",
            "https://relay.example/",
        ),
        (
            "a relay of one's own",
            "https://relay.example/",
            "https://relay.example/",
        ),
        (
            "a port, dotted",
            "https://relay.example.:8443/",
            "https://relay.example:8443/",
        ),
        (
            "a loopback relay",
            "http://127.0.0.1:3340/",
            "http://127.0.0.1:3340/",
        ),
        ("an IPv6 relay", "http://[::1]:3340/", "http://[::1]:3340/"),
    ];
    for (row, given, then) in rows {
        let when = dotless(given.parse().unwrap());
        assert_eq!(when.to_string(), then, "{row}");
    }
}

/// **A group never meets without the build's relays** (relay fallbacks;
/// GROUP_RELAYS §7): whatever the record names, none or some, the meeting
/// holds every relay the build gives, read dotless, where a member is
/// dialled, and never among the group's own, as a link from a computer
/// homed at the build's carries one (the run's take 1); and a link carries
/// the issuer's home first, then the group's own, else the build's first
/// alone. Planted: `Meeting::of` keeping the build's only when the group
/// names none, and the first two rows failed; and keeping a build's relay
/// in the own tier, and the own-tier assertion failed.
/// (A meeting written by hand does not compile: `Meeting`'s doc test.)
#[test]
fn a_group_never_meets_without_the_builds_relays() {
    let url = |u: &str| Url::parse(u).unwrap();
    let build = [
        url("https://use1-1.relay.n0.iroh.link./"),
        url("https://euc1-1.relay.n0.iroh.link/"),
    ];
    let (a, b) = (url("http://10.0.0.2:3340/"), url("http://10.0.0.3:3340/"));
    let dotless_build: Vec<Url> = build.iter().cloned().map(dotless).collect();
    for own in [
        vec![],
        vec![a.clone()],
        vec![a.clone(), b.clone(), build[1].clone()],
    ] {
        let meeting = Meeting::of(own.clone(), &build);
        let all: Vec<&Url> = meeting.relays().collect();
        for relay in dotless_build.iter().chain(&own) {
            assert!(all.contains(&relay), "{relay} in the meeting of {own:?}");
        }
    }
    let homed_at_n0 = Meeting::of([dotless_build[0].clone(), a.clone()], &build);
    assert_eq!(
        homed_at_n0.own(),
        &BTreeSet::from([a.clone()]),
        "a build's relay is never a group's own"
    );
    let mesh = Meeting::of([a.clone(), b.clone()], &build);
    assert_eq!(mesh.carried(Some(&b)), [b.clone(), a.clone()]);
    assert_eq!(
        mesh.carried(Some(&dotless_build[0])),
        [dotless_build[0].clone(), a.clone()]
    );
    assert_eq!(mesh.carried(None), [a.clone(), b.clone()]);
    let plain = Meeting::of([], &build);
    assert_eq!(
        plain.carried(Some(&dotless_build[1])),
        [dotless_build[1].clone()]
    );
    assert_eq!(
        plain.carried(Some(&a)),
        [dotless_build[1].clone()],
        "a home not the meeting's is not carried"
    );
    assert_eq!(mesh.join(plain).relays().count(), 4);
}

/// **A group meets at the build's fallback while it answers, and at its
/// own relays when it does not** (relay fallbacks; GROUP_RELAYS §7, the
/// charter's "keep a public fallback reachable while online"): the one
/// decision holds the build's tier alone while it answers, the own tier
/// alone when it does not, and the build's whatever answers when a group
/// joined in names none of its own, one endpoint serving every group; a
/// computer in no group is at the build's. Planted: the own tier preferred
/// while the build's answers, and the first row failed.
#[test]
fn a_group_meets_at_the_builds_fallback_while_it_answers() {
    let url = |u: &str| Url::parse(u).unwrap();
    let (a, n0) = (
        url("http://10.0.0.2:3340/"),
        url("https://use1-1.relay.n0.iroh.link/"),
    );
    let mesh = || Meeting::of([a.clone()], core::slice::from_ref(&n0));
    for (meeting, answers, tier, held) in [
        (mesh(), true, Tier::Fallback, BTreeSet::from([n0.clone()])),
        (mesh(), false, Tier::Own, BTreeSet::from([a.clone()])),
        (
            mesh().join(Meeting::of([], core::slice::from_ref(&n0))),
            true,
            Tier::Unowned,
            BTreeSet::from([n0.clone()]),
        ),
        (
            Meeting::of([], core::slice::from_ref(&n0)),
            true,
            Tier::Unowned,
            BTreeSet::from([n0.clone()]),
        ),
    ] {
        assert_eq!(meeting.decide(answers), (tier, held), "{tier:?}");
    }
    // A build with no relay at all answers nothing: a group's own relays
    // are held, and a group naming none holds none.
    assert_eq!(
        Meeting::of([a.clone()], &[]).decide(true),
        (Tier::Own, BTreeSet::from([a.clone()]))
    );
    assert_eq!(
        Meeting::of([], &[]).decide(true),
        (Tier::Unowned, BTreeSet::new())
    );
}
