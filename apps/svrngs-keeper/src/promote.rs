//! `keep <port>`, decided: what holds the port, which of its processes to
//! adopt, whether it can be kept at all, and the card it is kept from
//! (ralph/next/keeper/STATE.md, keeper-promote). Pure: text the host gave in
//! (`ss -ltnpH`, `/proc/<pid>/…`, `/props`), decisions out. `host.rs` reads
//! the host and does the handover.
//!
//! - **A program it knows**, llama-server started as itself, is composed from
//!   the card svrngs-keeper ships (`cards/llama-server`) and what it reports.
//! - **Anything else** is copied: the program, its first argument as its
//!   name, its arguments and its folder. A process's parents up to the shell
//!   come with it, so a wrapper script is adopted, not its child. No
//!   environment is carried; the settings the terminal added are named.

use std::path::Path;

/// One listening socket on the port, as `ss -ltnpH` printed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listen {
    /// Its local address without the port: `127.0.0.1`, `[::1]`, `*`.
    pub addr: String,
    /// The processes ss names as holding it: none when they are another
    /// account's.
    pub pids: Vec<u32>,
}

/// Each socket listening on `port` in `ss -ltnpH` output.
pub fn listens(ss: &str, port: u16) -> Vec<Listen> {
    let end = format!(":{port}");
    ss.lines()
        .filter_map(|line| {
            let local = line.split_whitespace().nth(3)?;
            let addr = local.strip_suffix(&end)?;
            let pids = line
                .split("pid=")
                .skip(1)
                .filter_map(|s| s.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok())
                .collect();
            Some(Listen {
                addr: addr.split('%').next().unwrap_or(addr).to_string(),
                pids,
            })
        })
        .collect()
}

/// The address a listener on `addr` is reached at here.
pub fn dial(addr: &str) -> String {
    match addr {
        "*" | "0.0.0.0" => "127.0.0.1".into(),
        "[::]" => "[::1]".into(),
        a => a.into(),
    }
}

/// `/proc/<pid>/stat`, the fields this reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stat {
    pub ppid: u32,
    pub session: u32,
    /// Its start, in clock ticks since boot: with the pid, which process it is.
    pub start: u64,
}

/// Read `/proc/<pid>/stat`. The name in parentheses may hold anything, so
/// the fields are counted from its last `)`.
pub fn stat(text: &str) -> Option<Stat> {
    let after: Vec<&str> = text.get(text.rfind(')')? + 1..)?.split_whitespace().collect();
    Some(Stat {
        ppid: after.get(1)?.parse().ok()?,
        session: after.get(3)?.parse().ok()?,
        start: after.get(19)?.parse().ok()?,
    })
}

/// A process, as the host read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proc {
    pub pid: u32,
    pub stat: Stat,
    /// `/proc/<pid>/exe`: the program file it runs.
    pub exe: String,
    /// `/proc/<pid>/cmdline`: its first argument, then the rest.
    pub argv: Vec<String>,
    /// `/proc/<pid>/cwd`.
    pub cwd: String,
    /// `/proc/<pid>/environ`, or why it could not be read.
    pub env: Result<Vec<(String, String)>, String>,
    /// Its cgroup v2 path (`/proc/<pid>/cgroup`'s `0::` line).
    pub cgroup: String,
    /// One of its threads waits to read a terminal on its standard input.
    pub typing: bool,
}

const SHELLS: [&str; 8] = ["sh", "bash", "dash", "zsh", "fish", "ksh", "mksh", "tcsh"];

fn base(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// A shell that runs a script or a `-c` command: its argv has an operand or
/// `-c`. A login shell is `-bash`; an interactive one has options at most.
fn shell_with_script(argv: &[String]) -> Option<Option<&str>> {
    let name = base(argv.first()?).trim_start_matches('-');
    if !SHELLS.contains(&name) {
        return None;
    }
    let mut rest = argv.iter().skip(1);
    let script = rest.find(|w| !w.starts_with('-') || w.as_str() == "-c");
    Some(script.map(|w| if w == "-c" { "" } else { w.as_str() }))
}

/// Whether `parent` of a process in session `session` comes with it: it is
/// in the same session, is neither its leader (the terminal's shell) nor
/// pid 1, and is not an interactive shell (`bash` typed in a terminal).
pub fn adopts(parent: &Proc, session: u32) -> bool {
    let p = parent.pid;
    let adopted = p != 1
        && p != parent.stat.session
        && parent.stat.session == session
        && !matches!(shell_with_script(&parent.argv), Some(None));
    tracing::debug!(pid = p, adopted, "parent of the port's process");
    adopted
}

/// What `keep <port>` declines, and why: each is said, nothing is changed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Declined {
    #[error("nothing listens on port {0} here")]
    Nothing(u16),
    #[error("a program of another account holds port {0}, so how it was started cannot be read")]
    Another(u16),
    #[error("port {0} is held by processes {1:?} at once, and svrngs-keeper copies one program")]
    Several(u16, Vec<u32>),
    #[error("port {port} is kept already, by the systemd unit {unit}")]
    Kept { port: u16, unit: String },
    #[error(
        "port {0} changes at each start: it is one the system hands out ({1}–{2}), and nothing it was started with names it"
    )]
    Changes(u16, u16, u16),
    #[error("{0} waits for someone to type at it, and a kept program has no one to type to")]
    Typed(String),
    #[error("{0}'s environment cannot be read ({1}), so it could not be started again as it was")]
    Environment(String, String),
    #[error("{0}")]
    Cannot(String),
}

