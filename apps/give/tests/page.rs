//! Give's page brought to a group (page.rs): its files written into the
//! group's folder at `apps/give/`, which svrngs carries to every member as
//! an app of the group's, and the shelf beside it as `cards.json`. No
//! podman and no svrngs here: these write files.

use std::path::Path;

use give::shelf::Shelf;
use serde_json::Value;

#[test]
fn the_page_and_its_shelf_go_into_the_groups_folder_once() {
    let folder = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("give-page-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    let standin = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/standin");
    let shelf = Shelf::with(&[standin]).unwrap();

    let wrote = give::page::bring(&folder, &shelf).unwrap();
    let names: Vec<_> = wrote
        .iter()
        .map(|p| p.strip_prefix(&folder).unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(
        names,
        [
            "apps/give/index.html",
            "apps/give/index.js",
            "apps/give/look.css",
            "apps/give/vendor/call.mjs",
            "apps/give/vendor/fold.js",
            "apps/give/cards.json"
        ]
    );
    let cards: Value =
        serde_json::from_str(&std::fs::read_to_string(folder.join("apps/give/cards.json")).unwrap()).unwrap();
    let kinds: Vec<_> = cards
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        ["factorio", "minecraft", "standin"],
        "give's own cards and the one named"
    );
    let minecraft = &cards[1];
    assert_eq!(minecraft["door"], "tcp");
    assert_eq!(minecraft["asks"][0]["link"], "https://aka.ms/MinecraftEULA");
    assert_eq!(cards[2]["door"], "http");
    assert_eq!(cards[2]["version"], shelf.get("standin").unwrap().version);

    assert_eq!(
        give::page::bring(&folder, &shelf).unwrap(),
        Vec::<std::path::PathBuf>::new(),
        "nothing written twice"
    );
    let _ = std::fs::remove_dir_all(&folder);
}
