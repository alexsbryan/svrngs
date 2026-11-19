//! The page's direct path (webrtc-page; RELAY_COMMONS design 4): a lend's
//! film arrives whole over the window's channel — `webrtc*`, the relay
//! beside it as backup — and the plant, the window withholding the
//! datagrams, keeps the same film relayed. The socket the computer opens
//! for the offer is webrtc-network's, so the lender's network must be
//! enabled here, as the home page turns it on; a host whose network cannot
//! be read is could-not-judge, said so.

use super::*;

/// Direct connections on for the network this host is on, as the home
/// page's control writes them (webrtc-network `direct.json`): the offer is
/// answered only on an enabled network, so without this every run would be
/// the plant.
fn direct_on(dir: &Path) {
    let network = svrngs_node::network::here().network.unwrap_or_else(|why| {
        panic!("could not judge: this host's network cannot be read ({why:?})")
    });
    std::fs::write(
        dir.join(svrngs_node::webrtc::direct::DIRECT_FILE),
        serde_json::to_vec(&vec![network]).unwrap(),
    )
    .unwrap();
}

/// A node lending a page and a film, as `playback.rs` lends them, with
/// direct connections on: the phone holding bo's key opens it, and the
/// film's part is fetched through its worker.
async fn lent_film(dir: &Path, relay: &Server, bo: &SigningKey) -> (std::path::PathBuf, u64) {
    let _ada = node(dir, relay, bo).await;
    let mp4 = dir.join("title.mp4");
    let size = film(&mp4);
    let (film_origin, _film_heads) = origins::kept_range_origin(mp4.clone(), size).await;
    let html = "<!doctype html><title>a page</title>\
                <video src=\"title.mp4\" playsinline preload=\"auto\"></video>";
    let page = one_origin(page_origin(html).await, film_origin).await;
    lend(dir, "page", page.port());
    (mp4, size)
}

in_both_engines!(
    /// **A lend's film arrives whole on `webrtc*`, with the relay as
    /// backup** (webrtc-page; RELAY_COMMONS design 4): the phone opens
    /// ada's page and waits for the worker's trace to say the direct path
    /// came up (the window's channel carried a fresh data endpoint's wire,
    /// and the connection's selected path is `webrtc` with the relay
    /// beside it); then the film's part, fetched through the worker, must
    /// arrive whole — its SHA-256 equal to the file's — over that
    /// connection. Planted: the window withholding the datagrams (the test
    /// after this) keeps the same film on the relay, and the trace says no
    /// direct path.
    a_lends_film_rides_the_windows_channel
);
async fn a_lends_film_rides_the_windows_channel(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let dir = tempfile::tempdir().unwrap();
    let (mp4, _size) = lent_film(dir.path(), &relay, &bo).await;
    direct_on(dir.path());

    let seen = drive(
        engine,
        "runtime.mjs",
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "open": "page",
            "film": film_args(),
            "direct": true,
        }),
        None,
    )
    .await;
    let mut shown = seen.clone();
    shown["trace"] = json!(format!(
        "{} lines",
        seen["trace"].as_str().map_or(0, |t| t.lines().count())
    ));
    eprintln!("the direct path's run: {shown}");
    assert!(at_origin(&seen["opened"], &label(1, "page")), "{shown:#}");
    film_answered(&seen["first"], &mp4, "on the window's channel", &seen);
    let trace = seen["trace"].as_str().unwrap_or_default();
    assert!(
        trace.contains("goes direct"),
        "the worker must say the direct path came up:\n{shown:#}"
    );
    assert!(
        trace.contains("\"kind\":\"webrtc\",\"selected\":true"),
        "the data connection's selected path must be webrtc:\n{shown:#}"
    );
}

in_both_engines!(
    /// **The window withholding datagrams keeps it relayed** (webrtc-page;
    /// the spike's Q4 plant): the same lend with `__svrngsNoWebrtc` set,
    /// so the shim passes no datagram between the channel and the worker —
    /// the data endpoint's dial never completes. The film must still
    /// arrive whole, over the relay, and the trace must say no direct path
    /// came up.
    the_window_withholding_datagrams_keeps_it_relayed
);
async fn the_window_withholding_datagrams_keeps_it_relayed(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let dir = tempfile::tempdir().unwrap();
    let (mp4, _size) = lent_film(dir.path(), &relay, &bo).await;
    direct_on(dir.path());

    let seen = drive(
        engine,
        "runtime.mjs",
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "open": "page",
            "film": film_args(),
            "withhold": true,
        }),
        None,
    )
    .await;
    let mut shown = seen.clone();
    shown["trace"] = json!(format!(
        "{} lines",
        seen["trace"].as_str().map_or(0, |t| t.lines().count())
    ));
    eprintln!("the withheld run: {shown}");
    film_answered(&seen["first"], &mp4, "withheld, over the relay", &seen);
    assert!(
        !seen["trace"]
            .as_str()
            .unwrap_or_default()
            .contains("goes direct"),
        "no direct path may come up while the window withholds the datagrams:\n{shown:#}"
    );
}
