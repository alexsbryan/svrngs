//! What a person gives Keep it running to keep: a program, its arguments and
//! the folder it starts in, each checked before anything is written (svrn-docs
//! rings/current/KEEPER_ENTRY_POINT.md, "The exact checks"). Both ways in end
//! here. A terminal's shell has already split the words, expanded `~` and
//! `$VAR` and named the folder (`svrngs keep <port> -- <command>`). The page's
//! line is split as a shell would split it ([`from_line`]), and anything only
//! a shell would expand is refused, since no shell will run it.
//!
//! What is kept runs at every boot, so whoever can change the program or its
//! folder chooses what runs as this person. Each file and folder on the way
//! is held to what OpenSSH's StrictModes asks of a key: owned by this person
//! or root, and writable by nobody else ([`Account::trusts`]). A group's
//! synced folder is refused outright, since anyone in the group can change it.

use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// Why what was given is not kept: each says what stopped it, why, and the
/// fix.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    #[error("The command has a quote that isn't closed. Close it, then press Start.")]
    Quotes,
    #[error("Give the command you started it with, then press Start.")]
    Empty,
    #[error(
        "The command starts with a setting ({0}), and only a shell can give one. In the terminal you started it in, put svrngs keep {1} -- in front of your command instead."
    )]
    Setting(String, u16),
    #[error(
        "The command uses {0}, which only a shell understands. In the terminal you started it in, put svrngs keep {1} -- in front of your command instead."
    )]
    Shell(String, u16),
    #[error("Give the folder's whole path, starting with / or ~: {0} doesn't.")]
    Relative(String),
    #[error("{0} {1}. Give the folder you started it from.")]
    Folder(PathBuf, &'static str),
    #[error("svrngs can't find {0} in your PATH. Give its whole path, such as /usr/bin/{0}.")]
    NotFound(String),
    #[error("{0} {1}, so svrngs can't start it.")]
    NotProgram(PathBuf, &'static str),
    #[error(
        "{0} is inside {1}, a folder your group syncs, so anyone in the group could change what runs as you. Move it out of {1}, then try again."
    )]
    Synced(PathBuf, PathBuf),
    #[error(
        "{0} can be changed by other accounts on this computer ({1}), so they could change what runs as you. Run chmod go-w {0}, then try again."
    )]
    Writable(PathBuf, &'static str),
    #[error(
        "{0} belongs to another account on this computer, so it could change what runs as you. Use a copy of your own, then try again."
    )]
    Owner(PathBuf),
    #[error("{0} couldn't be read ({1}), so nothing was kept.")]
    Unread(PathBuf, String),
}

fn refused<T>(refusal: Refusal) -> Result<T, Refusal> {
    tracing::debug!(%refusal, "given: refused");
    Err(refusal)
}

/// A program checked to keep: the file it runs, by the absolute path it was
/// found at; the word it was given as, its first argument; its other
/// arguments; and its folder, with every link in it resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Given {
    pub program: PathBuf,
    pub argv0: String,
    pub args: Vec<String>,
    pub folder: PathBuf,
}

impl Given {
    /// The line it runs, quoted as a shell would need it, for the person
    /// to confirm.
    pub fn shown(&self) -> String {
        let program = self.program.to_string_lossy();
        let words = std::iter::once(program.as_ref()).chain(self.args.iter().map(String::as_str));
        shlex::try_join(words).unwrap_or_else(|_| "(a command with a NUL in it)".into())
    }
}

/// Where the system's own programs are, whose files are not checked.
const SYSTEM: [&str; 5] = ["/usr", "/bin", "/sbin", "/opt/homebrew", "/usr/local"];

/// Words a shell reads as more than an argument: an operator standing alone.
const OPERATORS: [&str; 11] = ["|", "||", "&", "&&", ";", ";;", ">", ">>", "<", "<<", "2>&1"];

