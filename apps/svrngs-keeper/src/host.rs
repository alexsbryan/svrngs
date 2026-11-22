//! `keep <port>` on this computer: the reads and acts `promote.rs` decides
//! over. The port's holder is read with `ss -ltnpH` and its processes from
//! `/proc`; it is stopped by pid@start (a pidfd opened, then its start
//! checked), and started again, if its copy fails, with its arguments, folder
//! and full environment. Linux only: macOS has no `/proc` and no `ss`.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::time::{Duration, Instant};

use rustix::process::{Pid, PidfdFlags, Signal, pidfd_open, pidfd_send_signal};

use crate::promote::{Declined, Listen, Proc, adopts, listens, stat};

/// The sockets on `port`, from `ss -ltnpH sport = :<port>`.
pub fn ss(port: u16) -> Result<Vec<Listen>, String> {
    let out = std::process::Command::new("ss")
        .args(["-ltnpH", "sport", "=", &format!(":{port}")])
        .output()
        .map_err(|e| format!("ss: {e}"))?;
    if !out.status.success() {
        return Err(format!("ss failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(listens(&String::from_utf8_lossy(&out.stdout), port))
}

fn text(pid: u32, file: &str) -> Result<String, String> {
    std::fs::read_to_string(format!("/proc/{pid}/{file}")).map_err(|e| format!("/proc/{pid}/{file}: {e}"))
}

fn link(pid: u32, file: &str) -> Result<String, String> {
    let to = std::fs::read_link(format!("/proc/{pid}/{file}")).map_err(|e| format!("/proc/{pid}/{file}: {e}"))?;
    to.into_os_string()
        .into_string()
        .map_err(|_| format!("/proc/{pid}/{file} is not UTF-8"))
}

fn nul_words(bytes: &[u8]) -> Result<Vec<String>, String> {
    let bytes = bytes.strip_suffix(&[0]).unwrap_or(bytes);
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    bytes
        .split(|b| *b == 0)
        .map(|w| String::from_utf8(w.to_vec()).map_err(|_| "a word that is not UTF-8".to_string()))
        .collect()
}

/// `read(2)`'s number, as `/proc/<pid>/task/<tid>/syscall` gives it.
const READ: &str = if cfg!(target_arch = "aarch64") { "63" } else { "0" };

/// A process, read from `/proc/<pid>`.
pub fn process(pid: u32) -> Result<Proc, String> {
    let stat = stat(&text(pid, "stat")?).ok_or(format!("/proc/{pid}/stat is not one this reads"))?;
    let argv = nul_words(&std::fs::read(format!("/proc/{pid}/cmdline")).map_err(|e| e.to_string())?)
        .map_err(|e| format!("pid {pid}'s command has {e}"))?;
    let env = std::fs::read(format!("/proc/{pid}/environ"))
        .map_err(|e| e.to_string())
        .and_then(|b| nul_words(&b))
        .map(|words| {
            words
                .iter()
                .filter_map(|w| w.split_once('='))
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        });
    let cgroup = text(pid, "cgroup")?
        .lines()
        .find_map(|l| l.strip_prefix("0::"))
        .unwrap_or("")
        .to_string();
    let tty = link(pid, "fd/0").is_ok_and(|t| t.starts_with("/dev/pts/") || t.starts_with("/dev/tty"));
    let reading = || {
        std::fs::read_dir(format!("/proc/{pid}/task")).is_ok_and(|tasks| {
            tasks.flatten().any(|t| {
                std::fs::read_to_string(t.path().join("syscall"))
                    .is_ok_and(|s| s.strip_prefix(READ).is_some_and(|r| r.starts_with(" 0x0 ")))
            })
        })
    };
    Ok(Proc {
        pid,
        stat,
        exe: link(pid, "exe")?,
        argv,
        cwd: link(pid, "cwd")?,
        env,
        cgroup,
        typing: tty && reading(),
    })
}

/// The port's one listening process, and each parent that comes with it.
pub fn survey(port: u16) -> Result<(String, Vec<Proc>), Declined> {
    let sockets = ss(port).map_err(Declined::Cannot)?;
    tracing::debug!(port, ?sockets, "ss");
    let Some(first) = sockets.first() else {
        return Err(Declined::Nothing(port));
    };
    if sockets.iter().any(|s| s.pids.is_empty()) {
        return Err(Declined::Another(port));
    }
    let mut pids: Vec<u32> = sockets.iter().flat_map(|s| s.pids.clone()).collect();
    pids.sort();
    pids.dedup();
    if pids.len() > 1 {
        return Err(Declined::Several(port, pids));
    }
    let mut procs = vec![process(pids[0]).map_err(Declined::Cannot)?];
    loop {
        let ppid = procs[procs.len() - 1].stat.ppid;
        if ppid <= 1 {
            break;
        }
        let parent = match process(ppid) {
            Ok(p) => p,
            Err(why) => {
                tracing::debug!(ppid, why, "parent not read, so not adopted");
                break;
            }
        };
        if !adopts(&parent, procs[0].stat.session) {
            break;
        }
        procs.push(parent);
    }
    Ok((first.addr.clone(), procs))
}

/// `/proc/sys/net/ipv4/ip_local_port_range`: the ports the system hands out.
pub fn ephemeral() -> (u16, u16) {
    let range = std::fs::read_to_string("/proc/sys/net/ipv4/ip_local_port_range").unwrap_or_default();
    let mut it = range.split_whitespace().filter_map(|n| n.parse().ok());
    match (it.next(), it.next()) {
        (Some(lo), Some(hi)) => (lo, hi),
        // Linux's default, when the file can't be read.
        _ => (32768, 60999),
    }
}

/// A connection to the port, within a second.
#[expect(
    clippy::disallowed_methods,
    reason = "dial: the program its own person started, asked how it runs and whether it answers"
)]
fn connect(at: &str) -> Result<TcpStream, String> {
    let sock = at.parse().map_err(|e| format!("{at}: {e}"))?;
    TcpStream::connect_timeout(&sock, Duration::from_secs(1)).map_err(|e| format!("{at}: {e}"))
}

/// `GET <path>` over HTTP/1.0: the status and the body.
pub fn get(addr: &str, port: u16, path: &str) -> Result<(u16, String), String> {
    let at = format!("{addr}:{port}");
    let mut s = connect(&at)?;
    s.set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| e.to_string())?;
    write!(s, "GET {path} HTTP/1.0\r\nHost: {at}\r\n\r\n").map_err(|e| format!("{at}: {e}"))?;
    let mut got = Vec::new();
    let _ = s.read_to_end(&mut got);
    let got = String::from_utf8_lossy(&got);
    let (head, body) = got.split_once("\r\n\r\n").unwrap_or((&got, ""));
    let status = head
        .strip_prefix("HTTP/")
        .and_then(|h| h.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .ok_or(format!("{at} did not answer HTTP"))?;
    Ok((status, body.to_string()))
}

/// Whether the port answers: an HTTP status for an HTTP door, a connection
/// otherwise.
pub fn answers(addr: &str, port: u16, http: bool) -> bool {
    if http {
        return get(addr, port, "/").is_ok();
    }
    connect(&format!("{addr}:{port}")).is_ok()
}

/// Ask `ready` every half second until it says yes or `secs` pass.
#[expect(
    clippy::disallowed_methods,
    reason = "the handover's adapter waits on the clock for what it asks"
)]
pub fn within(secs: u64, mut ready: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    loop {
        if ready() {
            return true;
        }
        if Instant::now() >= end {
            return false;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

/// Stop `procs` by pid@start: each pidfd is opened and its process's start
/// checked against the one read, then SIGTERM goes to the adopted process
/// first and down to the port's. What still holds the port after 10 s is
/// sent SIGKILL.
pub fn stop(procs: &[Proc], port: u16) -> Result<(), String> {
    let mut fds = Vec::new();
    for p in procs.iter().rev() {
        let pid = i32::try_from(p.pid).ok().and_then(Pid::from_raw).ok_or("pid 0")?;
        let fd = pidfd_open(pid, PidfdFlags::empty()).map_err(|e| format!("pid {} is gone ({e})", p.pid))?;
        let now = stat(&text(p.pid, "stat")?).map(|s| s.start);
        if now != Some(p.stat.start) {
            tracing::debug!(
                pid = p.pid,
                ?now,
                read = p.stat.start,
                "refused: another process has its pid now"
            );
            return Err(format!("pid {} is another process now, so nothing was stopped", p.pid));
        }
        fds.push(fd);
    }
    for fd in &fds {
        let _ = pidfd_send_signal(fd, Signal::TERM);
    }
    let free = || ss(port).is_ok_and(|s| s.is_empty());
    if within(10, free) {
        return Ok(());
    }
    tracing::debug!(port, "still held 10 s after SIGTERM: SIGKILL");
    for fd in &fds {
        let _ = pidfd_send_signal(fd, Signal::KILL);
    }
    if within(2, free) {
        Ok(())
    } else {
        Err(format!("port {port} is still held after SIGKILL"))
    }
}

/// Whether `port` is held by `unit`'s own process running `exe`, and
/// answers.
pub fn held_by(port: u16, unit: &str, exe: &str, addr: &str, http: bool) -> bool {
    let Ok(sockets) = ss(port) else { return false };
    let pids: Vec<u32> = sockets.iter().flat_map(|s| s.pids.clone()).collect();
    let ours = !pids.is_empty()
        && pids
            .iter()
            .all(|pid| process(*pid).is_ok_and(|p| p.exe == exe && p.cgroup.ends_with(&format!("/{unit}"))));
    ours && answers(addr, port, http)
}

/// Start `top` again as it was: its program under its first argument, its
/// arguments, its folder and its whole environment, in a process group of
/// its own, its output appended to `log`.
pub fn start_again(top: &Proc, log: &Path) -> Result<u32, String> {
    let out = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
        .map_err(|e| format!("{}: {e}", log.display()))?;
    let err = out.try_clone().map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new(&top.exe);
    if let Some(argv0) = top.argv.first() {
        cmd.arg0(argv0);
    }
    let child = cmd
        .args(top.argv.iter().skip(1))
        .current_dir(&top.cwd)
        .env_clear()
        .envs(top.env.iter().flatten().map(|(k, v)| (k, v)))
        .stdin(std::process::Stdio::null())
        .stdout(out)
        .stderr(err)
        .process_group(0)
        .spawn()
        .map_err(|e| format!("start {}: {e}", top.exe))?;
    Ok(child.id())
}
