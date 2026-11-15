//! A new build reaches the group's page at an invitation's address
//! (web/tests/updated.mjs): the 2026-11-14 incident, where a browser that
//! had opened the page before ran the old build at a new build's
//! invitation until a hard reload.

use super::*;

in_both_engines!(
    /// **A new build runs at an invitation's address with no reload by
    /// hand.** The page opened at `#an-invitation` under build A's worker,
    /// build B deployed at the same origin, and the invitation opened again
    /// in a tab of its own: once B's worker holds the origin alone, both
    /// tabs run B, the one opened again and the one left open. Red on
    /// 8a5e1689, A's worker answering the page from its cache and moving
    /// the open pages with `navigate(page.url)`, which at an address with a
    /// fragment loads nothing (web/sw.js `answered`, `activated`): in WebKit
    /// both tabs ran A; in Chromium neither answered a read in 2 s, each
    /// held in the navigation Playwright never sees finish (reopen.mjs).
    a_new_build_runs_at_an_invitation_s_address_with_no_reload_by_hand
);
async fn a_new_build_runs_at_an_invitation_s_address_with_no_reload_by_hand(engine: Engine) {
    let seen = drive(engine, "updated.mjs", json!({ "site": site() }), None).await;
    let shown = json!({ "a": seen["a"], "b": seen["b"], "controlled": seen["controlled"],
        "alone": seen["alone"], "runs": seen["runs"], "urls": seen["urls"] });
    eprintln!("{shown:#}\n{:#}", seen["logs"]);
    assert_eq!(
        seen["controlled"], true,
        "A's worker took the page: {shown:#}"
    );
    assert_eq!(
        seen["alone"], true,
        "B's worker holds the origin alone: {shown:#}"
    );
    assert_eq!(
        seen["runs"],
        json!({ "opened": seen["b"], "left": seen["b"] }),
        "both tabs run build B: {shown:#}"
    );
}
