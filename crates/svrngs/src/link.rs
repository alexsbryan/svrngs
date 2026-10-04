// SPDX-License-Identifier: AGPL-3.0-or-later
// ported from commonwealth-ai mesh-join-vocab/src/deep_link.rs@e1a43bcd2
/// A parsed deep link.
#[derive(Debug, Clone)]
pub enum DeepLink {
    /// Use a mesh node's models as a GUEST: `sovereign://guest/<token>?url=…&exp=…&s=…`
    ///
    /// Not a join. The holder never becomes a member, never receives
    /// `mesh_secret`, and never learns an invite key — they present `token` as
    /// an `Authorization: Bearer` to `url` and get exactly what the issuing
    /// node's grant says, for as long as it says.
    Guest {
        /// The bearer. Opaque here — this crate never interprets it.
        token: String,
        /// Base URL of the issuing node's client API, no trailing `/v1`.
        ///
        /// Where the bearer goes **when `dial` is absent**. When `dial` is
        /// present this is provenance and display only — it names the machine
        /// that lent you something, and is never dialled. See [`Self::Guest`]'s
        /// `dial` field for why there is exactly one path, not a preference
        /// order.
        url: String,
        /// The lender's iroh dial string (`<hex-id>@<relay>[,<addr>…]`), when
        /// the plaintext client API is not the way in.
        ///
        /// An ENCRYPTED mesh forces its client listener loopback-only, so the
        /// address in `url` answers nothing — the sole ingress is the iroh
        /// acceptor. `mesh use` dials this, on `GUEST_ALPN`, and tunnels to
        /// the lender's bearer-checking guest listener.
        ///
        /// **Present means authoritative.** There is no fallback to `url`: a
        /// tunnel that fails is reported, never quietly downgraded to
        /// plaintext against a mesh that asked for encryption (§18.3). Old
        /// builds ignore the unknown param and fail on the unreachable `url`,
        /// which is the honest outcome for them.
        ///
        /// Additive, and the same shape a join link's `dial=` already uses.
        /// On the https fragment form the same string rides under the param
        /// name `iroh=` — the join link's dial param.
        dial: Option<String>,
        /// The issuing ring's digest marks (`<actor-hex>:<n>` pairs,
        /// comma-separated), when the link names the state a guest should
        /// expect. Opaque here, like `token`: this crate never interprets
        /// the marks, and they are never the payload — the checkpoint
        /// document is fetched separately. Present only on the https
        /// fragment form; `None` for the `sovereign://guest` form.
        at: Option<String>,
        /// Unix-seconds the grant lapses. Display + a local pre-flight so
        /// `mesh use` can refuse a dead link without a round-trip; the ISSUING
        /// NODE is the authority and rejects an expired token regardless.
        expires_at: u64,
        /// **Display only.** What the minting node said this buys, so `mesh
        /// use` can print it without a round-trip.
        ///
        /// Deliberately one opaque string rather than structured params: the
        /// grant's scope lives in the issuer's store, and a second
        /// machine-readable copy on the wire would be a second answer to "what
        /// does this link permit" (§10.6) — one that travels through a
        /// clipboard and can be edited. A link claiming more than the grant
        /// gives changes nothing; the request is still refused.
        ///
        /// This is also why a future scope needs no wire change at all.
        summary: Option<String>,
    },
}

/// Build the https form of a guest link:
/// `<base>#token=…&exp=…[&s=…][&at=…&iroh=…]`.
///
/// What a phone opens. The bearer rides the URL FRAGMENT, never the path or
/// query: a browser does not send the fragment to the server, so the token
/// stays out of access logs and `Referer` headers, and only the page's own
/// script reads it. `base` is kept verbatim — it is the page's address.
///
/// The couriers ride the fragment beside the token, on the same grounds:
/// `at` is the ring's digest marks (`<actor-hex>:<n>` pairs) and `iroh` the
/// lender's dial string (the join link's `iroh=` param). Both are appended
/// only when present — a call passing neither produces bytes identical to
/// the links this minted before the params existed. `path` is the DOOR route
/// a runtime page whose base is a bare page origin must fetch (the base
/// cannot spell both the runtime page and the app); appended last, so a call
/// that does not pass it is also byte-identical to before.
pub fn build_https_guest_link(
    token: &str,
    base: &str,
    expires_at: u64,
    summary: Option<&str>,
    at: Option<&str>,
    iroh: Option<&str>,
    path: Option<&str>,
) -> String {
    let mut params = vec![
        format!("token={}", percent_encode(token)),
        format!("exp={expires_at}"),
    ];
    if let Some(s) = summary {
        params.push(format!("s={}", percent_encode(s).replace(' ', "+")));
    }
    if let Some(a) = at {
        params.push(format!("at={}", percent_encode(a)));
    }
    if let Some(d) = iroh {
        params.push(format!("iroh={}", percent_encode(d)));
    }
    if let Some(p) = path {
        params.push(format!("path={}", percent_encode(p)));
    }
    format!("{base}#{}", params.join("&"))
}

