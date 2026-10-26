//! A navigation into a lend, judged as a fetch is (capability-navigation;
//! svrn-docs `rings/current/WHAT_CAN_ACT_AS_YOU.md` §3, shortfall 6): a
//! lend's page that frames another lend, or posts a form into it, asks as
//! that page, held to what its lender declared it reads and to the yes a
//! lend that asks needs, as its fetch is. A link the person follows stays
//! the person going there.

use super::*;

/// `node` lends `name` on `port`, as [`lend`] does, its declaration's
/// other fields `with`.
fn declare(dir: &Path, name: &str, port: u16, with: Value) {
    let lends = svrngs_node::lend::lends_dir(dir);
    std::fs::create_dir_all(&lends).unwrap();
    let mut body = json!({ "port": port, "limit": 3, "refused": [], "reads": [] });
    body.as_object_mut()
        .unwrap()
        .extend(with.as_object().unwrap().clone());
    std::fs::write(lends.join(format!("{name}.json")), body.to_string()).unwrap();
}

/// The first line of each request `heads` kept, its method and path.
fn asked(heads: &KeptHeads) -> Vec<String> {
    let heads = heads.lock().unwrap();
    let firsts = heads.iter().filter_map(|h| h.lines().next());
    firsts
        .map(|l| l.rsplit_once(' ').map_or(l, |(at, _)| at).to_string())
        .collect()
}

in_both_engines!(
    /// **A navigation from another lend's page is judged as its fetch**
    /// (shortfall 6, `origin.rs`'s `app_of`). Ada's computer lends a page
    /// reading kind `k`, a shelf of kind `j`, a box of kind `k` that asks
    /// its person first, to which ada said no yes, and notes of kind `k`.
    /// Opened by its typed address and moved within itself by a link, the
    /// page reaches each of the other three by a fetch, a frame and a form
    /// posted as the whole tab: the fetch is refused, the shelf because the
    /// page does not read it and the box because ada said no yes, and so are
    /// the frame and the form, which before this test reached each as the
    /// person going there; the notes, which the page reads, are reached all
    /// three ways, as the page. Kept: the typed address, the page moving
    /// within itself, and a link from the page into each, which the person
    /// follows as themselves. Planted: `app_of` taking every navigation for
    /// the lend's own page again, and the frame and the form reach the
    /// shelf and the box.
    a_navigation_from_another_lend_s_page_is_judged_as_its_fetch
);
async fn a_navigation_from_another_lend_s_page_is_judged_as_its_fetch(engine: Engine) {
    let relay = relay().await;
    let dir = tempfile::tempdir().unwrap();
    let ada = node(dir.path(), &relay, &signing(7)).await;
    let (page, on_page) = kept_page("<!doctype html><title>ada's page</title><body>").await;
    let (shelf, on_shelf) = kept_page("<!doctype html><title>the shelf</title><body>").await;
    let (boxed, on_box) = kept_page("<!doctype html><title>the box</title><body>").await;
    let (notes, on_notes) = kept_page("<!doctype html><title>the notes</title><body>").await;
    declare(
        dir.path(),
        "page",
        page.port(),
        json!({ "kind": "p", "reads": ["k"] }),
    );
    declare(dir.path(), "shelf", shelf.port(), json!({ "kind": "j" }));
    let asks = "the box shows what is in it to the page";
    declare(
        dir.path(),
        "box",
        boxed.port(),
        json!({ "kind": "k", "asks": asks }),
    );
    declare(dir.path(), "notes", notes.port(), json!({ "kind": "k" }));
    let port = ada.api_addr().port();
    let at = |name: &str| format!("http://{}.localhost:{port}/", label(1, name));

    let seen = drive(
        engine,
        "navigate.mjs",
        json!({ "url": at("page"), "into": [at("shelf"), at("box"), at("notes")] }),
        None,
    )
    .await;
    eprintln!("the navigations' run: {seen:#}");

    assert_eq!(
        seen["within"],
        format!("{}own/next", at("page")),
        "{seen:#}"
    );
    let on_page = asked(&on_page);
    assert!(
        on_page.iter().any(|l| l == "GET /own/next"),
        "the page moves within itself: {on_page:?}"
    );
    let lends = [("shelf", &on_shelf), ("box", &on_box), ("notes", &on_notes)];
    for (i, (name, heads)) in lends.into_iter().enumerate() {
        let into = &seen["into"][i];
        let heads = asked(heads);
        assert_eq!(
            into["linked"],
            format!("{}linked", at(name)),
            "a link from the page opens the {name}:\n{seen:#}"
        );
        assert!(
            heads.iter().any(|l| l == "GET /linked"),
            "the {name} is asked the link's path: {heads:?}"
        );
        assert!(
            !heads.iter().any(|l| l == "GET /bare"),
            "the {name} is never reached at its plain origin by a fetch: {heads:?}\n{seen:#}"
        );
        let read = name == "notes";
        for reached in ["GET /fetched", "GET /framed", "POST /posted"] {
            assert_eq!(
                heads.iter().any(|l| l == reached),
                read,
                "the page reached the {name} as `{reached}` only if it reads it: {heads:?}\n{seen:#}"
            );
        }
    }
    drop(ada);
}
