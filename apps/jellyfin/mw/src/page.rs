// SPDX-License-Identifier: AGPL-3.0-or-later
//! jellyfin-web's stored sign-in, written by the script the sign-in answers
//! (u5-3 finding 4, its third piece; demo/runs/mw/2026-10-08/README.md §1-2).
//! What jellyfin-web is signed in as lives in its own storage at the
//! address it is opened at (`jellyfin_credentials`); with none it asks
//! "Please sign in". So the answer carries a script, which svrngs's door
//! puts first in `<head>` of the person's pages, ahead of every script of
//! jellyfin-web's (all `defer`), naming the person's account, the server,
//! the address the page is opened at and [`STAND_IN`] for a token: what
//! U3's dry run's driver wrote (demo/runs/a/dry-2026-10-04/drive.mjs:41-47),
//! now written by what the lender runs, never by the infra. It names the
//! person's device too (`_deviceId2`, where jellyfin-web keeps its own),
//! so the socket jellyfin-web opens names the device the door signs in.

use std::net::SocketAddr;
use std::time::Duration;

/// What a browser holds in place of a token: svrngs's door's stand-in
/// (crates/svrngs-node/src/door/sign_in.rs, `STAND_IN`), which it swaps for
/// the person's token in what it carries in, and back in every answer.
/// Spelled here, not linked: svrngs ships no SDK (DESIGN §3, call 6).
pub(crate) const STAND_IN: &str = "svrngs-stand-in";

/// The server jellyfin-web's stored sign-in names, read once from
/// Jellyfin's `/System/Info/Public` as the middleware starts.
#[derive(Clone, Debug)]
pub(crate) struct Server {
    pub(crate) id: String,
    pub(crate) name: String,
}

impl Server {
    pub(crate) async fn read(jellyfin: SocketAddr) -> Result<Server, String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| format!("could not build an HTTP client: {e}"))?;
        let info: serde_json::Value = client
            .get(format!("http://{jellyfin}/System/Info/Public"))
            .send()
            .await
            .map_err(|e| format!("Jellyfin at {jellyfin} did not answer: {e}"))?
            .json()
            .await
            .map_err(|e| format!("Jellyfin's /System/Info/Public is not JSON: {e}"))?;
        let field = |k: &str| {
            info.get(k)
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .ok_or(format!("Jellyfin's /System/Info/Public has no {k}"))
        };
        Ok(Server {
            id: field("Id")?,
            name: field("ServerName")?,
        })
    }
}

/// A string as a script literal that cannot end the `<script>` it is in.
fn literal(s: &str) -> String {
    serde_json::Value::from(s).to_string().replace('<', "\\u003c")
}

/// The script that writes the stored sign-in: the person's account `user`
/// on `server`, at whatever address the page is opened at — a computer's
/// loopback origin or a phone's — with `token` for its token, on the
/// person's `device`. It writes over what is there: who the page is signed
/// in as is the door's word. Bare JavaScript: the door puts it in a
/// `<script>` of its own.
pub(crate) fn script(server: &Server, user: &str, token: &str, device: &str) -> String {
    format!(
        "/* svrngs's Jellyfin sign-in: signed in as the person svrngs's door named */\
         localStorage.setItem('_deviceId2',{});\
         localStorage.setItem('jellyfin_credentials',JSON.stringify({{Servers:[{{\
         ManualAddress:location.origin,LocalAddress:location.origin,LastConnectionMode:2,ManualAddressOnly:true,\
         Name:{},Id:{},DateLastAccessed:Date.now(),UserId:{},AccessToken:{}}}]}}));",
        literal(device),
        literal(&server.name),
        literal(&server.id),
        literal(user),
        literal(token)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_script_names_the_person_and_their_device_and_cannot_be_ended_by_a_name() {
        let s = script(
            &Server {
                id: "abc".into(),
                name: "Sam's </script><b>".into(),
            },
            "u1",
            STAND_IN,
            "svrngs-0123456789abcdef",
        );
        assert!(s.contains("UserId:\"u1\""));
        assert!(s.contains("AccessToken:\"svrngs-stand-in\""));
        assert!(s.contains("ManualAddress:location.origin"));
        assert!(s.contains("'_deviceId2',\"svrngs-0123456789abcdef\""));
        assert!(!s.to_ascii_lowercase().contains("</script"), "{s}");
    }
}
