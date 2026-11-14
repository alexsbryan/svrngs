//! What the home page's bring writes, on the wire: the node's half of
//! browser::carried::the_home_page_brings_things_to_the_table, the bring
//! posted as the page posts it (pages/shelf.test.mjs holds the page's).

use serde_json::{json, Value};

use super::computers::{found, round};
use super::on_wire::{self, http, start};

/// **Brought from the home page's shelf, Money, Events and House rules land
/// in house's own folder, each with the look, House rules with canon's
/// build as vendored, and the group lists each as its own**: Ana's computer
/// in house, its folder kept, posts `/shelf` `[house, [Money, Events,
/// House-rules]]` as the home page does, by each one's shelf.txt name; `apps/<name>/look.css` in each,
/// House rules's folder `House-rules`, its `vendor/canon.js` byte for byte
/// apps/canon's, and /v1/lends lists each, kind and group.
#[tokio::test]
async fn the_shelf_brings_each_app_with_its_look_and_house_rules_with_canon() {
    let dir = tempfile::tempdir().unwrap();
    let ana = start(1, dir.path()).await;
    let group = found("house", 9, &ana, &[]);
    round(&ana).await;
    let folder = dir.path().join("house");
    std::fs::create_dir(&folder).unwrap();
    let kept = http()
        .post(format!("http://{}/v1/folders", ana.api_addr()))
        .bearer_auth(ana.token())
        .json(&json!({ "dir": folder, "group": group }))
        .send()
        .await
        .unwrap();
    assert!(kept.status().is_success(), "{:?}", kept.text().await);
    let home = format!("svrngs.localhost:{}", ana.api_addr().port());
    let body = json!([group, ["Money", "Events", "House-rules"]]).to_string();
    let answer = http()
        .post(format!("http://{}/shelf", ana.api_addr()))
        .header("host", &home)
        .headers(on_wire::proved(&ana, None, "POST", "/shelf", &body).await)
        .header("sec-fetch-site", "same-origin")
        .header("sec-fetch-mode", "cors")
        .header("origin", format!("http://{home}"))
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(answer.status(), 200, "{}", answer.text().await.unwrap());
    for app in ["Money", "Events", "House-rules"] {
        let look = folder.join("apps").join(app).join("look.css");
        assert!(look.is_file(), "{app} brings the look: {}", look.display());
    }
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert_eq!(
        std::fs::read(folder.join("apps/House-rules/vendor/canon.js")).unwrap(),
        std::fs::read(repo.join("apps/canon/vendor/canon.js")).unwrap(),
        "House rules brings canon's build as vendored"
    );
    let mut listed = Vec::new();
    for _ in 0..30 {
        let url = format!("http://{}/v1/lends?namespace={group}", ana.api_addr());
        let seen: Value = (http().get(url).bearer_auth(ana.token().trim()).send())
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        listed = (seen["lent"].as_array().into_iter().flatten())
            .filter(|l| l["group"] == group.as_str())
            .map(|l| l["kind"].as_str().unwrap_or_default().to_string())
            .collect();
        listed.sort();
        if listed.len() == 3 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    assert_eq!(
        listed,
        ["Events", "House-rules", "Money"],
        "each the group's"
    );
    ana.net().close().await;
}
