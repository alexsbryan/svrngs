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
/// the issuer's home first, else the group's own first, and beside it the
/// build's first, or the group's own when the first is the build's (review
/// F2: watched red at 569373e5, `[b, a]` for `[b, euc1]`). Planted:
/// `Meeting::of` keeping the build's only when the group names none, and
/// the first two rows failed; and keeping a build's relay in the own tier,
/// and the own-tier assertion failed.
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
        let meeting = Meeting::of(own.clone(), &build, &[]);
        let all: Vec<&Url> = meeting.relays().collect();
        for relay in dotless_build.iter().chain(&own) {
            assert!(all.contains(&relay), "{relay} in the meeting of {own:?}");
        }
    }
    let homed_at_n0 = Meeting::of([dotless_build[0].clone(), a.clone()], &build, &[]);
    assert_eq!(
        homed_at_n0.own(),
        &BTreeSet::from([a.clone()]),
        "a build's relay is never a group's own"
    );
    let mesh = Meeting::of([a.clone(), b.clone()], &build, &[]);
    // The build's first, euc1 in the set's order, whatever the tier (F2).
    let first = &dotless_build[1];
    assert_eq!(mesh.carried(Some(&b)), [b.clone(), first.clone()]);
    assert_eq!(
        mesh.carried(Some(&dotless_build[0])),
        [dotless_build[0].clone(), a.clone()]
    );
    assert_eq!(mesh.carried(None), [a.clone(), first.clone()]);
    assert_eq!(
        Meeting::of([a.clone()], &[], &[]).carried(Some(&a)),
        [a.clone()]
    );
    let plain = Meeting::of([], &build, &[]);
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
/// alone when it does not, whether or not another group it holds names
/// none of its own (KEEP_GOING_AT_HOME 8: a public-only group does not pin
/// a computer to a relay that has gone), and the build's whatever answers
/// only when no group names one; a computer in no group is at the build's.
/// Planted: the own tier preferred while the build's answers, and the first
/// row failed; one unowned group holding the build's tier for all.
#[test]
fn a_group_meets_at_the_builds_fallback_while_it_answers() {
    let url = |u: &str| Url::parse(u).unwrap();
    let (a, n0) = (
        url("http://10.0.0.2:3340/"),
        url("https://use1-1.relay.n0.iroh.link/"),
    );
    let mesh = || Meeting::of([a.clone()], core::slice::from_ref(&n0), &[]);
    for (meeting, answers, tier, held) in [
        (mesh(), true, Tier::Fallback, BTreeSet::from([n0.clone()])),
        (mesh(), false, Tier::Own, BTreeSet::from([a.clone()])),
        (
            mesh().join(Meeting::of([], core::slice::from_ref(&n0), &[])),
            true,
            Tier::Fallback,
            BTreeSet::from([n0.clone()]),
        ),
        (
            mesh().join(Meeting::of([], core::slice::from_ref(&n0), &[])),
            false,
            Tier::Own,
            BTreeSet::from([a.clone()]),
        ),
        (
            Meeting::of([], core::slice::from_ref(&n0), &[]).join(mesh()),
            false,
            Tier::Own,
            BTreeSet::from([a.clone()]),
        ),
        (
            Meeting::of([], core::slice::from_ref(&n0), &[]),
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
        Meeting::of([a.clone()], &[], &[]).decide(true),
        (Tier::Own, BTreeSet::from([a.clone()]))
    );
    assert_eq!(
        Meeting::of([], &[], &[]).decide(true),
        (Tier::Unowned, BTreeSet::new())
    );
}

/// **n0's are the emergency, held only while neither ours nor a group's
/// own answers** (n0-emergency; RELAY_COMMONS 1, "n0's public relays come
/// in only while ours does not answer"): a group naming none of its own
/// meets at the build's while it answers and at n0's when it does not; a
/// group naming its own meets there when the build does not answer, and
/// n0's never, since iroh picks by speed and would choose n0 over it; a
/// build naming no relay of its own has nothing behind it; and a relay of
/// n0's is never a group's own, a link's or a file's (n0-emergency: a link
/// minted before the operator's switch). Planted: the emergency held while
/// ours answers, and the first row failed; the emergency behind a build
/// with no relay, and that row failed.
#[test]
fn n0_s_relays_are_the_emergency_behind_ours() {
    let url = |u: &str| Url::parse(u).unwrap();
    let (a, ours, n0) = (
        url("http://10.0.0.2:3340/"),
        url("https://relay.svrnme.sh/"),
        url("https://use1-1.relay.n0.iroh.link/"),
    );
    let (build, emergency) = (core::slice::from_ref(&ours), core::slice::from_ref(&n0));
    let unowned = Meeting::of([], build, emergency);
    assert_eq!(
        unowned.decide(true),
        (Tier::Unowned, BTreeSet::from([ours.clone()]))
    );
    assert_eq!(
        unowned.decide(false),
        (Tier::Emergency, BTreeSet::from([n0.clone()]))
    );
    let mesh = Meeting::of([a.clone()], build, emergency);
    assert_eq!(
        mesh.decide(true),
        (Tier::Fallback, BTreeSet::from([ours.clone()]))
    );
    assert_eq!(mesh.decide(false), (Tier::Own, BTreeSet::from([a.clone()])));
    assert_eq!(
        Meeting::of([], &[], emergency).decide(false),
        (Tier::Unowned, BTreeSet::new()),
        "nothing stands behind a build with no relay of its own"
    );
    assert_eq!(
        Meeting::of([n0.clone(), a.clone()], build, emergency).own(),
        &BTreeSet::from([a]),
        "a relay of n0's is never a group's own"
    );
}
