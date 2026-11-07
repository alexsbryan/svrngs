// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{answered, Answered};

/// **A computer running an older svrngs is named so, never read field by
/// field** (arch-version (b); AUDIT §2.5). Its answer of what it lends
/// carries no version, as every build before arch-version's did, so it is
/// `Older`, though its lends would read. One carrying this build's version,
/// or a later one, is read; one at 1 to 3, before the holders' flag day
/// (version 4, HOLDER_APPROVAL), is older too; one carrying it whose lends are
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
        matches!(said(Some(5), lend()), Answered::Lends(l, _) if l.len() == 1),
        "6 reads 5: a raise that is `Behind::Reads` cuts nobody off (version.rs STEPS)"
    );
    for before in 1..5 {
        assert!(
            matches!(said(Some(before), lend()), Answered::Older),
            "{before}: one before the holders' flag day (version 4) takes an app's entry unapproved"
        );
    }
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

/// The relayed direction of the scope fence (LENDING_SCOPE §"Updates,
/// migration and mixed versions"): a computer still on 4 answers its lends
/// unscoped, every one to every group; this computer names it older and
/// lists none of them on, so `/people/lends` never relays an old broad
/// grant. A 5 answer whose lend names no `groups` is read but listed to
/// nobody. Planted: VERSION left at 4, which read the 4 answer as lends.
#[test]
fn an_unscoped_answer_from_a_computer_on_4_is_relayed_to_nobody() {
    let shelf = serde_json::json!({
        "name": "shelf", "kind": "k", "limit": null, "using": 0, "upload_bps": null, "reads": [],
    });
    let said = |v: u32| {
        answered(
            "cy",
            &serde_json::to_vec(&serde_json::json!({ "version": v, "lends": [shelf] })).unwrap(),
        )
    };
    assert!(
        matches!(said(4), Answered::Older),
        "a 4 answer is older, not read"
    );
    let Answered::Lends(mut lends, 5) = said(5) else {
        panic!("a 5 answer is read")
    };
    assert!(
        !lends[0].listed(|_| true),
        "a lend with no groups is listed to nobody"
    );
}
