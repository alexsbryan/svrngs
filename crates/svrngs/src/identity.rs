// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai commonwealth/crates/commonwealth-transport/src/identity.rs@e1a43bcd2
//! Node identity keys — the Ed25519 keypair behind
//! [`commonwealth_core::ids::NodePubkey`].
//!
//! The 32-byte seed persisted at `<data_dir>/node_key` is, byte for
//! byte, a valid iroh `SecretKey`: when the dial-by-key transport
//! lands, THIS file is the node's transport identity — verifying a
//! peer's key and being able to dial it become the same fact. Until
//! then the pubkey rides along in `MemberRecord` so the trust ring
//! is transport-ready.
//!
//! Persistence mirrors `sovereign-mesh::persist::
//! load_or_generate_self_node_id` exactly: tmp-then-rename, 0600 on
//! Unix, graceful fallback to an ephemeral key on I/O errors (the
//! daemon stays usable; identity stability degrades until the file
//! can be written).

use std::fs;
use std::io::Write;
use std::path::Path;

use ed25519_dalek::SigningKey;

/// File name under the daemon data dir, sibling of `node_id`.
pub const NODE_KEY_FILE: &str = "node_key";

/// The pubkey for a signing key, as lowercase hex.
pub fn node_pubkey(key: &SigningKey) -> String {
    hex::encode(key.verifying_key().to_bytes())
}

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
                    format!(
                        "node_key at {} is {} bytes, expected 32",
                        path.display(),
                        bytes.len()
                    ),
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

/// Load-or-generate wrapper with graceful fallback. First boot
/// writes the seed; subsequent boots return the persisted key. On
/// I/O error writing the generated key, returns the fresh key
/// anyway and logs — the daemon is still usable, the node just
/// presents a different pubkey next session.
pub fn load_or_generate_node_key(data_dir: &Path) -> SigningKey {
    match load_key(data_dir) {
        Ok(Some(key)) => key,
        Ok(None) => {
            let mut seed = [0u8; 32];
            if let Err(e) = getrandom::fill(&mut seed) {
                // getrandom failing is catastrophic enough that an
                // expect matches NodeId::generate's posture.
                panic!("failed to generate node key entropy: {e}");
            }
            let fresh = SigningKey::from_bytes(&seed);
            if let Err(e) = save_key(data_dir, &fresh) {
                tracing::warn!(
                    error = %e,
                    data_dir = %data_dir.display(),
                    "node_key persistence failed — daemon will present a \
                     fresh identity key this session"
                );
            } else {
                tracing::info!(
                    node_pubkey = %node_pubkey(&fresh),
                    "node_key: generated + persisted identity key (first boot)"
                );
            }
            fresh
        }
        Err(e) => {
            tracing::warn!(
                error = %e,
                "node_key: failed to load persisted key — using fresh this session"
            );
            let mut seed = [0u8; 32];
            getrandom::fill(&mut seed).expect("failed to generate node key entropy");
            SigningKey::from_bytes(&seed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// covers: FE-3
    #[test]
    fn load_or_generate_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let first = load_or_generate_node_key(dir.path());
        let second = load_or_generate_node_key(dir.path());
        assert_eq!(first.to_bytes(), second.to_bytes(), "same key across boots");
        assert!(dir.path().join(NODE_KEY_FILE).exists());
    }

    #[cfg(unix)]
    #[test]
    fn key_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let _ = load_or_generate_node_key(dir.path());
        let mode = std::fs::metadata(dir.path().join(NODE_KEY_FILE))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn corrupt_key_file_falls_back_to_fresh() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(NODE_KEY_FILE), b"way too short").unwrap();
        // Must not panic; returns a usable (ephemeral) key.
        let _ = load_or_generate_node_key(dir.path());
    }
}
