//! Interpret listener observations: which sockets hold a port, and whose.

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
