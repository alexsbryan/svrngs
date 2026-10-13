// SPDX-License-Identifier: AGPL-3.0-or-later
//! Who the door asks for, parsed once at the edge (RUST_ARCH: parse, don't
//! validate), and the device of theirs Jellyfin signs them in on.
//!
//! The person is what svrngs's door sends its sign-in: `x-svrngs-key`, the
//! verified dialer's key as lowercase hex; `x-svrngs-person`, their name
//! in the register, to show and never to key on; `x-svrngs-user`, the name
//! of their account, unique by their key and decided by the door alone
//! (crates/svrngs-node/src/door.rs, `user_name`; bash-person (b));
//! `x-svrngs-device`, the key of the device of theirs asking, which names
//! the Jellyfin device it is signed in on (bash-two-at-once (b)); and
//! `x-svrngs-stand-in`, what a browser holds in place of a token, which the
//! door swaps for the person's token in what it carries in, and back in
//! every answer: read from the ask, not spelled here, so the door is its
//! one home (DESIGN §3, call 6).

use hyper::http::HeaderMap;

/// The door's header naming the verified key.
pub(crate) const KEY_HEADER: &str = "x-svrngs-key";
/// The door's header naming the person.
pub(crate) const PERSON_HEADER: &str = "x-svrngs-person";
/// The door's header naming the person's account.
pub(crate) const USER_HEADER: &str = "x-svrngs-user";
/// The door's header naming the device asking, by its own key.
pub(crate) const DEVICE_HEADER: &str = "x-svrngs-device";
/// The door's header naming its stand-in.
pub(crate) const STAND_IN_HEADER: &str = "x-svrngs-stand-in";

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

/// The key of the device asking, spelt as a [`PersonKey`] is: one of the
/// person's keys, the first device's the person's own.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct DeviceKey(String);

impl DeviceKey {
    pub(crate) fn parse(raw: &str) -> Option<DeviceKey> {
        PersonKey::parse(raw).map(|k| DeviceKey(k.0))
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
    /// Their account's name, as the door made it.
    pub(crate) user: String,
    /// The device of theirs asking.
    pub(crate) device: DeviceKey,
}

/// Why an ask is not one the door made.
#[derive(Debug, PartialEq)]
pub(crate) enum NotTheDoors {
    NoKey,
    NotAKey,
    NoPerson,
    NoUser,
    NoDevice,
    NotADevice,
}

impl std::fmt::Display for NotTheDoors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            NotTheDoors::NoKey => "the ask names no person: it carries no x-svrngs-key",
            NotTheDoors::NotAKey => "its x-svrngs-key is not a key as svrngs's door spells one",
            NotTheDoors::NoPerson => "it carries no x-svrngs-person",
            NotTheDoors::NoUser => "it carries no x-svrngs-user",
            NotTheDoors::NoDevice => "it carries no x-svrngs-device",
            NotTheDoors::NotADevice => "its x-svrngs-device is not a key as svrngs's door spells one",
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
        let text = |h, no| {
            (headers.get(h))
                .map(|v: &hyper::http::HeaderValue| String::from_utf8_lossy(v.as_bytes()).into_owned())
                .ok_or(no)
        };
        let name = text(PERSON_HEADER, NotTheDoors::NoPerson)?;
        let user = text(USER_HEADER, NotTheDoors::NoUser)?;
        let raw = headers.get(DEVICE_HEADER).ok_or(NotTheDoors::NoDevice)?;
        let device = (raw.to_str().ok().and_then(DeviceKey::parse)).ok_or(NotTheDoors::NotADevice)?;
        Ok(Who {
            key,
            name,
            user,
            device,
        })
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
    /// The person's device asking, named from its own key: one a device,
    /// since Jellyfin holds a token's session to the device it was signed
    /// in under (demo/runs/mw/2026-10-08/mw-1/README.md:14-26), so a
    /// person's two devices named as one were one session, each one's film
    /// overwriting the other's (demo/runs/two-at-once/2026-10-12/README.md,
    /// finding 1). jellyfin-web is told the same id (page.rs), so its
    /// socket names it too.
    pub(crate) fn of(who: &Who) -> Device {
        let name: String = who
            .name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
            .collect();
        Device {
            client: "svrngs".into(),
            name: format!("{} via svrngs", name.trim()),
            id: format!("svrngs-{}", &who.device.as_str()[..16]),
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
        let who = Who::of(&headers(&[
            (KEY_HEADER, &key()),
            (PERSON_HEADER, "Ana"),
            (USER_HEADER, "Ana-0123456789abcdef"),
            (DEVICE_HEADER, &key()),
        ]))
        .unwrap();
        assert_eq!(who.key.as_str(), key());
        assert_eq!(who.device.as_str(), key());
        assert_eq!(who.name, "Ana");
        assert_eq!(who.user, "Ana-0123456789abcdef", "the door's, as it came");
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
        assert_eq!(
            Who::of(&headers(&[(KEY_HEADER, &key()), (PERSON_HEADER, "Ana")])).unwrap_err(),
            NotTheDoors::NoUser
        );
        let named = [
            (KEY_HEADER, key()),
            (PERSON_HEADER, "Ana".into()),
            (USER_HEADER, "Ana-0".into()),
        ];
        let with = |device: Option<&str>| {
            let mut h: Vec<(&'static str, String)> = named.to_vec();
            h.extend(device.map(|d| (DEVICE_HEADER, d.to_string())));
            Who::of(&headers(&h.iter().map(|(k, v)| (*k, v.as_str())).collect::<Vec<_>>())).unwrap_err()
        };
        assert_eq!(with(None), NotTheDoors::NoDevice);
        assert_eq!(with(Some("ABC")), NotTheDoors::NotADevice);
    }

    #[test]
    fn each_person_s_device_is_named_from_their_key() {
        let who = Who::of(&headers(&[
            (KEY_HEADER, &key()),
            (PERSON_HEADER, "Ana \"x\""),
            (USER_HEADER, "Anax-0123456789abcdef"),
            (DEVICE_HEADER, &key()),
        ]))
        .unwrap();
        let d = Device::of(&who);
        assert_eq!(d.id, "svrngs-0123456789abcdef");
        assert_eq!(d.name, "Ana x via svrngs");
        assert_eq!(
            d.signed("t"),
            "MediaBrowser Client=\"svrngs\", Device=\"Ana x via svrngs\", \
             DeviceId=\"svrngs-0123456789abcdef\", Version=\"1\", Token=\"t\""
        );
    }

    /// Each of a person's devices is a Jellyfin device of its own, named
    /// from the key the door says it asks with (bash-two-at-once (b)):
    /// red at the row's base, both named from the person's key, so
    /// Jellyfin listed Ana's laptop and phone as one device
    /// (demo/runs/two-at-once/2026-10-12/README.md, finding 1).
    #[test]
    fn each_of_a_person_s_devices_is_its_own() {
        let on = |device: &str| {
            Device::of(
                &Who::of(&headers(&[
                    (KEY_HEADER, &key()),
                    (PERSON_HEADER, "Ana"),
                    (USER_HEADER, "Ana-0123456789abcdef"),
                    (DEVICE_HEADER, device),
                ]))
                .unwrap(),
            )
        };
        let (laptop, phone) = (on(&"fedcba9876543210".repeat(4)), on(&key()));
        assert_eq!(laptop.id, "svrngs-fedcba9876543210", "the laptop by its own key");
        assert_eq!(phone.id, "svrngs-0123456789abcdef", "the first device by the person's");
    }
}
