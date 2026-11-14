// SPDX-License-Identifier: AGPL-3.0-or-later
//! What is lent, asked of each computer now (u3-5, u4-5; DESIGN §3, "Lends
//! are asked, not stored"): A's `/lends`, a member's computer's own list
//! with where to reach each lend, and the people's lends A asks of each
//! lender's computer, each with its key and relay. One crossing, five
//! computers on one wire (common/on_wire.rs), A's service answered there; what each list keeps or leaves out is judged on
//! its own (`src/lend/tests.rs` `a_lend_is_listed_only_through_a_group_it_is_lent_to`,
//! `declarations_that_cannot_be_read_are_an_error_never_none`;
//! `src/door/limit/tests.rs`). Each step names the test it came from,
//! retired at its move (docs/seams.tsv).

use super::*;

/// What A lends, as B asks it now: `(name, kind, limit, using)`.
async fn lends_of(pair: &Pair) -> Vec<(String, String, u64, u64)> {
    let reply = pair.get("/lends", "x-svrngs-version: 5\r\n").await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    let doc: serde_json::Value =
        serde_json::from_str(reply.text.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    let lends = doc["lends"].as_array().unwrap().iter();
    lends
        .map(|l| {
            let text = |k: &str| l[k].as_str().unwrap().to_string();
            (
                text("name"),
                text("kind"),
                l["limit"].as_u64().unwrap(),
                l["using"].as_u64().unwrap(),
            )
        })
        .collect()
}

/// An origin that answers a request's head and first chunk, then holds the
/// rest of its answer until `release` fires.
async fn held_origin() -> (SocketAddr, tokio::sync::oneshot::Sender<()>) {
    let mut listener = origins::Listener::bind().await;
    let addr = listener.local_addr().unwrap();
    let (release, released) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let (mut sock, _) = listener.accept().await.unwrap();
        let mut head = Vec::new();
        while !head.ends_with(b"\r\n\r\n") {
            let mut byte = [0u8; 1];
            sock.read_exact(&mut byte).await.unwrap();
            head.push(byte[0]);
        }
        sock.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n")
            .await
            .unwrap();
        let _ = released.await;
        sock.write_all(b"0\r\n\r\n").await.unwrap();
    });
    (addr, release)
}

/// The people's lends A answers B now.
async fn peoples(pair: &Pair) -> serde_json::Value {
    let reply = pair
        .get(
            svrngs_node::node::PEOPLE_LENDS_ROUTE,
            "x-svrngs-version: 5\r\n",
        )
        .await;
    assert_eq!(reply.status, 200, "{}", reply.text);
    serde_json::from_str(reply.text.split("\r\n\r\n").nth(1).unwrap()).unwrap()
}

