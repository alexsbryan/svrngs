//! The home page's "Open shared files", on the wire: the node's half of
//! browser::the_home_page_opens_the_group_s_folder. The opener is a
//! planted `open` and `xdg-open` first on this process's `PATH` (a child
//! process, X, not a seam), so no window opens and what it was asked is
//! read beside it.

use std::os::unix::fs::PermissionsExt;

use serde_json::json;

use super::computers::found;
use super::on_wire::{http, start, Browser};

/// **The home page's open of the group's folder asks this computer's file
/// browser to open that folder, and no other**: Ana keeps house's folder;
/// POST /folder from her home page, as folder.js posts it, is 204 and the
/// planted opener was asked the folder; a directory that is not the
/// group's folder is refused `closed`, and nothing is asked.
#[tokio::test]
async fn the_home_page_opens_the_group_s_folder_by_this_computer_s_opener() {
    let dir = tempfile::tempdir().unwrap();
    let ana = start(1, dir.path()).await;
    let group = found("house", 33, &ana, &[]);
    let (folder, bin) = (dir.path().join("house"), dir.path().join("bin"));
    std::fs::create_dir(&folder).unwrap();
    std::fs::create_dir(&bin).unwrap();
    for opener in ["open", "xdg-open"] {
        let at = bin.join(opener);
        std::fs::write(&at, "#!/bin/sh\necho \"$1\" >> \"$1.opened\"\n").unwrap();
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    // The planted bin first: this binary runs no other opener.
    let path = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::iter::once(bin.clone()).chain(std::env::split_paths(&path));
    std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
    let kept = http()
        .post(format!("http://{}/v1/folders", ana.api_addr()))
        .bearer_auth(ana.token())
        .json(&json!({ "dir": folder, "group": group }))
        .send()
        .await
        .unwrap();
    assert!(kept.status().is_success(), "{:?}", kept.text().await);
    let browser = &Browser::of(&ana);
    let home = format!("svrngs.localhost:{}", ana.api_addr().port());
    let own =
        format!("Sec-Fetch-Site: same-origin\r\nSec-Fetch-Mode: cors\r\nOrigin: http://{home}\r\n");
    let asked = |dir: &std::path::Path| json!({ "group": group, "dir": dir }).to_string();
    let got = super::ask(browser, "POST", &home, "/folder", &own, &asked(&folder)).await;
    assert_eq!(got.status, 204, "{}", got.body);
    let opened = std::fs::read_to_string(dir.path().join("house.opened")).unwrap_or_default();
    assert_eq!(
        opened.lines().collect::<Vec<_>>(),
        [folder.display().to_string()]
    );
    let elsewhere = dir.path().join("elsewhere");
    std::fs::create_dir(&elsewhere).unwrap();
    let got = super::ask(browser, "POST", &home, "/folder", &own, &asked(&elsewhere)).await;
    assert_eq!(got.status, 403, "{}", got.body);
    assert!(got.body.contains("closed"), "{}", got.body);
    assert!(
        !dir.path().join("elsewhere.opened").exists(),
        "nothing else asked"
    );
    ana.net().close().await;
}
