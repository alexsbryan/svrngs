//! Play on the home page (give-play; svrn-docs rings/current/GIVE.md §3
//! scene 2, §6): a port a member of the group lends, UDP or TCP, is a row
//! of the home page's index with Play; pressed, the row says where a
//! program on this computer connects, `127.0.0.1` at the lend's own port
//! (`room::port_of`), and copies it, and "What acts as you" lists it with
//! its Stop, since anything on this computer reaches that lend as this
//! person while Play is on: a raw port carries no token. The restart, a
//! Stop's unbinding and a removal are the node's, judged without a page
//! (`crates/svrngs/tests/play.rs`). Run in Chromium alone: Play is a
//! computer's, and a phone's page offers none (the give queue's fence 13).

use std::net::{Ipv4Addr, SocketAddr};

use tokio::net::UdpSocket;

use super::*;

/// A UDP program on cy's computer that answers each packet with itself.
async fn udp_echo() -> u16 {
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let port = socket.local_addr().unwrap().port();
    tokio::spawn(async move {
        let mut buf = vec![0; 65_535];
        while let Ok((n, from)) = socket.recv_from(&mut buf).await {
            let _ = socket.send_to(&buf[..n], from).await;
        }
    });
    port
}

/// What a packet sent to `to` from a program on ada's computer hears back.
async fn udp_asks(to: SocketAddr, said: &[u8]) -> Option<Vec<u8>> {
    let program = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut buf = vec![0; 65_535];
    for _ in 0..10 {
        program.send_to(said, to).await.ok()?;
        let heard = tokio::time::timeout(Duration::from_millis(500), program.recv_from(&mut buf));
        if let Ok(Ok((n, _))) = heard.await {
            return Some(buf[..n].to_vec());
        }
    }
    None
}

in_chromium!(
    /// **A member presses Play, and its Stop is listed under "What acts as
    /// you"**. Cy lends `played`, a UDP port. Ada's home page lists it with
    /// Play, and no address to open in a tab, since a raw port is no page.
    /// Pressed, its row says "Connect to 127.0.0.1:<its port>", the address
    /// copied, and a program on ada's computer is answered there by cy's.
    /// Opened again, "What acts as you" says "Open on this computer: played
    /// (cy’s), while Play is on." with Stop; pressed, that Stop is the row's
    /// own: the row offers Play again, and nothing listens at the port.
    /// Red at d80a8ab9: no Play, and no line.
    a_member_presses_play_and_its_stop_is_listed_under_what_acts_as_you
);
async fn a_member_presses_play_and_its_stop_is_listed_under_what_acts_as_you(engine: Engine) {
    let relay = relay().await;
    let (ada_dir, cy_dir) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
    let ada = node(ada_dir.path(), &relay, &signing(7)).await;
    let cy = joined(cy_dir.path(), &relay, &ada, 44, "cy").await;
    let lends = svrngs_node::lend::lends_dir(cy_dir.path());
    std::fs::create_dir_all(&lends).unwrap();
    let body = json!({ "port": udp_echo().await, "raw": "udp", "kind": "echo", "limit": 3 });
    std::fs::write(lends.join("played.json"), body.to_string()).unwrap();
    let label = svrngs_core::pass::Label::of(signing(44).verifying_key().as_bytes(), "played");
    let at = SocketAddr::from((Ipv4Addr::LOCALHOST, svrngs_node::room::port_of(&label)));
    let row = "#home table.index td";
    let press = json!([
        format!("#home table.index tr:has-text('played') button:has-text('Play')"),
        null,
        true
    ]);
    let seen = home_page(engine, &ada, json!({ "all": row, "do": [press] })).await;
    eprintln!("the home page's run: {seen:#}");
    let before = seen["all"].to_string();
    assert!(before.contains("Play"), "{seen:#}");
    assert!(
        !before.contains("Copy its address for a program"),
        "{seen:#}"
    );
    let on = format!("Connect to {at}");
    assert!(seen["after"][0].to_string().contains(&on), "{seen:#}");
    assert_eq!(seen["clipboard"], at.to_string(), "{seen:#}");
    assert_eq!(udp_asks(at, b"hello").await.as_deref(), Some(&b"hello"[..]));

    let stop = json!(["#acts button:has-text('Stop')"]);
    let seen = home_page(
        engine,
        &ada,
        json!({ "all": row, "do": [stop], "text": true }),
    )
    .await;
    eprintln!("the home page's second run: {seen:#}");
    assert!(seen["all"].to_string().contains(&on), "{seen:#}");
    let text = seen["text"].as_str().unwrap_or_default();
    assert!(
        text.contains("Open on this computer: played (cy’s), while Play is on."),
        "{text}"
    );
    let after = seen["after"][0].to_string();
    assert!(!after.contains(&on) && after.contains("Play"), "{seen:#}");
    let free = tokio::time::timeout(Duration::from_secs(10), async {
        while UdpSocket::bind(at).await.is_err() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
    assert!(
        free.await.is_ok(),
        "nothing listens at {at} once Stop is pressed"
    );
    drop((ada, cy));
}
