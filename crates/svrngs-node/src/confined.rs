// SPDX-License-Identifier: AGPL-3.0-or-later
//! A synced folder's filesystem authority. cap-std resolves every parent beneath
//! the opened directory, including across symlinks; ambient export paths cannot.
use cap_std::fs::Dir;
use std::{io, path::Path, time::SystemTime};

pub(super) struct Root(Dir);

impl Root {
    pub(super) fn open(path: &Path) -> io::Result<Self> {
        Dir::open_ambient_dir(path, cap_std::ambient_authority())
            .map(Self)
            .inspect_err(|e| tracing::debug!(target: "folder", path = %path.display(), error = %e, "folder: refused — the synced folder could not be opened as its own root"))
    }

    pub(super) fn metadata(&self, path: &Path) -> io::Result<(u64, SystemTime)> {
        let meta = (self.0.open(path).and_then(|f| f.metadata())).inspect_err(|e| {
            tracing::debug!(target: "folder", path = %path.display(), error = %e, "folder: not read beneath the folder's root: absent, unreadable, or a path leading out of it");
        })?;
        if !meta.is_file() {
            tracing::debug!(target: "folder", path = %path.display(), "folder: refused — not a regular file");
            return Err(io::Error::other("not a regular file"));
        }
        Ok((meta.len(), meta.modified()?.into_std()))
    }

    pub(super) fn remove(&self, path: &Path) -> io::Result<()> {
        self.0.remove_file(path).inspect_err(|e| tracing::debug!(target: "folder", path = %path.display(), error = %e, "folder: refused — not removed beneath the folder's root"))
    }

    pub(super) fn unchanged(&self, path: &Path, expected: Option<(u64, SystemTime)>) -> bool {
        match expected {
            Some(was) => self.metadata(path).is_ok_and(|now| now == was),
            None => self
                .0
                .symlink_metadata(path)
                .is_err_and(|e| e.kind() == io::ErrorKind::NotFound),
        }
    }

    pub(super) fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let parent = path.parent().unwrap_or(Path::new(""));
        (self.0.create_dir_all(parent))
            .and_then(|()| self.0.write(path, bytes))
            .inspect_err(|e| tracing::debug!(target: "folder", path = %path.display(), error = %e, "folder: refused — not written beneath the folder's root"))
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "the filesystem adapter draws an unguessable partial file name"
    )]
    pub(super) async fn copy(
        &self,
        path: &Path,
        reader: impl tokio::io::AsyncRead + Unpin,
        expected: Option<(u64, SystemTime)>,
        size: u64,
    ) -> io::Result<()> {
        let parent = path.parent().unwrap_or(Path::new(""));
        self.0.create_dir_all(parent)?;
        let mut nonce = [0; 16];
        getrandom::fill(&mut nonce).map_err(io::Error::other)?;
        let partial = parent.join(format!(".svrngs-{}.partial", hex::encode(nonce)));
        let mut options = cap_std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let file = self.0.open_with(&partial, &options)?;
        let mut file = tokio::fs::File::from_std(file.into_std());
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut reader =
            tokio::io::BufReader::with_capacity(1024 * 1024, reader.take(size.saturating_add(1)));
        let copied = tokio::io::copy_buf(&mut reader, &mut file).await;
        let wrote = match copied {
            Ok(n) if n == size => match file.flush().await {
                Ok(()) => file.sync_all().await,
                Err(e) => Err(e),
            },
            Ok(n) => {
                tracing::debug!(target: "folder", path = %path.display(), size, got = n, "folder: refused — the blob's size is not its record's, so the partial is dropped");
                Err(io::Error::other(
                    "the blob's size does not match its record",
                ))
            }
            Err(e) => Err(e),
        };
        drop(file);
        let wrote = wrote.and_then(|()| {
            if self.unchanged(path, expected) {
                self.0.rename(&partial, &self.0, path)
            } else {
                tracing::debug!(target: "folder", path = %path.display(), "folder: refused — the file changed during its fetch, so it is kept as the person left it");
                Err(io::Error::other("the file changed during its fetch"))
            }
        });
        if let Err(e) = &wrote {
            tracing::debug!(target: "folder", path = %path.display(), error = %e, "folder: the fetched file not placed beneath the folder's root; its partial removed");
            let _ = self.0.remove_file(&partial);
        }
        wrote
    }
}

#[cfg(test)]
#[path = "confined_tests.rs"]
mod tests;
