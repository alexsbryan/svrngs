//! The people-at-once limit and each one's share, through the door (u3-6;
//! DESIGN §8, "Many people, one library"). A lends a streaming service to
//! three people at once, 30 Mbit/s declared; bo, cy, dee and eve, each on
//! their own computer standing in A's group, dial A by key over loopback
//! alone, as `super`'s pair does (#33). The window is
//! waited out once, for real: the use a stream holds ends only when its
//! answer is dropped, and nothing else here can see that it did. Its edge
//! is crossed on values in `src/door/limit/tests.rs`.

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::common::loopback_sockets;
use super::{clean, header_values, no_seed, origins, signing, Reply, NS};
use svrngs_node::door::limit::WINDOW;
use svrngs_node::door::SHARE_HEADER;
use svrngs_node::iroh::{Contact, RelayConfig};
use svrngs_node::lend;
use svrngs_node::node::Node;
use svrngs_node::{actor_of, Person, RailAct, Roster};

/// A third of 30 Mbit/s, in bits a second.
const A_THIRD: &str = "10000000";

/// A lender, and four people of its group, each with their bridge to it.
struct Crowd {
    a: Node,
    _members: Vec<Node>,
    bridges: Vec<SocketAddr>,
    _dirs: Vec<tempfile::TempDir>,
    heads: Arc<Mutex<Vec<String>>>,
}

const PEOPLE: [&str; 4] = ["bo", "cy", "dee", "eve"];

