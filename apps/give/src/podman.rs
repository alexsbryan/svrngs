//! podman's command line: the only way give reaches podman (GIVE.md §9,
//! "driving podman's CLI rather than libpod's REST API"). Each call is one
//! short command, run with nothing on its input, so it never waits on a
//! terminal; each is traced at debug with what it answered.

use std::ffi::OsString;
use std::process::{Command, Stdio};

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

/// The podman program give runs: `podman` on the PATH, or another for a test,
/// with what it adds to podman's environment.
#[derive(Debug, Clone)]
pub struct Podman {
    program: OsString,
    env: Vec<(OsString, OsString)>,
}

impl Default for Podman {
    fn default() -> Self {
        Podman::at("podman")
    }
}

impl Podman {
    pub fn at(program: impl Into<OsString>) -> Podman {
        Podman {
            program: program.into(),
            env: Vec::new(),
        }
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
        let out = Command::new(&self.program)
            .args(args)
            .envs(self.env.iter().map(|(k, v)| (k, v)))
            .stdin(Stdio::null())
            .output()
            .map_err(|e| PodmanError::Missing(format!("{}: {e}", self.program.to_string_lossy())))?;
        let status = out.status.code().unwrap_or(-1);
        let (stdout, stderr) = (
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        );
        tracing::debug!(command = args.join(" "), status, stderr = stderr.trim(), "podman");
        Ok((status, stdout, stderr))
    }
}
