//! A phone keeps going when the internet goes out at home (svrn-docs
//! rings/current/KEEP_GOING_AT_HOME.md): what it saved while everything
//! answered is what it reopens on (web/tests/reopen.mjs).

use super::*;

/// The relay `node` is homed at now, if any.
fn home(node: &Node) -> Option<String> {
    let addr = node.endpoint().addr();
    let url = addr.relay_urls().next().map(ToString::to_string);
    url
}

/// The browser half run on `args`, with what `cut` does done between its
/// browser's close and its reopening.
async fn reopened<F: std::future::Future<Output = ()> + Send + 'static>(
    engine: Engine,
    mut args: Value,
    cut: F,
) -> Value {
    let dir = tempfile::tempdir().unwrap();
    let ready = dir.path().join("ready");
    args["ready"] = json!(ready);
    let (go, wait) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        cut.await;
        let _ = go.send(());
    });
    drive(engine, "reopen.mjs", args, Some(wait)).await
}

in_webkit!(
    /// **A phone learns the group's relay with no new invitation**
    /// (KEEP_GOING_AT_HOME 4). Bo's link names only the public relay; the
    /// keeper has since said the group meets at the home relay. Opened once
    /// while the public relay answers, the phone keeps the home one on its
    /// group; the public relay goes and ada's computer moves; the browser
    /// reopened reaches the group through the relay no link names. Planted,
    /// watched failing: `/people/lends` naming no `meets`, so nothing is
    /// kept and the reopen ends in no relay answering.
    a_phone_learns_the_group_s_relay_with_no_new_invitation
);
async fn a_phone_learns_the_group_s_relay_with_no_new_invitation(engine: Engine) {
    let (public, house_relay) = (relay().await, relay().await);
    let dir = tempfile::tempdir().unwrap();
    let bo = signing(7);
    let (ada, id) = house(dir.path(), &public, &bo).await;
    let at_home = relay_url(&house_relay);
    let named: svrngs_node::iroh::RelayUrl = at_home.parse().unwrap();
    ada.host().meet(&id, &[named]).expect("ada keeps the group");

    let args = json!({
        "site": site(),
        "key": hex::encode(bo.to_bytes()),
        "token": token(&signing(1), &public),
        "id": id,
    });
    let ada = std::sync::Arc::new(ada);
    let (moved, moves) = (ada.clone(), at_home.clone());
    let seen = reopened(engine, args, async move {
        public.shutdown().await.unwrap();
        let start = std::time::Instant::now();
        while home(&moved).as_deref() != Some(moves.as_str()) {
            assert!(
                start.elapsed() < Duration::from_secs(90),
                "ada did not move"
            );
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await;
    eprintln!("{seen:#}");
    assert_eq!(
        seen["first"]["group"]["meets"],
        json!([at_home]),
        "{seen:#}"
    );
    assert!(
        seen["again"]["kind"].as_str().unwrap().contains("good"),
        "the reopened phone reached the group through the relay it learned: {seen:#}"
    );
}
