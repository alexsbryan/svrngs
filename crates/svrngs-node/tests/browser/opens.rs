//! Opening an app is the yes to what it does as you (capability-open;
//! svrn-docs `rings/current/WHAT_CAN_ACT_AS_YOU.md` §3 shortfalls 3 and 7,
//! §7.1 and §7.3, §9 "Opening is the yes"): an app another member lends,
//! whose lender declared it reads a kind (`--reads`), opens their lends of
//! that kind as you only once you said yes, and opening it from your home
//! page, by a button naming it with what it does said beside it, is that
//! yes. Beside the open of a carried app is what it reads and writes, as the
//! shelf words it. Each claim is now held without a browser (docs/seams.tsv
//! names where); left, the by-hand read of a real Immich.

use super::*;

/// A carried app reading a program that takes a key, against a real
/// Immich run by hand (the gate's fake retired: the key's header is
/// tests/door/main.rs's, its keeping tests/home.rs's): a real one is not in the gate, so
/// this is ignored and needs `SVRNGS_TEST_IMMICH_PORT`, a loopback port its
/// `/api/users/me` answers on (an `ssh -L` tunnel to one), and
/// `SVRNGS_TEST_IMMICH_KEY`, an API key of its. The kept key is the bytes
/// the lender pasted, exactly as a real paste is.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs a real Immich: SVRNGS_TEST_IMMICH_PORT and SVRNGS_TEST_IMMICH_KEY"]
async fn a_carried_app_reads_a_real_immich() {
    let port = std::env::var("SVRNGS_TEST_IMMICH_PORT").expect("SVRNGS_TEST_IMMICH_PORT");
    let key = std::env::var("SVRNGS_TEST_IMMICH_KEY").expect("SVRNGS_TEST_IMMICH_KEY");
    let seen = app_reads_with_a_key(
        Engine::Chromium,
        port.parse().unwrap(),
        &key,
        "/api/users/me",
    )
    .await;
    assert_eq!(seen["yes"]["films"]["status"], json!(200), "{seen:#}");
    let body = seen["yes"]["films"]["body"].as_str().unwrap_or_default();
    let body: serde_json::Value = serde_json::from_str(body).expect("a JSON answer");
    eprintln!("the app read this from a real Immich's /api/users/me: {body:#}");
    assert!(body["email"].as_str().is_some(), "{body:#}");
    assert!(
        !seen.to_string().contains(&key),
        "the key never reached the page: {seen:#}"
    );
}

/// Cy lends `photos` (program `immich`) at `port` with the key kept as
/// `key`; ada carries an app whose reads.txt names immich and opens it from
/// her home page, her yes; what the app's page read at `path` through its
/// own `/svrngs-lend/`, by the driver `carried-reads.mjs`, as "films".
async fn app_reads_with_a_key(engine: Engine, port: u16, key: &str, path: &str) -> Value {
    let relay = relay().await;
    let dirs = [(); 2].map(|()| tempfile::tempdir().unwrap());
    let ada = node(dirs[0].path(), &relay, &signing(9)).await;
    let cy = joined(dirs[1].path(), &relay, &ada, 2, "cy").await;
    let lends = svrngs_node::lend::lends_dir(dirs[1].path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "groups": [NS], "port": port, "kind": "immich", "limit": 3, "refused": [] });
    std::fs::write(lends.join("photos.json"), body.to_string()).unwrap();
    let name = svrngs_node::lend::LendName::parse("photos").unwrap();
    let program = svrngs_node::lend::Program::parse("immich").unwrap();
    assert!(svrngs_node::share::keep_key(
        dirs[1].path(),
        &name,
        svrngs_node::share::key_of(&program, ""),
        key
    )
    .unwrap());
    let folder = dirs[0].path().join("house");
    std::fs::create_dir(&folder).unwrap();
    keep_folder(&ada, &folder).await;
    let page = "<!doctype html><title>Probe</title><body>Probe";
    let files = [
        ("index.html", page.into()),
        ("reads.txt", "immich\n".into()),
    ];
    carried::carry(&folder, "Probe", &files);
    let origin = carried::carried_at(&ada, "Probe").await;
    let api = ada.api_addr().port();
    let seen = drive(
        engine,
        "carried-reads.mjs",
        json!({ "url": origin, "home": format!("http://svrngs.localhost:{api}/"), "words": "Open Probe",
            "tries": { "films": label(2, "photos") }, "path": path }),
        None,
    )
    .await;
    eprintln!("the carried app's read of the keyed program: {seen:#}");
    assert_eq!(seen["error"], Value::Null, "{seen:#}");
    assert_eq!(
        seen["before"]["films"]["refused"],
        json!("unasked"),
        "{seen:#}"
    );
    assert_eq!(
        seen["stopped"]["films"]["refused"],
        json!("unasked"),
        "{seen:#}"
    );
    drop(cy);
    drop(ada);
    seen
}
