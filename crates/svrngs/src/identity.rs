// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-transport/src/identity.rs@e1a43bcd2
//! Node identity keys — the Ed25519 keypair behind
//! [`commonwealth_core::ids::NodePubkey`].
//!
//! The 32-byte seed persisted at `<data_dir>/identity` is, byte for
//! byte, a valid iroh `SecretKey`: when the dial-by-key transport
//! lands, THIS file is the node's transport identity — verifying a
//! peer's key and being able to dial it become the same fact. Until
//! then the pubkey rides along in `MemberRecord` so the trust ring
//! is transport-ready.
//!
//! Persistence mirrors `sovereign-mesh::persist::
//! load_or_generate_self_node_id` exactly: tmp-then-rename, 0600 on
//! Unix.
//!
//! svrngs (DESIGN §8, U2's inventory, the identity.rs row): this is the
//! command's device key, and the key every group on this computer knows it
//! by (`svrngs_node::node`, "One key signs and dials"). Upstream fell back
//! to a fresh key nobody keeps when the file could not be read or the
//! fresh one could not be written; here both refuse, naming the file
//! ([`KeyError`]), so a computer never silently becomes someone new
//! (ARCH 6). Only an absent file mints a key.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use ed25519_dalek::SigningKey;
use svrngs_node::actor_of;

/// File name under the daemon data dir, sibling of `node_id`. (svrngs: the
/// file a refusal names, so it is called what a person may read — DESIGN
/// §1 keeps "node" and "key" from them; upstream's was `node_key`.)
pub const NODE_KEY_FILE: &str = "identity";

fn key_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join(NODE_KEY_FILE)
}

fn load_key(data_dir: &Path) -> std::io::Result<Option<SigningKey>> {
    let path = key_path(data_dir);
    match fs::read(&path) {
        Ok(bytes) => {
            let seed: [u8; 32] = bytes.as_slice().try_into().map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("it is {} bytes long, not 32", bytes.len()),
                )
            })?;
            Ok(Some(SigningKey::from_bytes(&seed)))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

fn save_key(data_dir: &Path, key: &SigningKey) -> std::io::Result<()> {
    fs::create_dir_all(data_dir)?;
    let target = key_path(data_dir);
    let tmp = target.with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(&key.to_bytes())?;
        f.sync_all()?;
    }
    fs::rename(&tmp, &target)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Best-effort: failure doesn't invalidate the write, the
        // file is just possibly group-readable.
        let _ = fs::set_permissions(&target, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// Why this computer has no key to run as.
#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("cannot read {path}, which says who this computer is in its groups: {source}")]
    Unreadable {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("cannot save {path}, which says who this computer is in its groups: {source}")]
    Unsaved {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// This computer's key: the one at `<data_dir>/identity`, or, when there
/// is no file there, a fresh one written there first. A file that cannot
/// be read, and a fresh key that cannot be written, refuse.
#[expect(clippy::disallowed_methods, reason = "draws a device key")]
pub fn load_or_generate_node_key(data_dir: &Path) -> Result<SigningKey, KeyError> {
    match load_key(data_dir) {
        Ok(Some(key)) => {
            tracing::debug!(
                target: "identity",
                node_pubkey = %actor_of(&key),
                "node_key: loaded the persisted key"
            );
            Ok(key)
        }
        Ok(None) => {
            let mut seed = [0u8; 32];
            if let Err(e) = getrandom::fill(&mut seed) {
                // getrandom failing is catastrophic enough that an
                // expect matches NodeId::generate's posture.
                panic!("failed to generate node key entropy: {e}");
            }
            let fresh = SigningKey::from_bytes(&seed);
            save_key(data_dir, &fresh).map_err(|source| {
                tracing::debug!(
                    target: "identity",
                    error = %source,
                    "node_key: refused — a fresh key that cannot be kept is not this computer's"
                );
                KeyError::Unsaved {
                    path: key_path(data_dir),
                    source,
                }
            })?;
            tracing::info!(
                node_pubkey = %actor_of(&fresh),
                "node_key: generated + persisted identity key (first boot)"
            );
            Ok(fresh)
        }
        Err(source) => {
            tracing::debug!(
                target: "identity",
                error = %source,
                "node_key: refused — the persisted key cannot be read, and no new one is made"
            );
            Err(KeyError::Unreadable {
                path: key_path(data_dir),
                source,
            })
        }
    }
}

#[cfg(test)]
#[path = "identity_tests.rs"]
mod tests;
