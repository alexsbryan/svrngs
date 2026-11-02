//! Every path, including the symlink target, is in a test-owned temporary tree.
use super::*;

#[tokio::test]
async fn reads_writes_and_deletes_cannot_follow_a_parent_outside_the_opened_root() {
    let sandbox = tempfile::tempdir().unwrap();
    let (inside, outside) = (
        sandbox.path().join("inside"),
        sandbox.path().join("outside"),
    );
    std::fs::create_dir(&inside).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("kept"), b"untouched").unwrap();
    std::os::unix::fs::symlink(&outside, inside.join("link")).unwrap();
    let root = Root::open(&inside).unwrap();
    assert!(root
        .copy(Path::new("good"), &b"data"[..], None, 4)
        .await
        .is_ok());
    assert_eq!(std::fs::read(inside.join("good")).unwrap(), b"data");
    assert!(root
        .copy(Path::new("link/new"), &b"data"[..], None, 4)
        .await
        .is_err());
    assert!(root.write(Path::new("link/new"), b"data").is_err());
    assert!(root.metadata(Path::new("link/kept")).is_err());
    assert!(root.remove(Path::new("link/kept")).is_err());
    assert!(!outside.join("new").exists());
    assert_eq!(std::fs::read(outside.join("kept")).unwrap(), b"untouched");
    assert!(root
        .copy(Path::new("bad-size"), &b"data"[..], None, 1)
        .await
        .is_err());
    assert!(!inside.join("bad-size").exists());
}

#[tokio::test]
async fn a_file_changed_while_its_blob_is_read_is_not_replaced() {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };
    struct Edited {
        path: std::path::PathBuf,
        read: bool,
    }
    impl tokio::io::AsyncRead for Edited {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _: &mut Context<'_>,
            out: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            if !self.read {
                std::fs::write(&self.path, b"edited during fetch")?;
                out.put_slice(b"new");
                self.read = true;
            }
            Poll::Ready(Ok(()))
        }
    }
    let sandbox = tempfile::tempdir().unwrap();
    let path = sandbox.path().join("file");
    std::fs::write(&path, b"old").unwrap();
    let root = Root::open(sandbox.path()).unwrap();
    let expected = root.metadata(Path::new("file")).unwrap();
    assert!(root
        .copy(
            Path::new("file"),
            Edited {
                path: path.clone(),
                read: false
            },
            Some(expected),
            3
        )
        .await
        .is_err());
    assert_eq!(std::fs::read(path).unwrap(), b"edited during fetch");
}
