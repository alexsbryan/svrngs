// SPDX-License-Identifier: AGPL-3.0-or-later
//! Whether a release's `SHA256SUMS` is signed by a key built into svrngs:
//! one pure check, no I/O (DESIGN §7, "A computer updates only from a
//! signed release"). It reads exactly one OpenPGP signature packet (RFC 4880
//! §5.2) of the shape the release key makes, as `gpg --list-packets` shows
//! v1.11.2's: version 4, over a binary document, EdDSA (algorithm 22) over
//! the digest, SHA-512 or SHA-256. Anything else is refused, never skipped.
//! Each refusal is one [`Refusal`], traced where the update raises it.

use ring::digest::{Algorithm, Context, SHA256, SHA512};
use ring::signature::{UnparsedPublicKey, ED25519};

/// The svrngs release key's Ed25519 point, fingerprint
/// 46B4D67C5E6621093BB1AB2B057BF9082AA03CFB. `svrngs-release-key.asc` is
/// its one home; a test reads that file and requires the two equal.
const RELEASE_KEY: &str = "149160e9661a330f0b5215e6379a7211ecb8b494cfc2fd95e77fad5d8aa01348";

/// The keys a release is signed by: the release key, or the one a build
/// pins as 64 hex characters in `SVRNGS_RELEASE_KEY` at build time, so a
/// fork that runs its own page updates from its own signatures. A second
/// entry is for rotation only.
pub const RELEASE_KEYS: &[[u8; 32]] = &[unhex(match option_env!("SVRNGS_RELEASE_KEY") {
    Some(key) => key,
    None => RELEASE_KEY,
})];

/// Sums a key signed, and when: made only by [`verify`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signed {
    created: u32,
}

impl Signed {
    /// When the signature says it was made, in seconds since 1970.
    pub fn created(&self) -> u32 {
        self.created
    }
}

/// Why a signature is refused, one for each way [`verify`] says no.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    #[error("it is not one signature packet")]
    Packet,
    #[error("bytes follow the signature")]
    Trailing,
    #[error("it is not a version 4 Ed25519 signature of a file, by SHA-512 or SHA-256")]
    Shape,
    #[error("it says no time it was made")]
    NoTime,
    #[error("it holds a critical subpacket svrngs does not read, type {0}")]
    Critical(u8),
    #[error("it is not over these sums")]
    Digest,
    #[error("no release key made it")]
    Key,
}

/// `sig`, one signature packet over `sums`, made by one of `keys`.
pub fn verify(sums: &[u8], sig: &[u8], keys: &[[u8; 32]]) -> Result<Signed, Refusal> {
    let (body, rest) = packet(sig)?;
    if !rest.is_empty() {
        return Err(Refusal::Trailing);
    }
    let &[4, 0x00, 22, hash, h0, h1, ..] = body else {
        return Err(Refusal::Shape);
    };
    let algorithm: &Algorithm = match hash {
        10 => &SHA512,
        8 => &SHA256,
        _ => return Err(Refusal::Shape),
    };
    // What is signed: version through the hashed subpackets.
    let (hashed, rest) = split(body, 6 + usize::from(u16::from_be_bytes([h0, h1])))?;
    let created = created(&hashed[6..])?;
    let (unhashed, rest) = number(rest, 2)?;
    let (_, rest) = split(rest, unhashed)?;
    let (prefix, rest) = split(rest, 2)?;
    let (r, rest) = mpi(rest)?;
    let (s, rest) = mpi(rest)?;
    if !rest.is_empty() {
        return Err(Refusal::Trailing);
    }
    let mut digest = Context::new(algorithm);
    digest.update(sums);
    digest.update(hashed);
    digest.update(&[4, 0xFF]);
    digest.update(&(hashed.len() as u32).to_be_bytes());
    let digest = digest.finish();
    if digest.as_ref()[..2] != *prefix {
        return Err(Refusal::Digest);
    }
    let mut rs = [0u8; 64];
    rs[32 - r.len()..32].copy_from_slice(r);
    rs[64 - s.len()..].copy_from_slice(s);
    let made = |key: &[u8; 32]| UnparsedPublicKey::new(&ED25519, key).verify(digest.as_ref(), &rs);
    match keys.iter().any(|key| made(key).is_ok()) {
        true => Ok(Signed { created }),
        false => Err(Refusal::Key),
    }
}

