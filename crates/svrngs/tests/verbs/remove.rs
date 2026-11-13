//! `svrngs remove` and `svrngs handover` on the wire: remove.rs's claims
//! whose decision is the running svrngs's removal router and the command's
//! words for its answer, asked by `cli::answer` of a node's routes in this
//! process (cli.rs `Here`, docs/seams.tsv). Nothing listens; what stays in
//! remove.rs is the verbs' parse and their words.

use svrngs::cli::{self, Refusal, RemoveKind, Verb};
use svrngs::group::Seed;
use svrngs_node::node::Wire;
use svrngs_node::{actor_of, SigningKey};

use crate::{computer, name, no_relay, Computer, Running};

/// Sam's computer on `wire`, and the group trip it made.
async fn sams_trip(wire: &Wire) -> (Computer, Seed) {
    let sam = computer(wire, 5).await;
    let relay = no_relay();
    let made = cli::new_group(&sam.node, name("trip"), name("Sam"), None, &[], &relay);
    let (seed, _) = made.await.unwrap();
    (sam, seed)
}

/// Bo let into trip as `bo` by a link Sam's computer mints, with no network.
fn let_bo_in(sam: &Computer, seed: &Seed, bo: &SigningKey) {
    let_in(sam, seed, bo, |redeem| Box::new(redeem.clone()));
}

/// As [`let_bo_in`], the `Admit` written as `admit` makes it of the asked
/// redemption: a second Bo's by hand, past the name a link refuses, as two
/// members' computers letting two Bos in at once would write.
fn let_in<C: svrngs_core::Command + 'static>(
    sam: &Computer,
    seed: &Seed,
    bo: &SigningKey,
    admit: impl FnOnce(&svrngs_core::Redeem) -> Box<C>,
) where
    C::Refusal: std::fmt::Debug,
{
    let (ns, rail) = (seed.id(), sam.node.rail());
    let keeper = actor_of(&sam.key);
    svrngs_core::tests_support::let_in(
        (&ns, &keeper),
        (bo, "Bo", svrngs_core::Device::Computer),
        |mint| {
            rail.commit(&ns, mint).unwrap().unwrap().lines()[0]
                .id
                .clone()
        },
        |redeem| drop(rail.commit(&ns, &*admit(redeem)).unwrap().unwrap()),
    );
}

fn argv(rest: &[&str]) -> Vec<String> {
    std::iter::once("svrngs".to_string())
        .chain(rest.iter().map(|s| s.to_string()))
        .collect()
}

/// **The command asks svrngs running here, and the fold decides** (U7
/// (e)). Sam makes trip and lets Bo in; Sam's `remove` of somebody not in
/// trip is refused by name, of Sam is refused as Sam decides, and of Bo is
/// the cut — Bo stands no more; with no svrngs running, the ask is a
/// precondition absent, exit 3. Watched failing on two plants: the removal
/// router's `Outcome::NotIn` answered as success (removal.rs), and `said`'s
/// `_ if ok` arm dropped so a cut is refused (cli.rs).
#[tokio::test]
async fn the_keepers_remove_is_the_cut() {
    let wire = Wire::default();
    let (sam, seed) = sams_trip(&wire).await;
    let relay = no_relay();
    let running = Running(&sam);
    let ask = |verb: Verb| cli::answer(verb, &running, &relay);
    let remove = |who: &str| Verb::Remove {
        kind: RemoveKind::Remove,
        person: name(who),
        group: "trip".into(),
        joined: None,
    };
    let bo = SigningKey::from_bytes(&[6; 32]);
    let_bo_in(&sam, &seed, &bo);
    let stands = || {
        let members = sam.node.rail().register(&seed.id()).unwrap().members;
        members.contains_key(&actor_of(&bo))
    };
    assert!(stands(), "control: Bo is in");

    let nobody = ask(remove("Dee")).await;
    assert!(
        matches!(&nobody, Err(Refusal::Removal(w)) if w == "Nobody called Dee is in trip."),
        "{nobody:?}"
    );
    let own = ask(remove("Sam")).await;
    assert!(
        matches!(&own, Err(Refusal::Removal(w)) if w.starts_with("Sam decides who is removed from trip")),
        "{own:?}"
    );
    assert!(ask(remove("Bo")).await.is_ok());
    assert!(!stands(), "Bo is out");

    // The product's own ask, over a data dir whose svrngs wrote no API:
    // nothing is dialled, and nothing answers.
    let away = cli::answer(remove("Bo"), &sam.dir.path().to_path_buf(), &relay).await;
    assert!(matches!(away, Err(Refusal::NotRunning)), "{away:?}");
    assert_eq!(away.unwrap_err().exit_code(), 3);
}

