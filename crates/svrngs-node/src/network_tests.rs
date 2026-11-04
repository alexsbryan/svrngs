// SPDX-License-Identifier: AGPL-3.0-or-later
//! What this computer reads of its network, over planted text: each
//! platform's way of keeping its router's hardware address. The address
//! picker's lists are judged beside the room (`room_tests.rs`), where the
//! trust floor holds them as a gate; a caller's address, beside the door's
//! count (`door/limit/tests.rs`).

use std::net::Ipv4Addr;

use super::{hardware_in, HardwareAddress};

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
