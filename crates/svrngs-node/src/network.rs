// SPDX-License-Identifier: AGPL-3.0-or-later
//! Where this computer is, and on which network: what anything here asks
//! that turns on "the network where this was set" or "a caller on my
//! network" (DESIGN §3, "The room" and "The local network is guessed").
//!
//! - [`Network`]: a network told apart by its router's hardware address,
//!   as macOS's own network signature tells it, so a café whose router
//!   gives this computer the same private address is not home.
//! - [`HardwareAddress`], parsed from what each platform prints, and
//!   [`Unread`], why a network cannot be read, so a caller can say which.
//! - [`here`], the edge and the one read of the world here: the network
//!   this computer is on, per platform ([`routers`]), and its address
//!   there; [`hardware_in`] reads what that finds, pure; [`now`], off the
//!   async runtime.
//! - [`pick`] and [`route`]: this computer's address for the devices beside
//!   it.
//! - [`guess`]: whether a caller's address is on this computer's network,
//!   the one decider of "this network" for the door's count and the room's
//!   listeners.
//!
//! New, in the shape of upstream's address listing and route probe, not
//! their code (PLAN §4, U6's inventory; `commonwealth-discovery/src/
//! mesh_discovery.rs:237-310` at `e1a43bcd2`, `local_ip_candidates`, whose
//! classes are strings there); upstream reads no router, so a network's
//! identity is svrngs's own.

use std::net::{IpAddr, Ipv4Addr};

use serde::{Deserialize, Serialize};

/// A device's hardware (MAC) address: six bytes, said as six pairs of hex
/// digits, `e4:f4:c6:0a:5e:a1`, and kept so.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct HardwareAddress([u8; 6]);

impl HardwareAddress {
    /// `text` as one: six octets in hex between colons, one digit or two
    /// (macOS writes `a` for `0a`); `None` for anything else, and for all
    /// zeros, which an ARP table writes for one it has not resolved.
    pub fn parse(text: &str) -> Option<Self> {
        let octets = text.split(':').map(|o| u8::from_str_radix(o, 16).ok());
        let octets: [u8; 6] = octets.collect::<Option<Vec<u8>>>()?.try_into().ok()?;
        (octets != [0; 6]).then_some(Self(octets))
    }
}

impl std::fmt::Display for HardwareAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [a, b, c, d, e, g] = self.0;
        write!(f, "{a:02x}:{b:02x}:{c:02x}:{d:02x}:{e:02x}:{g:02x}")
    }
}

impl From<HardwareAddress> for String {
    fn from(address: HardwareAddress) -> String {
        address.to_string()
    }
}

impl TryFrom<String> for HardwareAddress {
    type Error = String;
    fn try_from(text: String) -> Result<Self, String> {
        Self::parse(&text).ok_or(text)
    }
}

/// A network, by its router's hardware address: what tells home from a
/// café whose router gives this computer the same private address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Network(pub HardwareAddress);

/// Why the network this computer is on cannot be read: a closed set
/// (AGENTS 9), so a caller can say which.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unread {
    /// No default route, or no router on it.
    NoRouter,
    /// The router's hardware address is not kept here, or not resolved.
    NoHardwareAddress,
    /// The read stopped before it answered.
    Stopped,
}

/// Where this computer is now: the network it is on, or why that cannot be
/// read, and its address there for the devices beside it ([`pick`]): read
/// from its default route's interface (netdev), its router's hardware
/// address from where this computer keeps it ([`routers`],
/// [`hardware_in`]). The one read of the world here; never a guess.
pub fn here() -> (Result<Network, Unread>, Option<Ipv4Addr>) {
    let iface = netdev::get_default_interface();
    let router = (iface.as_ref().ok()).and_then(|i| i.gateway.as_ref()?.ipv4.first().copied());
    let addrs = |i: &netdev::Interface| i.ipv4.iter().map(|n| IpAddr::V4(n.addr())).collect();
    let listed: Vec<IpAddr> = iface.as_ref().map_or(Vec::new(), addrs);
    let network = match router {
        None => Err(Unread::NoRouter),
        Some(r) => (routers().and_then(|t| hardware_in(&t, r)).map(Network))
            .ok_or(Unread::NoHardwareAddress),
    };
    tracing::debug!(target: "network", error = ?iface.err(), ?router, ?network, ?listed, "network: read by its router's hardware address, or why not");
    (network, pick(&listed, route()))
}

/// [`here`], read off the async runtime, as its callers ask it.
pub async fn now() -> (Result<Network, Unread>, Option<Ipv4Addr>) {
    tokio::task::spawn_blocking(here)
        .await
        .unwrap_or((Err(Unread::Stopped), None))
}

