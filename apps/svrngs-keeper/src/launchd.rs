// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai svrn/crates/sovereign-service/src/lib.rs@8ed8d99b0
//! The login manager owns lifetime; only our own plist may be booted out.

use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use crate::{
    card::Native,
    kept::{Running, Starts, Status},
    runtime::Name,
};

pub fn label(name: &Name) -> String {
    format!("svrngs-keeper-{}", name.as_str())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\r', "&#13;")
}

fn render(label: &str, native: &Native, home: &str, log: &str) -> Result<String, String> {
    use std::os::unix::fs::PermissionsExt;
    let path = |s: &str| match s {
        "~" => home.to_owned(),
        s => s
            .strip_prefix("~/")
            .map_or_else(|| s.to_owned(), |tail| format!("{home}/{tail}")),
    };
    let string = |s: &str| format!("<string>{}</string>", xml_escape(s));
    let mut words = native.command.iter();
    let bin = path(words.next().expect("card has a command"));
    let folder = path(&native.folder);
    let search = crate::systemd::captured_path();
    // launchd's Program key takes an absolute path. Resolve a bare command
    // in the caller's PATH once, without asking a shell to expand arguments.
    let program = if bin.contains('/') {
        Path::new(&folder).join(&bin)
    } else {
        std::env::split_paths(&search)
            .map(|dir| Path::new(&folder).join(dir).join(&bin))
            .find(|file| fs::metadata(file).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0))
            .ok_or_else(|| format!("cannot find {bin} in PATH; give its full path"))?
    };
    let program = program.to_str().ok_or("the program path is not UTF-8")?;
    let args = std::iter::once(string(native.argv0.as_deref().unwrap_or(program)))
        .chain(words.map(|word| string(&path(word))))
        .collect::<String>();
    Ok(format!(
        include_str!("launchd.plist"),
        label = string(label),
        program = string(program),
        args = args,
        folder = string(&folder),
        log = string(log),
        path = string(&search),
    ))
}

pub struct Agent {
    domain: String,
    target: String,
    label: String,
    path: PathBuf,
    parked: PathBuf,
    card: PathBuf,
}

impl Agent {
    pub fn new(agents: &Path, name: &Name, card: PathBuf) -> Result<Self, String> {
        let uid = Command::new("/usr/bin/id")
            .arg("-u")
            .output()
            .map_err(|e| e.to_string())?;
        let uid = String::from_utf8_lossy(&uid.stdout);
        let uid = uid.trim().parse::<u32>().map_err(|_| "cannot read the account id")?;
        let label = label(name);
        let domain = format!("gui/{uid}");
        Ok(Self {
            target: format!("{domain}/{label}"),
            domain,
            path: agents.join(format!("{label}.plist")),
            parked: agents.join(format!("{label}.stopped")),
            label,
            card,
        })
    }

    fn call(&self, args: &[&str]) -> Result<Output, String> {
        let out = Command::new("/bin/launchctl")
            .args(args)
            .output()
            .map_err(|e| format!("launchctl: {e}"))?;
        tracing::debug!(?args, status = %out.status, "launchd:answered");
        Ok(out)
    }

