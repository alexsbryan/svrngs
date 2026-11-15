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
//!   this computer is on, per platform ([`routers`]), its address there,
//!   and the VPN's tunnel its traffic leaves by, if any ([`Here`]);
//!   [`lan`] chooses the interface, the one beside any VPN's tunnel, and
//!   [`hardware_in`] reads what that finds, both pure; [`now`], off the
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
    /// The outward route is a tunnel, and more than one network beside it
    /// has a router (WiFi and a cable, both on): which one the devices here
    /// are on is not told, so neither is guessed.
    SeveralNetworks,
    /// The router's hardware address is not kept here, or not resolved.
    NoHardwareAddress,
    /// The read stopped before it answered.
    Stopped,
}

/// Where this computer is now, as [`here`] reads it: the network it is
/// on, or why that cannot be read; its address there for the devices
/// beside it ([`pick`]); and the tunnel its outward route leaves by when
/// the network was read beside one: a VPN carrying this computer's
/// traffic, which may also refuse those devices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Here {
    pub network: Result<Network, Unread>,
    pub address: Option<Ipv4Addr>,
    pub tunnel: Option<String>,
}

/// The interface whose router names the network this computer is on, and
/// the tunnel its outward route leaves by when that is not it. The outward
/// route's interface (`default`) when it is on a network ([`on_a_network`]);
/// else, when a VPN carries the traffic, the one interface among `all` that
/// is: the VPN moves where traffic goes, not which network the devices
/// beside this computer are on, so a café's router still reads as the
/// café's. None on a network, [`Unread::NoRouter`]; more than one, which is
/// not told, [`Unread::SeveralNetworks`]. Pure, judged with netdev's own
/// interfaces, built (`network_tests.rs`).
pub fn lan<'a>(
    default: Option<&'a Face>,
    all: &'a [Face],
) -> Result<(&'a Face, Option<&'a str>), Unread> {
    if let Some(face) = default.filter(|f| on_a_network(f)) {
        return Ok((face, None));
    }
    let tunnel = default.filter(|f| tunnel(f)).map(|f| f.name.as_str());
    let beside: Vec<&Face> = all.iter().filter(|f| on_a_network(f)).collect();
    tracing::debug!(target: "network", default = ?default.map(|f| &f.name), ?tunnel, beside = ?beside.iter().map(|f| &f.name).collect::<Vec<_>>(),
        "network: the outward route's interface when it is on a network, else the one beside it that is, or why none");
    match beside[..] {
        [one] => Ok((one, tunnel)),
        [] => Err(Unread::NoRouter),
        _ => Err(Unread::SeveralNetworks),
    }
}

/// An interface, as netdev reads it.
pub type Face = netdev::Interface;

/// Whether a router on `face` can be told by its hardware address: an
/// interface on a network devices stand on, not a tunnel to one far off,
/// whatever made it.
fn on_a_network(face: &Face) -> bool {
    face.is_up() && !face.is_loopback() && !tunnel(face) && router_of(face).is_some()
}

/// Point-to-point, as every tunnel is (WireGuard's and OpenVPN's tun on
/// Linux, macOS's utun), or with no hardware address, which none has.
fn tunnel(face: &Face) -> bool {
    face.is_point_to_point() || face.mac_addr.is_none_or(|m| m.octets() == [0; 6])
}

/// `face`'s router, the default route's gateway on it.
fn router_of(face: &Face) -> Option<Ipv4Addr> {
    face.gateway.as_ref()?.ipv4.first().copied()
}

/// Where this computer is now ([`Here`]): the interface [`lan`] chooses,
/// its router's hardware address from where this computer keeps it for
/// that interface ([`routers`], [`hardware_in`]), and its address there
/// ([`pick`]). The one read of the world here; never a guess.
pub fn here() -> Here {
    let (default, all) = (netdev::get_default_interface(), netdev::get_interfaces());
    let chosen = lan(default.as_ref().ok(), &all);
    let face = chosen.as_ref().ok().map(|(f, _)| *f);
    let listed: Vec<IpAddr> =
        face.map_or(vec![], |f| f.ipv4.iter().map(|n| n.addr().into()).collect());
    let tunnel = chosen.as_ref().ok().and_then(|(_, t)| t.map(String::from));
    let read = |f: &Face| hardware_in(&routers(&f.name)?, router_of(f)?);
    let network = chosen.and_then(|(f, _)| read(f).map(Network).ok_or(Unread::NoHardwareAddress));
    tracing::debug!(target: "network", error = ?default.err(), ?network, ?listed, ?tunnel, "network: read by its router's hardware address, or why not");
    Here {
        network,
        address: pick(&listed, route()),
        tunnel,
    }
}

/// [`here`], read off the async runtime, as its callers ask it.
pub async fn now() -> Here {
    let read = tokio::task::spawn_blocking(here).await;
    read.unwrap_or(Here {
        network: Err(Unread::Stopped),
        address: None,
        tunnel: None,
    })
}

/// Where this computer keeps its router's hardware address for `iface`:
/// on macOS, the state in configd (`scutil`) of the network service on
/// `iface` ([`service_on`]), as macOS resolved it for the network's own
/// signature, since the ARP table is hidden from a program not let find
/// devices on the network (`arp -n` answered "no entry" to the tests'
/// process here, and its full answer to a terminal); on Linux, the ARP
/// table, `/proc/net/arp`, every interface's.
#[cfg(target_os = "macos")]
fn routers(iface: &str) -> Option<String> {
    use std::process::{Command, Stdio};
    let scutil = |command: &str| {
        let mut scutil = Command::new("/usr/sbin/scutil");
        let mut scutil = (scutil.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()).ok()?;
        std::io::Write::write_all(&mut scutil.stdin.take()?, format!("{command}\n").as_bytes())
            .ok()?;
        String::from_utf8(scutil.wait_with_output().ok()?.stdout).ok()
    };
    service_on(iface, scutil)
}
#[cfg(not(target_os = "macos"))]
fn routers(_iface: &str) -> Option<String> {
    std::fs::read_to_string("/proc/net/arp").ok()
}

/// The IPv4 state of the network service on `iface`, as `scutil` answers
/// each command given it: the primary service's when it is on `iface`, as
/// it is with no VPN; else the one service whose state names `iface` (a
/// VPN's service is primary while it carries the traffic). Pure but for
/// `scutil`, so it is judged with planted answers (`network_tests.rs`).
#[cfg(any(target_os = "macos", test))]
fn service_on(iface: &str, scutil: impl Fn(&str) -> Option<String>) -> Option<String> {
    // The value `scutil` prints after `key : ` in `text`.
    let value = |text: &str, key: &str| {
        text.lines()
            .find_map(|l| Some(l.trim().strip_prefix(key)?.to_string()))
    };
    let global = scutil("show State:/Network/Global/IPv4")?;
    let primary =
        value(&global, "PrimaryService : ").map(|s| format!("State:/Network/Service/{s}/IPv4"));
    let listed = scutil("list State:/Network/Service/[^/]+/IPv4").unwrap_or_default();
    let keys = listed
        .lines()
        .filter_map(|l| Some(l.split(" = ").nth(1)?.trim().to_string()));
    let on = |state: &String| value(state, "InterfaceName : ").as_deref() == Some(iface);
    let mut services = primary.into_iter().chain(keys);
    let found = services.find_map(|key| scutil(&format!("show {key}")).filter(on));
    tracing::debug!(target: "network", iface, found = found.is_some(), "network: the service on the interface, the primary's first");
    found
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