/// Why the chain `procs` (the port's process, then each parent adopted) on
/// `port` cannot be kept, if it cannot. `ephemeral` is the range the system
/// hands ports out from (`/proc/sys/net/ipv4/ip_local_port_range`).
pub fn declines(procs: &[Proc], port: u16, ephemeral: (u16, u16)) -> Option<Declined> {
    let top = procs.last()?;
    let said = |p: &Proc| format!("{} (pid {})", base(p.argv.first().unwrap_or(&p.exe)), p.pid);
    let unit = base(&procs[0].cgroup);
    if unit.starts_with("svrngs-keeper-") && unit.ends_with(".service") {
        return Some(Declined::Kept {
            port,
            unit: unit.into(),
        });
    }
    if let Some(p) = procs.iter().find(|p| p.typing) {
        return Some(Declined::Typed(said(p)));
    }
    if let Err(why) = &top.env {
        return Some(Declined::Environment(said(top), why.clone()));
    }
    let named = |w: &str| w.split(|c: char| !c.is_ascii_digit()).any(|d| d == port.to_string());
    let words = procs.iter().flat_map(|p| p.argv.iter().map(String::as_str));
    let values = top.env.iter().flatten().map(|(_, v)| v.as_str());
    let (lo, hi) = ephemeral;
    if (lo..=hi).contains(&port) && !words.chain(values).any(named) {
        return Some(Declined::Changes(port, lo, hi));
    }
    if top.exe.ends_with(" (deleted)") {
        return Some(Declined::Cannot(format!(
            "{}'s program file was replaced since it started, so the program it runs is not on disk",
            said(top)
        )));
    }
    if let Some(w) = top.argv.iter().skip(1).find(|w| w.starts_with("~/")) {
        return Some(Declined::Cannot(format!(
            "{} was given {w} as it is, and a kept card reads ~/ as your home folder",
            said(top)
        )));
    }
    None
}

const NOT_SETTINGS: [&str; 11] = [
    "PWD",
    "OLDPWD",
    "SHLVL",
    "_",
    "INVOCATION_ID",
    "JOURNAL_STREAM",
    "MANAGERPID",
    "MANAGERPIDFDID",
    "SYSTEMD_EXEC_PID",
    "MEMORY_PRESSURE_WATCH",
    "MEMORY_PRESSURE_WRITE",
];

/// The settings `env` has that the user's manager, whose environment a kept
/// unit starts with, does not have or has otherwise: by name, sorted. The
/// shell's own bookkeeping (`PWD`, `OLDPWD`, `SHLVL`, `_`) is no setting, nor
/// is what the manager sets for each unit it starts, the kept one too
/// (systemd.exec(5), "Environment Variables Set or Propagated by the Service
/// Manager").
/// `manager` is `systemctl --user show-environment`, whose quoted values
/// (`$'…'`) are compared as printed, so such a setting may be named though
/// the manager has it.
pub fn added(env: &[(String, String)], manager: &str) -> Vec<String> {
    let mut names: Vec<String> = env
        .iter()
        .filter(|(k, _)| !NOT_SETTINGS.contains(&k.as_str()))
        .filter(|(k, v)| !manager.lines().any(|l| l == format!("{k}={v}")))
        .map(|(k, _)| k.clone())
        .collect();
    names.sort();
    names.dedup();
    names
}

/// `path` as a card writes it: under `home`, `~/…`.
pub fn homed(path: &str, home: &str) -> String {
    match path.strip_prefix(home).and_then(|r| r.strip_prefix('/')) {
        Some(rest) => format!("~/{rest}"),
        None if path == home => "~".into(),
        None => path.into(),
    }
}