    fn checked(&self, args: &[&str]) -> Result<(), String> {
        let out = self.call(args)?;
        if out.status.success() {
            Ok(())
        } else {
            Err(format!(
                "launchctl {}: {}",
                args[0],
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        }
    }

    fn show(&self) -> Result<Option<String>, String> {
        let out = self.call(&["print", &self.target])?;
        if out.status.success() {
            Ok(Some(String::from_utf8_lossy(&out.stdout).into_owned()))
        } else if out.status.code() == Some(113) {
            Ok(None)
        } else {
            Err(format!(
                "launchctl print: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        }
    }

    fn ours(&self) -> Result<Option<String>, String> {
        let shown = self.show()?;
        if let Some(shown) = &shown {
            let path = value(shown, "path = ")
                .map(Path::new)
                .and_then(|p| p.canonicalize().ok());
            if path.is_none() || path != self.path.canonicalize().ok() || !regular(&self.path)? {
                tracing::debug!(target = %self.target, ?path, "launchd:refused_foreign_registration");
                return Err("that name is already registered from a different setup".into());
            }
        }
        Ok(shown)
    }

    pub fn installed(&self) -> bool {
        self.path.exists() || self.parked.exists()
    }

    fn owned_plist(&self) -> Result<&Path, String> {
        // A matching launchd path is not evidence that the keeper set it up.
        if !regular(&self.card)? {
            tracing::debug!(target = %self.target, "launchd:refused_no_card");
            return Err(format!("{} has no saved card here", self.label));
        }
        match (regular(&self.path)?, regular(&self.parked)?) {
            (true, false) => Ok(&self.path),
            (false, true) => Ok(&self.parked),
            (live, parked) => {
                tracing::debug!(target = %self.target, live, parked, "launchd:refused_plist_state");
                Err(format!("{} needs exactly one login or stopped setup", self.label))
            }
        }
    }

    pub fn require_new(&self) -> Result<(), String> {
        if regular(&self.path)? || regular(&self.parked)? || self.show()?.is_some() {
            tracing::debug!(target = %self.target, "launchd:existing_setup");
            return Err(
                "this name already has a setup here; start it again, or remove it before running a new command".into(),
            );
        }
        Ok(())
    }

    pub(super) fn install(&self, native: &Native) -> Result<(), String> {
        use std::os::unix::fs::OpenOptionsExt;
        let agents = self.path.parent().expect("agent has a directory");
        let home = agents
            .ancestors()
            .nth(2)
            .and_then(Path::to_str)
            .ok_or("the home path is not UTF-8")?;
        let log = self.log();
        // Made before launchd opens it, so what the program says is its
        // person's alone too: launchd appends to a file that is there.
        fs::OpenOptions::new()
            .append(true)
            .create(true)
            .mode(0o600)
            .open(&log)
            .and_then(|f| f.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600)))
            .map_err(|e| format!("write {}: {e}", log.display()))?;
        let content = render(
            &self.label,
            native,
            home,
            log.to_str().ok_or("the log path is not UTF-8")?,
        )?;
        fs::create_dir_all(agents).map_err(|e| format!("create {}: {e}", agents.display()))?;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&self.path)
            .and_then(|mut file| file.write_all(content.as_bytes()))
            .map_err(|e| format!("write {}: {e}", self.path.display()))?;
        self.start()
    }

    /// Where launchd writes what the program says, beside its card.
    pub fn log(&self) -> PathBuf {
        self.card.with_file_name("program.log")
    }

    /// The exit code launchd last saw it end with, as `launchctl print`
    /// says it, if it is registered.
    pub fn last_exit(&self) -> Option<String> {
        let shown = self.show().ok()??;
        value(&shown, "last exit code = ").map(str::to_owned)
    }

    pub fn start(&self) -> Result<(), String> {
        let plist = self.owned_plist()?;
        if self.ours()?.is_some() {
            return Ok(());
        }
        if plist == self.parked {
            self.move_plist(plist, &self.path)?;
        }
        let path = self.path.to_str().ok_or("the setup path is not UTF-8")?;
        self.checked(&["bootstrap", &self.domain, path])
    }

    fn bootout(&self) -> Result<(), String> {
        if self.ours()?.is_some() {
            self.checked(&["bootout", &self.target])?;
            // launchctl can return while teardown is still in flight (web/install.sh).
            let step = std::time::Duration::from_millis(100);
            for _ in 0..100 {
                if self.show()?.is_none() {
                    return Ok(());
                }
                std::thread::sleep(step);
            }
            return Err("launchd did not finish unregistering the program".into());
        }
        Ok(())
    }

    fn move_plist(&self, from: &Path, to: &Path) -> Result<(), String> {
        if regular(to)? {
            return Err(format!("{} already exists", to.display()));
        }
        fs::rename(from, to).map_err(|e| format!("move {} to {}: {e}", from.display(), to.display()))
    }

    pub fn stop(&self) -> Result<(), String> {
        let plist = self.owned_plist()?;
        self.bootout()?;
        // No login plist and no persistent launchctl override to leave behind.
        if plist == self.path {
            self.move_plist(plist, &self.parked)?;
        }
        Ok(())
    }

    pub fn restart(&self) -> Result<(), String> {
        self.owned_plist()?;
        if self.observe()?.1.is_some() {
            self.checked(&["kickstart", "-k", &self.target])?;
        }
        Ok(())
    }

    pub fn remove(&self) -> Result<(), String> {
        let plist = self.owned_plist()?;
        self.bootout()?;
        fs::remove_file(plist).map_err(|e| format!("remove {}: {e}", plist.display()))
    }

    pub fn observe(&self) -> Result<(Status, Option<u32>), String> {
        let Some(shown) = self.ours()? else {
            let starts = if self.parked.is_file() && !self.path.exists() {
                Starts::ByHand
            } else {
                return Err("its login setup is not loaded".into());
            };
            return Ok((
                Status {
                    running: Running::Stopped,
                    starts,
                    restarts: 0,
                },
                None,
            ));
        };
        let state = value(&shown, "state = ").ok_or("launchctl gave no state")?;
        let pid = value(&shown, "pid = ")
            .and_then(|p| p.parse::<u32>().ok())
            .filter(|p| *p > 0);
        let runs = value(&shown, "runs = ")
            .and_then(|p| p.parse::<u32>().ok())
            .ok_or("launchctl gave no run count")?;
        let running = match (state, pid) {
            ("running", Some(_)) => Running::Since(None),
            ("running", None) => return Err("launchctl gave no running pid".into()),
            _ => Running::Starting,
        };
        tracing::debug!(target = %self.target, state, ?pid, runs, "launchd:observed");
        Ok((
            Status {
                running,
                starts: Starts::AtLogin,
                restarts: runs.saturating_sub(1),
            },
            pid.filter(|_| state == "running"),
        ))
    }
}

fn regular(path: &Path) -> Result<bool, String> {
    let error = match fs::symlink_metadata(path) {
        Ok(m) if m.is_file() => return Ok(true),
        Ok(_) => format!("{} is not a regular file", path.display()),
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(false),
        Err(e) => format!("read {}: {e}", path.display()),
    };
    tracing::debug!(%error, "launchd:refused_file");
    Err(error)
}

fn value<'a>(shown: &'a str, key: &str) -> Option<&'a str> {
    // Only the outer job properties, not a nested endpoint or resource.
    shown
        .lines()
        .find_map(|line| line.strip_prefix('\t')?.strip_prefix(key))
}