/// Planted, each watched failing when its test was its own: the
/// declarations read once, when the door starts (the withdrawn lend still
/// listed); the use ended when the service answers rather than when the
/// answer ends; the whole audience listed; A's own lends left out of the
/// people's; the groups the asker does not stand in asked too; no relay
/// given; house's member taken for a key in two groups, which names no
/// relay.
#[tokio::test]
async fn what_is_lent_is_asked_of_each_computer_now() {
    let (pair, heads) = lending_heads(true).await;

    // a_member_asks_what_is_lent_and_how_many_use_it_now (main.rs:722-768).
    let (origin, release) = held_origin().await;
    pair.lend("slow", origin.port(), &[]);
    let row = |name: &str, using| (name.to_string(), "k".to_string(), 3, using);
    assert_eq!(lends_of(&pair).await, vec![row("shelf", 0), row("slow", 0)]);
    // The answer's head and first chunk read, the rest held.
    let asked = "GET /lend/slow/x HTTP/1.1\r\nHost: local\r\nConnection: close\r\n\r\n";
    let (_, mut body) = on_wire::raw_held(pair.bridge, asked).await.expect("let in");
    let first = http_body_util::BodyExt::frame(&mut body).await;
    let first = first.expect("the first chunk comes through").unwrap();
    assert_eq!(first.data_ref().map(|d| &d[..]), Some(&b"hello"[..]));
    let slow = |l: &Vec<(String, String, u64, u64)>| l.iter().find(|x| x.0 == "slow").unwrap().3;
    assert_eq!(
        slow(&lends_of(&pair).await),
        1,
        "bo is using it while it streams"
    );
    release.send(()).unwrap();
    axum::body::to_bytes(body, usize::MAX).await.unwrap();
    assert_eq!(
        slow(&lends_of(&pair).await),
        1,
        "the answer ended, and bo is counted until the window passes"
    );
    lend::withdraw(pair.dir.path(), &LendName::parse("slow").unwrap()).unwrap();
    assert_eq!(
        lends_of(&pair).await,
        vec![row("shelf", 0)],
        "a lend withdrawn is gone at the next ask"
    );

    // a_list_tells_a_member_only_the_groups_it_shares_with_the_lend (scope.rs:76-87).
    let yard = [("ada", 1), ("fay", 5)]
        .map(|(p, k)| (Person::parse(p).unwrap(), vec![actor_of(&signing(k))]));
    pair.a
        .rail()
        .set_seed("yard", &Roster::new(yard.into_iter().collect()))
        .unwrap();
    let shelf = LendName::parse("shelf").unwrap();
    assert!(lend::set_audience(pair.dir.path(), &shelf, vec![NS.into(), "yard".into()]).unwrap());
    let reply = pair.get("/lends", "x-svrngs-version: 5\r\n").await;
    let doc: serde_json::Value =
        serde_json::from_str(reply.text.split("\r\n\r\n").nth(1).unwrap()).unwrap();
    assert_eq!(doc["lends"][0]["groups"], serde_json::json!([NS]), "{doc}");

    // a_member_s_computer_lists_what_is_lent_with_where_to_reach_it (main.rs:779-836):
    // dan, whose computer is nowhere, not answering.
    svrngs_core::tests_support::let_in_by(NS, &signing(9), "dan", |act| {
        let act = svrngs_core::tests_support::Fixture { act };
        pair.a.rail().commit(NS, &act).unwrap().unwrap().lines()[0]
            .id
            .clone()
    });
    let seed = Roster::new(
        [(Person::parse("ada").unwrap(), vec![actor_of(&signing(1))])]
            .into_iter()
            .collect(),
    );
    pair.b.rail().set_seed(NS, &seed).unwrap();
    let synced = pair.b.host().sync_with(NS, &pair.addr()).await;
    assert!(synced.stop.is_none(), "{synced:?}");
    let lends = pair.b.lends().await;
    let unanswered = lends.unlisted.unanswered.iter().map(|n| &n.person);
    assert_eq!(
        unanswered.collect::<Vec<_>>(),
        [&Person::parse("dan").unwrap()]
    );
    assert_eq!(lends.lent.len(), 1, "{lends:?}");
    let lent = &lends.lent[0];
    assert_eq!(lent.person, Person::parse("ada").unwrap());
    assert_eq!(
        (lent.offered.name.as_str(), lent.offered.kind.as_str()),
        ("shelf", "k")
    );
    assert_eq!(lent.offered.using, 0);
    let label = svrngs_core::pass::Label::of(pair.a.net().id().as_bytes(), "shelf");
    let host = format!("{label}.localhost");
    assert_eq!(
        lent.address,
        format!("http://{host}:{}/", pair.b.api_addr().port())
    );
    // As a program on B asks it, with B's token (give-session), its name
    // resolved to B's API, in this process.
    let reply = on_wire::http()
        .get(format!("{}whoami", lent.address))
        .bearer_auth(pair.b.token().trim())
        .send()
        .await
        .unwrap();
    assert_eq!(reply.status(), 200);
    clean(reply.text().await.unwrap().as_bytes());
    assert_eq!(header_values(&last(&heads), "x-svrngs-person"), vec!["bo"]);
    let file = lend::lends_dir(pair.dir.path()).join("shelf.json");
    let declared = std::fs::read(&file).unwrap();
    lend::withdraw(pair.dir.path(), &shelf).unwrap();
    assert!(
        pair.b.lends().await.lent.is_empty(),
        "a lend withdrawn is gone at the next ask"
    );
    std::fs::write(&file, declared).unwrap();

    // a_member_asks_what_the_group_s_people_lend_each_with_its_lender_s_key
    // (main.rs:889-954): cy lends pad in the house; eve, of work with ada
    // alone, lends desk; ada's relay in the record, with the root's dot.
    pair.a.stop_sync();
    let (cy, cy_dir) = beside(&pair, NS, 3, "cy").await;
    lends_on(cy_dir.path(), "pad", 1);
    let (eve, eve_dir) = beside(&pair, "work", 4, "eve").await;
    lends_on(eve_dir.path(), "desk", 1);
    let at = svrngs_core::RelayAt {
        key: actor_of(&signing(1)),
        url: svrngs_core::relay::parse("https://relay.example./").ok(),
    };
    pair.a.rail().commit(NS, &at).unwrap().unwrap();
    let doc = peoples(&pair).await;
    let told: Vec<(&str, &str, &str)> = (doc["lent"].as_array().unwrap().iter())
        .map(|l| {
            (
                l["person"].as_str().unwrap(),
                l["key"].as_str().unwrap(),
                l["name"].as_str().unwrap(),
            )
        })
        .collect();
    let (ada, cy_key) = (actor_of(&signing(1)), actor_of(&signing(3)));
    assert_eq!(
        told,
        vec![
            ("ada", ada.as_str(), "shelf"),
            ("cy", cy_key.as_str(), "pad")
        ],
        "{doc:#}"
    );
    let dan = serde_json::json!([{ "person": "dan", "person_key": actor_of(&signing(9)) }]);
    assert_eq!(doc["unanswered"], dan, "{doc:#}");
    let relays: Vec<&serde_json::Value> = doc["lent"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| &l["relay"])
        .collect();
    let want = [
        serde_json::json!("https://relay.example/"),
        serde_json::Value::Null,
    ];
    assert_eq!(relays, want.iter().collect::<Vec<_>>(), "{doc:#}");

    // a_key_in_two_groups_that_disagree_is_dialled_and_told_by_one_relay
    // (main.rs:977-1013): bo and cy let into work, which names a relay for cy.
    for (person, me) in [("bo", 2), ("cy", 3)] {
        svrngs_core::tests_support::let_in_by("work", &signing(me), person, |act| {
            let act = svrngs_core::tests_support::Fixture { act };
            pair.a.rail().commit("work", &act).unwrap().unwrap().lines()[0]
                .id
                .clone()
        });
    }
    let relay = "https://relay.work.example/";
    let said = RailAct::Relay {
        url: Some(relay.into()),
    };
    let said = svrngs_core::tests_support::signed_in("work", &signing(3), unix_now() + 5, 0, said);
    pair.a.rail().ingest_all("work", &[said]).unwrap();
    let doc = peoples(&pair).await;
    let told = (doc["lent"].as_array().unwrap().iter())
        .find(|l| l["key"] == cy_key.as_str())
        .unwrap_or_else(|| panic!("cy's pad not told: {doc:#}"));
    assert_eq!(told["relay"], serde_json::json!(relay), "{doc:#}");
    let want = vec![relay.to_string()];
    assert_eq!(
        pair.a.relays_dialled(&cy_key),
        [want.clone(), want],
        "a folder's fetch and the origin, then the round and the lend asks"
    );
    for node in [cy, eve] {
        node.net().close().await;
    }
    pair.close().await;
}
