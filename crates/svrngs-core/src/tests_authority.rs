//! Native source meaning is signature-covered, not an unsigned parser hint.
use crate::*;

#[test]
fn a_new_group_commits_its_source_policy_in_its_actual_seed_identity() {
    let key = SigningKey::from_bytes(&[1; 32]);
    let made = seed::Seed::from_parts(
        Name::parse("house").unwrap(),
        Name::parse("ana").unwrap(),
        key.verifying_key(),
        [7; 16],
    );
    // An older svrngs's seed file: the same fields, and no policy.
    let mut file: serde_json::Value = serde_json::from_str(&made.to_text()).unwrap();
    file.as_object_mut().unwrap().remove("policy");
    let legacy = seed::Seed::from_text(&file.to_string()).expect("an older svrngs's seed reads");
    assert!(legacy.older() && !made.older());
    let original = legacy.to_text();
    file["policy"] = serde_json::json!("sources_v1");
    let source_text = serde_json::to_string(&file).unwrap();
    let source = seed::Seed::from_text(&source_text)
        .expect("an explicit source-policy seed is a supported new group, never a legacy upgrade");
    assert_eq!(made, source);
    assert!(authority::SourceRoots::from_seed(&legacy).is_err());
    let roots = authority::SourceRoots::from_seed(&source).unwrap();
    assert_eq!(roots.namespace(), source.id());
    let held = ActSet::default();
    let graph = authority::NativeAuthority::new(&roots, &held, &Ed25519Verifier);
    assert_eq!(graph.members().len(), 1);
    assert!(
        matches!(graph.office(), authority::SourceOffice::Held { key: holder, source: authority::AuthorityRef::Genesis } if holder == actor_of(&key))
    );
    assert_ne!(source.id(), legacy.id());
    // Independent Python blake3 producer, recorded in the source-graph run.
    assert_eq!(
        legacy.id(),
        "084969014b79276d234481f59cd965b1d408beeaf96874dcec6370259bac3e7e"
    );
    assert_eq!(
        source.id(),
        "0b30f9662c070504d9983a47acd352d9a27a0bccceb0ca61c737f4667f575b65"
    );
    assert!(seed::answered(&legacy.id(), &source_text).is_none());
    assert!(seed::answered(&source.id(), &original).is_none());
    assert_eq!(
        seed::answered(&source.id(), &source_text),
        Some(source.clone())
    );
    assert_eq!(
        seed::Seed::from_text(&original).unwrap().to_text(),
        original
    );
    file["policy"] = serde_json::json!("sources_v999");
    assert!(seed::Seed::from_text(&serde_json::to_string(&file).unwrap()).is_err());
}

/// **A group an older svrngs made stands nobody, and refuses a write and a
/// join in words that say the way on** (svrn-docs EUDOXIA_JUDGE §2a). Alex's
/// seed names no policy, as an older svrngs wrote every seed; its journal
/// holds alex's record and alex's link. Nobody stands and nothing counts,
/// and the fold says why; alex's write and a joiner's redemption of the
/// link are refused `Older`, said in the same words, and nothing is
/// written. Planted: `Roster::older` answering false.
#[test]
fn an_older_svrngs_group_stands_nobody_and_refuses_a_write_and_a_join() {
    use crate::tests_support::{after, invite_of, key, p, record, signed, Fixture, Memory, NS};
    let alex = key(1);
    let members = [(p("alex"), alloc::vec![actor_of(&alex)])].into();
    let seed = Roster {
        members,
        policy: None,
    };
    let wrote = signed(&alex, 100, 0, record("milk"));
    let link = after(&alex, &wrote, 101, invite_of(&key(9), 10_000, None));
    let store = Memory::holding(&[wrote, link.clone()]);
    let a = admit(&store.0.borrow(), &[], &seed, NS, &Ed25519Verifier);
    let words = "This group was made by an older svrngs, so nothing in it changes now. To carry on, make it again and invite people by link.";
    assert!(
        a.membership().standing.is_empty(),
        "nobody stands: {:?}",
        a.membership()
    );
    assert!(a.ops().is_empty(), "no act counts");
    assert_eq!(a.gaps(), [RailGap::Older]);
    assert_eq!(a.gaps()[0].to_string(), words);
    assert_eq!(RailError::Older.to_string(), words);
    assert_eq!(
        made_older("Pier"),
        "Pier was made by an older svrngs, so nothing in it changes now. To carry on, make it again and invite people by link."
    );
    let author = Author {
        signer: &alex,
        roster: &seed,
        namespace: NS,
        now: 200,
    };
    let Ok(write) = commit(
        &store,
        &Fixture {
            act: record("bread"),
        },
        &author,
    );
    assert_eq!(write.err(), Some(Refused::Older), "a write");
    let joining = key(7);
    let consent = Redemption::sign(&joining, NS, &link.id, &p("bo"), Device::Computer);
    let redeem = Redeem {
        invite: link.id.clone(),
        person: p("bo"),
        key: actor_of(&joining),
        cosig: consent.sig,
        device: Device::Computer,
        link: actor_of(&key(9)),
        issuer: actor_of(&alex),
    };
    let Ok(join) = commit(&store, &redeem, &author);
    assert_eq!(join.err(), Some(Refused::Older), "a join");
    assert_eq!(store.0.borrow().len(), 2, "nothing written");
}
