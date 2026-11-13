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

/// `unshare` of a name nothing is lent by says what is lent, or nothing
/// more when nothing is.
#[test]
fn unshare_of_nothing_lent_says_what_is() {
    let lend = |n: &str| svrngs_node::lend::LendName::parse(n).unwrap();
    let said = |lent: Vec<_>| {
        Refusal::NotLent {
            name: lend("pad"),
            lent,
        }
        .to_string()
    };
    assert_eq!(
        said(vec![]),
        "you lend nothing called pad from this computer"
    );
    assert_eq!(
        said(vec![lend("films"), lend("tea")]),
        "you lend nothing called pad from this computer; you lend films, tea"
    );
}

/// Sam's computer on `wire` holding Lab, its svrngs running: the data
/// dir's lock held and its API's address written, as `svrngs` keeps them.
async fn sams_lab(wire: &Wire) -> (crate::Computer, std::fs::File) {
    let sam = computer(wire, 5).await;
    let relay = no_relay();
    let made = cli::new_group(&sam.node, name("Lab"), name("Sam"), None, &[], &relay);
    made.await.expect("Sam makes Lab");
    let held = cli::claim(sam.dir.path()).expect("svrngs holds the data dir");
    let api = format!("http://{}", sam.node.api_addr());
    std::fs::write(sam.dir.path().join(svrngs_node::api::API_FILE), api).unwrap();
    (sam, held)
}

/// Sam at the terminal as the command reaches it ([`crate::Running`]):
/// `typed` at every prompt, and each prompt's words kept.
struct Lender<'a> {
    running: crate::Running<'a>,
    typed: &'a str,
    asked: std::cell::RefCell<Vec<String>>,
}

impl<'a> Lender<'a> {
    fn of(sam: &'a crate::Computer, typed: &'a str) -> Self {
        let (running, asked) = (crate::Running(sam), Default::default());
        Self {
            running,
            typed,
            asked,
        }
    }
}

impl cli::Here for Lender<'_> {
    fn root(&self) -> &std::path::Path {
        self.running.root()
    }
    async fn ask(&self, route: &str, body: &impl serde::Serialize) -> cli::Asked {
        self.running.ask(route, body).await
    }
    fn listens(&self, port: NonZeroU16) -> bool {
        self.running.listens(port)
    }
    fn prompt(&self, _: &svrngs_node::lend::LendName, asked: &str, _: bool) -> cli::Prompted {
        self.asked.borrow_mut().push(asked.to_string());
        Ok(Some(self.typed.to_string()))
    }
}

/// `svrngs share <args>` as Sam types it, asked of `lender`: what it says.
async fn shares(lender: &Lender<'_>, args: &[&str]) -> Result<String, Refusal> {
    let argv = ["svrngs", "share"]
        .iter()
        .chain(args)
        .map(|s| s.to_string());
    let verb = cli::parse(argv).expect("share parses");
    cli::answer(verb, lender, &no_relay()).await
}

/// Every file under `dir` but `except`, by path, holding `needle`.
fn holding(dir: &std::path::Path, needle: &[u8], except: &std::path::Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path != except {
                let bytes = std::fs::read(&path).unwrap_or_default();
                if bytes.windows(needle.len()).any(|w| w == needle) {
                    found.push(path.display().to_string());
                }
            }
        }
    }
    found
}

/// Nothing a person reads says a word DESIGN §1 keeps from them.
fn plain(said: &str) {
    for line in said.lines() {
        let lower = line.to_lowercase();
        for word in svrngs_node::link::KEPT_WORDS {
            assert!(!lower.contains(word), "{line:?} says {word}");
        }
    }
}

/// The trace `heard` kept, as text.
fn traced(said: &crate::node_common::Said) -> String {
    String::from_utf8(said.0.lock().unwrap().clone()).unwrap()
}