async fn start(seed: u8, dir: &std::path::Path) -> Node {
    Node::start(
        dir.to_path_buf(),
        signing(seed),
        &RelayConfig::from_parts(Vec::new(), Some("none"))
            .unwrap()
            .on_loopback_only(),
        no_seed,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .expect("a node starts")
}

impl Crowd {
    async fn new() -> Self {
        let dirs: Vec<tempfile::TempDir> = (0..5).map(|_| tempfile::tempdir().unwrap()).collect();
        let a = start(1, dirs[0].path()).await;
        let seed = Roster::new(
            [(Person::parse("ada").unwrap(), vec![actor_of(&signing(1))])]
                .into_iter()
                .collect(),
        );
        a.rail().set_seed(NS, &seed).unwrap();
        let (mut members, mut bridges) = (Vec::new(), Vec::new());
        for (i, who) in PEOPLE.iter().enumerate() {
            let key = 2 + u8::try_from(i).unwrap();
            let admit = svrngs_core::tests_support::Fixture {
                act: RailAct::Admit {
                    person: Person::parse(who).unwrap(),
                    key: actor_of(&signing(key)),
                    invite: None,
                },
                on_behalf_of: None,
            };
            a.rail().commit(NS, &admit).unwrap().unwrap();
            let member = start(key, dirs[i + 1].path()).await;
            let base = member
                .transport()
                .base_url(&Contact {
                    key: a.endpoint().id(),
                    relay_url: None,
                    iroh_direct_addrs: loopback_sockets(a.endpoint()),
                })
                .await
                .expect("a contact with an address resolves");
            bridges.push(base.trim_start_matches("http://").parse().unwrap());
            members.push(member);
        }
        let (origin, heads) = origins::streaming_origin().await;
        let dir = lend::lends_dir(dirs[0].path());
        std::fs::create_dir_all(&dir).unwrap();
        let body = serde_json::json!({
            "port": origin.port(), "kind": "k", "limit": 3, "upload_bps": 30_000_000,
        });
        std::fs::write(dir.join("shelf.json"), body.to_string()).unwrap();
        Crowd {
            a,
            _members: members,
            bridges,
            _dirs: dirs,
            heads,
        }
    }

    fn bridge(&self, who: &str) -> SocketAddr {
        self.bridges[PEOPLE.iter().position(|p| *p == who).unwrap()]
    }

    /// `who` starts streaming: the answer's head and first chunk read, the
    /// rest left to come for as long as the socket is held.
    async fn stream(&self, who: &str) -> (TcpStream, String) {
        let mut sock = TcpStream::connect(self.bridge(who)).await.unwrap();
        sock.write_all(b"GET /lend/shelf/stream HTTP/1.1\r\nHost: local\r\n\r\n")
            .await
            .unwrap();
        let mut got = Vec::new();
        while !got.windows(5).any(|w| w == b"hello") {
            let mut buf = [0u8; 1024];
            let n = tokio::time::timeout(Duration::from_secs(20), sock.read(&mut buf))
                .await
                .expect("the stream starts")
                .unwrap();
            assert!(n > 0, "{who}'s stream ended early: {got:?}");
            got.extend_from_slice(&buf[..n]);
        }
        clean(&got);
        (sock, String::from_utf8_lossy(&got).into_owned())
    }

    /// `who` asks once more beside their stream, as a seek does.
    async fn seek(&self, who: &str) -> Reply {
        let mut sock = TcpStream::connect(self.bridge(who)).await.unwrap();
        sock.write_all(
            b"GET /lend/shelf/seek HTTP/1.1\r\nHost: local\r\nConnection: close\r\n\r\n",
        )
        .await
        .unwrap();
        let mut reply = Vec::new();
        tokio::time::timeout(Duration::from_secs(30), sock.read_to_end(&mut reply))
            .await
            .expect("the door answers")
            .unwrap();
        Reply::of(reply)
    }

    /// The share the service was told on the last request it received.
    fn told_the_service(&self) -> Vec<String> {
        let heads = self.heads.lock().unwrap();
        let head = heads.last().expect("the service was asked");
        header_values(head, SHARE_HEADER)
            .into_iter()
            .map(str::to_string)
            .collect()
    }
}

fn share_in(head: &str) -> Vec<&str> {
    header_values(head.split("\r\n\r\n").next().unwrap(), SHARE_HEADER)
}

/// Three people streaming are each told a third of the declared upload —
/// on the answer they read and on the request the service reads — each
/// asking beside their own stream, so four requests are three people. A
/// fourth is refused with a reason naming three. One stops, and after the
/// window the fourth is let in, and each is told a third. Planted: the
/// share divided by requests (bo, seeking, told a quarter).
#[tokio::test]
async fn three_streaming_are_told_a_third_and_a_fourth_waits_for_one_to_stop() {
    let crowd = Crowd::new().await;
    let mut streams = Vec::new();
    for who in ["bo", "cy", "dee"] {
        streams.push(crowd.stream(who).await);
    }
    assert_eq!(
        share_in(&streams[2].1),
        vec![A_THIRD],
        "dee, the third, is told a third"
    );
    for who in ["bo", "cy", "dee"] {
        let reply = crowd.seek(who).await;
        assert_eq!(reply.status, 200, "{}", reply.text);
        assert_eq!(
            share_in(&reply.text),
            vec![A_THIRD],
            "{who}: {}",
            reply.text
        );
        assert_eq!(crowd.told_the_service(), vec![A_THIRD], "{who}");
    }

    let refused = crowd.seek("eve").await;
    assert_eq!(refused.status, 503, "{}", refused.text);
    assert!(
        refused
            .text
            .contains("shelf is lent to 3 people at once, and 3 are using it now"),
        "{}",
        refused.text
    );
    assert!(share_in(&refused.text).is_empty(), "{}", refused.text);

    let (cy, _) = streams.remove(1);
    drop(cy);
    tokio::time::sleep(WINDOW + Duration::from_secs(2)).await;
    for who in ["eve", "bo", "dee"] {
        let reply = crowd.seek(who).await;
        assert_eq!(reply.status, 200, "{who}: {}", reply.text);
        assert_eq!(
            share_in(&reply.text),
            vec![A_THIRD],
            "{who}: {}",
            reply.text
        );
    }
    drop(streams);
    crowd.a.endpoint().close().await;
}
