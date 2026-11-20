// SPDX-License-Identifier: AGPL-3.0-or-later
//! Which program may receive a share's traffic (svrn-docs SHARE_BINDING
//! §5.9): the two decisions, over what svrngs read of the share's port.
//! No I/O, no clock and no system's types, only ids in sets, so the module
//! moves to `eudoxia-community` whole. [`crate::uid`] reads the port, and
//! [`crate::door::bound`] and [`crate::share`] act on what these decide.
//!
//! - [`approve`]: the sockets on the port, each with the processes holding
//!   it, and the process the person chose, if any: what is approved, or
//!   why nothing is.
//! - [`check`]: what was approved, the sockets on the port now, and what
//!   took this connection: carry it, or why it is held.
//!
//! A socket is approved by its id where the system gives it one that no
//! later socket gets while it runs (Linux's cookie), else by the processes
//! holding it, each named by its id and start, which no later process
//! shares (a Mac's). The door reaches it at this computer's own address of
//! a family it takes (`Approved::at`), and only the sockets that take what
//! is sent there are judged: one on another address gets none of it.

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use serde::{Deserialize, Serialize};

/// A process: its id and its start, which no later process shares.
pub type Process = (u32, u64);

/// One socket on a share's port, as read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Socket {
    /// Linux's cookie; a Mac's handle, which names it in this read alone.
    pub id: u64,
    /// The cgroup it was made in, where one is read (Linux).
    pub cgroup: Option<u64>,
    /// The processes holding it, where they were read.
    pub holders: BTreeSet<Process>,
    /// The address it is bound to.
    pub at: IpAddr,
    /// On every IPv6 address, it takes IPv4's too: not `IPV6_V6ONLY`.
    pub dual: bool,
}

/// A share's port as read once: the sockets on it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Read {
    /// The boot the ids belong to, where they last for one (Linux): the
    /// first 64 bits of its id.
    pub boot: Option<u64>,
    /// Whether a socket's id is never another's while the system runs.
    pub lasting: bool,
    pub sockets: Vec<Socket>,
    /// Something answers on the port that the read can't list: another
    /// account's program, on a Mac.
    pub unlisted: bool,
}

/// What a person approved for a share: the sockets, or the processes
/// holding them, the cgroups they were made in, and the address the door
/// reaches them at. Kept as its fields, each set only where it holds any; a
/// field it doesn't know is refused.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approved {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot: Option<u64>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub sockets: BTreeSet<u64>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub cgroups: BTreeSet<u64>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub processes: BTreeSet<Process>,
    /// 127.0.0.1 or ::1, which an approved socket takes.
    pub at: IpAddr,
    /// A program not approved was seen there since: held until the person
    /// approves again, whatever is there by then.
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    pub held: bool,
}

/// The process a person chose on the port, and what runs now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chosen {
    pub pid: u32,
    /// Its start when it was shown to the person.
    pub start: u64,
    /// The start of the process with that id now; none when none runs.
    pub now: Option<u64>,
    /// Each process's parent, read with the sockets.
    pub parents: BTreeMap<u32, u32>,
}

/// Why nothing is approved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// Nothing answers on the port.
    Nothing,
    /// What answers isn't a program of the account svrngs runs as, and
    /// only that account's can be read here.
    NotYours,
    /// The process chosen stopped: another has its id, or none does.
    Stopped,
    /// A socket there isn't held by the process chosen or a child.
    Another,
    /// No socket on the port takes what is sent to this computer's own
    /// address; one is on the address given.
    Elsewhere(IpAddr),
}

/// What took the connection the door made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Took {
    /// A datagram, which the sockets read alone may receive.
    Datagram,
    /// The cgroup its server end carries (Linux).
    Cgroup(u64),
    /// The process holding its server end, then its parent and theirs,
    /// each by its id and start (a Mac).
    Lineage(Vec<Process>),
    /// Its server end isn't a socket yet, waiting on the door's first bytes
    /// (Linux's `TCP_DEFER_ACCEPT`): the listener its handshake reached
    /// makes it, and that listener is among the sockets read, listening or
    /// bound to the port, while it can make it (uid.rs `sockets`).
    Waiting,
}

/// Why a connection is held.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Held {
    /// Nothing is approved for the share.
    Unapproved,
    /// A program not approved is there, or took the connection.
    Another,
    /// A socket there whose holder can't be read now.
    Unjudged,
    /// Nothing is there now.
    Empty,
}

