use ed25519_dalek::{Signer, SigningKey};
use serde_json::json;

use crate::*;

const NOW: u64 = 1_790_000_000;

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

fn actor(k: &SigningKey) -> Actor {
    Actor::from_key(&k.verifying_key())
}

fn sign(k: &SigningKey, d: Draft) -> VerifiedAct {
    let sig = Sig::from_bytes(k.sign(d.id().as_bytes()).to_bytes());
    d.seal(sig).unwrap()
}

/// A group whose starter (key 1) admitted a Bridge key (key 2).
fn group() -> (Journal, SigningKey, SigningKey) {
    let (person, bridge) = (key(1), key(2));
    let genesis = Genesis {
        key: actor(&person),
        nonce: Nonce::from_bytes([7; 32]),
        name: Name::try_from("the house").unwrap(),
    };
    let mut j = Journal::new(genesis);
    let name = Name::try_from("the chat").unwrap();
    let msg = admit_message(&actor(&person), &actor(&bridge), Role::Bridge, &name);
    let cosig = Sig::from_bytes(bridge.sign(&msg).to_bytes());
    let admit = Body::Admit(Admit {
        key: actor(&bridge),
        role: Role::Bridge,
        name,
        cosig,
    });
    let act = sign(&person, j.draft(&actor(&person), &admit, NOW).unwrap());
    hold(&mut j, act);
    (j, person, bridge)
}

fn hold(j: &mut Journal, act: VerifiedAct) {
    match j.decide(act, NOW) {
        Ingest::Held(h) => j.apply(h),
        other => panic!("not held: {other:?}"),
    }
}

fn record(op: serde_json::Value, interaction: Option<&str>) -> Body {
    Body::Record(Record {
        app: AppId::try_from("expenses").unwrap(),
        op: Payload::new(op).unwrap(),
        on_behalf_of: interaction.map(|i| OnBehalfOf {
            speaker: Speaker::try_from("discord:1").unwrap(),
            interaction: Interaction::try_from(i).unwrap(),
        }),
    })
}

fn write(j: &mut Journal, k: &SigningKey, body: &Body, now: u64) -> VerifiedAct {
    let act = sign(k, j.draft(&actor(k), body, now).unwrap());
    hold(j, act.clone());
    act
}

#[test]
fn a_line_round_trips_and_its_id_derives_from_it() {
    let (mut j, _, bridge) = group();
    let act = write(&mut j, &bridge, &record(json!({"b": 1, "a": 2}), Some("i1")), NOW);
    assert_eq!(VerifiedAct::parse(&act.to_line()).unwrap(), act);
}

#[test]
fn a_tampered_line_is_refused() {
    let (mut j, _, bridge) = group();
    let line = write(&mut j, &bridge, &record(json!({"cents": 100}), Some("i1")), NOW).to_line();
    let changed = line.replace("\"cents\":100", "\"cents\":900");
    assert!(matches!(
        VerifiedAct::parse(&changed),
        Err(ParseError::IdMismatch { .. })
    ));
    let mut v: serde_json::Value = serde_json::from_str(&line).unwrap();
    let sig = v["sig"].as_str().unwrap().replacen('0', "1", 1).replacen('a', "b", 1);
    v["sig"] = json!(sig);
    assert_eq!(VerifiedAct::parse(&v.to_string()), Err(ParseError::BadSignature));
    let unknown = line.replacen("{", "{\"extra\":1,", 1);
    assert!(matches!(VerifiedAct::parse(&unknown), Err(ParseError::Malformed(_))));
}

