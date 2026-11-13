//! `svrngs share` and `unshare` on the wire: share.rs's claims whose
//! decision is `cli::audience_of`'s, `cli::share_lend`'s or `cli::unshare`'s
//! over groups `cli::new_group` made (docs/seams.tsv). The port a lend
//! names is said to listen by the probe `share::lend_of` is handed, so no
//! service answers on it; that a port is probed is share.rs's.

use std::num::NonZeroU16;

use svrngs::cli::{self, Refusal, Said};
use svrngs::share::{self, Share};
use svrngs_node::lend::Program as LentProgram;
use svrngs_node::node::Wire;

use crate::{computer, name, no_relay};

/// Sam's password for Sam's own service: Jellyfin 12's form.
const SECRET: &str = r#"MediaBrowser Token="sams-own-1234""#;

/// The port Sam's pad is lent at, nothing listening there.
const PAD: u16 = 8123;

/// The probe `share` asks whether a port listens: here, it does.
fn listens(_: NonZeroU16) -> bool {
    true
}

/// What `svrngs share <port> --kind <kind>` asks, nothing else said.
fn share_of(port: u16, kind: &str) -> Share {
    Share {
        port: NonZeroU16::new(port).unwrap(),
        name: None,
        kind: Some(LentProgram::parse(kind).unwrap()),
        limit: None,
        refused: Vec::new(),
        upload: None,
        reads: Vec::new(),
        asks: None,
        sign_in: false,
        readers: false,
        raw: None,
        audience: svrngs_node::lend::Audience::Legacy,
    }
}

/// Who can open a lend is said, never assumed (LENDING_SCOPE §"User
/// interface and commands"): `share` with no `--group` is refused with the
/// command that lends it; each `--group` is a group held here, by name, kept
/// by its id; `unshare --group` takes only that group from the audience, its
/// secret kept, and bare `unshare` still withdraws it. Planted: no
/// `--group` read as the legacy lend, paused, the checkpoint's parse.
#[tokio::test(flavor = "multi_thread")]
async fn a_share_names_who_can_open_it_and_unshare_takes_one_group_away() {
    use svrngs_node::lend::{self, Audience};
    let wire = Wire::default();
    let sam = computer(&wire, 1).await;
    let (mut seeds, relay) = (Vec::new(), no_relay());
    for group in ["house", "trip"] {
        let made = cli::new_group(&sam.node, name(group), name("Sam"), None, &[], &relay);
        seeds.push(made.await.expect("Sam makes the group").0.id());
    }
    let (dir, port) = (sam.dir.path(), std::num::NonZeroU16::new(8096).unwrap());
    let none = cli::audience_of(dir, port, &[]).unwrap_err();
    let next = "say which groups can open it: `svrngs share 8096 --group <group>`, once for each";
    assert_eq!((none.to_string().as_str(), none.exit_code()), (next, 1));
    let named = |g: &[&str]| g.iter().map(|g| g.to_string()).collect::<Vec<_>>();
    let both = cli::audience_of(dir, port, &named(&["house", "trip"])).unwrap();
    assert_eq!(both, Audience::Selected(seeds.clone()));
    let yard = cli::audience_of(dir, port, &named(&["yard"]));
    assert!(matches!(yard, Err(Refusal::NoGroup(_))), "{yard:?}");

    let asked = Share {
        audience: both,
        ..share_of(PAD, "pad")
    };
    let pad = share::lend_of(&asked, listens, None, "").unwrap();
    cli::share_lend(dir, &pad, Some(SECRET)).unwrap();
    let narrowed = cli::unshare(dir, &pad.name, &named(&["trip"])).unwrap();
    assert_eq!(narrowed, Said::Narrowed(pad.name.clone()));
    let kept = lend::read(dir, &pad.name).unwrap().unwrap();
    assert_eq!(kept.audience, Audience::Selected(vec![seeds[0].clone()]));
    let secret = svrngs_node::declared::read_declared_in(&lend::secret_dir(dir, &pad.name));
    assert!(secret.iter().any(|(_, v)| v == SECRET), "{secret:?}");
    let gone = cli::unshare(dir, &pad.name, &[]).unwrap();
    assert_eq!(gone, Said::Unshared(pad.name.clone()));
}

