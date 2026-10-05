// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use svrngs_node::link::client_link;
use svrngs_node::SigningKey;

fn issuer() -> PublicKey {
    let key = SigningKey::from_bytes(&[1u8; 32]);
    PublicKey::from_bytes(&key.verifying_key().to_bytes()).unwrap()
}

/// A client's link with no relay is short: at most 90 characters.
#[test]
fn a_clients_link_with_no_relay_is_at_most_90_characters() {
    let link = client_link(&mint(issuer(), None));
    eprintln!("a client's link, {} characters: {link}", link.len());
    assert!(link.len() <= 90, "{} characters: {link}", link.len());
}

/// A minted token spells no word DESIGN §1 keeps from people, in any case,
/// draw after draw.
#[test]
fn a_minted_token_spells_no_kept_word() {
    for _ in 0..10_000 {
        let token = mint(issuer(), None);
        assert!(!token.spells_a_kept_word(), "{}", token.encode());
    }
}
