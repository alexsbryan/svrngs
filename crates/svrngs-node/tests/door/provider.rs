// SPDX-License-Identifier: AGPL-3.0-or-later
//! The local provider scene (`demo/provider-lending.md`). The baseline uses
//! the same origin on both sides to distinguish refusal from an unavailable
//! service. Its result is not independent-provider acceptance.

use super::{bridge_as, header_values, lends_on, origins, raw_to, signing, start, unix_now, Pair};
use svrngs_core::alpn::Alpn;
use svrngs_core::pass::{Label, Pass};
use svrngs_node::actor_of;
use svrngs_node::lend::{self, LendName};

#[tokio::test]
async fn scoped_control_answers_but_a_library_without_a_group_refuses_a_pass() {
    let control = Pair::new().await;
    let (origin, heads) = origins::head_origin().await;
    control.lend("booth", origin.port(), &[]);
    let reply = control.get("/lend/booth/availability", "").await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    assert!(reply.text.ends_with("\r\n\r\nok"), "{}", reply.text);
    assert_eq!(heads.lock().unwrap().len(), 1);
    eprintln!("Scoped control: 200, one request reached the booth service.");

    let root = tempfile::tempdir().unwrap();
    let library = start(11, root.path()).await;
    assert!(library.rail().namespaces().unwrap().is_empty());
    lends_on(root.path(), "booth", origin.port());
    let booth = LendName::parse("booth").unwrap();
    assert!(lend::set_audience(root.path(), &booth, vec![]).unwrap());
    // Planted: a different, undeclared label was a 404, not a pass refusal.
    let label = Label::of(library.endpoint().id().as_bytes(), booth.as_str());
    let holder = signing(12);
    let pass = Pass::sign(
        &signing(2),
        &actor_of(&holder),
        (label.clone(), vec![]),
        unix_now() + 60,
    );
    let bridge = bridge_as(&holder, &library, Alpn::Lend).await;
    let reply = raw_to(
        bridge.local_addr(),
        format!(
            "GET /lend/{label}/availability HTTP/1.1\r\nHost: local\r\nX-Svrngs-Pass: {}\r\nConnection: close\r\n\r\n",
            pass.encode()
        ),
    )
    .await;
    assert_eq!(reply.status, 403, "{}", reply.text);
    assert_eq!(header_values(&reply.text, "x-svrngs-refused"), ["pass"]);
    assert_eq!(
        heads.lock().unwrap().len(),
        1,
        "the refused request reached the booth"
    );
    eprintln!(
        "Independent library: 403 to a signed outside pass, zero extra requests at the booth."
    );
    eprintln!("PROVIDER_BASELINE=passed");
    library.net().close().await;
    control.close().await;
}
