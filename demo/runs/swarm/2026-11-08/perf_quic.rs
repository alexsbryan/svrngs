// Step 1's diagnostic, as it ran: dropped into crates/svrngs-node/tests/perf.rs
// beside perf_blobs, with `noq-proto = "=1.0.1"` under [dev-dependencies].
#[test]
#[ignore = "an instrument the perf run's driver starts"]
fn perf_quic() {
    // Step 1's diagnostic: one stream of PERF_SIZE over a shaped path, with
    // PERF_CC bbr3 or cubic, printing the sender's cwnd and controller each
    // second, so a run's ping can be read against what the controller did.
    use iroh::endpoint::{presets, QuicTransportConfig};
    use noq_proto::congestion::{Bbr3Config, CubicConfig};
    use std::sync::Arc;
    let size: u64 = env("PERF_SIZE").parse().unwrap();
    let addr_file = std::path::PathBuf::from(env("PERF_DIR")).join("quic-addr");
    let cc: Arc<dyn iroh::endpoint::ControllerFactory + Send + Sync> = match env("PERF_CC").as_str() {
        "bbr3" => Arc::new(Bbr3Config::default()),
        _ => Arc::new(CubicConfig::default()),
    };
    let cfg = QuicTransportConfig::builder().congestion_controller_factory(cc).build();
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async move {
        let ep = iroh::Endpoint::builder(presets::Minimal)
            .crypto_provider(svrngs_node::guest::ring_crypto_provider())
            .relay_mode(iroh::RelayMode::Disabled)
            .alpns(vec![b"q".to_vec()])
            .transport_config(cfg)
            .bind()
            .await
            .unwrap();
        if std::env::var("PERF_ROLE").as_deref() == Ok("serve") {
            let port = ep.bound_sockets()[0].port();
            std::fs::write(&addr_file, format!("{}\n{}", ep.id(), port)).unwrap();
            let conn = ep.accept().await.unwrap().await.unwrap();
            let mut send = conn.open_uni().await.unwrap();
            let conn2 = conn.clone();
            tokio::spawn(async move {
                let t0 = std::time::Instant::now();
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    let (mut cwnd, mut lost) = (0u64, 0u64);
                    for p in conn2.paths().iter() {
                        let s = p.stats();
                        cwnd = s.cwnd.max(cwnd);
                        lost += s.lost_packets;
                    }
                    let cc = conn2.congestion_state(iroh::endpoint::PathId::ZERO).map(|c| format!("{c:?}"));
                    let name = cc.as_deref().unwrap_or("?");
                    let name = name.split(|c: char| !c.is_alphanumeric()).next().unwrap_or("?").to_string();
                    println!("QUIC t={:.0} cwnd={cwnd} lost={lost} controller={name}", t0.elapsed().as_secs_f64());
                }
            });
            let piece = vec![0u8; 64 * 1024];
            let (mut sent, t0) = (0u64, std::time::Instant::now());
            while sent < size {
                send.write_all(&piece).await.unwrap();
                sent += piece.len() as u64;
            }
            send.finish().unwrap();
            println!("QUIC done {sent} in {:?}", t0.elapsed());
            conn.closed().await;
        } else {
            let text = loop {
                if let Ok(t) = std::fs::read_to_string(&addr_file) {
                    break t;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            };
            let (id, port) = text.split_once('\n').unwrap();
            let id: iroh::EndpointId = id.parse().unwrap();
            let sock: std::net::SocketAddr = format!("{}:{}", env("PERF_PEER"), port).parse().unwrap();
            let conn = ep.connect(iroh::EndpointAddr::new(id).with_ip_addr(sock), b"q").await.unwrap();
            let mut recv = conn.accept_uni().await.unwrap();
            let t0 = std::time::Instant::now();
            let bytes = recv.read_to_end(usize::MAX).await.unwrap();
            println!("QUIC got {} in {:?}", bytes.len(), t0.elapsed());
            conn.close(0u32.into(), b"done");
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    });
}

