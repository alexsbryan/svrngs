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

    /// A one-time code the door's authorize path would mint, for `lend`'s
    /// client and the person, with its redirect, PKCE challenge and nonce:
    /// a test's way to the token route before the door's path exists. The
    /// challenge and method are read as the door reads them
    /// ([`crate::oidc::provider::S256Challenge::parse`]): for a pair the
    /// door refuses, such as a method other than S256, no code is minted,
    /// and what is answered is a value the token route never minted.
    #[allow(clippy::too_many_arguments)]
    pub fn oidc_code(
        &self,
        lend: &str,
        client: &str,
        sub: &str,
        name: &str,
        username: &str,
        group: svrngs_core::token::Group,
        redirect: &str,
        challenge: Option<&str>,
        method: Option<&str>,
        nonce: Option<&str>,
    ) -> String {
        let Ok(challenge) = crate::oidc::provider::S256Challenge::parse(challenge, method) else {
            return "no-code-the-door-refuses-this-pkce-method".into();
        };
        let who = svrngs_core::token::Subject {
            sub: sub.into(),
            name: name.into(),
            username: username.into(),
            group,
        };
        self.asking.oidc.mint(crate::oidc::Code {
            lend: crate::lend::LendName::parse(lend).expect("a test's lend name"),
            client: client.into(),
            who,
            redirect: redirect.into(),
            challenge,
            nonce: nonce.map(Into::into),
        })
    }

    /// Run one round now, beside the interval's, and say what it moved.
    pub async fn round(&self) -> crate::ring_sync::RoundOutcome {
        crate::ring_sync::run_one_round(self.host.as_ref(), &|| {}, &mut Default::default()).await
    }
}

/// A loopback port whose every connection is spliced to a new bi-stream to
/// `to` on `alpn`, dialled from `endpoint` — so as its key: how a test
/// drives raw HTTP, or a socket, at a door. The product has no such port
/// (arch-peer (d): through one, any program on the computer reached a
/// friend's door as its person, `tests/peer.rs`); a test's lives as long as
/// its runtime.
pub async fn splice(endpoint: Endpoint, to: EndpointAddr, alpn: &'static [u8]) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a loopback port");
    let at = listener.local_addr().expect("a bound port");
    tokio::spawn(async move {
        while let Ok((tcp, _)) = listener.accept().await {
            let (endpoint, to) = (endpoint.clone(), to.clone());
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt;
                let Ok(conn) = endpoint.connect(to, alpn).await else {
                    return;
                };
                let Ok((mut send, mut recv)) = conn.open_bi().await else {
                    return;
                };
                let (mut from, mut to) = tcp.into_split();
                // Each way on its own, as the bridge's pump copied: a door
                // that answers 413 and stops reading still has its answer read.
                let up = async {
                    let _ = tokio::io::copy(&mut from, &mut send).await;
                    let _ = send.shutdown().await;
                };
                let down = async {
                    let _ = tokio::io::copy(&mut recv, &mut to).await;
                    let _ = to.shutdown().await;
                };
                tokio::join!(up, down);
            });
        }
    });
    at
}

impl IrohTransport {
    /// A [`splice`] to the member `peer` names on the members' protocol,
    /// from this transport's endpoint, as `http://<port>`; `None` with no
    /// path to it. What a test that drives raw HTTP at a member asks, where
    /// it asked the bridge's `base_url` before arch-peer (d).
    pub async fn spliced(&self, peer: &Contact) -> Option<String> {
        let to = self.endpoint_addr_for(peer)?;
        Some(format!(
            "http://{}",
            splice(self.endpoint.clone(), to, crate::iroh::ALPN).await
        ))
    }
}
