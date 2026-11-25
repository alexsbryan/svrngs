//! Interpret listener observations, and the arguments a listener was
//! started with, for the person to review as the command Keeper would run.

/// One listening socket on the port, as `ss -ltnpH --cgroup` printed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listen {
    /// Its local address without the port: `127.0.0.1`, `[::1]`, `*`.
    pub addr: String,
    /// The processes ss names as holding it: none when they are another account's.
    pub pids: Vec<u32>,
    pub cgroup: Option<String>,
}

pub fn listens(ss: &str, port: u16) -> Vec<Listen> {
    let end = format!(":{port}");
    ss.lines()
        .filter_map(|line| {
            let local = line.split_whitespace().nth(3)?;
            let addr = local.strip_suffix(&end)?;
            let cgroup = line
                .split_whitespace()
                .last()?
                .strip_prefix("cgroup:")
                .map(str::to_owned);
            let pids = line
                .split("pid=")
                .skip(1)
                .filter_map(|s| s.split(|c: char| !c.is_ascii_digit()).next()?.parse().ok())
                .collect();
            Some(Listen {
                addr: addr.split('%').next().unwrap_or(addr).to_string(),
                pids,
                cgroup,
            })
        })
        .collect()
}

/// A Linux `/proc/<pid>/cmdline`: each argument ends in a NUL. Empty is a
/// kernel thread's, or one that has exited.
pub fn cmdline(bytes: &[u8]) -> Result<Vec<String>, String> {
    if bytes.is_empty() {
        return Err("it gave no arguments, so it may have exited".into());
    }
    // A program that rewrites its title leaves no final NUL.
    let args = bytes.strip_suffix(&[0]).unwrap_or(bytes);
    args.split(|b| *b == 0).map(text).collect()
}

/// A Mac's `KERN_PROCARGS2` (xnu `sysctl_procargsx`): the argument count,
/// the executable's path, NULs to align, then that many arguments, each
/// ending in a NUL, then its environment, which is never read.
pub fn procargs(bytes: &[u8]) -> Result<Vec<String>, String> {
    const UNREAD: &str = "its arguments could not be read";
    let (argc, rest) = bytes.split_first_chunk::<4>().ok_or(UNREAD)?;
    let argc = usize::try_from(i32::from_ne_bytes(*argc)).map_err(|_| UNREAD)?;
    let path = rest.iter().position(|b| *b == 0).ok_or(UNREAD)?;
    let first = rest[path..].iter().position(|b| *b != 0).ok_or(UNREAD)?;
    let args: Vec<&[u8]> = rest[path + first..].split(|b| *b == 0).take(argc).collect();
    if argc == 0 || args.len() < argc {
        return Err(UNREAD.into());
    }
    args.into_iter().map(text).collect()
}

fn text(arg: &[u8]) -> Result<String, String> {
    String::from_utf8(arg.to_vec()).map_err(|_| "one of its arguments is not UTF-8 text".into())
}
