// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use iroh::address_lookup::MemoryLookup;
use iroh::{EndpointAddr, SecretKey, TransportAddr};

const N0_DOTTED: [&str; 4] = [
    "https://use1-1.relay.n0.iroh.link./",
    "https://usw1-1.relay.n0.iroh.link./",
    "https://euc1-1.relay.n0.iroh.link./",
    "https://aps1-1.relay.n0.iroh.link./",
];
const N0_DOTLESS: [&str; 4] = [
    "https://use1-1.relay.n0.iroh.link/",
    "https://usw1-1.relay.n0.iroh.link/",
    "https://euc1-1.relay.n0.iroh.link/",
    "https://aps1-1.relay.n0.iroh.link/",
];

fn urls(map: &RelayMap) -> Vec<String> {
    let mut urls: Vec<String> = map
        .urls::<Vec<_>>()
        .iter()
        .map(ToString::to_string)
        .collect();
    urls.sort();
    urls
}

fn sorted(urls: [&str; 4]) -> Vec<String> {
    let mut urls: Vec<String> = urls.iter().map(ToString::to_string).collect();
    urls.sort();
    urls
}

/// n0's four, given / when / then: given n0's relays as iroh spells them,
/// or written dotless, when this computer makes its relay map, then each
/// is dotless, with iroh's QUIC settings for it kept. The control: iroh's
/// own map is the dotted four, so the row reads what iroh ships. Watched
/// failing with [`dotless`] answering its input: the map kept every dot.
#[test]
fn n0s_four_relays_are_made_dotless() {
    let iroh = iroh::RelayMode::Default.relay_map();
    assert_eq!(
        urls(&iroh),
        sorted(N0_DOTTED),
        "control: iroh spells n0's relays with the dot"
    );
    let rows = [
        ("as iroh spells them", iroh.clone()),
        (
            "written dotless",
            N0_DOTLESS
                .iter()
                .map(|u| u.parse::<RelayUrl>().unwrap())
                .collect(),
        ),
    ];
    for (row, given) in rows {
        let then = dotless_map(&given);
        assert_eq!(urls(&then), sorted(N0_DOTLESS), "{row}");
    }
    let made = n0_relays();
    assert_eq!(
        urls(&made),
        sorted(N0_DOTLESS),
        "the map this computer uses"
    );
    for relay in made.relays::<Vec<_>>() {
        let theirs = iroh
            .relays::<Vec<_>>()
            .into_iter()
            .find(|r| dotless(r.url.clone()) == relay.url)
            .unwrap();
        assert_eq!(
            relay.quic, theirs.quic,
            "{}: iroh's QUIC settings kept",
            relay.url
        );
    }
}

/// An advertised address, given / when / then: given the address another
/// computer advertised — its relay n0's as an earlier build spells it,
/// dotless as this one does, or one of its own — when iroh's lookup
/// answers it through [`Dotless`], then its relay is dotless and its other
/// addresses are as they came. Watched failing with [`dotless`] answering
/// its input.
#[tokio::test]
async fn a_lookups_answer_is_read_dotless() {
    let rows = [
        (
            "n0's, as an earlier build advertised it",
            N0_DOTTED[0],
            N0_DOTLESS[0],
        ),
        (
            "n0's, as this build advertises it",
            N0_DOTLESS[0],
            N0_DOTLESS[0],
        ),
        (
            "a relay of its own, dotted",
            "https://relay.example./",
            "https://relay.example/",
        ),
        (
            "a loopback relay",
            "http://127.0.0.1:3340/",
            "http://127.0.0.1:3340/",
        ),
    ];
    for (i, (row, given, then)) in rows.into_iter().enumerate() {
        let id = SecretKey::from_bytes(&[i as u8 + 1; 32]).public();
        let direct: std::net::SocketAddr = "192.0.2.7:4433".parse().unwrap();
        let advertised = EndpointAddr::from_parts(
            id,
            [
                TransportAddr::Relay(given.parse().unwrap()),
                TransportAddr::Ip(direct),
            ],
        );
        let lookup = Dotless(MemoryLookup::from_endpoint_info([advertised]));
        let answer = lookup
            .resolve(id)
            .expect("the lookup knows the key")
            .next()
            .await
            .expect("an answer")
            .expect("not an error");
        let relays: Vec<String> = answer.relay_urls().map(ToString::to_string).collect();
        assert_eq!(relays, vec![then.to_string()], "{row}");
        assert_eq!(
            answer.ip_addrs().collect::<Vec<_>>(),
            vec![&direct],
            "{row}"
        );
    }
}

/// A relay this computer is told — the operator's, a contact's, a dial
/// string's — is read dotless, and one that is no URL is refused as before.
#[test]
fn a_relay_this_computer_is_told_is_read_dotless() {
    assert_eq!(parse(N0_DOTTED[0]).unwrap().to_string(), N0_DOTLESS[0]);
    assert_eq!(parse(N0_DOTLESS[0]).unwrap().to_string(), N0_DOTLESS[0]);
    assert!(parse("not a url").is_err());
}
