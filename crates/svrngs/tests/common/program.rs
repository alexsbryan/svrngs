// SPDX-License-Identifier: AGPL-3.0-or-later
//! The program itself, as a person runs it: started over a data dir, its
//! output read line by line as it comes. Moved from tests/share.rs,
//! behaviour kept, so other tests can run the command against a svrngs
//! already running (tests/join_by_link.rs).

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command as Process, Stdio};
use std::time::Duration;

/// The program, its output read line by line as it comes.
pub struct Program {
    pub child: Child,
    lines: std::sync::mpsc::Receiver<String>,
    pub seen: Vec<String>,
}

impl Program {
    /// `svrngs <args>` over `dir`, tracing everything to `log`, with `input`
    /// on its standard input. (Allowed dead: this file is included by each
    /// test binary whole, as the browser harness is, and not each one runs
    /// the program without a home of its own — pages/main.rs's allow.)
    #[allow(dead_code)]
    pub fn start(dir: &Path, log: &Path, args: &[&str], input: &str) -> Self {
        Self::at(dir, log, args, input, None)
    }

    /// [`Program::start`], with `$HOME` set to `home` when one is given, so
    /// a group's folder is placed there, not in the machine's own home.
    pub fn at(dir: &Path, log: &Path, args: &[&str], input: &str, home: Option<&Path>) -> Self {
        let mut asked = Process::new(env!("CARGO_BIN_EXE_svrngs"));
        let mut asked = asked
            .args(args)
            .env("SVRNGS_DIR", dir)
            .env("RUST_LOG", "trace");
        if let Some(home) = home {
            asked = asked.env("HOME", home);
        }
        let mut child = asked
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(std::fs::File::create(log).unwrap())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let out = child.stdout.take().unwrap();
        let (tx, lines) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    return;
                }
            }
        });
        Program {
            child,
            lines,
            seen: Vec::new(),
        }
    }

    /// Read its output until a line holds `text`.
    pub fn until(&mut self, text: &str) {
        while !self.seen.iter().any(|l| l.contains(text)) {
            match self.lines.recv_timeout(Duration::from_secs(30)) {
                Ok(l) => self.seen.push(l),
                Err(_) => panic!("no {text:?} in {:?}", self.seen),
            }
        }
    }

    /// Wait for it to end by itself; its exit code and whole output.
    pub fn ends(mut self) -> (i32, Vec<String>) {
        let status = self.child.wait().unwrap();
        while let Ok(l) = self.lines.recv_timeout(Duration::from_secs(5)) {
            self.seen.push(l);
        }
        (status.code().unwrap_or(-1), std::mem::take(&mut self.seen))
    }
}

/// Dropped, the program is stopped: a test that fails before it stops one
/// would otherwise leave it running, orphaned, holding its port.
impl Drop for Program {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