#[test]
fn an_admit_its_key_did_not_cosign_is_refused() {
    let (person, bridge) = (key(1), key(2));
    let j = Journal::new(Genesis {
        key: actor(&person),
        nonce: Nonce::from_bytes([7; 32]),
        name: Name::try_from("g").unwrap(),
    });
    let name = Name::try_from("the chat").unwrap();
    // Co-signed as a Person, presented as a Bridge.
    let msg = admit_message(&actor(&person), &actor(&bridge), Role::Person, &name);
    let cosig = Sig::from_bytes(bridge.sign(&msg).to_bytes());
    let admit = Body::Admit(Admit {
        key: actor(&bridge),
        role: Role::Bridge,
        name,
        cosig,
    });
    let d = j.draft(&actor(&person), &admit, NOW).unwrap();
    let sig = Sig::from_bytes(person.sign(d.id().as_bytes()).to_bytes());
    assert_eq!(d.seal(sig), Err(ParseError::BadCosignature));
}

#[test]
fn ingest_offers_again_what_it_cannot_hold_yet() {
    let (mut j, _, bridge) = group();
    let stranger = key(9);
    let act = sign(
        &stranger,
        j.draft(&actor(&stranger), &record(json!({}), None), NOW).unwrap(),
    );
    assert!(matches!(j.decide(act, NOW), Ingest::OfferAgain(Why::Unbound(_))));

    let first = sign(
        &bridge,
        j.draft(&actor(&bridge), &record(json!({"n": 1}), None), NOW).unwrap(),
    );
    let mut ahead = j.clone();
    hold(&mut ahead, first.clone());
    let second = sign(
        &bridge,
        ahead
            .draft(&actor(&bridge), &record(json!({"n": 2}), None), NOW)
            .unwrap(),
    );
    assert!(matches!(
        j.decide(second.clone(), NOW),
        Ingest::OfferAgain(Why::PrevNotHeld(_))
    ));
    hold(&mut j, first);
    hold(&mut j, second.clone());
    assert!(matches!(j.decide(second, NOW), Ingest::AlreadyHeld(_)));

    let late = sign(
        &bridge,
        j.draft(&actor(&bridge), &record(json!({}), None), NOW + MAX_SKEW_SECS + 1)
            .unwrap(),
    );
    assert!(matches!(
        j.decide(late, NOW),
        Ingest::OfferAgain(Why::FromTheFuture { .. })
    ));
}

#[test]
fn a_key_never_stamps_before_its_admission() {
    let (j, _, bridge) = group();
    let d = j.draft(&actor(&bridge), &record(json!({}), None), NOW - 1000).unwrap();
    let act = sign(&bridge, d);
    assert_eq!(act.ts(), NOW);
    assert!(matches!(j.decide(act, NOW), Ingest::Held(_)));
}

#[test]
fn the_interaction_a_bridge_act_answers_is_taken() {
    let (mut j, person, bridge) = group();
    let interaction = Interaction::try_from("i1").unwrap();
    assert_eq!(j.taken(&interaction), None);
    // A Person key speaking "for" someone takes nothing: on_behalf_of is the Bridge's.
    write(&mut j, &person, &record(json!({}), Some("i1")), NOW);
    assert_eq!(j.taken(&interaction), None);
    let act = write(&mut j, &bridge, &record(json!({}), Some("i1")), NOW);
    assert_eq!(j.taken(&interaction), Some(act.id()));
}

#[test]
fn fold_orders_by_ts_actor_seq_id_and_voids_by_correct() {
    let (mut j, person, bridge) = group();
    let b1 = write(&mut j, &bridge, &record(json!({"n": 1}), Some("i1")), NOW + 5);
    let p1 = write(&mut j, &person, &record(json!({"n": 2}), None), NOW + 1);
    let fix = Body::Correct(Correct {
        app: AppId::try_from("expenses").unwrap(),
        corrects: *b1.id(),
        replacement: None,
        on_behalf_of: None,
    });
    let c = write(&mut j, &person, &fix, NOW + 6);
    let admit_id = *fold(&j)[0].act.id();
    let other_app = Body::Correct(Correct {
        app: AppId::try_from("tap").unwrap(),
        corrects: *p1.id(),
        replacement: None,
        on_behalf_of: None,
    });
    write(&mut j, &person, &other_app, NOW + 7);
    let membership = Body::Correct(Correct {
        app: AppId::try_from("expenses").unwrap(),
        corrects: admit_id,
        replacement: None,
        on_behalf_of: None,
    });
    write(&mut j, &person, &membership, NOW + 8);

    let f = fold(&j);
    let ids: Vec<&ActId> = f.iter().map(|x| x.act.id()).collect();
    assert_eq!(&ids[..4], &[&admit_id, p1.id(), b1.id(), c.id()]);
    let voided: Vec<&ActId> = f.iter().filter(|x| x.voided).map(|x| x.act.id()).collect();
    assert_eq!(
        voided,
        vec![b1.id()],
        "only a Correct of the same app voids, and never a membership act"
    );
    assert!(f[2].speaker.is_some() && f[1].speaker.is_none());
}