/// The page's command line and folder, as a shell would split them, with `~`
/// and `~/` at the start of a word or the folder read as `home`. A first word
/// shaped `NAME=value`, an operator, `$` or a backtick is refused: each needs
/// a shell, and the terminal route has one.
pub fn from_line(line: &str, folder: &str, home: &Path, port: u16) -> Result<(Vec<String>, PathBuf), Refusal> {
    let Some(words) = shlex::split(line) else {
        return refused(Refusal::Quotes);
    };
    let Some(first) = words.first() else {
        return refused(Refusal::Empty);
    };
    let setting = first
        .split_once('=')
        .is_some_and(|(name, _)| !name.is_empty() && name.chars().all(|c| c == '_' || c.is_ascii_alphanumeric()));
    if setting {
        return refused(Refusal::Setting(first.clone(), port));
    }
    let shell = |w: &&String| {
        OPERATORS.contains(&w.as_str()) || w.contains(['$', '`']) || w.ends_with(';') || w.starts_with(['>', '<', '|'])
    };
    if let Some(word) = words.iter().find(shell) {
        return refused(Refusal::Shell(word.clone(), port));
    }
    let tilde = |w: &str| match w {
        "~" => home.to_path_buf(),
        _ => w
            .strip_prefix("~/")
            .map_or_else(|| PathBuf::from(w), |rest| home.join(rest)),
    };
    let words = words.iter().map(|w| tilde(w).to_string_lossy().into_owned()).collect();
    Ok((words, tilde(folder.trim())))
}

/// Whose files may say what runs: this account, and its own group if it has
/// one nobody else is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Account {
    pub uid: u32,
    /// The group named as this account is, with no other member, as a
    /// distribution that gives each account its own group makes it; group
    /// write by it is this person's alone.
    pub own_group: Option<u32>,
}

impl Account {
    /// This process's account, and its own group as `/etc/passwd` and
    /// `/etc/group` say. One they don't list has none: its group's write
    /// bit is refused.
    pub fn read() -> Result<Account, Refusal> {
        let uid = uid()?;
        let read = |p: &str| std::fs::read_to_string(p).unwrap_or_default();
        let own_group = own_group(&read("/etc/passwd"), &read("/etc/group"), uid);
        tracing::debug!(uid, ?own_group, "given: the account whose files may say what runs");
        Ok(Account { uid, own_group })
    }

    /// `path`, a file or folder, and every folder above it, owned by this
    /// account or root and writable by nobody else: root's sticky folders
    /// (`/tmp`) aside, where nobody may replace another's file.
    pub fn trusts(&self, path: &Path) -> Result<(), Refusal> {
        for at in path.ancestors() {
            let m = std::fs::symlink_metadata(at).map_err(|e| Refusal::Unread(at.into(), e.to_string()));
            let m = match m {
                Ok(m) => m,
                Err(refusal) => return refused(refusal),
            };
            let mode = m.permissions().mode();
            let sticky = m.is_dir() && mode & 0o1000 != 0 && m.uid() == 0;
            if m.uid() != self.uid && m.uid() != 0 {
                return refused(Refusal::Owner(at.into()));
            }
            if mode & 0o002 != 0 && !sticky {
                return refused(Refusal::Writable(at.into(), "anyone can write to it"));
            }
            // Root's own group, and a Mac's admin group, which can become
            // root anyway: what root lets them write (python.org's
            // /Library/Frameworks) is root's.
            let admin = m.uid() == 0 && (m.gid() == 0 || cfg!(target_os = "macos") && m.gid() == 80);
            if mode & 0o020 != 0 && !sticky && !admin && self.own_group != Some(m.gid()) {
                return refused(Refusal::Writable(at.into(), "its group can write to it"));
            }
        }
        tracing::debug!(path = %path.display(), "given: owned and written by this account or root alone");
        Ok(())
    }
}

/// The account's own group: the one `passwd` gives it, named as it is, with
/// no member but it.
pub fn own_group(passwd: &str, group: &str, uid: u32) -> Option<u32> {
    let fields = |line: &str| line.split(':').map(str::to_owned).collect::<Vec<_>>();
    let user = passwd
        .lines()
        .map(fields)
        .find(|f| f.len() > 3 && f[2] == uid.to_string())?;
    let gid: u32 = user[3].parse().ok()?;
    let own = group.lines().map(fields).find(|f| f.len() > 3 && f[2] == user[3])?;
    let alone = own[3].split(',').all(|m| m.is_empty() || m == user[0]);
    (own[0] == user[0] && alone).then_some(gid)
}

#[cfg(target_os = "linux")]
fn uid() -> Result<u32, Refusal> {
    std::fs::metadata("/proc/self")
        .map(|m| m.uid())
        .map_err(|e| Refusal::Unread("/proc/self".into(), e.to_string()))
}

