//! A computer's home page driven in a browser, written once
//! (REVIEW-audit-groups-auto-1): included by each test binary that reads
//! the home page, beside the browser harness it runs through.

use serde_json::{json, Value};

use crate::harness;

/// Drive `web/tests/home.mjs` in `engine` at `page`, as `args` say, its
/// browser opened by the svrngs that holds `token` at the home page and at
/// `page`'s host, as `svrngs open` and the home page open them
/// (give-session; web/tests/page.mjs `launch`).
pub async fn home_page(engine: harness::Engine, page: &str, token: &str, mut args: Value) -> Value {
    args["engine"] = json!(engine.as_str());
    args["ask"] = json!(page);
    let host = page
        .strip_prefix("http://")
        .and_then(|p| p.split('/').next());
    let (name, port) = host
        .and_then(|h| h.split_once(':'))
        .expect("a page on loopback");
    let name = name
        .strip_suffix(".localhost")
        .expect("a page at a .localhost host");
    let names = if name == "svrngs" {
        vec![name]
    } else {
        vec!["svrngs", name]
    };
    let opens =
        json!([{ "port": port.parse::<u16>().unwrap(), "token": token.trim(), "names": names }]);
    let out = tokio::process::Command::new("node")
        .arg(harness::repo().join("web/tests/home.mjs"))
        .arg(args.to_string())
        .env("SVRNGS_BROWSERS", harness::browsers())
        .env("SVRNGS_OPENS", opens.to_string())
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