/// **`share --sign-in-with-svrngs` turns it on for scripts** (SIGN_IN_WITH_
/// SVRNGS §5 "Turning it on", §8 9): the command makes a client id and
/// secret from the allowed alphabet, keeps them in the lend's declaration
/// at 0600, prints the issuer, the client and the deviations once, and
/// asks no password; the secret is in no other file and no trace. Asked
/// through `cli::Here` with svrngs running, no port probed and nothing at
/// the terminal. Watched failing on two plants: the client never put in
/// the lend (cli.rs `answer`'s Share arm), and the setup words claiming
/// credentials completed app setup (cli.rs `Said`).
#[cfg(unix)]
#[tokio::test]
async fn share_turns_on_signing_people_in_with_svrngs() {
    use std::os::unix::fs::PermissionsExt;
    let (heard, _heard) = crate::node_common::heard("cli=debug,share=debug");
    let wire = Wire::default();
    let (sam, _held) = sams_lab(&wire).await;
    let lender = Lender::of(&sam, SECRET);
    let args = [
        "9001",
        "--kind",
        "pad",
        "--sign-in-with-svrngs",
        "--group",
        "Lab",
    ];
    let seen = shares(&lender, &args).await.expect("pad is lent");

    let name = svrngs_node::lend::LendName::parse("pad").unwrap();
    let declared = svrngs_node::lend::read(sam.dir.path(), &name)
        .unwrap()
        .unwrap();
    let client = declared
        .sign_in_with_svrngs
        .expect("the declaration holds the client");
    let id = client.client_id;
    let secret = client.client_secret.expect("a secret");
    for made in [&id, &secret] {
        assert!(
            made.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-._~".contains(&b)),
            "{made:?} is made from the allowed alphabet"
        );
    }
    let wrote = seen
        .lines()
        .find(|l| l.contains("issuer http://127.0.0.1:") && l.contains("/oidc/pad"))
        .unwrap_or_else(|| panic!("the issuer is not printed: {seen}"));
    assert!(
        wrote.contains("Sign-in credentials created; app setup still required.")
            && wrote.contains("install any required OpenID Connect plugin")
            && wrote.contains("from the app server on this computer"),
        "credential creation must explain the remaining app setup"
    );
    assert!(!wrote.contains("It signs people in with svrngs."));
    assert!(wrote.contains(&id) && wrote.contains(&secret), "{wrote}");
    assert!(
        seen.lines()
            .any(|l| l.contains("plain http") && l.contains("ES256") && l.contains("no email")),
        "the deviations are in the lender's words: {seen}"
    );
    assert!(!seen.contains("Its password stays"), "{seen}");
    assert_eq!(lender.asked.borrow().len(), 0, "no password was asked");
    let file = sam.dir.path().join("lends/pad.json");
    let mode = std::fs::metadata(&file).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "the declaration holding a secret");
    assert_eq!(
        holding(sam.dir.path(), secret.as_bytes(), &file),
        Vec::<String>::new(),
        "the secret is in no file but its declaration"
    );
    assert!(
        !traced(&heard).contains(&secret),
        "the secret is in no trace"
    );
}

/// A service that keeps its own accounts (u5-4; DESIGN §3, "Lent apps",
/// call 4): `share` asks no password, keeps none though one would be
/// typed, and tells its lender how to make each person an account there,
/// in DESIGN §1's words. Watched failing on two plants: its accounts
/// unread, so a password is asked (cli.rs `answer`'s Share arm), and the
/// accounts' words changed (cli.rs `Said`).
#[tokio::test]
async fn a_service_with_its_own_accounts_is_lent_with_no_password() {
    let (heard, _heard) = crate::node_common::heard("cli=debug");
    let wire = Wire::default();
    let (sam, _held) = sams_lab(&wire).await;
    let lender = Lender::of(&sam, SECRET);
    let args = ["9002", "--kind", "jellyfin", "--group", "Lab"];
    let seen = shares(&lender, &args).await.expect("jellyfin is lent");

    let how = seen
        .lines()
        .find(|l| l.starts_with("jellyfin asks each person to sign in once, with an account you make them. In jellyfin: "))
        .unwrap_or_else(|| panic!("the lender is not told how to make accounts: {seen}"));
    assert!(
        how.contains("Dashboard, Users and +") && how.ends_with("Then tell each person theirs."),
        "{how}"
    );
    assert!(!seen.contains("Its password stays"), "{seen}");
    let dir = sam.dir.path();
    assert!(
        dir.join("lends/jellyfin.json").exists(),
        "jellyfin is declared"
    );
    assert!(
        !dir.join("secrets/lends/jellyfin").exists(),
        "a password was kept for jellyfin"
    );
    assert_eq!(
        holding(dir, b"sams-own", std::path::Path::new("")),
        Vec::<String>::new()
    );
    assert_eq!(lender.asked.borrow().len(), 0, "a password was asked");
    assert!(
        traced(&heard).contains("cli: share — no password asked"),
        "the branch is traced"
    );
    plain(&seen);
}

/// **Navidrome by its port alone** (bash-person (d); Demo C's dry run,
/// finding 2): `share 4533`, said to listen, lends it as navidrome, asks
/// no password and keeps none though one would be typed, and prints both
/// settings its lender sets in it so it takes each person from the header
/// svrngs sets. Nothing binds 4533: the probe is `cli::Here`'s. Watched
/// failing on two plants: the well-known port's kind dropped (cli.rs
/// `answer`'s Share arm, refused "say what is on port 4533"), and its
/// accounts unread, so a password is asked (the same arm).
#[tokio::test]
async fn share_4533_alone_lends_navidrome_asking_nothing_and_says_what_to_set() {
    let wire = Wire::default();
    let (sam, _held) = sams_lab(&wire).await;
    let lender = Lender::of(&sam, SECRET);
    let seen = shares(&lender, &["4533", "--group", "Lab"]).await;
    let seen = seen.expect("navidrome is lent");

    assert!(
        seen.lines()
            .any(|l| l.starts_with("You lend navidrome (port 4533)")),
        "{seen}"
    );
    let set = seen
        .lines()
        .find(|l| l.contains("ND_EXTAUTH_USERHEADER=X-Svrngs-User"))
        .unwrap_or_else(|| panic!("the lender is not told what to set: {seen}"));
    assert!(
        set.contains("ND_EXTAUTH_TRUSTEDSOURCES=127.0.0.1/32"),
        "{set}"
    );
    assert!(!seen.contains("Its password stays"), "{seen}");
    let dir = sam.dir.path();
    assert!(dir.join("lends/navidrome.json").exists());
    assert!(!dir.join("secrets/lends/navidrome").exists());
    assert_eq!(lender.asked.borrow().len(), 0, "a password was asked");
    plain(&seen);
}
