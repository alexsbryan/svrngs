// SPDX-License-Identifier: AGPL-3.0-or-later
//! Who the door asks for, parsed once at the edge (RUST_ARCH: parse, don't
//! validate), and the device of theirs Jellyfin signs them in on.
//!
//! The person is what svrngs's door sends its sign-in: `x-svrngs-key`, the
//! verified dialer's key as lowercase hex, and `x-svrngs-person`, their
//! name in the register (crates/svrngs-node/src/door/sign_in.rs, `ask_of`).

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

/// The person the door says it asks for.
#[derive(Clone, Debug)]
pub(crate) struct Who {
    pub(crate) key: PersonKey,
    pub(crate) name: String,
}

/// Why an ask is not one the door made.
#[derive(Debug, PartialEq)]
pub(crate) enum NotTheDoors {
    NoKey,
    NotAKey,
    NoPerson,
}

impl std::fmt::Display for NotTheDoors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            NotTheDoors::NoKey => "the ask names no person: it carries no x-svrngs-key",
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

impl Device {
    /// The person's own device, named from their key: one a person, since
    /// the door sets one `authorization` on each of their requests, and
    /// Jellyfin holds a token's session to the device it was signed in
    /// under (demo/runs/mw/2026-10-08/mw-1/README.md:14-26). jellyfin-web
    /// is told the same id (page.rs), so its socket names it too.
    pub(crate) fn of(who: &Who) -> Device {
        let name: String = who
            .name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
            .collect();
        Device {
            client: "svrngs".into(),
            name: format!("{} via svrngs", name.trim()),
            id: format!("svrngs-{}", &who.key.as_str()[..16]),
            version: "1".into(),
        }
    }

    /// The one device the `one-device` plant signs everyone in on, as a lend
    /// keeping one token was (`apps/jellyfin/check/check.sh plant one-device`).
    pub(crate) fn one() -> Device {
        Device {
            client: "svrngs".into(),
            name: "one device".into(),
            id: "svrngs-mw-one-device".into(),
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
    fn an_ask_the_door_did_not_make_is_named_as_such() {
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
    fn each_person_s_device_is_named_from_their_key() {
        let who = Who::of(&headers(&[(KEY_HEADER, &key()), (PERSON_HEADER, "Ana \"x\"")])).unwrap();
        let d = Device::of(&who);
        assert_eq!(d.id, "svrngs-0123456789abcdef");
        assert_eq!(d.name, "Ana x via svrngs");
        assert_eq!(
            d.signed("t"),
            "MediaBrowser Client=\"svrngs\", Device=\"Ana x via svrngs\", \
             DeviceId=\"svrngs-0123456789abcdef\", Version=\"1\", Token=\"t\""
        );
    }
}
