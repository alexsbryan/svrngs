// SPDX-License-Identifier: AGPL-3.0-or-later
//! What a device signs for one write of an app's database in a group, a
//! change set (svrn-docs `rings/current/APP_STATE.md` §4, "Sealed and
//! signed"), and the two names in it that every copy derives alike: its
//! database's site and its schema's hash. Forked from the app-state spike's
//! `svrngs-core/src/state.rs` (svrn-docs
//! `rings/current/app-state-spike/spike.patch:188-233`), its message kept;
//! new are the nonce and the schema in what is signed, and the site derived.
//!
//! **A site is derived, never claimed.** cr-sqlite names each copy's writes
//! by a 16-byte site. Here a site is the hash of the group, the app, the
//! device's key and a nonce the device made with that database ([`site`]).
//! So only that device's signature carries it, and every copy derives it
//! from the set alone, where the spike bound a site to the first device a
//! copy saw claim it. A database made again under the same key, as when
//! its state directory is deleted, makes a new nonce and so a new site.
//! cr-sqlite refuses a copy's own site's rows above its own version
//! (`vendor/cr-sqlite/core/rs/core/src/db_version.rs`, `insert_db_version`),
//! and the new database's versions start at 1 again (svrn-docs
//! `app-state-diligence/b-sync-build-2026-11-22/`, step 0). A phone's worker
//! derives its site here too, its nonce kept beside its count.
//!
//! The changes are bytes the database chose, signed by their hash
//! ([`changes`]), so a set's head, all it signs, checks without them: a copy
//! keeps the highest head it has checked for each site and a member vouches
//! for a mark by one (svrn-docs `rings/current/APP_STATE.md` §4, "What
//! travels"). The rest is framed by length ([`crate::field`]) under a domain
//! of its own, so a set's signature is never an act's, an app's or a pass's,
//! and one lifted into another group, app, database or seq fails. Pure: the
//! key is an argument.

use alloc::string::String;
use alloc::vec::Vec;

use crate::sig::{PREPARED_DOMAIN, SCHEMA_DOMAIN, SET_DOMAIN, SITE_DOMAIN};
use crate::{field, sign_message, verify_hex, ContentHash, SigningKey};

/// What a device makes with each database it keeps, so the database's site
/// is its own ([`site`]).
pub type Nonce = [u8; 16];

/// A database's site, as cr-sqlite names the writes made there.
pub type Site = [u8; 16];

/// A set's place, all of it signed: its group and app; its device's key,
/// as the record spells a signer; the nonce of the database it was written
/// in; its seq there, cr-sqlite's `db_version`; and the hash of the
/// `schema.sql` it was written under ([`schema`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct At<'a> {
    pub group: &'a str,
    pub app: &'a str,
    pub device: &'a str,
    pub nonce: &'a Nonce,
    pub seq: u64,
    pub schema: &'a [u8; 32],
}

impl At<'_> {
    /// What is signed: the domain, then each part, the variable ones
    /// framed, and last the hash of the set's changes ([`changes`]).
    pub fn message(&self, changes: &[u8; 32]) -> Vec<u8> {
        let mut msg = SET_DOMAIN.to_vec();
        for part in [self.group, self.app, self.device] {
            field(&mut msg, part.as_bytes());
        }
        msg.extend_from_slice(self.nonce);
        msg.extend_from_slice(&self.seq.to_be_bytes());
        msg.extend_from_slice(self.schema);
        msg.extend_from_slice(changes);
        msg
    }

    /// `key`'s signature over the set and its `changes`' hash, in hex: the
    /// set's own device's, where `device` is `key`'s.
    pub fn sign(&self, key: &SigningKey, changes: &[u8; 32]) -> String {
        sign_message(key, &self.message(changes))
    }

    /// Whether `sig` is the set's device's over it and its `changes`' hash.
    pub fn check(&self, changes: &[u8; 32], sig: &str) -> bool {
        verify_hex(self.device, &self.message(changes), sig)
    }
}

/// The site of the database `device` made with `nonce` for `app` in
/// `group`: the first 16 bytes of their hash.
pub fn site(group: &str, app: &str, device: &str, nonce: &Nonce) -> Site {
    let mut bytes = SITE_DOMAIN.to_vec();
    for part in [group.as_bytes(), app.as_bytes(), device.as_bytes(), nonce] {
        field(&mut bytes, part);
    }
    let hash = ContentHash::of(&bytes);
    hash.as_bytes()[..16].try_into().expect("a hash is longer")
}

/// What the computer that prepares a phone's write to `app` in `group`
/// signs, its answer's `text` (svrn-docs `rings/current/APP_STATE.md` §4,
/// "Phones"): so the phone's signer takes the changes it checks from a
/// computer of the group, never from the app's own page.
pub fn prepared(group: &str, app: &str, text: &str) -> Vec<u8> {
    let mut msg = PREPARED_DOMAIN.to_vec();
    for part in [group, app, text] {
        field(&mut msg, part.as_bytes());
    }
    msg
}

/// The hash of a set's changes, as its head signs them.
pub fn changes(bytes: &[u8]) -> [u8; 32] {
    *ContentHash::of(bytes).as_bytes()
}

/// The hash of a `schema.sql`'s text, as each set written under it carries it.
pub fn schema(text: &str) -> [u8; 32] {
    *ContentHash::of(&[SCHEMA_DOMAIN, text.as_bytes()].concat()).as_bytes()
}

#[cfg(test)]
#[path = "set_tests.rs"]
mod tests;
