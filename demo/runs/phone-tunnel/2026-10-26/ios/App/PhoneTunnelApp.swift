// SPDX-License-Identifier: AGPL-3.0-or-later
// The spike's app (../../DESIGN.md §4): one switch for the tunnel, the plant
// it starts with, and the extension's two files shared out.
import NetworkExtension
import SwiftUI

let appGroup = "group.sh.svrnme.phonetunnel"

enum Plant: String, CaseIterable, Identifiable {
    case none, stranger, leak
    var id: String { rawValue }
}

/// The ladder's step (../../../phone-replica/2026-10-27/PREDICTIONS.md, "The ladder").
enum Rung: String, CaseIterable, Identifiable {
    case bare, relay, periodic, stock
    var id: String { rawValue }
}

@MainActor
final class Tunnel: ObservableObject {
    @Published var on = false
    @Published var plant = Plant.none
    @Published var rung = Rung.stock
    @Published var said = ""
    private var manager: NETunnelProviderManager?

    func load() async {
        manager = (try? await NETunnelProviderManager.loadAllFromPreferences())?.first
        let given = (manager?.protocolConfiguration as? NETunnelProviderProtocol)?.providerConfiguration?["plant"]
        plant = (given as? String).flatMap(Plant.init(rawValue:)) ?? .none
        let step = (manager?.protocolConfiguration as? NETunnelProviderProtocol)?.providerConfiguration?["rung"]
        rung = (step as? String).flatMap(Rung.init(rawValue:)) ?? .stock
        on = [.connected, .connecting, .reasserting].contains(manager?.connection.status)
    }

    func turn(_ on: Bool) async {
        let manager = manager ?? NETunnelProviderManager()
        let proto = NETunnelProviderProtocol()
        proto.providerBundleIdentifier = "sh.svrnme.phonetunnel.tunnel"
        proto.serverAddress = "127.0.0.1"
        proto.providerConfiguration = ["plant": plant.rawValue, "rung": rung.rawValue]
        manager.protocolConfiguration = proto
        manager.localizedDescription = "Phone tunnel spike"
        manager.isEnabled = true
        // iOS starts it again whenever it stops it, as a shipped app would ask; off with the switch.
        manager.onDemandRules = [NEOnDemandRuleConnect()]
        manager.isOnDemandEnabled = on
        do {
            try await manager.saveToPreferences()
            // A manager just saved starts only once loaded again.
            try await manager.loadFromPreferences()
            self.manager = manager
            if on { try manager.connection.startVPNTunnel() } else { manager.connection.stopVPNTunnel() }
            said = on ? "Started, with the plant \(plant.rawValue), on the step \(rung.rawValue)" : "Stopped"
        } catch {
            self.on = !on
            said = error.localizedDescription
        }
    }
}

@main
struct PhoneTunnelApp: App {
    @StateObject private var tunnel = Tunnel()
    private let logs = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroup)

    var body: some Scene {
        WindowGroup {
            Form {
                Toggle("Tunnel", isOn: Binding(get: { tunnel.on }, set: { on in
                    tunnel.on = on
                    Task { await tunnel.turn(on) }
                }))
                Picker("Plant", selection: $tunnel.plant) {
                    ForEach(Plant.allCases) { Text($0.rawValue).tag($0) }
                }.disabled(tunnel.on)
                Picker("Step", selection: $tunnel.rung) {
                    ForEach(Rung.allCases) { Text($0.rawValue).tag($0) }
                }.disabled(tunnel.on)
                if let logs {
                    ShareLink(items: [logs.appending(path: "samples.jsonl"), logs.appending(path: "trace.log")]) {
                        Label("Share samples and trace", systemImage: "square.and.arrow.up")
                    }
                }
                Text(tunnel.said)
            }.task { await tunnel.load() }
        }
    }
}
