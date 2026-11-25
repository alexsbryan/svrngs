// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn terminal_run_points_to_share_by_port() {
    let verdict =
        serde_json::from_str(r#"{"kind":"verified","unit":"coder.service","port":8101}"#).unwrap();
    let said = started("coder", RunView::Terminal, verdict).unwrap();
    assert_eq!(
        said,
        "coder is running here. To share it: `svrngs share 8101 --program <program> --name coder --group <group>`.\n"
    );
}

#[test]
fn page_run_says_svrngs_runs_it_and_no_more() {
    let verdict =
        serde_json::from_str(r#"{"kind":"verified","unit":"coder.service","port":8101}"#).unwrap();
    let said = started("coder", RunView::Page, verdict).unwrap();
    assert_eq!(said, "svrngs runs coder now.");
}