/// The body of the signature packet (tag 2) `sig` begins with, and what
/// follows it: its length said whole, in the old format or the new, never
/// in parts nor left to the end.
fn packet(sig: &[u8]) -> Result<(&[u8], &[u8]), Refusal> {
    let (&tag, rest) = sig.split_first().ok_or(Refusal::Packet)?;
    let (len, rest) = match tag {
        0x88 => number(rest, 1)?,
        0x89 => number(rest, 2)?,
        0x8A => number(rest, 4)?,
        0xC2 => length(rest, 224)?,
        _ => return Err(Refusal::Packet),
    };
    split(rest, len)
}

/// The one creation time among the hashed subpackets, refusing a critical
/// one this check does not read (RFC 4880 §5.2.3.1). The issuer, by key ID
/// or fingerprint, is read by trying each key instead.
fn created(mut at: &[u8]) -> Result<u32, Refusal> {
    let mut created = None;
    while !at.is_empty() {
        let (len, rest) = length(at, 255)?;
        let (sub, rest) = split(rest, len)?;
        at = rest;
        let (&kind, body) = sub.split_first().ok_or(Refusal::Packet)?;
        match (kind & 0x7F, body) {
            (2, &[a, b, c, d]) if created.is_none() => {
                created = Some(u32::from_be_bytes([a, b, c, d]))
            }
            (2, _) => return Err(Refusal::Packet),
            (16 | 33, _) => {}
            (other, _) if kind & 0x80 != 0 => return Err(Refusal::Critical(other)),
            _ => {}
        }
    }
    created.ok_or(Refusal::NoTime)
}

/// A new-format length (RFC 4880 §4.2.2, §5.2.3.1) and what follows it: two
/// bytes from 192 up to `parts`, where a packet's partial lengths begin.
fn length(at: &[u8], parts: u8) -> Result<(usize, &[u8]), Refusal> {
    match *at.first().ok_or(Refusal::Packet)? {
        o if o < 192 => number(at, 1),
        o if o < parts => number(at, 2).map(|(n, rest)| (n - (192 << 8) + 192, rest)),
        255 => number(&at[1..], 4),
        _ => Err(Refusal::Packet),
    }
}

/// An MPI of at most 256 bits, R or S: its bytes, and what follows.
fn mpi(at: &[u8]) -> Result<(&[u8], &[u8]), Refusal> {
    let (bits, rest) = number(at, 2)?;
    if bits > 256 {
        return Err(Refusal::Packet);
    }
    split(rest, bits.div_ceil(8))
}

/// The big-endian number in `at`'s first `n` bytes, and what follows.
fn number(at: &[u8], n: usize) -> Result<(usize, &[u8]), Refusal> {
    let (bytes, rest) = split(at, n)?;
    Ok((bytes.iter().fold(0, |a, &b| a << 8 | usize::from(b)), rest))
}

fn split(at: &[u8], n: usize) -> Result<(&[u8], &[u8]), Refusal> {
    at.split_at_checked(n).ok_or(Refusal::Packet)
}

/// 64 hex characters as 32 bytes, at build time: anything else stops the
/// build, so no build pins a key it did not mean.
const fn unhex(hex: &str) -> [u8; 32] {
    let hex = hex.as_bytes();
    assert!(hex.len() == 64, "SVRNGS_RELEASE_KEY is 64 hex characters");
    let mut key = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        key[i] = nibble(hex[2 * i]) << 4 | nibble(hex[2 * i + 1]);
        i += 1;
    }
    key
}

const fn nibble(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => panic!("SVRNGS_RELEASE_KEY is 64 hex characters"),
    }
}
