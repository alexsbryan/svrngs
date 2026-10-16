//! A computer's home page driven in a browser, written once
//! (REVIEW-audit-groups-auto-1): included by each test binary that reads
//! the home page, beside the browser harness it runs through.

use serde_json::{json, Value};

use crate::harness;

/// Drive `web/tests/home.mjs` in `engine` at `page`, as `args` say.
pub async fn home_page(engine: harness::Engine, page: &str, mut args: Value) -> Value {
    args["engine"] = json!(engine.as_str());
    args["ask"] = json!(page);
    let out = tokio::process::Command::new("node")
        .arg(harness::repo().join("web/tests/home.mjs"))
        .arg(args.to_string())
        .output()
        .await
        .expect("node runs: the browser tests need Node 18 or later");
    assert!(
        out.status.success(),
        "the browser half failed ({}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    serde_json::from_str(stdout.lines().last().expect("printed")).expect("JSON")
}
