//! ported from commonwealth-ai sovereign/crates/sovereign-service/src/lib.rs@b4c23c4d6
//! (`install_launchd`, `install_systemd`, the PATH capture of host-kit's
//! `captured_path`): keep `svrngs serve` running on this computer, as a
//! launchd agent on macOS and a systemd user unit on Linux (DESIGN C11 step
//! 3, "enables the service and warns if the computer sleeps").

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, bail};
use tracing::{debug, info};

use crate::Home;

const LABEL: &str = "org.svrngs.serve";

/// Whether this computer sleeps when idle, as far as it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sleep {
    Never,
    AfterMinutes(u32),
    /// The computer did not say, and why.
    CannotTell(String),
}

/// Register `svrngs serve --home <home>` with this computer's service manager
/// and start it, replacing one registered before. Returns the file written.
pub fn enable(home: &Home) -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe()?.canonicalize()?;
    std::fs::create_dir_all(home.log_dir())?;
    // The service manager starts programs with a minimal PATH, and serve
    // runs `node` from it: the PATH of the shell that ran setup is the one
    // known to find it (host-kit `captured_path`).
    let path = std::env::var("PATH").ok().filter(|p| !p.trim().is_empty());
    let path = path.context("PATH is not set, so the service could not find Node.js")?;
    let user_home = std::env::var_os("HOME").context("HOME is not set")?;
    if cfg!(target_os = "macos") {
        let file = Path::new(&user_home)
            .join("Library/LaunchAgents")
            .join(format!("{LABEL}.plist"));
        write(&file, &plist(&exe, home, &path))?;
        // Unload first, so a second setup picks up a new binary or chat;
        // failing because it was never loaded is expected.
        let unloaded = run("launchctl", &["unload", &file.to_string_lossy()]);
        debug!(?unloaded, "service:unload");
        run("launchctl", &["load", "-w", &file.to_string_lossy()])?;
        info!(file = %file.display(), "service:launchd");
        Ok(file)
    } else if cfg!(target_os = "linux") {
        let file = Path::new(&user_home).join(".config/systemd/user/svrngs.service");
        write(&file, &unit(&exe, home, &path))?;
        run("systemctl", &["--user", "daemon-reload"])?;
        run("systemctl", &["--user", "enable", "svrngs.service"])?;
        run("systemctl", &["--user", "restart", "svrngs.service"])?;
        info!(file = %file.display(), "service:systemd");
        Ok(file)
    } else {
        bail!("this computer's system has no service svrngs knows how to set up")
    }
}

fn write(file: &Path, content: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(file.parent().context("a service file needs a directory")?)?;
    std::fs::write(file, content).with_context(|| format!("{}", file.display()))
}

fn run(program: &str, args: &[&str]) -> anyhow::Result<()> {
    let out = Command::new(program)
        .args(args)
        .output()
        .with_context(|| program.to_string())?;
    if !out.status.success() {
        bail!(
            "{program} {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Restarted when it exits with a failure — the bridge stopping — and not
/// when it was stopped on purpose.
fn plist(exe: &Path, home: &Home, path: &str) -> String {
    let x = |p: &Path| xml_escape(&p.to_string_lossy());
    let log = x(&home.log_dir().join("serve.log"));
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key><string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array><string>{exe}</string><string>serve</string><string>--home</string><string>{home}</string></array>
    <key>RunAtLoad</key><true/>
    <key>KeepAlive</key><dict><key>SuccessfulExit</key><false/></dict>
    <key>StandardOutPath</key><string>{log}</string>
    <key>StandardErrorPath</key><string>{log}</string>
    <key>EnvironmentVariables</key><dict><key>PATH</key><string>{path}</string></dict>
</dict>
</plist>
"#,
        exe = x(exe),
        home = x(home.root()),
        path = xml_escape(path),
    )
}

fn unit(exe: &Path, home: &Home, path: &str) -> String {
    // `%` is a unit-file specifier (host-kit `escape_specifiers`).
    let e = |s: &str| s.replace('%', "%%");
    format!(
        "[Unit]\nDescription=svrngs: keeps the group open for the chat\n\n\
         [Service]\nExecStart=\"{exe}\" serve --home \"{home}\"\nEnvironment=\"PATH={path}\"\n\
         Restart=on-failure\nRestartSec=10\n\n[Install]\nWantedBy=default.target\n",
        exe = e(&exe.to_string_lossy()),
        home = e(&home.root().to_string_lossy()),
        path = e(path),
    )
}

/// Ask this computer whether it sleeps when idle: `pmset -g` on macOS.
pub fn sleep() -> Sleep {
    if !cfg!(target_os = "macos") {
        return Sleep::CannotTell("only macOS is asked".into());
    }
    match Command::new("pmset").arg("-g").output() {
        Ok(out) if out.status.success() => read_pmset(&String::from_utf8_lossy(&out.stdout)),
        Ok(out) => Sleep::CannotTell(format!("pmset -g: {}", out.status)),
        Err(e) => Sleep::CannotTell(format!("pmset -g: {e}")),
    }
}

/// The `sleep` line of `pmset -g`: minutes idle before the computer sleeps,
/// `0` for never.
fn read_pmset(text: &str) -> Sleep {
    let line = text.lines().find_map(|l| {
        let mut words = l.split_whitespace();
        (words.next() == Some("sleep")).then(|| words.next())
    });
    match line.flatten().map(str::parse::<u32>) {
        Some(Ok(0)) => Sleep::Never,
        Some(Ok(m)) => Sleep::AfterMinutes(m),
        _ => Sleep::CannotTell("pmset -g has no sleep line".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pmset_says_when_the_computer_sleeps() {
        let text = " standby  1\n Sleep On Power Button 1\n disksleep  10\n sleep  1 (sleep prevented by powerd)\n";
        assert_eq!(read_pmset(text), Sleep::AfterMinutes(1));
        assert_eq!(read_pmset(" sleep                0\n"), Sleep::Never);
        assert!(matches!(read_pmset(" disksleep 10\n"), Sleep::CannotTell(_)));
    }

    #[test]
    fn the_service_runs_serve_for_this_home_with_the_setup_path() {
        let home = Home::new("/Users/a&b/.svrngs");
        let p = plist(Path::new("/bin/svrngs"), &home, "/opt/node/bin:/usr/bin");
        assert!(
            p.contains("<string>serve</string><string>--home</string><string>/Users/a&amp;b/.svrngs</string>"),
            "{p}"
        );
        assert!(
            p.contains("<key>PATH</key><string>/opt/node/bin:/usr/bin</string>"),
            "{p}"
        );
        let u = unit(Path::new("/bin/svrngs"), &Home::new("/home/a%b"), "/usr/bin");
        assert!(
            u.contains("ExecStart=\"/bin/svrngs\" serve --home \"/home/a%%b\""),
            "{u}"
        );
    }
}