/// Two groups of one name never share a grant (LENDING_SCOPE §"Policy and
/// representation": a duplicate display name must not retarget it): Sam
/// holds two called House. `--group House` is refused as ambiguous, naming
/// both ids' starts; the start of one's id chooses that one alone, and the
/// declaration keeps its id, which a rename or a third House never moves.
/// Planted: a name read as the first group that has it.
#[tokio::test(flavor = "multi_thread")]
async fn two_groups_of_one_name_are_chosen_by_id_never_by_name() {
    use svrngs_node::lend::Audience;
    let wire = Wire::default();
    let sam = computer(&wire, 1).await;
    let (mut ids, relay) = (Vec::new(), no_relay());
    for _ in 0..2 {
        let made = cli::new_group(&sam.node, name("House"), name("Sam"), None, &[], &relay);
        ids.push(made.await.expect("Sam makes a House").0.id());
    }
    let (dir, port) = (sam.dir.path(), std::num::NonZeroU16::new(8096).unwrap());
    let both = cli::audience_of(dir, port, &["House".to_string()]);
    assert!(
        matches!(&both, Err(Refusal::Ambiguous { ids: i, .. }) if i.len() == 2),
        "{both:?}"
    );
    let second = cli::audience_of(dir, port, &[ids[1][..8].to_string()]).unwrap();
    assert_eq!(second, Audience::Selected(vec![ids[1].clone()]));
}

/// `unshare --group` on a legacy lend, which a start now lends to its
/// computer's groups, so only one made since: that group cannot open it, and
/// the words say that, not that a change was made (audit F12). It still
/// needs selection.
#[tokio::test(flavor = "multi_thread")]
async fn unshare_a_group_from_a_legacy_lend_leaves_it_lent_to_none() {
    use svrngs_node::lend::{self, Audience};
    let wire = Wire::default();
    let sam = computer(&wire, 1).await;
    let relay = no_relay();
    let made = cli::new_group(&sam.node, name("house"), name("Sam"), None, &[], &relay);
    made.await.expect("Sam makes house");
    let pad = share::lend_of(&share_of(PAD, "pad"), listens, None, "").unwrap();
    cli::share_lend(sam.dir.path(), &pad, None).unwrap();
    let said = cli::unshare(sam.dir.path(), &pad.name, &["house".to_string()]);
    said.expect("true already: a legacy lend reaches no group");
    let kept = lend::read(sam.dir.path(), &pad.name).unwrap().unwrap();
    assert_eq!(kept.audience, Audience::Legacy);
}

/// **A folder is synced by asking svrngs running here, in one group**
/// (gf-1, sy-1, syp-4): `share --folder <dir> --group <group>` takes the
/// folder's absolute path and the group as typed, and only `share` takes
/// `--folder`, never without `--group` (a folder is one group's); asked
/// through `cli::Here` of the running node's routes, a group it does not
/// hold is refused by name and one it holds is answered; with no svrngs
/// running here (the product's `Here` over a data dir that wrote no API)
/// the ask is refused as a precondition absent, never answered empty.
/// Watched failing on two plants: the folder route's answer read as
/// refused whatever it said (cli.rs `folder`), and the route answering a
/// held group 409 (folder.rs `ask`).
#[tokio::test]
async fn a_folder_is_synced_by_asking_svrngs_running_here() {
    use svrngs::cli::Verb;
    let argv = |rest: &[&str]| {
        std::iter::once("svrngs".to_string())
            .chain(rest.iter().map(|s| s.to_string()))
            .collect::<Vec<_>>()
    };
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(
        cli::parse(argv(&["share", "--folder", "films", "--group", "house"])),
        Ok(Verb::Folder {
            dir: cwd.join("films"),
            group: "house".into()
        })
    );
    assert!(cli::parse(argv(&["share", "--folder", "films"])).is_err());
    assert!(cli::parse(argv(&["share", "--folder", "films", "--both-ways"])).is_err());
    assert!(cli::parse(argv(&["share", "--group", "house"])).is_err());
    assert!(cli::parse(argv(&["unshare", "--folder", "films"])).is_err());
    assert!(cli::parse(argv(&["link", "x", "--folder", "films"])).is_err());

    let wire = Wire::default();
    let sam = computer(&wire, 5).await;
    let relay = no_relay();
    let films = tempfile::tempdir().unwrap();
    let folder = |group: &str| Verb::Folder {
        dir: films.path().to_path_buf(),
        group: group.into(),
    };
    cli::new_group(&sam.node, name("house"), name("Sam"), None, &[], &relay)
        .await
        .unwrap();
    let running = crate::Running(&sam);
    let nope = cli::answer(folder("nope"), &running, &relay).await;
    assert!(
        matches!(&nope, Err(Refusal::NoGroup(g)) if g == "nope"),
        "{nope:?}"
    );
    let synced = cli::answer(folder("house"), &running, &relay).await;
    assert!(synced.is_ok(), "{synced:?}");

    let away = sam.dir.path().to_path_buf();
    let refused = cli::answer(folder("house"), &away, &relay).await;
    assert!(matches!(refused, Err(Refusal::NotRunning)), "{refused:?}");
    assert_eq!(refused.unwrap_err().exit_code(), 3);
}
