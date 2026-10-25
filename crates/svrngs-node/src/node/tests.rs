// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{answered, Answered};

/// **A computer running an older svrngs is named so, never read field by
/// field** (arch-version (b); AUDIT §2.5). Its answer of what it lends
/// carries no version, as every build before arch-version's did, so it is
/// `Older`, though its lends would read. One carrying this build's version,
/// or a later one, is read, and one at 1, before a holder's link, is read
/// with its version (capability-signed); one carrying it whose lends are
/// not lends is refused, as before. Planted, watched failing: the version check taken
/// out of `answered`, which read the versionless answer as lends.
#[test]
fn a_computer_answering_an_older_version_is_named_older() {
    let lend = || {
        serde_json::json!({
            "name": "shelf", "kind": "k", "limit": null, "using": 0,
            "upload_bps": null, "reads": [],
        })
    };
    let said = |version: Option<u32>, lends: serde_json::Value| {
        let mut body = serde_json::json!({ "lends": [lends] });
        if let Some(v) = version {
            body["version"] = v.into();
        }
        answered("cy", &serde_json::to_vec(&body).unwrap())
    };
    assert!(
        matches!(said(None, lend()), Answered::Older),
        "the published build's"
    );
    let now = crate::version::VERSION;
    assert!(matches!(said(Some(now), lend()), Answered::Lends(l, _) if l.len() == 1));
    assert!(
        matches!(said(Some(now + 1), lend()), Answered::Lends(l, _) if l.len() == 1),
        "a later one"
    );
    assert!(
        matches!(said(Some(1), lend()), Answered::Lends(l, 1) if l.len() == 1),
        "one before a holder's link (capability-signed), read with its version"
    );
    assert!(!crate::version::links(Some(1)) && crate::version::links(Some(now)));
    assert!(
        matches!(
            said(Some(now), serde_json::json!({ "name": "shelf" })),
            Answered::Refused
        ),
        "lends that are not lends"
    );
    assert!(matches!(answered("cy", b"not json"), Answered::Refused));
}
