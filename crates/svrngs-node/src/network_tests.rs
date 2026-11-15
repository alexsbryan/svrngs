// SPDX-License-Identifier: AGPL-3.0-or-later
//! What this computer reads of its network, over planted text: each
//! platform's way of keeping its router's hardware address. The address
//! picker's lists are judged beside the room (`room_tests.rs`), where the
//! trust floor holds them as a gate; a caller's address, beside the door's
//! count (`door/limit/tests.rs`).

use std::net::Ipv4Addr;

use super::{hardware_in, lan, service_on, Face, HardwareAddress, Unread};

/// **The router's hardware address, as each platform keeps it** (`look`):
/// configd's state for the primary service on macOS, as `scutil` prints
/// it, and Linux's `/proc/net/arp`, its octets read to one spelling; the
/// router absent, or its address incomplete (all zeros), none. Planted,
/// watched failing: the octets kept as printed, `a` apart from `0a`.
#[test]
fn the_routers_hardware_address_is_read_as_each_platform_keeps_it() {
    let router: Ipv4Addr = "192.168.1.1".parse().unwrap();
    let mac = "<dictionary> {\n  ARPResolvedHardwareAddress : e4:f4:c6:a:5e:a1\n  \
               ARPResolvedIPAddress : 192.168.1.1\n  Router : 192.168.1.1\n}\n";
    let linux = "IP address       HW type     Flags       HW address            Mask     Device\n\
                 192.168.1.7      0x1         0x2         11:22:33:44:55:66     *        wlan0\n\
                 192.168.1.1      0x1         0x2         e4:f4:c6:0a:5e:a1     *        wlan0\n";
    let want = HardwareAddress::parse("e4:f4:c6:0a:5e:a1");
    assert_eq!(hardware_in(mac, router), want);
    assert_eq!(hardware_in(linux, router), want);
    let zeros = "192.168.1.1 0x1 0x0 00:00:00:00:00:00 * wlan0\n";
    assert_eq!(hardware_in(zeros, router), None);
    assert_eq!(hardware_in(linux, "192.168.1.9".parse().unwrap()), None);
    assert_eq!(hardware_in("<dictionary> {\n}\n", router), None);
}

