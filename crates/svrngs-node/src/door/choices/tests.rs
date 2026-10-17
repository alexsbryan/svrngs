// SPDX-License-Identifier: AGPL-3.0-or-later
//! The door's one check, row by row of the design's §4 table;
//! `tests/door/choices.rs` runs it through the door between two nodes.

use super::*;

/// Each row: whether the lend asks, the app named, the choice kept, and
/// what the door does.
#[test]
fn each_row_of_the_table() {
    let own = Label::of(&[1; 32], "taste");
    let app = Label::of(&[2; 32], "night");
    let unasked = Err(LendRefusal::Unasked.as_str());
    for (row, asks, named, said, want) in [
        ("asks nobody, no app", false, None, None, Ok(())),
        (
            "asks nobody, an app stopped",
            false,
            Some(&app),
            Some(Said::Stop),
            Ok(()),
        ),
        ("its own pages", true, Some(&own), None, Ok(())),
        (
            "its own pages, stopped",
            true,
            Some(&own),
            Some(Said::Stop),
            Ok(()),
        ),
        ("no app named", true, None, Some(Said::Yes), unasked),
        ("an app, yes", true, Some(&app), Some(Said::Yes), Ok(())),
        (
            "an app, stopped",
            true,
            Some(&app),
            Some(Said::Stop),
            Err("stopped"),
        ),
        ("an app, nothing said", true, Some(&app), None, unasked),
    ] {
        let got = judge(asks, &own, named, said).map_err(LendRefusal::as_str);
        assert_eq!(got, want, "{row}");
    }
}
