// SPDX-License-Identifier: AGPL-3.0-or-later
//! Every relay URL this computer makes or reads, without the DNS root's
//! trailing dot (u4-9; svrngs-core's [`svrngs_core::relay`] says why: WebKit
//! refuses TLS to a host so spelled, and iroh spells n0's relays so).
//! Each enters through one of these: n0's relays as iroh lists them
//! ([`n0_relays`]), a relay this computer is told — the operator's, a
//! contact's, a dial string's ([`parse`]) — and an address iroh's lookup
//! answers ([`DotlessLookup`]). A link's relay is read dotless by
//! svrngs-core's token. This computer's own relay is one of n0's or the
//! operator's, so the address it advertises is dotless too. The page
//! compiles this same file (`web/src/lib.rs`, by `#[path]`), so a phone
//! reads n0's relays and a lookup's answers as this computer does; it uses
//! only iroh, n0-future and svrngs-core, which both builds have.

use iroh::address_lookup::{
    AddressLookup, AddressLookupBuilder, AddressLookupBuilderError, EndpointData, EndpointInfo,
    Error, Item,
};
use iroh::{Endpoint, EndpointId, RelayMap, RelayUrl, RelayUrlParseError};
use n0_future::boxed::BoxStream;
use n0_future::StreamExt as _;

/// `url` without its host's trailing dot ([`svrngs_core::relay::dotless`]).
pub fn dotless(url: RelayUrl) -> RelayUrl {
    RelayUrl::from(svrngs_core::relay::dotless(url.into()))
}

/// A relay's URL as written, read dotless.
pub fn parse(raw: &str) -> Result<RelayUrl, RelayUrlParseError> {
    raw.parse().map(dotless)
}

/// n0's relays, each as iroh configures it, dotless: the relays a computer
/// with n0's services uses, in place of iroh's own spelling
/// (`iroh-1.0.2/src/defaults.rs:27-33`). iroh's choice of production or
/// staging relays is kept (`iroh::endpoint::default_relay_mode`).
pub fn n0_relays() -> RelayMap {
    dotless_map(&iroh::endpoint::default_relay_mode().relay_map())
}

/// `map`, its relays' URLs dotless and nothing else changed.
pub fn dotless_map(map: &RelayMap) -> RelayMap {
    map.relays::<Vec<_>>()
        .into_iter()
        .map(|relay| {
            let mut relay = (*relay).clone();
            relay.url = dotless(relay.url);
            relay
        })
        .collect()
}

/// An address lookup, built as `B` builds it, whose answers' relays are
/// dotless. What it publishes passes through unchanged: this computer's
/// own relay is dotless already.
#[derive(Debug)]
pub struct DotlessLookup<B>(pub B);

impl<B: AddressLookupBuilder> AddressLookupBuilder for DotlessLookup<B> {
    fn into_address_lookup(
        self,
        endpoint: &Endpoint,
    ) -> Result<impl AddressLookup, AddressLookupBuilderError> {
        Ok(Dotless(self.0.into_address_lookup(endpoint)?))
    }
}

/// A built lookup, its answers' relays made dotless as they arrive.
#[derive(Debug)]
pub struct Dotless<L>(pub L);

impl<L: AddressLookup> AddressLookup for Dotless<L> {
    fn publish(&self, data: &EndpointData) {
        self.0.publish(data);
    }

    fn resolve(&self, endpoint_id: EndpointId) -> Option<BoxStream<Result<Item, Error>>> {
        let answers = self.0.resolve(endpoint_id)?;
        // `Box::pin`, not `.boxed()`: in a browser iroh's stream is not
        // `Send` (n0-future-0.3.2/src/lib.rs:64-72), and the page builds
        // this file too.
        Some(Box::pin(answers.map(|answer| answer.map(dotless_item))))
    }
}

/// `item` with each relay it names dotless, else as it came.
fn dotless_item(item: Item) -> Item {
    let info = item.endpoint_info();
    let mut data = info.data.clone();
    data.clear_relay_urls();
    for relay in info.relay_urls() {
        data.add_relay_url(dotless(relay.clone()));
    }
    tracing::debug!(
        target: "transport",
        peer = %info.endpoint_id.fmt_short(),
        relays = ?data.relay_urls().map(ToString::to_string).collect::<Vec<_>>(),
        "relay: a lookup's answer, its relays read dotless"
    );
    Item::new(
        EndpointInfo::from_parts(info.endpoint_id, data),
        item.provenance(),
        item.last_updated(),
    )
}

#[cfg(test)]
#[path = "relay_tests.rs"]
mod tests;
