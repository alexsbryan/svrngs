//! Pure calibration moved from the browser target. No host measurement, media,
//! browser or relay is required to judge these explicit observations.
#[path = "browser/load.rs"]
#[allow(dead_code)]
mod load;
use load::seeks_judged;
use serde_json::{json, Value};

const QUIET: &str = "2026-10-12T05:00:00Z 22:00  up 3 days, 2 users, load averages: 35.9 20.1 12.0";
const BUSY_MAC: &str =
    "2026-10-12T05:00:00Z 22:00  up 3 days, 2 users, load averages: 62.1 40.3 30.1";
const BUSY_LINUX: &str =
    "2026-10-12T05:00:00Z 05:00:00 up 3 days, 2 users, load average: 62.10, 40.30, 30.10";
const UNREAD: &str = "2026-10-12T05:00:00Z";

fn seeks(ms: [Value; 3]) -> Vec<Value> {
    ms.into_iter().map(|ms| json!({ "ms": ms })).collect()
}

#[test]
fn a_seek_missed_at_low_load_fails() {
    let missed = seeks([json!(1853), json!(2701), json!(11424)]);
    let never = seeks([json!(null), json!(null), json!(531)]);
    for load in [QUIET, UNREAD] {
        assert_eq!(
            seeks_judged(&missed, load, 12),
            Err(r#"{"ms":11424}"#.to_string())
        );
        assert_eq!(
            seeks_judged(&never, load, 12),
            Err(r#"{"ms":null}, {"ms":null}"#.to_string())
        );
    }
}

#[test]
fn a_seek_missed_past_the_bar_could_not_judge() {
    let missed = seeks([json!(1853), json!(2701), json!(11424)]);
    for load in [BUSY_MAC, BUSY_LINUX] {
        assert_eq!(
            seeks_judged(&missed, load, 12),
            Ok(Some(
                "could-not-judge, load 62.1 on 12 cores, past 3 a core".to_string()
            ))
        );
    }
}

#[test]
fn seeks_held_passed_at_any_load() {
    let held = seeks([json!(1853), json!(2701), json!(5000)]);
    for load in [QUIET, BUSY_MAC, UNREAD] {
        assert_eq!(seeks_judged(&held, load, 12), Ok(None));
    }
}
