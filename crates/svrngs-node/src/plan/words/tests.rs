// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

/// A Run kept says so and when it comes back, the keeper's words (the
/// command ends the line itself); one refused says the keeper's why and nothing else. Planted,
/// watched failing: `back` left out of the line.
#[test]
fn a_run_says_it_runs_and_when_it_comes_back_or_why_not() {
    let verified =
        r#"{"kind":"verified","unit":"coder.service","port":8101,"back":"when you log in"}"#;
    let page = started(
        "coder",
        RunView::Page,
        serde_json::from_str(verified).unwrap(),
    );
    assert_eq!(
        page.unwrap(),
        "svrngs runs coder now. It comes back when you log in."
    );
    let refused = r#"{"kind":"refused","why":"Port 8101 is in use by python3."}"#;
    let said = started(
        "coder",
        RunView::Page,
        serde_json::from_str(refused).unwrap(),
    );
    assert_eq!(said.unwrap_err(), "Port 8101 is in use by python3.");
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
