// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{answered, Answered};

/// **A computer running an older svrngs is named so, never read field by
/// field** (arch-version (b); AUDIT §2.5). Its answer of what it lends
/// carries no version, as every build before arch-version's did: the
/// published one's lends say nothing of signing in, and one since says it,
/// and both are `Older`, though the second's lends would read. One
/// carrying this build's version, or a later one, is read; one carrying it
/// whose lends are not lends is refused, as before. Planted, watched
/// failing: the version check taken out of `answered`, which read the
/// second as lends and the published one's as refused.
#[test]
fn a_computer_answering_an_older_version_is_named_older() {
    let lend = |signs_in: Option<bool>| {
        let mut l = serde_json::json!({
            "name": "shelf", "kind": "k", "limit": null, "using": 0,
            "upload_bps": null, "reads": [],
        });
        if let Some(s) = signs_in {
            l["signs_in"] = s.into();
        }
        l
    };
    let said = |version: Option<u32>, signs_in: Option<bool>| {
        let mut body = serde_json::json!({ "lends": [lend(signs_in)] });
        if let Some(v) = version {
            body["version"] = v.into();
        }
        answered("cy", &serde_json::to_vec(&body).unwrap())
    };
    assert!(
        matches!(said(None, None), Answered::Older),
        "the published build's"
    );
    assert!(
        matches!(said(None, Some(true)), Answered::Older),
        "one before arch-version's"
    );
    let now = crate::version::VERSION;
    assert!(
        matches!(said(Some(now), Some(true)), Answered::Lends(l) if l.len() == 1 && l[0].signs_in)
    );
    assert!(
        matches!(said(Some(now + 1), Some(false)), Answered::Lends(l) if !l[0].signs_in),
        "a later one"
    );
    assert!(
        matches!(said(Some(now), None), Answered::Refused),
        "lends that are not lends"
    );
    assert!(matches!(answered("cy", b"not json"), Answered::Refused));
}
