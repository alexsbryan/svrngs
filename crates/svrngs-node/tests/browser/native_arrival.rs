//! Native welcome reads the real issuer without admitting a browser identity.
use super::*;

in_chromium!(native_arrival_names_once_and_shows_the_actual_folder);
async fn native_arrival_names_once_and_shows_the_actual_folder(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let (ada, group) = house(dir.path(), &relay, &signing(7)).await;
    let folder = dir.path().join("already-used-name-2");
    std::fs::create_dir(&folder).unwrap();
    keep_folder_in(&ada, &folder, &group).await;
    let token = ada
        .host()
        .link(&group, false, None, None)
        .unwrap()
        .token
        .encode();
    let journal = ada
        .rail()
        .group_dir(&group)
        .unwrap()
        .join("ring_oplog.jsonl");
    let before = std::fs::read(&journal).unwrap();
    let destination = format!("/#group-{group}");
    let open = drive::open_at(ada.api_addr().port(), "svrngs", &destination).unwrap();
    let seen = drive(
        engine,
        "native-arrival.mjs",
        json!({
            "site": site, "token": token, "group": group, "folder": folder, "open": open,
        }),
        None,
    )
    .await;
    eprintln!("native arrival: {seen:#}");
    assert_eq!(
        std::fs::read(&journal).unwrap(),
        before,
        "the browser did not redeem"
    );
    ada.endpoint().close().await;
}