/// Parse the https form built by [`build_https_guest_link`] into a
/// [`DeepLink::Guest`] whose `url` is the part before the fragment and whose
/// `at` and `dial` come from the fragment's `at=` / `iroh=` (or are `None`).
/// `token` and `exp` are required, as for the `sovereign://guest` form; one
/// rule covers every protected param — `token`, `at`, `iroh` — anywhere but
/// the fragment is not read, because the parser never looks left of the `#`.
pub fn parse_https_guest_link(link: &str) -> Option<DeepLink> {
    let (base, fragment) = link.split_once('#')?;
    if !(base.starts_with("https://") || base.starts_with("http://")) {
        return None;
    }
    let params = parse_query_params(Some(fragment));
    let token = params.get("token").filter(|t| !t.is_empty())?.clone();
    let expires_at = params.get("exp").and_then(|s| s.parse::<u64>().ok())?;
    Some(DeepLink::Guest {
        token,
        url: base.to_string(),
        dial: params
            .get("iroh")
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty()),
        at: params
            .get("at")
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty()),
        expires_at,
        summary: params.get("s").cloned(),
    })
}

fn parse_query_params(query: Option<&str>) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    if let Some(q) = query {
        for pair in q.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                map.insert(percent_decode(key), percent_decode(value));
            }
        }
    }
    map
}

/// Best-effort percent-decoder for query values.
///
/// Browsers and most clipboard-capable chat clients round-trip URLs
/// through `URL`/`encodeURIComponent`, which percent-escapes reserved
/// characters the builder didn't bother to encode — most importantly
/// the `:` in `relay=100.64.0.2:9742` (becomes `%3A`) and `'` in
/// mesh names (becomes `%27`). Without this decode the parser treats
/// `100.64.0.2%3A9742` as a DNS hostname, the join handshake
/// fails, and the user sees a "no peer at that address" error that
/// looks like a networking problem but is actually an encoding one.
///
/// Scope: handles `%XX` sequences and translates `+` to space (the
/// traditional HTML form-encoding convention — existing code already
/// does this for the `name` param). Invalid escapes pass through as
/// literals rather than erroring, because our input is user-pasted
/// and a partial-success link is better than a hard reject.
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let hi = (bytes[i + 1] as char).to_digit(16);
                let lo = (bytes[i + 2] as char).to_digit(16);
                if let (Some(h), Some(l)) = (hi, lo) {
                    out.push(((h << 4) | l) as u8);
                    i += 3;
                    continue;
                }
                out.push(b'%');
                i += 1;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    // Decoded bytes are only valid UTF-8 if the original encoder
    // produced valid UTF-8 — true for anything `encodeURIComponent`
    // emits. On malformed input, fall back to lossy conversion so we
    // still return something the caller can pattern-match.
    String::from_utf8(out).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

/// Percent-encode a query value — the inverse of [`percent_decode`].
/// Encodes everything outside the unreserved set (`ALPHA DIGIT - . _ ~`),
/// so the iroh dial string's `@ , : /` are escaped on the way out and
/// `parse_query_params` decodes them back verbatim.
fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for &b in input.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
// svrngs: one variant of `DeepLink` came over, so upstream's `let … else`
// over it cannot fail.
#[allow(irrefutable_let_patterns)]
mod tests {
    use super::*;

    #[test]
    fn percent_decode_passes_through_invalid_escapes() {
        // A lone `%` with no two hex digits shouldn't panic or drop
        // characters — we'd rather hand back a partial value than
        // reject the whole link. Guard for accidental truncation.
        assert_eq!(percent_decode("abc%"), "abc%");
        assert_eq!(percent_decode("abc%ZZ"), "abc%ZZ");
    }

    #[test]
    fn percent_encode_round_trips_dial_string() {
        // The dial string carries every reserved char that breaks a
        // naive query value: `@ : / , .`. Encode → decode must be
        // identity, or the joiner dials a corrupt founder address.
        let dial = "aabbccddeeff00112233@https://relay.example./,10.0.0.5:9742,[fd7a::1]:9742";
        assert_eq!(percent_decode(&percent_encode(dial)), dial);
    }

