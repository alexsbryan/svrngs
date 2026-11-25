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

/// A program svrngs runs is one row of the page's Already running here,
/// its button its action and its name: Stop while it runs or starts,
/// Start once stopped or failed. Planted, watched failing: the button said
/// by `Action::verb`, "stop coder".
#[test]
fn a_run_row_s_button_names_its_action_and_the_program() {
    let row = |running: &str| {
        let kept = format!(r#"{{"name":"coder","port":8101,"status":{{"running":{running}}}}}"#);
        owned(serde_json::from_str(&kept).unwrap())
    };
    let said = |v: serde_json::Value| (v["text"].clone(), v["action"].clone(), v["button"].clone());
    use serde_json::json;
    assert_eq!(
        said(row(r#"{"since":"9:00"}"#)),
        (
            json!("Run by svrngs, since 9:00."),
            json!("stop"),
            json!("Stop coder")
        )
    );
    assert_eq!(
        said(row(r#""starting""#)),
        (json!("Starting…"), json!("stop"), json!("Stop coder"))
    );
    assert_eq!(
        said(row(r#""stopped""#)),
        (json!("Stopped."), json!("start"), json!("Start coder"))
    );
}
