//! The room door's two decisions, over planted inputs (the u6-1 row): the
//! address picked from an interface list a node built on loopback alone
//! cannot show, and the caller let in or refused. The listeners themselves
//! are judged in `tests/room.rs`.

use std::net::{IpAddr, SocketAddr};

use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use super::{from_this_network, pick};

fn ips(raw: &[&str]) -> Vec<IpAddr> {
    raw.iter().map(|a| a.parse().unwrap()).collect()
}

/// Loopback, a container's bridge, tailscale, public, link-local and the
/// home network's, tailscale first, as `ip_addrs()` may list them.
const EVERY: [&str; 7] = [
    "100.101.12.7",
    "127.0.0.1",
    "172.17.0.1",
    "203.0.113.9",
    "169.254.10.3",
    "192.168.1.4",
    "fe80::1",
];

/// **The private LAN's address, or none.** With the route leaving by the
/// home network, that address, never tailscale's though it is listed first,
/// nor the bridge's, though it is private too; with the route unknown and
/// the bridge beside the LAN, absence, never a guess; with the bridge gone,
/// the one private address; with no private address at all, absence.
/// Planted, each watched failing: tailscale picked first (`is_shared`
/// accepted beside `is_private`), and the first private listed taken over
/// the route's.
#[test]
fn picks_the_private_lan_address_or_none() {
    let every = ips(&EVERY);
    let lan = Some("192.168.1.4".parse().unwrap());
    assert_eq!(
        pick(&every, lan).map(|a| a.to_string()).as_deref(),
        Some("192.168.1.4")
    );
    assert_eq!(
        pick(&every, None),
        None,
        "the bridge and the LAN, and no route to tell them apart"
    );
    let no_bridge = ips(&[
        "100.101.12.7",
        "127.0.0.1",
        "203.0.113.9",
        "169.254.10.3",
        "192.168.1.4",
    ]);
    assert_eq!(
        pick(&no_bridge, None).map(|a| a.to_string()).as_deref(),
        Some("192.168.1.4")
    );
    let tailscale = Some("100.101.12.7".parse().unwrap());
    assert_eq!(
        pick(&no_bridge, tailscale)
            .map(|a| a.to_string())
            .as_deref(),
        Some("192.168.1.4")
    );
    let none = ips(&[
        "100.101.12.7",
        "127.0.0.1",
        "203.0.113.9",
        "169.254.10.3",
        "fe80::1",
    ]);
    assert_eq!(pick(&none, tailscale), None, "no private address: absent");
    assert_eq!(pick(&none, Some("203.0.113.9".parse().unwrap())), None);
    assert_eq!(pick(&[], None), None);
}

/// What the room door answers a caller at `caller`, or with no address.
async fn answered(caller: Option<&str>) -> StatusCode {
    let routes = axum::Router::new()
        .fallback(|| async { "the lend" })
        .layer(axum::middleware::from_fn(from_this_network));
    let mut request = Request::new(axum::body::Body::empty());
    if let Some(caller) = caller {
        let addr: SocketAddr = caller.parse().unwrap();
        request.extensions_mut().insert(ConnectInfo(addr));
    }
    routes.oneshot(request).await.unwrap().status()
}

/// **A request from off this network is refused** (the u6-1 row): a
/// public caller, a tailscale one, loopback and one whose address is
/// unknown are refused 403; one on the home network is let in. Planted,
/// watched failing: an off-network source let in (the check answering
/// every caller).
#[tokio::test]
async fn a_caller_off_this_network_is_refused() {
    for off in ["203.0.113.9:5000", "100.101.12.7:5000", "127.0.0.1:5000"] {
        assert_eq!(answered(Some(off)).await, StatusCode::FORBIDDEN, "{off}");
    }
    assert_eq!(answered(None).await, StatusCode::FORBIDDEN, "no address");
    assert_eq!(answered(Some("192.168.1.9:5000")).await, StatusCode::OK);
}
