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
/// film's part is fetched through its worker. The node is returned with
/// them: the drive runs after, and a node dropped here would be off
/// before the phone dials it (the first run of this test failed that way,
/// "the computer that sent your link did not answer").
async fn lent_film(
    dir: &Path,
    relay: &Server,
    bo: &SigningKey,
    me: u8,
) -> (Node, std::path::PathBuf, u64) {
    let ada = computer(dir, relay, bo, me, "ada").await;
    let mp4 = dir.join("title.mp4");
    let size = film(&mp4);
    let (film_origin, _film_heads) = origins::kept_range_origin(mp4.clone(), size).await;
    // `preload="none"`: WebKit's `<video>` otherwise asks the whole film in
    // one request, and the worker streams all of it through the same event
    // loop while the direct path is being set up — WebKit's runs of this
    // test then stalled the data connection ("page goes direct" but the
    // fetch's body never arrived, the peer idling out) until the film's own
    // fetch, below, was judged; the film's fetch is this test's subject,
    // and playing it is the playback step's (playback.rs).
    let html = "<!doctype html><title>a page</title>\
                <video src=\"title.mp4\" playsinline preload=\"none\"></video>";
    let page = one_origin(page_origin(html).await, film_origin).await;
    lend(dir, "page", page.port());
    (ada, mp4, size)
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
    let (ada, mp4, _size) = lent_film(dir.path(), &relay, &bo, 1).await;
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
    eprintln!(
        "the direct path's trace:\n{}",
        seen["trace"].as_str().unwrap_or_default()
    );
    assert!(at_origin(&seen["opened"], &label(1, "page")), "{shown:#}");
    film_answered(&seen["first"], &mp4, "on the window's channel", &seen);
    let trace = seen["trace"].as_str().unwrap_or_default();
    assert!(
        trace.contains("goes direct"),
        "the worker must say the direct path came up:\n{shown:#}"
    );
    assert!(
        webrtc_selected(&goes_direct_paths(trace)),
        "the data connection's selected path must be webrtc:\n{shown:#}"
    );
    drop(ada);
}

in_both_engines!(
    /// **The channel closed mid-film completes over the relay**
    /// (webrtc-page; RELAY_COMMONS design 4): the film's part rides
    /// `webrtc*` (the worker waits for that selection, `Direct::go`), and
    /// the window closes the channel once `__svrngsCloseWebrtc` datagrams
    /// have come through it — a count of the wire's own, so the close
    /// lands mid-film however far the page's reading runs behind; the
    /// test witnesses the channel actually closing, the lender giving up
    /// its custom path, and the exact range arriving with the relay left.
    the_channel_closed_mid_film_completes_over_the_relay
);
async fn the_channel_closed_mid_film_completes_over_the_relay(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let dir = tempfile::tempdir().unwrap();
    // Distinct lenders let us select each engine's iroh events even when
    // every browser case is running against the common trace log.
    let me = match engine {
        Engine::Chromium => 101,
        Engine::Webkit => 102,
    };
    let (ada, mp4, _size) = lent_film(dir.path(), &relay, &bo, me).await;
    direct_on(dir.path());

    let seen = drive(
        engine,
        "runtime.mjs",
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(me), &relay),
            "open": "page",
            // 6 MiB, a fifth of the film, with the window closing the
            // channel once 2000 datagrams have come through it: the
            // wire's own count, so the close lands mid-film (about
            // 2.5 MiB in) however far the page's reading runs behind,
            // leaving bytes that the relay must carry.
            "close": {
                "path": "title.mp4",
                "range": format!("bytes=0-{}", 6 * MIB - 1),
                "after": 2000,
            },
            "direct": true,
            "fallback": true,
        }),
        None,
    )
    .await;
    let mut shown = seen.clone();
    shown["trace"] = json!(format!(
        "{} lines",
        seen["trace"].as_str().map_or(0, |t| t.lines().count())
    ));
    eprintln!("the closed channel's run: {shown}");
    eprintln!(
        "the closed channel's trace:\n{}",
        seen["trace"].as_str().unwrap_or_default()
    );
    assert!(at_origin(&seen["opened"], &label(me, "page")), "{shown:#}");
    assert!(
        seen["first"]["closed"]["state"] == "closed"
            && seen["first"]["closed"]["wireBytes"]
                .as_u64()
                .is_some_and(|bytes| (1..6 * MIB).contains(&bytes)),
        "the WebRTC channel must actually close before it can carry the whole range:\n{shown:#}"
    );
    film_part_answered(
        &seen["first"],
        &mp4,
        (0, 6 * MIB - 1),
        "closed mid-film, over the relay",
        &seen,
    );
    let trace = seen["trace"].as_str().unwrap_or_default();
    assert!(
        trace.contains("goes direct"),
        "the channel closed mid-film is only a check with a direct path to close:\n{shown:#}"
    );
    assert!(
        webrtc_selected(&goes_direct_paths(trace)),
        "the film must be riding the window's channel before it is closed:\n{shown:#}"
    );
    // iroh prints each custom address as transport ID + the full peer key.
    // Use the local side, including when it appears as the previous path.
    let lender = format!("_{}->", ada.endpoint().id());
    let log = std::fs::read_to_string(harness::trace_log()).unwrap();
    let paths: Vec<_> = log
        .lines()
        .filter(|line| line.contains(&lender) && line.contains("iroh::_events::path::"))
        .collect();
    eprintln!("the closed channel's lender paths: {paths:#?}");
    let abandoned = paths.iter().position(|line| {
        line.contains("path::abandoned:") && line.contains("network_path=Custom(")
    });
    assert!(
        abandoned.is_some_and(|i| paths[i + 1..].iter().any(|line| {
            line.contains("path::selected:") && line.contains("network_path=Relay(")
        })),
        "iroh must give up the closed WebRTC path and select the relay: {paths:#?}"
    );
    drop(ada);
}

