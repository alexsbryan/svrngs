// SPDX-License-Identifier: AGPL-3.0-or-later
// The spike's packet tunnel (../../DESIGN.md §4): it routes nothing but
// 10.77.0.1, and is up only so the system keeps it running; the Rust in it
// serves the lend on 127.0.0.1:8096.
import NetworkExtension
import Tunnel

enum Refused: Error {
    case noContainer, unwritable, started(Int32)
}

final class PacketTunnelProvider: NEPacketTunnelProvider {
    override func startTunnel(options: [String: NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        let settings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "127.0.0.1")
        let ipv4 = NEIPv4Settings(addresses: ["10.77.0.2"], subnetMasks: ["255.255.255.255"])
        ipv4.includedRoutes = [NEIPv4Route(destinationAddress: "10.77.0.1", subnetMask: "255.255.255.255")]
        settings.ipv4Settings = ipv4
        setTunnelNetworkSettings(settings) { error in
            if let error { return completionHandler(error) }
            let container = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: "group.sh.svrnme.phonetunnel")
            guard let logs = container else { return completionHandler(Refused.noContainer) }
            // An absent plant goes as null, for the Rust to refuse: never a default.
            let given = (self.protocolConfiguration as? NETunnelProviderProtocol)?.providerConfiguration
            let (plant, rung) = (given?["plant"], given?["rung"])
            let config: [String: Any] = [
                "seed_hex": Key.seedHex, "lender": Key.lender, "lend": Key.lend,
                "port": 8096, "plant": plant ?? NSNull(), "log_dir": logs.path, "probe": Key.probe ?? NSNull(), "rung": rung ?? NSNull(),
            ]
            guard let json = try? JSONSerialization.data(withJSONObject: config),
                  let text = String(data: json, encoding: .utf8)
            else { return completionHandler(Refused.unwritable) }
            let started = tunnel_start(text)
            // Console.app is the only place a 1 is seen: it means no trace file could be opened.
            NSLog("phone-tunnel: tunnel_start %d", started)
            completionHandler(started == 0 ? nil : Refused.started(started))
        }
    }

    // Counted for the probe's vitals: whether iOS sleeps the provider with a locked phone (PHONE_REPLICA §4).
    override func sleep(completionHandler: @escaping () -> Void) {
        tunnel_slept()
        completionHandler()
    }

    override func wake() {
        tunnel_woke()
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        NSLog("phone-tunnel: stopping, reason %d", reason.rawValue)
        tunnel_stop()
        completionHandler()
    }
}
