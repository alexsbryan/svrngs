//! The group's directory (DESIGN D4): `genesis.json` and `journal.jsonl`, one
//! signed act per line, append-only, readable with no tool. It holds no key.
//! Only the journal actor writes it.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use svrngs_core::{Genesis, Ingest, Journal, VerifiedAct};
use tracing::{info, warn};

pub const GENESIS: &str = "genesis.json";
pub const JOURNAL: &str = "journal.jsonl";

/// A group's directory, open for appending.
pub struct GroupDir {
    path: PathBuf,
    journal: File,
}

impl GroupDir {
    /// Write a new group's directory: its genesis and its first acts.
    pub fn create(path: &Path, genesis: &Genesis, acts: &[VerifiedAct]) -> anyhow::Result<()> {
        if path.exists() {
            bail!("{} already exists", path.display());
        }
        fs::create_dir_all(path)?;
        fs::write(path.join(GENESIS), serde_json::to_string_pretty(genesis)? + "\n")?;
        let lines: String = acts.iter().map(|a| a.to_line() + "\n").collect();
        fs::write(path.join(JOURNAL), lines)?;
        Ok(())
    }

    /// Read the directory back into a journal, refusing a genesis that does
    /// not hash to the directory's name and any line ingest would not hold.
    pub fn open(path: &Path, now: u64) -> anyhow::Result<(Self, Journal)> {
        let gpath = path.join(GENESIS);
        let genesis: Genesis = serde_json::from_str(&fs::read_to_string(&gpath)?)
            .with_context(|| format!("{} is not a genesis", gpath.display()))?;
        let id = genesis.group_id().to_string();
        if path.file_name().and_then(|n| n.to_str()) != Some(id.as_str()) {
            bail!("{} hashes to group {id}, not to its directory's name", gpath.display());
        }
        let mut journal = Journal::new(genesis);
        let jpath = path.join(JOURNAL);
        for (n, line) in fs::read_to_string(&jpath)?.lines().enumerate() {
            let at = format!("{}:{}", jpath.display(), n + 1);
            let act = VerifiedAct::parse(line).with_context(|| format!("{at} is refused"))?;
            match journal.decide(act, now) {
                Ingest::Held(h) => journal.apply(h),
                Ingest::AlreadyHeld(dup) => warn!(%at, id = ?dup, "journal:duplicate_line"),
                Ingest::OfferAgain(why) => bail!("{at} cannot be held: {why}"),
            }
        }
        info!(group = %id, acts = journal.len(), "journal:open");
        let file = OpenOptions::new().append(true).open(&jpath)?;
        Ok((
            Self {
                path: path.to_path_buf(),
                journal: file,
            },
            journal,
        ))
    }

    /// One line, written whole and synced before the act counts as held.
    pub fn append(&mut self, act: &VerifiedAct) -> io::Result<()> {
        self.journal.write_all((act.to_line() + "\n").as_bytes())?;
        self.journal.sync_data()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