/// What `read` approves, with `chosen` when the person chose a process:
/// the sockets that take what is sent to 127.0.0.1 when one does, else ::1
/// ([`on`]), each held by it or a child of it.
pub fn approve(read: &Read, chosen: Option<&Chosen>) -> Result<Approved, Refused> {
    let Some(first) = read.sockets.first() else {
        return Err(if read.unlisted {
            Refused::NotYours
        } else {
            Refused::Nothing
        });
    };
    let mine = [
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(Ipv6Addr::LOCALHOST),
    ];
    let at = mine
        .into_iter()
        .find(|to| !on(read, *to).is_empty())
        .ok_or(Refused::Elsewhere(first.at))?;
    let on = on(read, at);
    // A pid@start read before a reboot may name another process after it
    // (Linux's start counts from the boot): accepted (SHARE_BINDING_REVIEWS
    // §6 #11a), since a selection is made on a page this boot's svrngs
    // served, and what is approved is still only the sockets read now.
    if let Some(chosen) = chosen {
        if chosen.now != Some(chosen.start) {
            return Err(Refused::Stopped);
        }
        let within = |&(pid, _): &Process| descends(pid, chosen.pid, &chosen.parents);
        if !on.iter().all(|s| s.holders.iter().any(within)) {
            return Err(Refused::Another);
        }
    }
    let lasting = on.iter().filter(|_| read.lasting);
    let named = on.iter().filter(|_| !read.lasting);
    Ok(Approved {
        boot: read.boot,
        sockets: lasting.map(|s| s.id).collect(),
        cgroups: on.iter().filter_map(|s| s.cgroup).collect(),
        processes: named.flat_map(|s| s.holders.iter().copied()).collect(),
        at,
        held: false,
    })
}

/// The sockets of `read` that take what is sent to `to`, this computer's
/// own address of a family: bound to it, or to every address of its family,
/// or dual, to every IPv6 address. Only these can receive the door's
/// traffic; one on another address neither holds a share nor is approved.
fn on(read: &Read, to: IpAddr) -> Vec<&Socket> {
    let takes = |s: &&Socket| {
        let at = s.at.to_canonical();
        let every = at.is_ipv4() == to.is_ipv4() || s.dual && to.is_ipv4();
        at == to || at.is_unspecified() && every
    };
    read.sockets.iter().filter(takes).collect()
}

/// Whether `pid` is `from` or descends from it, by `parents`, each step a
/// parent that isn't itself, as many as there are parents.
fn descends(pid: u32, from: u32, parents: &BTreeMap<u32, u32>) -> bool {
    let up = |at: &u32| parents.get(at).copied().filter(|parent| parent != at);
    core::iter::successors(Some(pid), up)
        .take(parents.len() + 1)
        .any(|at| at == from)
}

/// Whether a connection `took` may be carried, by what is `approved` and
/// what takes what is sent to its address `now` ([`on`]): nothing not
/// approved seen since; the same boot; at least one socket there, every one
/// approved, by its id or a process holding it; and what took it the
/// approved cgroup, an approved process or one it started, or the listener
/// its handshake reached, among them. A socket there that isn't approved is
/// another's when it is read by its id or held by a process read, and
/// unjudged when neither: a Mac's, whose last user has exited.
pub fn check(approved: Option<&Approved>, now: &Read, took: &Took) -> Result<(), Held> {
    let approved = approved.ok_or(Held::Unapproved)?;
    if approved.held || approved.boot != now.boot {
        return Err(Held::Another);
    }
    let on = on(now, approved.at);
    let by_id = |s: &Socket| now.lasting && approved.sockets.contains(&s.id);
    let ok = |s: &&Socket| by_id(s) || !s.holders.is_disjoint(&approved.processes);
    // Whether each unapproved socket there was seen: read by its id, or
    // held by a process read.
    let unapproved = on.iter().filter(|s| !ok(s));
    let seen = unapproved.map(|s| now.lasting || !s.holders.is_empty());
    match seen.max() {
        _ if on.is_empty() => Err(Held::Empty),
        Some(true) => Err(Held::Another),
        Some(false) => Err(Held::Unjudged),
        None => match took {
            Took::Datagram | Took::Waiting => Ok(()),
            Took::Cgroup(cgroup) if approved.cgroups.contains(cgroup) => Ok(()),
            Took::Lineage(up) if up.iter().any(|p| approved.processes.contains(p)) => Ok(()),
            Took::Cgroup(_) | Took::Lineage(_) => Err(Held::Another),
        },
    }
}

#[cfg(test)]
#[path = "binding_tests.rs"]
mod tests;
