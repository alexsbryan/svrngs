//! A way to a relay that a test can cut: relay_tiers.rs's.

use std::net::SocketAddr;

/// A plain TCP way to the relay at `to`, and its switch: while cut, each
/// connection through it is closed and each new one held unanswered, then
/// carried once it is not, so iroh's dial waits rather than backs off.
pub async fn cuttable(to: SocketAddr) -> (String, tokio::sync::watch::Sender<bool>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (url, cut) = (
        format!("http://{}/", listener.local_addr().unwrap()),
        tokio::sync::watch::Sender::new(false),
    );
    let switch = cut.clone();
    tokio::spawn(async move {
        while let Ok((mut asked, _)) = listener.accept().await {
            let mut cut = switch.subscribe();
            tokio::spawn(async move {
                let _ = cut.wait_for(|c| !c).await;
                let mut relay = tokio::net::TcpStream::connect(to).await?;
                tokio::select! {
                    _ = tokio::io::copy_bidirectional(&mut asked, &mut relay) => {}
                    _ = cut.wait_for(|c| *c) => {}
                }
                std::io::Result::Ok(())
            });
        }
    });
    (url, cut)
}
