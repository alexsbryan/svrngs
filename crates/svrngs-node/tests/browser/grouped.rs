//! The home layout keeps every control of frozen layout A reachable across its places.
use super::*;

in_chromium!(grouped_home_keeps_capabilities_and_group_identity);
async fn grouped_home_keeps_capabilities_and_group_identity(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, group) = house(dir.path(), &relay, &signing(7)).await;
    let open = drive::open_at(ada.api_addr().port(), "svrngs", "/").unwrap();
    let seen = drive(
        engine,
        "grouped.mjs",
        json!({ "open": open, "group": group, "artifacts": repo().join("target") }),
        None,
    )
    .await;
    eprintln!("grouped home: {seen:#}");
    ada.endpoint().close().await;
}
