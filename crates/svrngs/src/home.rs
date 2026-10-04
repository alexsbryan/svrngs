//! The host's home directory, and the one accessor for each path in it
//! (ARCH_PRINCIPLES 8). Groups live in `groups/<id>/`; their keys, the
//! bridge's capability token and the chat's token sit beside them, never
//! inside a group's directory (DESIGN D4: it holds no key). `chat.json` says
//! which channel the bot reads, once `svrngs start` has asked.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use svrngs_core::{Actor, Admit, Body, Genesis, GroupId, Ingest, Journal, Name, Nonce, Role, Sig, admit_message};
use tracing::info;

use crate::bridge::Token;
use crate::store::GroupDir;

/// What the bridge's key is called in the group.
const BRIDGE_NAME: &str = "the chat";

/// The host's two keys in one group: its Person key, which started the group
/// and keeps it, and the Bridge-role key the chat's acts are signed with.
pub struct HostKeys {
    pub person: SigningKey,
    pub bridge: SigningKey,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KeysFile {
    person: String,
    bridge: String,
}

pub struct Home {
    root: PathBuf,
}

/// The chat the bot speaks in: the channel `svrngs start` picked, and whose
/// computer this is, as the greeting says it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chat {
    pub channel: String,
    pub owner: String,
}

impl Home {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// `$SVRNGS_HOME`, else `~/.svrngs`.
    pub fn from_env() -> anyhow::Result<Self> {
        if let Some(h) = std::env::var_os("SVRNGS_HOME") {
            return Ok(Self::new(h));
        }
        let home = std::env::var_os("HOME").context("neither SVRNGS_HOME nor HOME is set")?;
        Ok(Self::new(Path::new(&home).join(".svrngs")))
    }

    fn groups(&self) -> PathBuf {
        self.root.join("groups")
    }

    pub fn group_dir(&self, id: &GroupId) -> PathBuf {
        self.groups().join(id.to_string())
    }

    fn keys_file(&self, id: &GroupId) -> PathBuf {
        self.root.join("keys").join(format!("{id}.json"))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn token_file(&self) -> PathBuf {
        self.root.join("secrets").join("bridge-token")
    }

    /// The Discord bot's token, which the person pasted into `svrngs start`.
    pub fn discord_token_file(&self) -> PathBuf {
        self.root.join("secrets").join("discord-token")
    }

    fn chat_file(&self) -> PathBuf {
        self.root.join("chat.json")
    }

    /// Where the service writes what `svrngs serve` and the bridge log.
    pub fn log_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// The chat, once one is set up; `None` before.
    pub fn chat(&self) -> anyhow::Result<Option<Chat>> {
        let path = self.chat_file();
        match fs::read(&path) {
            Ok(bytes) => Ok(Some(
                serde_json::from_slice(&bytes).with_context(|| format!("{}", path.display()))?,
            )),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("{}", path.display())),
        }
    }

    pub fn set_chat(&self, chat: &Chat) -> anyhow::Result<()> {
        fs::create_dir_all(&self.root)?;
        fs::write(self.chat_file(), serde_json::to_vec_pretty(chat)?)?;
        info!(channel = %chat.channel, "home:set_chat");
        Ok(())
    }

