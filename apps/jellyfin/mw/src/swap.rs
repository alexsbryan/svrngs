// SPDX-License-Identifier: AGPL-3.0-or-later
//! Each person's token kept out of what Jellyfin answers and put back in
//! what it is asked (u5-3 finding 4, its fourth piece; README §2). Jellyfin
//! writes the token a request was signed in with into what it answers —
//! `PlaybackInfo`'s `TranscodingUrl`, `master.m3u8`, `main.m3u8`, 43 places
//! for one converted film (u5-3 finding 3) — and the browser asks those
//! addresses back. So every textual answer has the person's token swapped
//! for [`STAND_IN`], and every request's query has its credential set to
//! the person's own, whatever the client put there: what a request is
//! signed in as is the door's word, never the client's (forward.rs drops
//! the client's credential headers likewise).

/// What a browser holds in place of a token: jellyfin-web's stored sign-in
/// names it (page.rs), and every address Jellyfin wrote a token into
/// carries it. It names nobody: the door names the person.
pub(crate) const STAND_IN: &str = "svrngs-stand-in";

/// The query names a credential travels under in what jellyfin-web asks:
/// `api_key` (its socket, its streams) and `ApiKey` (Jellyfin's own
/// addresses), compared without case as Jellyfin reads them.
const QUERY_CREDENTIALS: [&str; 2] = ["api_key", "apikey"];

/// Whether the query carries a credential: the request asks to be signed
/// in, whether or not it names its device.
pub(crate) fn asks_sign_in(query: Option<&str>) -> bool {
    query.is_some_and(|q| {
        q.split('&').any(|pair| {
            let name = pair.split_once('=').map_or(pair, |(n, _)| n);
            QUERY_CREDENTIALS.iter().any(|c| name.eq_ignore_ascii_case(c))
        })
    })
}

/// The request target `target` (path and query) with every query
/// credential set to `token`, or dropped when the request is signed in as
/// nobody. `None` when nothing changed.
pub(crate) fn inward(target: &str, token: Option<&str>) -> Option<String> {
    let (path, query) = target.split_once('?')?;
    let mut changed = false;
    let pairs: Vec<String> = query
        .split('&')
        .filter_map(|pair| {
            let name = pair.split_once('=').map_or(pair, |(n, _)| n);
            if !QUERY_CREDENTIALS.iter().any(|c| name.eq_ignore_ascii_case(c)) {
                return Some(pair.to_string());
            }
            changed = true;
            token.map(|t| format!("{name}={t}"))
        })
        .collect();
    changed.then(|| {
        if pairs.is_empty() {
            path.to_string()
        } else {
            format!("{path}?{}", pairs.join("&"))
        }
    })
}

/// Whether an answer of this `Content-Type` is read for a token: JSON,
/// a playlist, a page, any text. A film's bytes, an image's and a script
/// served as a file are not: Jellyfin writes no token into them.
pub(crate) fn textual(content_type: &str) -> bool {
    let t = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    (t.starts_with("text/") && t != "text/javascript" && t != "text/css")
        || t == "application/json"
        || t.ends_with("+json")
        || t.ends_with("mpegurl")
        || t == "application/xml"
        || t.ends_with("+xml")
}

/// `body` with every `token` swapped for [`STAND_IN`]: the new body and how
/// many it swapped, or `None` when it held none.
pub(crate) fn outward(body: &[u8], token: &str) -> Option<(Vec<u8>, usize)> {
    let token = token.as_bytes();
    if token.is_empty() || body.len() < token.len() {
        return None;
    }
    let mut out = Vec::with_capacity(body.len());
    let (mut at, mut n) = (0, 0);
    while let Some(i) = body[at..].windows(token.len()).position(|w| w == token) {
        out.extend_from_slice(&body[at..at + i]);
        out.extend_from_slice(STAND_IN.as_bytes());
        at += i + token.len();
        n += 1;
    }
    (n > 0).then(|| {
        out.extend_from_slice(&body[at..]);
        (out, n)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_query_credential_is_the_person_s_whatever_the_client_sent() {
        assert_eq!(
            inward("/socket?api_key=svrngs-stand-in&deviceId=d", Some("T")).as_deref(),
            Some("/socket?api_key=T&deviceId=d")
        );
        assert_eq!(
            inward("/Videos/1/master.m3u8?DeviceId=d&ApiKey=someone-else", Some("T")).as_deref(),
            Some("/Videos/1/master.m3u8?DeviceId=d&ApiKey=T")
        );
        // Signed in as nobody: the credential goes, the rest stays.
        assert_eq!(inward("/x?api_key=a&b=c", None).as_deref(), Some("/x?b=c"));
        assert_eq!(inward("/x?API_KEY=a", None).as_deref(), Some("/x"));
        // Nothing to set: unchanged.
        assert_eq!(inward("/x?b=c", Some("T")), None);
        assert_eq!(inward("/x", Some("T")), None);
        assert!(asks_sign_in(Some("ApiKey=svrngs-stand-in")));
        assert!(asks_sign_in(Some("a=b&api_key=x")));
        assert!(!asks_sign_in(Some("a=b")));
        assert!(!asks_sign_in(None));
    }

    #[test]
    fn the_token_is_swapped_out_of_an_answer_every_time_it_appears() {
        let body = b"#EXTM3U\nhls1/main/0.ts?api_key=0123abcd&x=1\nhls1/main/1.ts?api_key=0123abcd\n";
        let (out, n) = outward(body, "0123abcd").unwrap();
        assert_eq!(n, 2);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "#EXTM3U\nhls1/main/0.ts?api_key=svrngs-stand-in&x=1\nhls1/main/1.ts?api_key=svrngs-stand-in\n"
        );
        assert_eq!(outward(b"no token here", "0123abcd"), None);
        assert_eq!(outward(b"", "0123abcd"), None);
    }

    #[test]
    fn only_text_is_read_for_a_token() {
        for t in [
            "application/json; charset=utf-8",
            "application/x-mpegURL",
            "application/vnd.apple.mpegurl",
            "text/html",
            "text/plain",
        ] {
            assert!(textual(t), "{t}");
        }
        for t in [
            "video/mp4",
            "image/jpeg",
            "application/octet-stream",
            "text/javascript",
            "text/css",
            "",
        ] {
            assert!(!textual(t), "{t}");
        }
    }
}