/// **A hardware address is six bytes, said padded** (`HardwareAddress`):
/// one digit or two per octet read alike, said back as two, and kept as
/// that text in room.json; anything else, and all zeros, is none.
#[test]
fn a_hardware_address_is_six_bytes_said_padded() {
    let mac = HardwareAddress::parse("e4:f4:c6:a:5e:a1").unwrap();
    assert_eq!(mac.to_string(), "e4:f4:c6:0a:5e:a1");
    assert_eq!(
        serde_json::to_string(&mac).unwrap(),
        r#""e4:f4:c6:0a:5e:a1""#
    );
    assert_eq!(
        serde_json::from_str::<HardwareAddress>(r#""e4:f4:c6:0a:5e:a1""#).unwrap(),
        mac
    );
    for not in [
        "e4:f4:c6:0a:5e",
        "e4:f4:c6:0a:5e:a1:00",
        "e4:f4:c6:0a:5e:zz",
        "00:00:00:00:00:00",
        "",
    ] {
        assert_eq!(HardwareAddress::parse(not), None, "{not}");
    }
}

// The interface flags netdev reads them by, alike on Linux and macOS
// (`<net/if.h>`): up, loopback, point-to-point.
const UP: u32 = 0x1;
const LOOPBACK: u32 = 0x8;
const POINT_TO_POINT: u32 = 0x10;

/// An interface as netdev reads one, `name` up on a network with `router`
/// and a hardware address, unless the case changes it.
fn on(name: &str, router: &str, ip: &str) -> Face {
    let mut face = Face::dummy();
    face.name = name.into();
    face.flags = UP;
    face.mac_addr = Some(netdev::MacAddr::new(2, 0, 0, 0, 0, 1));
    face.gateway = Some(netdev::NetworkDevice {
        mac_addr: netdev::MacAddr::zero(),
        ipv4: vec![router.parse().unwrap()],
        ipv6: vec![],
    });
    face.ipv4 = vec![netdev::ipnet::Ipv4Net::new(ip.parse().unwrap(), 24).unwrap()];
    face
}

/// A tunnel: point-to-point, with no hardware address, and a router of its
/// own when `router` names one (OpenVPN's tun can have one; WireGuard's
/// has none).
fn tunnel(name: &str, router: Option<&str>, ip: &str) -> Face {
    let mut face = on(name, router.unwrap_or("0.0.0.0"), ip);
    face.flags = UP | POINT_TO_POINT;
    face.mac_addr = None;
    if router.is_none() {
        face.gateway = None;
    }
    face
}

/// `face` with no router.
fn routerless(mut face: Face) -> Face {
    face.gateway = None;
    face
}

/// **The network is the one the devices beside this computer are on,
/// whatever VPN carries its traffic** (`lan`): with no VPN, the outward
/// route's interface; with any VPN's tunnel carrying the traffic
/// (WireGuard's, with no router, as Mullvad's `wg0-mullvad` on the Fedora
/// box, 2026-11-14; OpenVPN's tun, with one; macOS's utun), the one
/// interface beside it on a network, and the tunnel said; Tailscale with
/// no exit node, beside, changes nothing. None on a network is no router;
/// WiFi and a cable both on behind a VPN is not told, never guessed; an
/// interface down, loopback, or with no router is never chosen. Planted,
/// watched failing: before `lan`, the outward route's interface alone,
/// `wg0-mullvad`, read as no router.
#[test]
fn the_network_is_the_one_beside_this_computer_whatever_vpn_carries_its_traffic() {
    let wifi = on("wlp192s0", "192.168.1.1", "192.168.1.12");
    let tailscale = tunnel("tailscale0", None, "100.115.12.21");
    let mut lo = routerless(on("lo", "0.0.0.0", "127.0.0.1"));
    lo.flags = UP | LOOPBACK;
    let cable_unplugged = routerless(on("enp191s0", "0.0.0.0", "192.168.2.5"));
    let pick = |default: &Face, all: &[Face]| {
        lan(Some(default), all).map(|(f, t)| (f.name.clone(), t.map(String::from)))
    };
    let named = |face: &str, tunnel: Option<&str>| Ok((face.into(), tunnel.map(String::from)));

    let home = [
        lo.clone(),
        wifi.clone(),
        cable_unplugged.clone(),
        tailscale.clone(),
    ];
    assert_eq!(pick(&wifi, &home), named("wlp192s0", None), "no VPN");

    let wireguard = tunnel("wg0-mullvad", None, "10.156.8.102");
    let mut all = home.to_vec();
    all.push(wireguard.clone());
    assert_eq!(
        pick(&wireguard, &all),
        named("wlp192s0", Some("wg0-mullvad")),
        "WireGuard"
    );

    let openvpn = tunnel("tun0", Some("10.8.0.1"), "10.8.0.6");
    let all = [wifi.clone(), openvpn.clone()];
    assert_eq!(
        pick(&openvpn, &all),
        named("wlp192s0", Some("tun0")),
        "OpenVPN, a router of its own"
    );

    let utun = tunnel("utun4", None, "10.64.0.2");
    let en0 = on("en0", "192.168.1.1", "192.168.1.20");
    assert_eq!(
        pick(&utun, &[en0, utun.clone()]),
        named("en0", Some("utun4")),
        "macOS"
    );

    let exit_node = tunnel("tailscale0", None, "100.115.12.21");
    assert_eq!(
        pick(&exit_node, &home),
        named("wlp192s0", Some("tailscale0")),
        "a Tailscale exit node"
    );

    let cafe = on("wlp192s0", "10.0.0.1", "10.0.0.40");
    assert_eq!(
        pick(&wireguard, &[cafe, wireguard.clone()]),
        named("wlp192s0", Some("wg0-mullvad")),
        "the café's own router"
    );

    assert_eq!(
        pick(&wireguard, &[lo.clone(), wireguard.clone()]),
        Err(Unread::NoRouter),
        "no network"
    );
    let cable = on("enp191s0", "192.168.1.1", "192.168.1.30");
    assert_eq!(
        pick(&wireguard, &[wifi.clone(), cable, wireguard.clone()]),
        Err(Unread::SeveralNetworks),
        "WiFi and a cable"
    );
    let mut down = wifi.clone();
    down.flags = 0;
    assert_eq!(
        pick(&wireguard, &[down, wireguard.clone()]),
        Err(Unread::NoRouter),
        "WiFi down"
    );
    assert_eq!(
        pick(&cable_unplugged, &[cable_unplugged.clone()]),
        Err(Unread::NoRouter),
        "a cable with no router"
    );
    assert_eq!(
        lan(None, &[wifi.clone()]).map(|(f, t)| (f.name.clone(), t)),
        Ok(("wlp192s0".into(), None)),
        "no outward route"
    );
}

/// **On macOS, the router is read from the service on the interface
/// chosen** (`service_on`): the primary service's state when it is on that
/// interface, as with no VPN; else, a VPN's service primary, the one whose
/// state names the interface, as `scutil`'s `list` names each; none found,
/// none. Planted, watched failing: the primary's alone, a VPN's, with no
/// router's hardware address in it. Unread on a Mac yet: the shapes are
/// `scutil`'s, as `show` prints a service's state.
#[test]
fn on_macos_the_router_is_read_from_the_service_on_the_interface_chosen() {
    let state = |iface: &str, extra: &str| {
        format!("<dictionary> {{\n  Addresses : <array> {{\n  }}\n  InterfaceName : {iface}\n{extra}}}\n")
    };
    let wifi = state(
        "en0",
        "  ARPResolvedHardwareAddress : e4:f4:c6:a:5e:a1\n  Router : 192.168.1.1\n",
    );
    let vpn = state("utun4", "  Router : 10.64.0.1\n");
    let scutil = |primary: &'static str| {
        let (wifi, vpn) = (wifi.clone(), vpn.clone());
        move |command: &str| {
            match command {
            "show State:/Network/Global/IPv4" => Some(format!("<dictionary> {{\n  PrimaryInterface : x\n  PrimaryService : {primary}\n}}\n")),
            "list State:/Network/Service/[^/]+/IPv4" => Some("  subKey [0] = State:/Network/Service/VPN-1/IPv4\n  subKey [1] = State:/Network/Service/WIFI-1/IPv4\n".into()),
            "show State:/Network/Service/WIFI-1/IPv4" => Some(wifi.clone()),
            "show State:/Network/Service/VPN-1/IPv4" => Some(vpn.clone()),
            _ => None,
        }
        }
    };
    assert_eq!(
        service_on("en0", scutil("WIFI-1")),
        Some(wifi.clone()),
        "no VPN: the primary"
    );
    assert_eq!(
        service_on("en0", scutil("VPN-1")),
        Some(wifi.clone()),
        "a VPN primary: the service on en0"
    );
    assert_eq!(
        service_on("utun4", scutil("WIFI-1")),
        Some(vpn.clone()),
        "whichever is on the interface asked"
    );
    assert_eq!(service_on("en7", scutil("VPN-1")), None, "none on it");
    let router: Ipv4Addr = "192.168.1.1".parse().unwrap();
    let read = service_on("en0", scutil("VPN-1")).and_then(|t| hardware_in(&t, router));
    assert_eq!(read, HardwareAddress::parse("e4:f4:c6:0a:5e:a1"));
}
