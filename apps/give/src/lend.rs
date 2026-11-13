//! Lending an app's door through svrngs: `svrngs share` and `svrngs
//! unshare`, the commands a person runs (CHARTER fence 4). Give stores no
//! lend of its own: the svrngs running here lends it from the moment
//! `share` returns, keeps it across its own restarts, and stops when
//! `unshare` does (GIVE.md §7a).

use std::ffi::OsString;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;

use crate::card::Protocol;

#[derive(Debug, thiserror::Error)]
pub enum LendError {
    #[error("svrngs's command could not be run as {program}: {why}")]
    Spawn { program: String, why: String },
    #[error("svrngs {args} exited {code}: {said}")]
    Failed { args: String, code: i32, said: String },
}

/// What runs svrngs's command: the program a person runs, or a test's
/// stand-in that answers as it does (apps/give/tests/fake).
pub trait Runs: std::fmt::Debug + Send + Sync {
    /// `svrngs <args>`, an empty line on its input: its exit code, none
    /// where a signal ended it, and what it said, its output then its
    /// errors.
    fn run(&self, args: &[&str]) -> Result<(Option<i32>, String), LendError>;
}

/// svrngs's program.
#[derive(Debug)]
struct Program(OsString);

impl Runs for Program {
    fn run(&self, args: &[&str]) -> Result<(Option<i32>, String), LendError> {
        let program = self.0.to_string_lossy().to_string();
        let mut child = (Command::new(&self.0).args(args))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| LendError::Spawn {
                program: program.clone(),
                why: e.to_string(),
            })?;
        // `share` asks for the password a service takes, if any: an app give
        // keeps takes none from svrngs, so the answer is an empty line.
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(b"\n");
        }
        let out = child.wait_with_output().map_err(|e| LendError::Spawn {
            program,
            why: e.to_string(),
        })?;
        let said = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        Ok((out.status.code(), said))
    }
}

/// svrngs's command: `svrngs` on the PATH, as a person runs it, or what a
/// test gives it.
#[derive(Debug, Clone)]
pub struct Svrngs {
    runs: Arc<dyn Runs>,
}

impl Default for Svrngs {
    fn default() -> Svrngs {
        Svrngs::by(Arc::new(Program("svrngs".into())))
    }
}

impl Svrngs {
    /// The svrngs command `runs` answers for.
    pub fn by(runs: Arc<dyn Runs>) -> Svrngs {
        Svrngs { runs }
    }

    fn run(&self, args: &[&str]) -> Result<String, LendError> {
        let (code, said) = self.runs.run(args)?;
        tracing::debug!(?args, code, said, "svrngs: ran");
        if code != Some(0) {
            return Err(LendError::Failed {
                args: args.join(" "),
                code: code.unwrap_or(-1),
                said: said.trim().to_string(),
            });
        }
        Ok(said)
    }

    /// Lend what listens on this computer's `port` as `name` to `group`, by
    /// its id, the one group that keeps it: a page for an HTTP door, a port
    /// as it is for a TCP or UDP one.
    pub fn share(&self, port: u16, name: &str, door: Protocol, group: &str) -> Result<(), LendError> {
        let port = port.to_string();
        let mut args = vec![
            "share",
            port.as_str(),
            "--name",
            name,
            "--program",
            name,
            "--group",
            group,
        ];
        match door {
            Protocol::Http => {}
            Protocol::Tcp => args.push("--tcp"),
            Protocol::Udp => args.push("--udp"),
        }
        self.run(&args).map(drop)
    }

    /// Stop lending `name`.
    pub fn unshare(&self, name: &str) -> Result<(), LendError> {
        self.run(&["unshare", name]).map(drop)
    }
}