    /// The Discord bot's token, if one was pasted.
    pub fn discord_token(&self) -> anyhow::Result<Option<String>> {
        let path = self.discord_token_file();
        match fs::read_to_string(&path) {
            Ok(t) => Ok(Some(t.trim().to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e).with_context(|| format!("{}", path.display())),
        }
    }

    /// Keep the Discord bot's token, replacing one kept before.
    pub fn set_discord_token(&self, token: &str) -> anyhow::Result<()> {
        let path = self.discord_token_file();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        write_secret(&path, token.trim().as_bytes())?;
        info!("home:set_discord_token");
        Ok(())
    }

    /// E0 keeps one group per host (DESIGN D7 E0); a second is refused by name.
    pub fn the_group(&self) -> anyhow::Result<Option<GroupId>> {
        let Ok(entries) = fs::read_dir(self.groups()) else {
            return Ok(None);
        };
        let mut ids = Vec::new();
        for e in entries {
            let name = e?.file_name().to_string_lossy().into_owned();
            ids.push(GroupId::try_from(name.clone()).map_err(|e| anyhow::anyhow!("{name}: {e}"))?);
        }
        match ids.as_slice() {
            [] => Ok(None),
            [one] => Ok(Some(*one)),
            _ => bail!(
                "{} holds {} groups; this version keeps one",
                self.groups().display(),
                ids.len()
            ),
        }
    }

    /// Start a group: the genesis names the Person key, whose first act admits
    /// the Bridge key, co-signed by it.
    pub fn create_group(&self, name: &str, now: u64) -> anyhow::Result<GroupId> {
        let keys = HostKeys {
            person: SigningKey::from_bytes(&random()?),
            bridge: SigningKey::from_bytes(&random()?),
        };
        let (person, bridge) = (
            Actor::from_key(&keys.person.verifying_key()),
            Actor::from_key(&keys.bridge.verifying_key()),
        );
        let genesis = Genesis {
            key: person,
            nonce: Nonce::from_bytes(random()?),
            name: text(name)?,
        };
        let id = genesis.group_id();
        let mut journal = Journal::new(genesis.clone());
        let bridge_name = text(BRIDGE_NAME)?;
        let cosig = sign(
            &keys.bridge,
            &admit_message(&person, &bridge, Role::Bridge, &bridge_name),
        );
        let admit = Body::Admit(Admit {
            key: bridge,
            role: Role::Bridge,
            name: bridge_name,
            cosig,
        });
        let draft = journal.draft(&person, &admit, now)?;
        let act = draft.clone().seal(sign(&keys.person, draft.id().as_bytes()))?;
        match journal.decide(act.clone(), now) {
            Ingest::Held(h) => journal.apply(h),
            other => bail!("the group's first act was not held: {other:?}"),
        }
        let file = KeysFile {
            person: hex::encode(keys.person.to_bytes()),
            bridge: hex::encode(keys.bridge.to_bytes()),
        };
        write_secret(&self.keys_file(&id), &serde_json::to_vec(&file)?)?;
        GroupDir::create(&self.group_dir(&id), &genesis, &[act])?;
        info!(group = %id, "home:create_group");
        Ok(id)
    }

    pub fn keys(&self, id: &GroupId) -> anyhow::Result<HostKeys> {
        let path = self.keys_file(id);
        let file: KeysFile = serde_json::from_slice(&fs::read(&path).with_context(|| format!("{}", path.display()))?)?;
        let key = |h: &str| -> anyhow::Result<SigningKey> {
            let mut b = [0u8; 32];
            hex::decode_to_slice(h, &mut b).with_context(|| format!("a key in {}", path.display()))?;
            Ok(SigningKey::from_bytes(&b))
        };
        Ok(HostKeys {
            person: key(&file.person)?,
            bridge: key(&file.bridge)?,
        })
    }

    /// The bridge's capability token, minted on first use.
    pub fn token(&self) -> anyhow::Result<Token> {
        let path = self.token_file();
        if !path.exists() {
            write_secret(&path, hex::encode(random()?).as_bytes())?;
            info!(path = %path.display(), "home:mint_token");
        }
        Token::parse(fs::read_to_string(&path)?.trim()).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))
    }
}

fn random() -> anyhow::Result<[u8; 32]> {
    let mut b = [0u8; 32];
    getrandom::getrandom(&mut b).map_err(|e| anyhow::anyhow!("no randomness: {e}"))?;
    Ok(b)
}

pub(crate) fn sign(key: &SigningKey, msg: &[u8]) -> Sig {
    Sig::from_bytes(key.sign(msg).to_bytes())
}

fn text(s: &str) -> anyhow::Result<Name> {
    Name::try_from(s).map_err(anyhow::Error::msg)
}

/// Readable by this user only, and never overwritten.
fn write_secret(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    fs::create_dir_all(path.parent().context("a secret needs a directory")?)?;
    let mut opts = OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    let mut f = opts.open(path).with_context(|| format!("{}", path.display()))?;
    f.write_all(bytes)?;
    f.sync_all()?;
    Ok(())
}
