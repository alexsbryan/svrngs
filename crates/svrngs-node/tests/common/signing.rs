// SPDX-License-Identifier: AGPL-3.0-or-later
//! Signatures a test makes over a page's `SHA256SUMS` with a key of its
//! own, shaped as the release key's (update/signed.rs `verify` reads
//! them): tests/update.rs's, and the upgrade gate's, whose last release
//! pins such a key (crates/svrngs/tests/upgrade.rs).

use ring::digest;
use ring::signature::{Ed25519KeyPair, KeyPair};

/// `body` as one signature packet, in the old format the release key's is.
pub fn framed(body: Vec<u8>) -> Vec<u8> {
    [vec![0x88, u8::try_from(body.len()).unwrap()], body].concat()
}

/// A key of the test's own, never the release key.
pub fn test_key(seed: u8) -> Ed25519KeyPair {
    Ed25519KeyPair::from_seed_unchecked(&[seed; 32]).unwrap()
}

pub fn point(key: &Ed25519KeyPair) -> [u8; 32] {
    key.public_key().as_ref().try_into().unwrap()
}

/// A signature packet made by hand, shaped as the release key's: its
/// fields, and its hashed subpackets as (type, body).
pub struct Packet {
    pub version: u8,
    pub kind: u8,
    pub algorithm: u8,
    pub hash: u8,
    pub hashed: Vec<(u8, Vec<u8>)>,
}

impl Packet {
    /// The release key's shape, made at `created`.
    pub fn at(created: u32) -> Self {
        Self {
            version: 4,
            kind: 0x00,
            algorithm: 22,
            hash: 10,
            hashed: vec![(33, vec![4; 21]), (2, created.to_be_bytes().to_vec())],
        }
    }

    /// The packet over `sums`, by `key`: its digest SHA-256 where it says
    /// 8, else SHA-512, whatever it says.
    pub fn sign(&self, sums: &[u8], key: &Ed25519KeyPair) -> Vec<u8> {
        let mut body = vec![self.version, self.kind, self.algorithm, self.hash];
        let subpackets: Vec<u8> = self
            .hashed
            .iter()
            .flat_map(|(kind, said)| [vec![said.len() as u8 + 1, *kind], said.clone()].concat())
            .collect();
        body.extend(u16::try_from(subpackets.len()).unwrap().to_be_bytes());
        body.extend(subpackets);
        let algorithm = match self.hash {
            8 => &digest::SHA256,
            _ => &digest::SHA512,
        };
        let mut digest = digest::Context::new(algorithm);
        digest.update(sums);
        digest.update(&body);
        digest.update(&[4, 0xFF]);
        digest.update(&u32::try_from(body.len()).unwrap().to_be_bytes());
        let digest = digest.finish();
        let made = key.sign(digest.as_ref());
        // Unhashed: the issuer's key ID, as gpg puts it, read by no one.
        body.extend([0, 10, 9, 16, 0, 0, 0, 0, 0, 0, 0, 0]);
        body.extend(&digest.as_ref()[..2]);
        for half in made.as_ref().chunks(32) {
            let first = half.iter().position(|&b| b != 0).unwrap();
            let bits = (32 - first) * 8 - half[first].leading_zeros() as usize;
            body.extend(u16::try_from(bits).unwrap().to_be_bytes());
            body.extend(&half[first..]);
        }
        framed(body)
    }
}