#[test]
fn a_person_key_cannot_speak_for_the_chat() {
    let (mut j, person, _) = group();
    write(&mut j, &person, &record(json!({}), Some("i9")), NOW);
    assert!(fold(&j).iter().all(|f| f.speaker.is_none()));
}

#[test]
fn payload_is_canonical_and_whole() {
    let a = Payload::new(json!({"b": {"y": 1, "x": 2}, "a": [3, 1]})).unwrap();
    assert_eq!(
        String::from_utf8(a.to_bytes()).unwrap(),
        r#"{"a":[3,1],"b":{"x":2,"y":1}}"#
    );
    assert!(matches!(
        Payload::new(json!({"a": [1, 2.5]})),
        Err(PayloadError::Fractional(_))
    ));
    assert_eq!(Payload::new(json!([1])), Err(PayloadError::NotAnObject));
}

fn guest(speaker: &str, name: &str, event: GuestEvent) -> Guest {
    Guest {
        speaker: Speaker::try_from(speaker).unwrap(),
        name: Name::try_from(name).unwrap(),
        event,
    }
}

#[test]
fn a_removal_sticks_until_a_join_lifts_it() {
    let (mut j, _, bridge) = group();
    let ama = Speaker::try_from("discord:1").unwrap();
    assert!(
        j.guests().may_speak(&ama),
        "someone never seen may act: acting is how they arrive"
    );
    let seen = guest("discord:1", "Ama", GuestEvent::Seen);
    assert_eq!(j.guests().change(&seen), GuestChange::Arrives);
    write(&mut j, &bridge, &Body::Guest(seen.clone()), NOW);
    assert_eq!(j.guests().change(&seen), GuestChange::Unchanged);

    let banned = guest("discord:1", "Ama", GuestEvent::Banned);
    assert_eq!(j.guests().change(&banned), GuestChange::Removes);
    write(&mut j, &bridge, &Body::Guest(banned.clone()), NOW + 1);
    assert!(!j.guests().may_speak(&ama));
    assert_eq!(j.guests().change(&banned), GuestChange::Unchanged);
    assert_eq!(j.guests().change(&seen), GuestChange::RemovalSticks);
    // A Seen written anyway — a second bridge racing this one — lifts nothing.
    write(&mut j, &bridge, &Body::Guest(seen.clone()), NOW + 2);
    assert!(!j.guests().may_speak(&ama));
    assert_eq!(j.guests().present().count(), 0);

    let joined = guest("discord:1", "Ama", GuestEvent::Joined);
    assert_eq!(j.guests().change(&joined), GuestChange::Arrives);
    write(&mut j, &bridge, &Body::Guest(joined), NOW + 3);
    assert!(j.guests().may_speak(&ama));
    assert_eq!(
        j.guests().change(&guest("discord:1", "Ama B", GuestEvent::Seen)),
        GuestChange::Renamed
    );
}

#[test]
fn only_a_bridge_key_writes_the_guest_book() {
    let (mut j, person, _) = group();
    let ama = Speaker::try_from("discord:1").unwrap();
    write(
        &mut j,
        &person,
        &Body::Guest(guest("discord:1", "Ama", GuestEvent::Banned)),
        NOW,
    );
    assert!(
        j.guests().may_speak(&ama),
        "a Person key's guest act is not the guest book's"
    );
    assert_eq!(j.guests().entry(&ama), None);
}