in_both_engines!(
    /// **The page announcing itself after the channel closed goes direct
    /// again** (webrtc-page; demo/runs/phone-home/2026-11-20, "What it
    /// found" 1): the film's part rides `webrtc*` and the window closes the
    /// channel mid-film, as the test before this one does; once the part
    /// has arrived over the relay, the page is reloaded, so the shim
    /// announces itself again, and a second attempt must go direct, its
    /// selected path `webrtc`, and the film's part arrive whole. Planted:
    /// the worker answering every announce with the attempt it holds
    /// (`directs.has`), the phone's finding, and no second attempt came.
    the_page_announcing_itself_after_the_channel_closed_goes_direct_again
);
async fn the_page_announcing_itself_after_the_channel_closed_goes_direct_again(engine: Engine) {
    let site = site();
    let relay = relay().await;
    let bo = signing(7);
    let dir = tempfile::tempdir().unwrap();
    let (ada, mp4, _size) = lent_film(dir.path(), &relay, &bo, 1).await;
    direct_on(dir.path());

    let seen = drive(
        engine,
        "runtime.mjs",
        json!({
            "site": site,
            "key": hex::encode(bo.to_bytes()),
            "token": token(&signing(1), &relay),
            "open": "page",
            // As the closed channel's test: 6 MiB, closed after 2000 datagrams.
            "close": {
                "path": "title.mp4",
                "range": format!("bytes=0-{}", 6 * MIB - 1),
                "after": 2000,
            },
            "direct": true,
            "again": true,
            "film": film_args(),
        }),
        None,
    )
    .await;
    let mut shown = seen.clone();
    shown["trace"] = json!(format!(
        "{} lines",
        seen["trace"].as_str().map_or(0, |t| t.lines().count())
    ));
    eprintln!("the announce again's run: {shown}");
    eprintln!(
        "the announce again's trace:\n{}",
        seen["trace"].as_str().unwrap_or_default()
    );
    film_part_answered(
        &seen["first"],
        &mp4,
        (0, 6 * MIB - 1),
        "closed mid-film, over the relay",
        &seen,
    );
    let trace = seen["trace"].as_str().unwrap_or_default();
    let lines = goes_direct_lines(trace);
    assert!(
        lines.first().is_some_and(webrtc_selected),
        "the film must ride the window's channel before it is closed:\n{shown:#}"
    );
    assert_eq!(
        lines.len(),
        2,
        "the page announcing itself again must go direct afresh:\n{shown:#}"
    );
    assert!(
        webrtc_selected(&lines[1]),
        "the second attempt's selected path must be webrtc:\n{shown:#}"
    );
    film_answered(&seen["again"], &mp4, "after it went direct again", &seen);
    drop(ada);
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
    let (ada, mp4, _size) = lent_film(dir.path(), &relay, &bo, 1).await;
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
    eprintln!(
        "the withheld trace:\n{}",
        seen["trace"].as_str().unwrap_or_default()
    );
    film_answered(&seen["first"], &mp4, "withheld, over the relay", &seen);
    assert!(
        !seen["trace"]
            .as_str()
            .unwrap_or_default()
            .contains("goes direct"),
        "no direct path may come up while the window withholds the datagrams:\n{shown:#}"
    );
    drop(ada);
}

/// The paths the trace's first `goes direct` line holds, as JSON
/// (`Direct::paths`): serde_json orders an object's keys, so `kind` and
/// `selected` are not adjacent, and a path is read rather than matched as
/// text (the first runs of these tests failed on that).
fn goes_direct_paths(trace: &str) -> Value {
    goes_direct_lines(trace)
        .into_iter()
        .next()
        .expect("the direct path's line holds its paths as JSON")
}

/// The paths of every `goes direct` line in the trace, in order: one per
/// attempt that went direct.
fn goes_direct_lines(trace: &str) -> Vec<Value> {
    trace
        .lines()
        .filter_map(|l| l.split_once("goes direct: ").map(|(_, paths)| paths))
        .filter_map(|paths| serde_json::from_str(paths).ok())
        .collect()
}

/// Whether `paths` names `webrtc` as the selected one.
fn webrtc_selected(paths: &Value) -> bool {
    paths.as_array().is_some_and(|paths| {
        paths
            .iter()
            .any(|path| path["kind"] == "webrtc" && path["selected"] == true)
    })
}