/// What a refusal's words say to add to the command, between backticks,
/// as a shell would split it: each `"…"` one argument.
fn ways(words: &str) -> Vec<Vec<String>> {
    (words.split('`').skip(1).step_by(2))
        .map(|way| {
            (way.split('"').enumerate())
                .flat_map(|(i, part)| match i % 2 {
                    0 => part.split_whitespace().map(str::to_string).collect(),
                    _ => vec![part.to_string()],
                })
                .collect()
        })
        .collect()
}

/// **The keeper removes one of two people of one name** (bash-same-name
/// (a)). Sam lets two people into trip as Bo, a second apart, each by
/// hand, as a member's hand or a race at two computers gives them. Sam's
/// `remove Bo` and `handover Bo` are refused — svrngs never guesses which
/// Bo — in words that name, between backticks, what to add to the command
/// to name one; following the first, one Bo is out and the other stands,
/// and following the second, the other is out. Watched failing on two
/// plants: the removal router answering two Bos as nobody of the name
/// (removal.rs `cut`), and `said`'s `_ if ok` arm dropped (cli.rs).
#[tokio::test]
async fn the_keeper_removes_one_of_two_people_of_one_name() {
    let wire = Wire::default();
    let (sam, seed) = sams_trip(&wire).await;
    let relay = no_relay();
    let running = Running(&sam);
    let ask = |verb: Verb| cli::answer(verb, &running, &relay);
    let bos = [6u8, 7].map(|b| SigningKey::from_bytes(&[b; 32]));
    let_bo_in(&sam, &seed, &bos[0]);
    tokio::time::sleep(std::time::Duration::from_millis(1_100)).await;
    let_in(&sam, &seed, &bos[1], |r| {
        let invite = svrngs_core::Redemption {
            id: r.invite.clone(),
            sig: r.cosig.clone(),
            device: r.device,
        };
        let (person, key) = (r.person.clone(), r.key.clone());
        let act = svrngs_core::RailAct::Admit {
            person,
            key,
            invite: Some(invite),
        };
        Box::new(svrngs_core::tests_support::Fixture { act })
    });
    let stand = || {
        let members = sam.node.rail().register(&seed.id()).unwrap().members;
        bos.each_ref().map(|bo| members.contains_key(&actor_of(bo)))
    };
    assert_eq!(stand(), [true, true], "control: both Bos are in");

    let verb = |v: &str, more: &[String]| {
        let mut args = argv(&[v, "Bo", "--group", "trip"]);
        args.extend(more.iter().cloned());
        cli::parse(args)
    };
    for v in ["handover", "remove"] {
        let refused = ask(verb(v, &[]).unwrap()).await;
        let Err(Refusal::Removal(words)) = &refused else {
            panic!("{v} Bo: {refused:?}")
        };
        eprintln!("{v} Bo: {words}");
        for kept in ["key", "member", "keeper", "ring", " act", "journal"] {
            assert!(
                !words.to_lowercase().contains(kept),
                "{v} says {kept}: {words}"
            );
        }
        assert_eq!(stand(), [true, true], "{v} Bo changed nothing");
        assert_eq!(
            ways(words).len(),
            2,
            "{v} Bo: the refusal names a way to name each Bo: {words}"
        );
    }
    let Err(Refusal::Removal(words)) = ask(verb("remove", &[]).unwrap()).await else {
        unreachable!("refused above")
    };
    let [first, second] = <[Vec<String>; 2]>::try_from(ways(&words)).unwrap();
    let out = ask(verb("remove", &first).unwrap()).await;
    assert!(out.is_ok(), "remove Bo {first:?}: {out:?}");
    assert_eq!(stand(), [false, true], "the first Bo out, the other stands");
    let out = ask(verb("remove", &second).unwrap()).await;
    assert!(out.is_ok(), "remove Bo {second:?}: {out:?}");
    assert_eq!(stand(), [false, false], "the second Bo out");
}
