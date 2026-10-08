// SPDX-License-Identifier: AGPL-3.0-or-later
//! Who a request is from and which device it names, parsed once at the
//! edge (RUST_ARCH: parse, don't validate).
//!
//! The person is what svrngs's door set: `x-svrngs-key`, the verified
//! dialer's key as lowercase hex, and `x-svrngs-person`, their name in the
//! register (`crates/svrngs-node/src/door.rs:103-111`; it strips any
//! `x-svrngs-*` a client typed, `:468-496`). The device is the client's
//! own, from the `authorization` jellyfin-web sends, so Jellyfin's sessions
//! show each person's device and not one of the middleware's.

use hyper::http::HeaderMap;

/// The door's header naming the verified key.
pub(crate) const KEY_HEADER: &str = "x-svrngs-key";
/// The door's header naming the person.
pub(crate) const PERSON_HEADER: &str = "x-svrngs-person";

/// A key as the door spells it: 64 lowercase hex.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct PersonKey(String);

impl PersonKey {
    pub(crate) fn parse(raw: &str) -> Option<PersonKey> {
        (raw.len() == 64 && raw.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
            .then(|| PersonKey(raw.to_string()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// The person the door says is asking.
#[derive(Clone, Debug)]
pub(crate) struct Who {
    pub(crate) key: PersonKey,
    pub(crate) name: String,
}

/// Why a request is not one the door forwarded.
#[derive(Debug, PartialEq)]
pub(crate) enum NotTheDoors {
    NoKey,
    NotAKey,
    NoPerson,
}

impl std::fmt::Display for NotTheDoors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            NotTheDoors::NoKey => "the request names no person: it carries no x-svrngs-key",
            NotTheDoors::NotAKey => "its x-svrngs-key is not a key as svrngs's door spells one",
            NotTheDoors::NoPerson => "it carries no x-svrngs-person",
        })
    }
}

impl Who {
    pub(crate) fn of(headers: &HeaderMap) -> Result<Who, NotTheDoors> {
        let raw = headers.get(KEY_HEADER).ok_or(NotTheDoors::NoKey)?;
        let key = raw
            .to_str()
            .ok()
            .and_then(PersonKey::parse)
            .ok_or(NotTheDoors::NotAKey)?;
        let name = headers
            .get(PERSON_HEADER)
            .map(|v| String::from_utf8_lossy(v.as_bytes()).into_owned())
            .ok_or(NotTheDoors::NoPerson)?;
        Ok(Who { key, name })
    }
}

/// A device as Jellyfin's `MediaBrowser` authorization names one.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Device {
    pub(crate) client: String,
    pub(crate) name: String,
    pub(crate) id: String,
    pub(crate) version: String,
}

/// Where a request's device came from, for the trace.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DeviceFrom {
    /// The client's own `authorization`.
    Authorization,
    /// A `deviceId` in the query, as jellyfin-web's socket names its own.
    Query,
}

/// A value put back between quotes in a header: printable ASCII with no
/// quote, comma or backslash, as jellyfin-web's are (it URL-encodes them).
fn plain(v: &str) -> bool {
    !v.is_empty()
        && v.bytes()
            .all(|b| b.is_ascii_graphic() && !matches!(b, b'"' | b',' | b'\\') || b == b' ')
}

