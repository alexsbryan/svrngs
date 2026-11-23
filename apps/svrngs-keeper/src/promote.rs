//! Interpret listener observations and draft a llama-server card for review.

use std::path::Path;

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

/// Wildcard listeners are reached at loopback; the draft always uses loopback.
pub fn dial(addr: &str) -> &str {
    match addr {
        "*" | "0.0.0.0" => "127.0.0.1",
        "[::]" => "[::1]",
        a => a,
    }
}

pub fn props(body: &str) -> Result<String, String> {
    let v: serde_json::Value = serde_json::from_str(body).map_err(|e| format!("/props is not JSON: {e}"))?;
    v["model_path"]
        .as_str()
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "/props gave no model_path".into())
}

fn homed(path: &str, home: &str) -> String {
    match path.strip_prefix(home).and_then(|r| r.strip_prefix('/')) {
        Some(rest) => format!("~/{rest}"),
        None if path == home => "~".into(),
        None => path.into(),
    }
}

/// A relative model path is relative to the program's folder, not ours.
pub fn model(folder: &Path, reported: &str, home: &str) -> String {
    homed(&folder.join(reported).to_string_lossy(), home)
}

const LLAMA: &str = include_str!("../cards/llama-server/compose.yaml");

fn q(s: &str) -> String {
    // A JSON string is a YAML double-quoted scalar, escapes and all.
    serde_json::Value::from(s).to_string()
}

pub fn llama_card(name: &str, folder: &Path, port: u16, reported: &str, home: &str) -> String {
    let model = model(folder, reported, home);
    [
        ("@name@", q(name)),
        ("@about@", q(&format!("llama-server with your model {model}"))),
        ("@model@", q(&model)),
        ("@port@", port.to_string()),
        ("@folder@", q(&homed(&folder.to_string_lossy(), home))),
    ]
    .iter()
    .fold(LLAMA.to_string(), |card, (k, v)| card.replace(k, v))
}
