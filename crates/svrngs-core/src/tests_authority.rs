//! Native source meaning is signature-covered, not an unsigned parser hint.
use crate::*;

#[test]
fn an_unadopted_legacy_epoch_cannot_authorize_source_bound_data() {
    let key = SigningKey::from_bytes(&[1; 32]);
    let actor = actor_of(&key);
    let line = sign_act(
        &key,
        "legacy-source",
        1,
        0,
        None,
        RailAct::Record {
            payload: Payload::new(serde_json::json!({"kind":"notes/item"})).unwrap(),
        },
        None,
        None,
        None,
    );
    let mut kind = line.kind;
    kind.act = RailAct::Accept {
        hand_on: crate::tests_support::stray_id(1),
    };
    let mut sourced = unsigned_act(actor.clone(), "legacy-source", 1, kind);
    sourced.kind.sig = sign_ring_op(&key, &sourced.id);
    let seed = Roster::new([(Person::parse("founder").unwrap(), alloc::vec![actor])].into());
    let result = admit(
        &ActSet::of([&sourced]),
        &[],
        &seed,
        "legacy-source",
        &Ed25519Verifier,
    );
    assert!(
        !result.membership().counts(&sourced.id),
        "schema support alone cannot select new authority"
    );
    assert!(result.gaps().iter().any(|gap| matches!(gap,
        RailGap::AuthorityUnavailable { id } if id == &sourced.id
    )));
    assert!(
        !result.is_complete(),
        "uninterpreted source meaning is reported, never silently dropped"
    );
}

#[test]
fn a_new_group_commits_its_source_policy_in_its_actual_seed_identity() {
    let key = SigningKey::from_bytes(&[1; 32]);
    let legacy = seed::Seed::from_parts(
        Name::parse("house").unwrap(),
        Name::parse("ana").unwrap(),
        key.verifying_key(),
        [7; 16],
    );
    let original = legacy.to_text();
    let mut file: serde_json::Value = serde_json::from_str(&original).unwrap();
    file["policy"] = serde_json::json!("sources_v1");
    let source_text = serde_json::to_string(&file).unwrap();
    let source = seed::Seed::from_text(&source_text)
        .expect("an explicit source-policy seed is a supported new group, never a legacy upgrade");
    assert_eq!(legacy.clone().with_source_policy(), source);
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
