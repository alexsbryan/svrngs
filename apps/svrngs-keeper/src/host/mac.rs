use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

use crate::promote::Listen;

fn read(program: &str, args: &[&str], empty: bool) -> Result<String, String> {
    let out = Command::new(program)
        .args(args)
        .env("LC_ALL", "C")
        .output()
        .map_err(|e| format!("{program}: {e}"))?;
    if out.status.success() || (empty && out.status.code() == Some(1) && out.stdout.is_empty() && out.stderr.is_empty())
    {
        String::from_utf8(out.stdout).map_err(|e| format!("{program}: {e}"))
    } else {
        Err(format!(
            "{program}: {} ({})",
            String::from_utf8_lossy(&out.stderr).trim(),
            out.status
        ))
    }
}

pub fn listeners(port: u16) -> Result<Vec<Listen>, String> {
    let text = read(
        "/usr/sbin/lsof",
        &["-nP", "-a", &format!("-iTCP:{port}"), "-sTCP:LISTEN", "-Fpn"],
        true,
    )?;
    let end = format!(":{port}");
    let mut pid = None;
    let mut sockets = Vec::new();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix('p') {
            pid = Some(value.parse().map_err(|_| "lsof gave no process id")?);
        } else if let Some(value) = line.strip_prefix('n') {
            let addr = value.strip_suffix(&end).ok_or("lsof gave another port")?;
            sockets.push(Listen {
                addr: addr.into(),
                pids: vec![pid.ok_or("lsof gave a socket without its holder")?],
                cgroup: None,
            });
        }
    }
    tracing::debug!(port, ?sockets, "host:listeners");
    Ok(sockets)
}

pub fn busy(port: u16) -> Result<Option<String>, String> {
    // lsof can name this account's holders; netstat also sees other accounts.
    let text = read("/usr/sbin/netstat", &["-an", "-p", "tcp"], false)?;
    let end = format!(".{port}");
    let occupied = text.lines().any(|line| {
        let mut fields = line.split_whitespace();
        matches!(fields.next(), Some("tcp4" | "tcp6" | "tcp46"))
            && fields.nth(2).is_some_and(|local| local.ends_with(&end))
            && fields.nth(1) == Some("LISTEN")
    });
    let sockets = listeners(port)?;
    tracing::debug!(port, occupied, "host:kernel_listener");
    Ok((occupied || !sockets.is_empty()).then(|| {
        sockets
            .first()
            .and_then(|s| s.pids.first())
            .and_then(|pid| program(*pid).ok())
            .map(|name| name.chars().filter(|c| !c.is_control()).collect::<String>())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "another program".into())
    }))
}

fn program(pid: u32) -> Result<String, String> {
    let text = read("/bin/ps", &["-p", &pid.to_string(), "-o", "comm="], false)?;
    Path::new(text.trim())
        .file_name()
        .and_then(|p| p.to_str())
        .map(str::to_owned)
        .ok_or("ps gave no program".into())
}

pub fn llama_folder(pid: u32) -> Result<PathBuf, String> {
    if program(pid)? != "llama-server" {
        return Err("no known program is listening".into());
    }
    let text = read(
        "/usr/sbin/lsof",
        &["-a", "-p", &pid.to_string(), "-d", "cwd", "-Fn"],
        false,
    )?;
    let path = text
        .lines()
        .find_map(|l| l.strip_prefix('n'))
        .map(PathBuf::from)
        .filter(|p| p.is_absolute());
    path.ok_or("lsof gave no working folder".into())
}

pub fn owns(port: u16, job: u32) -> Result<bool, String> {
    let sockets = listeners(port)?;
    let text = read("/bin/ps", &["-axo", "pid=,ppid="], false)?;
    let parents: HashMap<u32, u32> = text
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            Some((fields.next()?.parse().ok()?, fields.next()?.parse().ok()?))
        })
        .collect();
    let descends = |mut pid| {
        for _ in 0..parents.len() {
            if pid == job {
                return true;
            }
            let Some(parent) = parents.get(&pid) else {
                break;
            };
            pid = *parent;
        }
        false
    };
    Ok(parents.contains_key(&job)
        && !sockets.is_empty()
        && sockets.iter().all(|s| s.pids.iter().copied().all(descends)))
}