/// Where this computer keeps its router's hardware address: on macOS, the
/// primary network service's state in configd (`scutil`), as macOS
/// resolved it for the network's own signature, since the ARP table is
/// hidden from a program not let find devices on the network (`arp -n`
/// answered "no entry" to the tests' process here, and its full answer to a
/// terminal); on Linux, the ARP table, `/proc/net/arp`.
#[cfg(target_os = "macos")]
fn routers() -> Option<String> {
    use std::process::{Command, Stdio};
    let show = |key: &str| {
        let mut scutil = Command::new("/usr/sbin/scutil");
        let mut scutil = (scutil.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()).ok()?;
        std::io::Write::write_all(
            &mut scutil.stdin.take()?,
            format!("show {key}\n").as_bytes(),
        )
        .ok()?;
        String::from_utf8(scutil.wait_with_output().ok()?.stdout).ok()
    };
    let global = show("State:/Network/Global/IPv4")?;
    let service = global
        .lines()
        .find_map(|l| l.trim().strip_prefix("PrimaryService : "))?;
    show(&format!("State:/Network/Service/{service}/IPv4"))
}
#[cfg(not(target_os = "macos"))]
fn routers() -> Option<String> {
    std::fs::read_to_string("/proc/net/arp").ok()
}

/// The hardware address of `router` in `text`, as [`routers`] reads it:
/// configd's `ARPResolvedHardwareAddress` line (macOS), or the router's row
/// of the ARP table (Linux); `None` when absent or incomplete (all zeros).
pub fn hardware_in(text: &str, router: Ipv4Addr) -> Option<HardwareAddress> {
    let router = router.to_string();
    let word = text
        .lines()
        .find_map(|l| match l.split_whitespace().collect::<Vec<_>>()[..] {
            ["ARPResolvedHardwareAddress", ":", mac] => Some(mac),
            [ip, _, _, mac, ..] if ip == router => Some(mac),
            _ => None,
        })?;
    HardwareAddress::parse(word)
}

/// The address a room's devices reach this computer at, from the
/// addresses its default route's interface lists ([`look`]) and the one
/// its outward route leaves from ([`route`]): that one when it is a
/// private IPv4 address listed; else the one private IPv4 address listed;
/// else `None`, absence, said so by the caller. Never loopback, a wildcard,
/// link-local, tailscale's 100.64/10 — a device on the room's WiFi cannot
/// reach it, so "the LAN before tailscale" upstream measured is "instead
/// of" here — or a public address. A container's bridge is private too,
/// and is told from the LAN only by the route, which does not leave by it.
pub fn pick(listed: &[IpAddr], route: Option<IpAddr>) -> Option<Ipv4Addr> {
    let v4 = |a: &IpAddr| match a.to_canonical() {
        IpAddr::V4(v4) if v4.is_private() => Some(v4),
        _ => None,
    };
    let private: Vec<Ipv4Addr> = listed.iter().filter_map(v4).collect();
    let routed = route.as_ref().and_then(v4).filter(|r| private.contains(r));
    let picked = routed.or(match private[..] {
        [one] => Some(one),
        _ => None,
    });
    tracing::debug!(target: "network", ?listed, ?route, ?private, ?picked, "network: the address picked — the route's if private and listed, else the one private, else none");
    picked
}

/// The address this computer's outward route leaves from: a UDP socket
/// connected to a public address, which sends nothing (upstream's
/// fallback, `commonwealth-discovery/src/mesh_discovery.rs:290-298`).
pub fn route() -> Option<IpAddr> {
    let probe = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    probe.connect("1.1.1.1:80").ok()?;
    probe.local_addr().ok().map(|a| a.ip())
}

/// Where the door guesses a viewer is, from their path's address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Whereabouts {
    /// In the lender's home: a private, link-local or unique-local address.
    Home,
    /// Anywhere else — a public address, a relayed path, or no path seen.
    Away,
}

/// `addr`, the address a viewer's path ends at, or `None` when it ends at a
/// relay or none was seen: home when it is private (10/8, 172.16/12,
/// 192.168/16), link-local (169.254/16, fe80::/10) or unique-local
/// (fc00::/7), an IPv4 address written as IPv6 read as IPv4; else away.
/// Loopback is away: no other computer reaches this one over it.
pub fn guess(addr: Option<IpAddr>) -> Whereabouts {
    let home = match addr.map(|a| a.to_canonical()) {
        Some(IpAddr::V4(v4)) => v4.is_private() || v4.is_link_local(),
        Some(IpAddr::V6(v6)) => v6.is_unique_local() || v6.is_unicast_link_local(),
        None => false,
    };
    if home {
        Whereabouts::Home
    } else {
        Whereabouts::Away
    }
}

#[cfg(test)]
#[path = "network_tests.rs"]
mod tests;