impl Device {
    /// The request's device: its `authorization`'s, else its query's
    /// `deviceId`, else none. None is not made up: signing a person in on a
    /// device of the middleware's own opens a session of its own in
    /// Jellyfin (`Smoke via svrngs`, demo/runs/mw/2026-10-08/mw-1/README.md),
    /// so a request that names no device goes on signed in as nobody.
    pub(crate) fn of(headers: &HeaderMap, query: Option<&str>, who: &Who) -> Option<(Device, DeviceFrom)> {
        if let Some(d) = headers
            .get(hyper::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(Device::from_authorization)
        {
            return Some((d, DeviceFrom::Authorization));
        }
        let id = query.and_then(device_in_query)?;
        Some((Device::named(who, id), DeviceFrom::Query))
    }

    /// `MediaBrowser Client="…", Device="…", DeviceId="…", Version="…"[,
    /// Token="…"]`: `None` without a `DeviceId`, or with a value that cannot
    /// be put back as it came.
    pub(crate) fn from_authorization(value: &str) -> Option<Device> {
        let (scheme, params) = value.trim().split_once(' ')?;
        if !scheme.eq_ignore_ascii_case("mediabrowser") && !scheme.eq_ignore_ascii_case("emby") {
            return None;
        }
        let mut d = Device {
            client: "svrngs".into(),
            name: "svrngs".into(),
            id: String::new(),
            version: "1".into(),
        };
        for part in params.split(',') {
            let Some((k, v)) = part.split_once('=') else {
                continue;
            };
            let v = v.trim().trim_matches('"').to_string();
            let slot = match k.trim().to_ascii_lowercase().as_str() {
                "client" => &mut d.client,
                "device" => &mut d.name,
                "deviceid" => &mut d.id,
                "version" => &mut d.version,
                _ => continue,
            };
            if !plain(&v) {
                return None;
            }
            *slot = v;
        }
        (!d.id.is_empty()).then_some(d)
    }

    /// A device the query names by id alone, its name the person's.
    fn named(who: &Who, id: String) -> Device {
        let name: String = who
            .name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
            .collect();
        Device {
            client: "svrngs".into(),
            name: format!("{} via svrngs", name.trim()),
            id,
            version: "1".into(),
        }
    }

    /// The `authorization` that names this device and nothing more.
    pub(crate) fn header(&self) -> String {
        format!(
            "MediaBrowser Client=\"{}\", Device=\"{}\", DeviceId=\"{}\", Version=\"{}\"",
            self.client, self.name, self.id, self.version
        )
    }

    /// The `authorization` that signs this device in with `token`.
    pub(crate) fn signed(&self, token: &str) -> String {
        format!("{}, Token=\"{token}\"", self.header())
    }
}

fn device_in_query(query: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k.eq_ignore_ascii_case("deviceid") && plain(v)).then(|| v.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::http::HeaderValue;

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.insert(*k, HeaderValue::from_str(v).unwrap());
        }
        h
    }

    fn key() -> String {
        "0123456789abcdef".repeat(4)
    }

    #[test]
    fn the_door_s_headers_name_the_person() {
        let who = Who::of(&headers(&[(KEY_HEADER, &key()), (PERSON_HEADER, "Ana")])).unwrap();
        assert_eq!(who.key.as_str(), key());
        assert_eq!(who.name, "Ana");
    }

    #[test]
    fn a_request_the_door_did_not_forward_is_named_as_such() {
        assert_eq!(Who::of(&headers(&[])).unwrap_err(), NotTheDoors::NoKey);
        assert_eq!(
            Who::of(&headers(&[(KEY_HEADER, "ABC"), (PERSON_HEADER, "Ana")])).unwrap_err(),
            NotTheDoors::NotAKey
        );
        assert_eq!(
            Who::of(&headers(&[(KEY_HEADER, &key().to_uppercase())])).unwrap_err(),
            NotTheDoors::NotAKey
        );
        assert_eq!(
            Who::of(&headers(&[(KEY_HEADER, &key())])).unwrap_err(),
            NotTheDoors::NoPerson
        );
    }

    #[test]
    fn jellyfin_web_s_own_device_is_kept_and_its_token_dropped() {
        let d = Device::from_authorization(
            "MediaBrowser Client=\"Jellyfin Web\", Device=\"Chrome\", DeviceId=\"TW96aWxsYQ11\", \
             Version=\"10.11.0\", Token=\"someone-else\"",
        )
        .unwrap();
        assert_eq!(d.id, "TW96aWxsYQ11");
        assert_eq!(
            d.signed("t"),
            "MediaBrowser Client=\"Jellyfin Web\", Device=\"Chrome\", DeviceId=\"TW96aWxsYQ11\", \
             Version=\"10.11.0\", Token=\"t\""
        );
    }

    #[test]
    fn a_device_is_found_in_the_query_or_else_is_none() {
        let who = Who::of(&headers(&[(KEY_HEADER, &key()), (PERSON_HEADER, "Ana \"x\"")])).unwrap();
        let (d, from) = Device::of(&headers(&[]), Some("api_key=x&deviceId=abc"), &who).unwrap();
        assert_eq!((d.id.as_str(), from), ("abc", DeviceFrom::Query));
        assert_eq!(d.name, "Ana x via svrngs");
        assert_eq!(Device::of(&headers(&[]), None, &who), None);
        // An authorization with no device, or one that cannot be put back,
        // names none.
        let auth = headers(&[("authorization", "MediaBrowser Token=\"t\"")]);
        assert_eq!(Device::of(&auth, Some("deviceId=a\"b"), &who), None);
        assert_eq!(Device::from_authorization("MediaBrowser DeviceId=\"a\\\"b\""), None);
        assert_eq!(Device::from_authorization("Bearer DeviceId=\"a\""), None);
    }
}