/// The name a kept copy of `argv` is given when none is: its first
/// argument's file name, or a shell's script's, without its extension.
pub fn name_for(argv: &[String]) -> String {
    let word = match shell_with_script(argv) {
        Some(Some(script)) if !script.is_empty() => script,
        _ => argv.first().map_or("", String::as_str),
    };
    let file = base(word);
    crate::kept::name_of(file.split_once('.').map_or(file, |(stem, _)| stem))
}

/// What llama-server reports at `/props`: its model and its context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Props {
    pub model_path: String,
    pub n_ctx: u64,
}

/// Read llama-server's `/props` body.
pub fn props(body: &str) -> Result<Props, String> {
    let v: serde_json::Value = serde_json::from_str(body).map_err(|e| format!("/props is not JSON: {e}"))?;
    let model_path = v["model_path"].as_str().filter(|p| !p.is_empty());
    let n_ctx = v["default_generation_settings"]["n_ctx"].as_u64();
    match (model_path, n_ctx) {
        (Some(m), Some(n)) => Ok(Props {
            model_path: m.into(),
            n_ctx: n,
        }),
        _ => Err("/props gave no model_path or no default_generation_settings.n_ctx".into()),
    }
}

/// Whether `listener` is a program svrngs-keeper ships a card for: started
/// as itself, with no wrapper adopted above it.
pub fn known(procs: &[Proc]) -> bool {
    procs.len() == 1 && base(&procs[0].exe) == "llama-server"
}

const LLAMA: &str = include_str!("../cards/llama-server/compose.yaml");

fn q(s: &str) -> String {
    // A JSON string is a YAML double-quoted scalar, escapes and all.
    serde_json::Value::from(s).to_string()
}

/// The model `listener` reported, as a card writes it: a relative path is
/// its folder's.
pub fn model(listener: &Proc, props: &Props, home: &str) -> String {
    homed(
        &Path::new(&listener.cwd).join(&props.model_path).to_string_lossy(),
        home,
    )
}

/// The llama-server card, filled: `listener` listening on `addr`:`port`
/// reported `props`.
pub fn llama_card(name: &str, listener: &Proc, addr: &str, port: u16, props: &Props, home: &str) -> String {
    let model = model(listener, props, home);
    let host = match addr {
        "*" => "0.0.0.0",
        a => a.trim_start_matches('[').trim_end_matches(']'),
    };
    [
        ("@name@", q(name)),
        ("@about@", q(&format!("llama-server with your model {model}"))),
        ("@program@", q(&listener.exe)),
        ("@model@", q(&model)),
        ("@host@", q(host)),
        ("@port@", port.to_string()),
        ("@ctx@", props.n_ctx.to_string()),
        ("@folder@", q(&homed(&listener.cwd, home))),
    ]
    .iter()
    .fold(LLAMA.to_string(), |card, (k, v)| card.replace(k, v))
}

/// A copy of `top`, the process adopted, as a native card: its program, its
/// first argument as its name when that is not the program, its arguments,
/// its folder. `http` when the port answered HTTP.
pub fn copy_card(name: &str, top: &Proc, port: u16, http: bool, home: &str) -> String {
    let mut command = vec![q(&top.exe)];
    command.extend(top.argv.iter().skip(1).map(|w| q(w)));
    let argv0 = match top.argv.first() {
        Some(a) if *a != top.exe => format!("    argv0: {}\n", q(a)),
        _ => String::new(),
    };
    format!(
        "# svrngs-keeper copied this card from how {name} was started.\n\
         x-svrngs:\n  title: {}\n  about: {}\n  door: {{ port: {port}, protocol: {} }}\n  state: []\n  native:\n    \
         command: [{}]\n{argv0}    folder: {}\n",
        q(name),
        q(&format!("{}, as it was started", top.argv.join(" "))),
        if http { "http" } else { "tcp" },
        command.join(", "),
        q(&homed(&top.cwd, home)),
    )
}

/// The words of `argv` a shell would read back as they are.
pub fn shown(argv: &[String]) -> String {
    argv.iter()
        .map(|w| {
            let plain = !w.is_empty() && w.chars().all(|c| c.is_ascii_alphanumeric() || "-_./=:,+@%".contains(c));
            if plain {
                w.clone()
            } else {
                format!("'{}'", w.replace('\'', r"'\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The flags llama-server was started with that its composed card does
/// not give it, so the person is told the kept one runs without them.
pub fn not_carried(argv: &[String]) -> Vec<String> {
    const CARRIED: [&str; 6] = ["-m", "--model", "--host", "--port", "-c", "--ctx-size"];
    argv.iter()
        .skip(1)
        .filter(|w| w.starts_with('-') && !CARRIED.contains(&w.split('=').next().unwrap_or(w)))
        .cloned()
        .collect()
}
