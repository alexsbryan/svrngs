//! The fold's cases, the keeper program's build: every case in cases.json,
//! asked through the JSON boundary the page's build answers too
//! (`give_fold::answer`; ../fold.test.mjs runs the same file against the
//! JavaScript), so the two builds are held to one set of answers.

use serde_json::{Value, json};

const CASES: &str = include_str!("../cases.json");

/// How many cases cases.json holds; ../fold.test.mjs counts the same, so a
/// case either build skipped would show.
const COUNT: usize = 17;

#[test]
fn every_case_folds_as_written() {
    let cases: Vec<Value> = serde_json::from_str(CASES).expect("cases.json is JSON");
    assert_eq!(cases.len(), COUNT, "cases.json holds {COUNT} cases");
    let mut failed = Vec::new();
    for case in &cases {
        let asked = json!({ "ops": case["ops"] }).to_string();
        let answered: Value = serde_json::from_str(&give_fold::answer(&asked)).expect("an answer is JSON");
        if answered != json!({ "ok": case["kept"] }) {
            failed.push(format!(
                "{}\n  wanted {}\n  folded {answered}",
                case["name"], case["kept"]
            ));
        }
    }
    assert!(
        failed.is_empty(),
        "{} of {COUNT} cases folded otherwise:\n{}",
        failed.len(),
        failed.join("\n")
    );
}

#[test]
fn a_request_that_is_not_ops_is_said_so() {
    let answered: Value = serde_json::from_str(&give_fold::answer("{\"acts\": []}")).unwrap();
    assert_eq!(
        answered,
        json!({ "error": "a request is {\"ops\": [...]}, the acts in the record's order" })
    );
}
