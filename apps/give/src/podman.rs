//! podman's command line: the only way give reaches podman (GIVE.md §9,
//! "driving podman's CLI rather than libpod's REST API"). Each call is one
//! short command, run with nothing on its input, so it never waits on a
//! terminal; each is traced at debug with what it answered.

use std::ffi::OsString;
use std::process::{Command, Stdio};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum PodmanError {
    #[error("Podman is not installed on this computer ({0})")]
    Missing(String),
    #[error("`podman {command}` exited {status}: {stderr}")]
    Failed {
        command: String,
        status: i32,
        stderr: String,
    },
}

/// What runs podman's command line: the program itself, or a test's
/// stand-in that answers as podman does (apps/give/tests/fake). The one
/// place a podman command leaves give.
pub trait Runs: std::fmt::Debug + Send + Sync {
    /// `podman <args>`, with `env` added to its environment: its exit
    /// status, its output and its errors, or why it could not be run.
    fn run(&self, args: &[&str], env: &[(OsString, OsString)]) -> Result<(i32, String, String), String>;
}

/// A podman program, run with nothing on its input.
#[derive(Debug)]
struct Program(OsString);

impl Runs for Program {
    fn run(&self, args: &[&str], env: &[(OsString, OsString)]) -> Result<(i32, String, String), String> {
        let out = Command::new(&self.0)
            .args(args)
            .envs(env.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("{}: {e}", self.0.to_string_lossy()))?;
        let status = out.status.code().unwrap_or(-1);
        let (stdout, stderr) = (
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        );
        Ok((status, stdout, stderr))
    }
}

/// The podman give runs: `podman` on the PATH, another program, or what
/// a test gives it, with what it adds to podman's environment.
#[derive(Debug, Clone)]
pub struct Podman {
    runs: Arc<dyn Runs>,
    env: Vec<(OsString, OsString)>,
}

impl Default for Podman {
    fn default() -> Self {
        Podman::at("podman")
    }
}

impl Podman {
    pub fn at(program: impl Into<OsString>) -> Podman {
        Podman::by(Arc::new(Program(program.into())))
    }

    /// The podman `runs` answers for.
    pub fn by(runs: Arc<dyn Runs>) -> Podman {
        Podman { runs, env: Vec::new() }
    }

    /// The same podman, run with `key` set to `value`.
    pub fn with(&self, key: &str, value: impl Into<OsString>) -> Podman {
        let mut with = self.clone();
        with.env.push((key.into(), value.into()));
        with
    }

    /// Run it; its output, trimmed, when it exits 0.
    pub fn run(&self, args: &[&str]) -> Result<String, PodmanError> {
        let (status, out, err) = self.exec(args)?;
        if status != 0 {
            return Err(PodmanError::Failed {
                command: args.join(" "),
                status,
                stderr: err.trim().to_string(),
            });
        }
        Ok(out.trim().to_string())
    }

    /// For podman's yes-or-no verbs (`exists`, `healthcheck run`): exit 0 is
    /// yes and 1 is no; anything else is a failure, never a no.
    pub fn answers(&self, args: &[&str]) -> Result<bool, PodmanError> {
        match self.exec(args)? {
            (0, ..) => Ok(true),
            (1, ..) => Ok(false),
            (status, _, err) => Err(PodmanError::Failed {
                command: args.join(" "),
                status,
                stderr: err.trim().to_string(),
            }),
        }
    }

    fn exec(&self, args: &[&str]) -> Result<(i32, String, String), PodmanError> {
        let (status, stdout, stderr) = self.runs.run(args, &self.env).map_err(PodmanError::Missing)?;
        tracing::debug!(command = args.join(" "), status, stderr = stderr.trim(), "podman");
        Ok((status, stdout, stderr))
    }
}
