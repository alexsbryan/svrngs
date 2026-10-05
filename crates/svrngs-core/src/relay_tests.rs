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
