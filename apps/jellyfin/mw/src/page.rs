// SPDX-License-Identifier: AGPL-3.0-or-later
//! jellyfin-web's stored sign-in, written by the page the middleware
//! answers (u5-3 finding 4, its third piece; README §1-2). What jellyfin-web
//! is signed in as lives in its own storage at the address it is opened at
//! (`jellyfin_credentials`); with none it asks "Please sign in". So the
//! middleware puts a script of its own first in `<head>` of the page it
//! forwards, ahead of every script of jellyfin-web's (all `defer`), naming
//! the person's account, the server, the address the page is opened at
//! and [`swap::STAND_IN`] for a token: what U3's dry run's driver wrote
//! (demo/runs/a/dry-2026-10-04/drive.mjs:41-47), now written by what the
//! lender runs, never by the infra.

use std::net::SocketAddr;
use std::time::Duration;

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
/// loopback origin or a phone's — with `token` for its token. It writes
/// over what is there: who the page is signed in as is the door's word.
pub(crate) fn script(server: &Server, user: &str, token: &str) -> String {
    format!(
        "<script>/* svrngs's Jellyfin middleware: signed in as the person svrngs's door named */\
         localStorage.setItem('jellyfin_credentials',JSON.stringify({{Servers:[{{\
         ManualAddress:location.origin,LocalAddress:location.origin,LastConnectionMode:2,ManualAddressOnly:true,\
         Name:{},Id:{},DateLastAccessed:Date.now(),UserId:{},AccessToken:{}}}]}}));</script>",
        literal(&server.name),
        literal(&server.id),
        literal(user),
        literal(token)
    )
}

/// `page` with `script` put just after its `<head…>`, or after `<html…>`
/// or at its start when it has none; `None` when it is not text.
pub(crate) fn written(page: &[u8], script: &str) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(page).ok()?;
    let lower = text.to_ascii_lowercase();
    let after = |tag: &str| {
        let open = lower.find(tag)?;
        lower[open..].find('>').map(|close| open + close + 1)
    };
    let at = after("<head").or_else(|| after("<html")).unwrap_or(0);
    Some([&text[..at], script, &text[at..]].concat().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_script_goes_first_in_head() {
        let page =
            b"<!DOCTYPE html><html lang=\"en\"><head><script defer=\"defer\" src=\"a.js\"></script></head></html>";
        let out = String::from_utf8(written(page, "<script>S</script>").unwrap()).unwrap();
        assert_eq!(
            out,
            "<!DOCTYPE html><html lang=\"en\"><head><script>S</script><script defer=\"defer\" src=\"a.js\"></script></head></html>"
        );
        let bare = String::from_utf8(written(b"<p>x</p>", "<s>").unwrap()).unwrap();
        assert_eq!(bare, "<s><p>x</p>");
    }

    #[test]
    fn the_script_names_the_person_and_cannot_be_ended_by_a_name() {
        let s = script(
            &Server {
                id: "abc".into(),
                name: "Sam's </script><b>".into(),
            },
            "u1",
            "svrngs-stand-in",
        );
        assert!(s.contains("UserId:\"u1\""));
        assert!(s.contains("AccessToken:\"svrngs-stand-in\""));
        assert!(s.contains("ManualAddress:location.origin"));
        assert_eq!(s.matches("</script>").count(), 1, "{s}");
    }
}
