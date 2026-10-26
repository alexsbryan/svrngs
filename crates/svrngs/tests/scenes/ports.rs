//! **Ports, played** (give-play-scene; svrn-docs rings/current/GIVE.md §3
//! scene 2, §6; the people runs are demo/factorio.md and
//! demo/minecraft.md): Ana runs a UDP program and lends its port, `svrngs
//! share <port> --udp --name walkie`; Bo runs a TCP program and lends
//! his, `svrngs share <port> --tcp --name board`. Each, on their own
//! computer's home page, presses Play on the other's row, which says
//! "Connect to 127.0.0.1:<port>." and copies it; a program there sends to
//! that address, and the bytes cross to the other's computer and come
//! back. Then Bo presses Stop under "What acts as you", and nothing
//! listens at his address.
//!
//! The stand-ins are echo programs, a few lines each, standing for the
//! two games the people runs play: a UDP one for Factorio, a TCP one for
//! Minecraft Java. Quality on a relayed path is judged by play, in those
//! runs, never here (GIVE.md §6).
//!
//! The invariant it relies on: **a port lent opens on a member's computer
//! only while Play is on** (play.rs): a raw port carries no token, so
//! anything on that computer reaches the lend as its person while it is
//! open, and nothing does once it is off.

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

use crate::harness::Engine;
use crate::people::{late, sees, Person, Scene};

/// A UDP program on this computer that answers each packet with itself:
/// its port.
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

/// A TCP program on this computer that answers each connection with what
/// it is sent: its port.
async fn tcp_echo() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut c, _)) = listener.accept().await {
            tokio::spawn(async move {
                let (mut from, mut to) = c.split();
                let _ = tokio::io::copy(&mut from, &mut to).await;
            });
        }
    });
    port
}

/// `who` lends `port` as `name`, `how` (`--udp` or `--tcp`), as the
/// command lends one beside the svrngs running there.
async fn lends(scene: &Scene, who: &Person, port: u16, how: &str, name: &str) {
    let port = port.to_string();
    let args = ["share", &port, how, "--kind", "echo", "--name", name];
    let shared = scene.command(who, &args, "\n").await;
    let raw = how.trim_start_matches("--").to_uppercase();
    assert!(
        shared.contains(&format!("You lend {name} ({raw} port {port})")),
        "{} lends {name}: {shared}",
        who.name
    );
}

/// `who` presses Play on `name`'s row of their home page: the address its
/// row says to connect to, which the page copied.
async fn plays(engine: Engine, who: &Person, name: &str) -> SocketAddr {
    let press = json!([
        format!("#home table.index tr:has-text('{name}') button:has-text('Play')"),
        null,
        true
    ]);
    let args = json!({ "ask": who.home_page(), "all": "#home table.index td", "do": [press] });
    let seen = sees(engine, "home.mjs", args).await;
    eprintln!("{}'s home page, Play on {name}: {seen:#}", who.name);
    let after = seen["after"][0].to_string();
    let port = after.split("Connect to 127.0.0.1:").nth(1);
    let port = port.map(|p| {
        p.chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
    });
    let port = port.and_then(|p| p.parse::<u16>().ok());
    let port = port.unwrap_or_else(|| {
        panic!(
            "{}'s row for {name} says where to connect: {seen:#}",
            who.name
        )
    });
    let at = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    assert_eq!(
        seen["clipboard"],
        at.to_string(),
        "{}'s page copies {name}'s address: {seen:#}",
        who.name
    );
    at
}

/// A packet sent to `to` by a program on this computer until it comes
/// back, a datagram being one that may be lost: what came back, within
/// 20 s.
async fn udp_asks(to: SocketAddr, said: &[u8]) -> Option<Vec<u8>> {
    let program = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut buf = vec![0; 65_535];
    for _ in 0..40 {
        program.send_to(said, to).await.ok()?;
        let heard = tokio::time::timeout(Duration::from_millis(500), program.recv_from(&mut buf));
        if let Ok(Ok((n, _))) = heard.await {
            return Some(buf[..n].to_vec());
        }
    }
    None
}

/// What a TCP program on this computer hears back from `to`, within 20 s.
async fn tcp_asks(to: SocketAddr, said: &[u8]) -> Option<Vec<u8>> {
    let asked = async {
        let mut program = TcpStream::connect(to).await.ok()?;
        program.write_all(said).await.ok()?;
        let mut heard = vec![0; said.len()];
        program.read_exact(&mut heard).await.ok()?;
        Some(heard)
    };
    tokio::time::timeout(Duration::from_secs(20), asked)
        .await
        .ok()?
}

scene!(
    /// Planted, watched failing: a hand edit, play.rs `opened` dropping
    /// the carrying it made (`Ok(carried) => drop(carried)`), which
    /// unbinds the listener while the row still says where to connect;
    /// reverted after.
    ports
);
async fn ports(engine: Engine) {
    let scene = Scene::new("ports", engine).await;
    let (ana, group) = scene.founds("Ana", "House").await;
    let bo = scene.joins("Bo", &ana.invites(&group).await, "House").await;

    // Each lends a port of a program on their own computer.
    lends(&scene, &ana, udp_echo().await, "--udp", "walkie").await;
    lends(&scene, &bo, tcp_echo().await, "--tcp", "board").await;
    bo.lent_by(&group, "Ana", "walkie").await;
    ana.lent_by(&group, "Bo", "board").await;

    // Bo presses Play on Ana's walkie, and a program on his computer is
    // answered by hers, by UDP.
    let walkie = plays(engine, &bo, "walkie").await;
    let said = b"over, by UDP";
    let Some(heard) = udp_asks(walkie, said).await else {
        let log = bo.log();
        let why = format!("Bo's program at {walkie} is not answered by Ana's walkie within 20 s");
        late(format!("{why}; Bo's log is {}", log.display())).await
    };
    assert_eq!(heard, said, "Ana's walkie answers Bo's program");

    // Ana presses Play on Bo's board, and a program on her computer is
    // answered by his, by TCP.
    let board = plays(engine, &ana, "board").await;
    let said = b"pinned, by TCP";
    let Some(heard) = tcp_asks(board, said).await else {
        let log = ana.log();
        let why = format!("Ana's program at {board} is not answered by Bo's board within 20 s");
        late(format!("{why}; Ana's log is {}", log.display())).await
    };
    assert_eq!(heard, said, "Bo's board answers Ana's program");

    // Bo's "What acts as you" names Ana's walkie; its Stop closes it.
    let stop = json!(["#acts button:has-text('Stop')"]);
    let args = json!({ "ask": bo.home_page(), "all": "#acts p", "do": [stop], "unfold": true });
    let seen: Value = sees(engine, "home.mjs", args).await;
    eprintln!("Bo's home page, Stop: {seen:#}");
    let line = "Open on this computer: walkie (Ana’s), while Play is on. Stop";
    assert!(
        seen["all"].to_string().contains(line),
        "Bo's page lists Ana's walkie as acting as him: {seen:#}"
    );
    let free = tokio::time::timeout(Duration::from_secs(10), async {
        while UdpSocket::bind(walkie).await.is_err() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });
    assert!(
        free.await.is_ok(),
        "nothing listens at {walkie} once Bo presses Stop"
    );
    drop((ana, bo));
}
