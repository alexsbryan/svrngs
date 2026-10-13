// SPDX-License-Identifier: AGPL-3.0-or-later
//! What only a test asks of a [`Node`], behind `test-support` as
//! svrngs-core's fixtures are: moved here from node.rs in lean-3, since no
//! path of the product reaches them.

use super::*;

impl Node {
    /// The routes served at [`Node::api_addr`], behind their loopback
    /// check: what a test hands a request whose caller it names.
    pub fn api_routes(&self) -> axum::Router {
        self.routes.clone()
    }

    /// The dial-by-key transport the round dials through; a caller seeds a
    /// member's addresses here when no address lookup can find it.
    pub fn transport(&self) -> &IrohTransport {
        &self.host.transport
    }

    /// The relay this computer dials `actor` through, each way it dials:
    /// a folder's fetch and a lend's origin ([`Host::contact`]), then the
    /// round and the lend asks ([`Host::standing_peers`]) (#127).
    pub fn relays_dialled(&self, actor: &str) -> [Option<String>; 2] {
        let groups = self.host.standings().unwrap_or_default();
        let round = (self.host.standing_peers(&groups).into_iter())
            .find_map(|(a, _, contact)| (a == actor).then_some(contact.relay_url)?);
        [self.host.contact(actor).and_then(|c| c.relay_url), round]
    }

    /// Run one round now, beside the interval's, and say what it moved.
    pub async fn round(&self) -> crate::ring_sync::RoundOutcome {
        crate::ring_sync::run_one_round(self.host.as_ref(), &|| {}).await
    }
}