    #[test]
    fn https_guest_link_round_trips_with_the_token_only_in_the_fragment() {
        let base = "http://192.168.1.10:9750/app/ring-doc/";
        let link = build_https_guest_link(
            "deadbeef",
            base,
            1_787_900_000,
            Some("rail:wall"),
            None,
            None,
            None,
        );
        let (before, fragment) = link.split_once('#').expect("a fragment");
        assert!(
            !before.contains("deadbeef"),
            "the token must never reach the path or query: {link}"
        );
        assert!(fragment.contains("token=deadbeef"), "{link}");
        let DeepLink::Guest {
            token,
            url,
            dial,
            at,
            expires_at,
            summary,
        } = parse_https_guest_link(&link).unwrap()
        else {
            panic!("expected a guest link")
        };
        assert_eq!(token, "deadbeef");
        assert_eq!(url, base);
        assert!(dial.is_none());
        assert!(at.is_none());
        assert_eq!(expires_at, 1_787_900_000);
        assert_eq!(summary.as_deref(), Some("rail:wall"));
        // A token outside the fragment is not a guest link.
        assert!(parse_https_guest_link("http://h/?token=deadbeef&exp=1").is_none());
    }

    /// The couriers ride the fragment beside the token and come back
    /// verbatim: the marks (`at=`) and the dial string (`iroh=`) survive the
    /// percent-encoder, and the built link is exactly these bytes.
    #[test]
    fn https_guest_link_round_trips_the_couriers_exactly() {
        let base = "http://192.168.1.10:9750/app/ring-doc/";
        let at = "aa11:1,bb22:2,cc33:3";
        let iroh = "3b1f0a@https://relay.example:443/,192.168.1.10:41234";
        let link = build_https_guest_link(
            "deadbeef",
            base,
            1_787_900_000,
            None,
            Some(at),
            Some(iroh),
            None,
        );
        assert_eq!(
            link,
            "http://192.168.1.10:9750/app/ring-doc/#token=deadbeef&exp=1787900000\
             &at=aa11%3A1%2Cbb22%3A2%2Ccc33%3A3\
             &iroh=3b1f0a%40https%3A%2F%2Frelay.example%3A443%2F%2C192.168.1.10%3A41234"
        );
        let DeepLink::Guest {
            at: marks, dial, ..
        } = parse_https_guest_link(&link).unwrap()
        else {
            panic!("expected a guest link")
        };
        assert_eq!(marks.as_deref(), Some(at));
        assert_eq!(dial.as_deref(), Some(iroh));
    }

    /// A call with neither courier produces bytes identical to the link this
    /// builder minted before the params existed — every existing QR and
    /// pasted link stays valid.
    #[test]
    fn https_guest_link_without_the_new_params_is_byte_identical() {
        assert_eq!(
            build_https_guest_link(
                "deadbeef",
                "http://192.168.1.10:9750/app/ring-doc/",
                1_787_900_000,
                Some("rail:wall"),
                None,
                None,
                None,
            ),
            "http://192.168.1.10:9750/app/ring-doc/#token=deadbeef&exp=1787900000&s=rail%3Awall"
        );
    }

    /// `path=` is the door route a bare runtime page must fetch, and it is
    /// appended last and percent-encoded like every other courier. A caller
    /// that does not pass one mints the same bytes as before (asserted
    /// above), which is what keeps this addition from invalidating old QRs.
    #[test]
    fn https_guest_link_carries_the_door_route_in_path() {
        let link = build_https_guest_link(
            "deadbeef",
            "https://svrnme.sh/ring/",
            1_787_900_000,
            None,
            None,
            Some("3b1f0a@https://relay.example./,10.0.0.5:9742"),
            Some("/ring/ring-doc/"),
        );
        assert_eq!(
            link,
            "https://svrnme.sh/ring/#token=deadbeef&exp=1787900000\
             &iroh=3b1f0a%40https%3A%2F%2Frelay.example.%2F%2C10.0.0.5%3A9742\
             &path=%2Fring%2Fring-doc%2F"
        );
    }

    /// The marks are read from the fragment only: an `at=` in the query —
    /// the part a browser sends to the server — is not the marks.
    #[test]
    fn an_at_outside_the_fragment_is_not_read() {
        let link = "http://h:9750/app/ring-doc/?at=aa11%3A7#token=deadbeef&exp=1";
        let DeepLink::Guest { at, .. } = parse_https_guest_link(link).unwrap() else {
            panic!("expected a guest link")
        };
        assert!(at.is_none());
    }

    /// Same rule for the dial string: an `iroh=` in the query is ignored.
    #[test]
    fn an_iroh_outside_the_fragment_is_not_read() {
        let link = "http://h:9750/app/ring-doc/?iroh=zz11%40relay%3A1#token=deadbeef&exp=1";
        let DeepLink::Guest { dial, .. } = parse_https_guest_link(link).unwrap() else {
            panic!("expected a guest link")
        };
        assert!(dial.is_none());
    }
}