#[cfg(not(target_os = "linux"))]
fn uid() -> Result<u32, Refusal> {
    let unread = |why: String| Refusal::Unread("/usr/bin/id".into(), why);
    let out = std::process::Command::new("/usr/bin/id").arg("-u").output();
    let out = out.map_err(|e| unread(e.to_string()))?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.trim()
        .parse()
        .map_err(|_| unread("it gave no account number".into()))
}

/// `path` outside every folder in `synced`.
fn unsynced(path: &Path, synced: &[PathBuf]) -> Result<(), Refusal> {
    for dir in synced {
        let dir = dir.canonicalize().unwrap_or_else(|_| dir.clone());
        if path.starts_with(&dir) {
            return refused(Refusal::Synced(path.into(), dir));
        }
    }
    Ok(())
}

/// The program and folder `words` and `folder` give, checked to keep: the
/// folder absolute, there and a folder; the program found as a shell finds
/// it (in `folder` when its word has a `/`, else in `path`), a file that
/// runs; neither inside a `synced` folder, and each, through every link on
/// the way to the program, one only `account` can change, unless the program
/// is the system's ([`SYSTEM`]).
pub fn check(
    words: Vec<String>,
    folder: &Path,
    synced: &[PathBuf],
    path: &str,
    account: &Account,
) -> Result<Given, Refusal> {
    if !folder.is_absolute() {
        return refused(Refusal::Relative(folder.display().to_string()));
    }
    let Ok(folder) = folder.canonicalize() else {
        return refused(Refusal::Folder(folder.into(), "isn't there"));
    };
    if !folder.is_dir() {
        return refused(Refusal::Folder(folder, "isn't a folder"));
    }
    unsynced(&folder, synced)?;
    account.trusts(&folder)?;
    let mut words = words.into_iter();
    let argv0 = words.next().filter(|w| !w.is_empty()).ok_or(Refusal::Empty)?;
    let runs = |p: &Path| std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0);
    let program = if argv0.contains('/') {
        folder.join(&argv0)
    } else {
        let found = std::env::split_paths(path)
            .map(|dir| folder.join(dir).join(&argv0))
            .find(|p| runs(p));
        match found {
            Some(found) => found,
            None => return refused(Refusal::NotFound(argv0)),
        }
    };
    let mut at = program.clone();
    // The system's own programs are root's to change, links and all, as
    // its package manager (or Homebrew, its owner's) keeps them.
    let system = SYSTEM.iter().any(|dir| program.starts_with(dir));
    tracing::debug!(program = %program.display(), system, "given: a system program's files are not checked");
    // Each link's folder is checked as it is followed, so nobody else can
    // point any link on the way somewhere else.
    for _ in 0..16 {
        let dir = at.parent().and_then(|d| d.canonicalize().ok());
        let Some(dir) = dir else {
            return refused(Refusal::NotProgram(program, "isn't there"));
        };
        if !system {
            account.trusts(&dir)?;
        }
        let name = at.file_name().map(PathBuf::from).unwrap_or_default();
        let m = match std::fs::symlink_metadata(dir.join(&name)) {
            Ok(m) => m,
            Err(_) => return refused(Refusal::NotProgram(program, "isn't there")),
        };
        if m.is_symlink() {
            let to =
                std::fs::read_link(dir.join(&name)).map_err(|e| Refusal::Unread(dir.join(&name), e.to_string()))?;
            at = dir.join(to);
            continue;
        }
        let file = dir.join(name);
        if !m.is_file() {
            return refused(Refusal::NotProgram(program, "isn't a file"));
        }
        if m.permissions().mode() & 0o111 == 0 {
            return refused(Refusal::NotProgram(program, "isn't marked to run (chmod +x)"));
        }
        unsynced(&file, synced)?;
        if !system {
            account.trusts(&file)?;
        }
        let program = std::path::absolute(&program).unwrap_or(program);
        tracing::debug!(program = %program.display(), file = %file.display(), folder = %folder.display(), "given: checked to keep");
        return Ok(Given {
            program,
            argv0,
            args: words.collect(),
            folder,
        });
    }
    refused(Refusal::NotProgram(program, "is a link to a link, too many times over"))
}
