//! `svrngs share` and `unshare` on the wire: share.rs's claims whose
//! decision is `cli::audience_of`'s, `cli::share_lend`'s or `cli::unshare`'s
//! over groups `made::new_group` made (docs/seams.tsv). The port a lend
//! names is said to listen by the probe `share::lend_of` is handed, so no
//! service answers on it; that a port is probed is share.rs's.

use crate::made;
use std::num::NonZeroU16;

use svrngs::cli::{self, Refusal, Refused, Said, Unjudged};
use svrngs::share::{self, Share};
use svrngs_node::lend::Program as LentProgram;
use svrngs_node::node::Wire;

use crate::{computer, name, no_relay};

/// Sam's password for Sam's own service: Jellyfin 12's form.
const SECRET: &str = r#"MediaBrowser Token="sams-own-1234""#;

/// The port Sam's pad is lent at, nothing listening there.
const PAD: u16 = 8123;

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
        pid: None,
        replace: false,
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
    let mut seeds = Vec::new();
    for group in ["house", "trip"] {
        let made = made::new_group(&sam.node, name(group), name("Sam"), None, &[]);
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
    assert!(
        matches!(yard, Err(Refusal::Refused(Refused::NoGroup(_)))),
        "{yard:?}"
    );

    let asked = Share {
        audience: both,
        ..share_of(PAD, "pad")
    };
    let mut pad = share::lend_of(&asked, None, "").unwrap();
    svrngs_node::node::approve(&mut pad);
    cli::share_lend(dir, &pad, Some(SECRET)).unwrap();
    let narrowed = cli::unshare(dir, &pad.name, &named(&["trip"])).unwrap();
    assert_eq!(narrowed, Said::Narrowed(pad.name.clone()));
    let kept = lend::read(dir, &pad.name).unwrap().unwrap();
    assert_eq!(
        kept.terms.audience,
        Audience::Selected(vec![seeds[0].clone()])
    );
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
    let mut ids = Vec::new();
    for _ in 0..2 {
        let made = made::new_group(&sam.node, name("House"), name("Sam"), None, &[]);
        ids.push(made.await.expect("Sam makes a House").0.id());
    }
    let (dir, port) = (sam.dir.path(), std::num::NonZeroU16::new(8096).unwrap());
    let both = cli::audience_of(dir, port, &["House".to_string()]);
    assert!(
        matches!(&both, Err(Refusal::Refused(Refused::Ambiguous { ids: i, .. })) if i.len() == 2),
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
    let made = made::new_group(&sam.node, name("house"), name("Sam"), None, &[]);
    made.await.expect("Sam makes house");
    let mut pad = share::lend_of(&share_of(PAD, "pad"), None, "").unwrap();
    svrngs_node::node::approve(&mut pad);
    cli::share_lend(sam.dir.path(), &pad, None).unwrap();
    let said = cli::unshare(sam.dir.path(), &pad.name, &["house".to_string()]);
    said.expect("true already: a legacy lend reaches no group");
    let kept = lend::read(sam.dir.path(), &pad.name).unwrap().unwrap();
    assert_eq!(kept.terms.audience, Audience::Legacy);
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
    made::new_group(&sam.node, name("house"), name("Sam"), None, &[])
        .await
        .unwrap();
    let running = crate::Running(&sam);
    let nope = cli::answer(folder("nope"), &running, &relay).await;
    assert!(
        matches!(&nope, Err(Refusal::Refused(Refused::NoGroup(g))) if g == "nope"),
        "{nope:?}"
    );
    let synced = cli::answer(folder("house"), &running, &relay).await;
    assert!(synced.is_ok(), "{synced:?}");

    let away = sam.dir.path().to_path_buf();
    let refused = cli::answer(folder("house"), &away, &relay).await;
    assert!(
        matches!(refused, Err(Refusal::Unjudged(Unjudged::NotRunning))),
        "{refused:?}"
    );
    assert_eq!(refused.unwrap_err().exit_code(), 3);
}

/// `unshare` of a name nothing is lent by says what is lent, or nothing
/// more when nothing is.
#[test]
fn unshare_of_nothing_lent_says_what_is() {
    let lend = |n: &str| svrngs_node::lend::LendName::parse(n).unwrap();
    let said = |lent: Vec<_>| {
        Refusal::Refused(Refused::NotLent {
            name: lend("pad"),
            lent,
        })
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

/// Sam's computer on `wire` holding Lab, its folder placed, its svrngs
/// running: the data dir's lock held and its API's address written, as
/// `svrngs` keeps them.
async fn sams_lab(wire: &Wire) -> (crate::Computer, std::fs::File) {
    let sam = computer(wire, 5).await;
    let made = made::new_group(&sam.node, name("Lab"), name("Sam"), None, &[]);
    let (lab, _) = made.await.expect("Sam makes Lab");
    let (_, placed) = sam.node.folders().place(&lab);
    placed.expect("Lab's folder, placed as `svrngs new` places it");
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
    /// What answers on each port, where a test says; else as `running`'s.
    answers: Option<crate::Answered>,
    /// What answers once a prompt is asked, where a test says: what changed
    /// while its person read it.
    changed: Option<crate::Answered>,
    /// Whether a person is at the terminal ([`cli::Here::terminal`]).
    terminal: bool,
}

impl<'a> Lender<'a> {
    fn of(sam: &'a crate::Computer, typed: &'a str) -> Self {
        let (running, asked) = (crate::Running(sam), Default::default());
        Self {
            running,
            typed,
            asked,
            answers: None,
            changed: None,
            terminal: false,
        }
    }
}

impl cli::Here for Lender<'_> {
    fn root(&self) -> &std::path::Path {
        self.running.root()
    }
    async fn asked(&self, route: &str, body: Option<String>) -> cli::Answered {
        self.running.asked(route, body).await
    }
    fn answering(
        &self,
        raw: svrngs_node::lend::Raw,
        port: NonZeroU16,
        pid: Option<u32>,
    ) -> crate::Answered {
        let changed = self
            .changed
            .clone()
            .filter(|_| !self.asked.borrow().is_empty());
        (changed.or_else(|| self.answers.clone()))
            .unwrap_or_else(|| self.running.answering(raw, port, pid))
    }
    fn terminal(&self) -> bool {
        self.terminal
    }
    async fn bound(&self, lend: &svrngs_node::lend::Lend) -> crate::Checked {
        self.running.bound(lend).await
    }
    fn prompt(&self, _: &svrngs_node::lend::LendName, asked: &str, _: bool) -> cli::Prompted {
        self.asked.borrow_mut().push(asked.to_string());
        Ok(Some(self.typed.to_string()))
    }
    fn browse(&self, what: &std::ffi::OsStr) -> bool {
        self.running.browse(what)
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
        .terms
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

/// Until svrngs running on `sam`'s computer answers `group`'s
/// `programs/<word>.txt` with `text` in it, as its record now places it
/// (`/v1/programs`, the ask `share` makes); its answer.
async fn until_read(sam: &crate::Computer, group: &str, word: &str, text: &str) -> String {
    let asked = serde_json::json!([[group], word]);
    for _ in 0..600 {
        let (_, said) = crate::ask(
            &sam.node,
            "POST",
            svrngs_node::folder::PROGRAMS_ROUTE,
            &asked,
        )
        .await;
        if said.contains(text) {
            return said;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("{word}: never read with {text}");
}

/// **`svrngs share` keeps what a group's own `programs/<word>.txt` adds,
/// and says whose words they are** (CONTRACTS §2), svrngs running and
/// asked through `cli::Here`: Sam writes `programs/kokoro.txt` in Lab's
/// folder; once svrngs reads it through Lab's record, `share --program
/// kokoro` keeps back `/admin`, saying Lab's file keeps it, beside the
/// paths svrngs ships for kokoro; the lend's own file holds it. Lab's
/// `programs/tea.txt` says how tea's lender makes each person an account,
/// and `share` says it as Sam's words, in Lab's file. A file outside the
/// closed set is refused, its line and group named, and nothing lent; Lab
/// and Trip each with a different `programs/notes.txt` refuse a lend to
/// both, naming them. With svrngs not running, what `share` says keeps
/// only what ships, and says so. Watched failing on two plants: `terms_of`
/// never asked (cli.rs `answer`'s Share arm: no `/admin` line, the bad
/// file lent), and who wrote the file left out of what `share` says
/// (cli.rs `Said`).
#[tokio::test]
async fn share_keeps_what_a_group_s_programs_file_adds_and_says_so() {
    let wire = Wire::default();
    let (sam, _held) = sams_lab(&wire).await;
    let made = made::new_group(&sam.node, name("Trip"), name("Sam"), None, &[]);
    let (trip, _) = made.await.expect("Sam makes Trip");
    sam.node.folders().place(&trip).1.expect("Trip's folder");
    let lender = Lender::of(&sam, "");
    let ids: std::collections::BTreeMap<String, String> = svrngs::group::groups(sam.dir.path())
        .into_iter()
        .map(|s| (s.name().to_string(), s.id()))
        .collect();
    let folders = sam.dir.path().join("svrngs");
    let (lab, trip) = (folders.join("Lab/programs"), folders.join("Trip/programs"));
    std::fs::create_dir_all(&lab).unwrap();
    std::fs::create_dir_all(&trip).unwrap();

    std::fs::write(
        lab.join("kokoro.txt"),
        "# Lab's kokoro\n- kokoro keeps: /admin\n",
    )
    .unwrap();
    until_read(&sam, &ids["Lab"], "kokoro", "/admin").await;
    let said = shares(&lender, &["9011", "--program", "kokoro", "--group", "Lab"]).await;
    let said = said.expect("kokoro is lent");
    assert!(
        said.lines().any(
            |l| l == "They cannot open /admin: your groups' programs/kokoro.txt keeps it back."
        ),
        "{said}"
    );
    assert!(
        said.lines().any(
            |l| l.starts_with("They cannot open /docs, /openapi.json: svrngs keeps these back")
        ),
        "{said}"
    );
    let lent = std::fs::read_to_string(sam.dir.path().join("lends/kokoro.json")).unwrap();
    assert!(lent.contains("/admin") && lent.contains("/docs"), "{lent}");

    std::fs::write(
        lab.join("tea.txt"),
        "- tea accounts: Settings, People, Add for each person\n",
    )
    .unwrap();
    until_read(&sam, &ids["Lab"], "tea", "Add for each person").await;
    let said = shares(&lender, &["9014", "--program", "tea", "--group", "Lab"]).await;
    let said = said.expect("tea is lent");
    assert!(
        said.lines().any(|l| l == "tea asks each person to sign in once, with an account you make them. In tea, as Sam wrote in Lab's programs/tea.txt: Settings, People, Add for each person. Then tell each person theirs."),
        "{said}"
    );

    std::fs::write(lab.join("kokoro.txt"), "- kokoro opens: /admin\n").unwrap();
    until_read(&sam, &ids["Lab"], "kokoro", "is no term").await;
    let args = [
        "9012",
        "--program",
        "kokoro",
        "--name",
        "bad",
        "--group",
        "Lab",
    ];
    let refused = shares(&lender, &args)
        .await
        .expect_err("the bad file refuses it");
    assert_eq!(refused.exit_code(), 1);
    let told = refused.to_string();
    assert!(
        told.contains(
            "In Lab, programs/kokoro.txt can't be used: `kokoro opens: /admin` is no term"
        ),
        "{told}"
    );
    assert!(
        !sam.dir.path().join("lends/bad.json").exists(),
        "nothing lent"
    );

    std::fs::write(lab.join("notes.txt"), "- notes keeps: /lab\n").unwrap();
    std::fs::write(trip.join("notes.txt"), "- notes keeps: /trip\n").unwrap();
    until_read(&sam, &ids["Lab"], "notes", "/lab").await;
    until_read(&sam, &ids["Trip"], "notes", "/trip").await;
    let args = [
        "9013",
        "--program",
        "notes",
        "--group",
        "Lab",
        "--group",
        "Trip",
    ];
    let refused = shares(&lender, &args)
        .await
        .expect_err("two files refuse it");
    assert_eq!(refused.exit_code(), 1);
    let told = refused.to_string();
    assert!(
        told.contains("Lab and Trip say different things in programs/notes.txt"),
        "{told}"
    );
    assert!(
        !sam.dir.path().join("lends/notes.json").exists(),
        "nothing lent"
    );

    // svrngs not running: no terms read, and `share` says so before it runs.
    let pad = share_of(9010, "pad");
    let lend = share::lend_of(&pad, None, "").unwrap();
    let said = Said::Shared {
        lend: Box::new(lend),
        answers: Vec::new(),
        secret: false,
        accounts: None,
        client: None,
        issuer: None,
        terms: None,
    };
    assert!(
        said.to_string().lines().any(|l| l.starts_with(
            "svrngs wasn't running, so this keeps only the terms svrngs ships for pad."
        )),
        "{said}"
    );
}

/// What answers on a port as a test says it: socket `id`, held by process
/// `pid` started at `start`, shown as `program` in `~/dev/<program>`; `pid`'s
/// start now `now`.
fn answering(
    id: u64,
    (pid, start): (u32, u64),
    program: &str,
    now: Option<u64>,
) -> crate::Answered {
    let socket = svrngs_node::binding::Socket {
        id,
        cgroup: None,
        holders: [(pid, start)].into(),
        at: std::net::Ipv4Addr::LOCALHOST.into(),
        dual: false,
        peer: svrngs_node::binding::Peer::Unread,
    };
    let read = svrngs_node::binding::Read {
        boot: None,
        lasting: true,
        sockets: vec![socket],
        unlisted: false,
    };
    let shown = svrngs_node::lend::Answer::Program {
        pid,
        start,
        program: program.into(),
        folder: Some(format!("~/dev/{program}")),
    };
    Ok(svrngs_node::uid::Answering {
        read,
        parents: Default::default(),
        start: now,
        shown: vec![(id, shown)],
        beside: false,
    })
}

/// **A share is kept to what answers on its port, and says what that is**
/// (SHARE_BINDING §5.1): `share 4533` approves the socket read there and
/// says it is navidrome in its folder; read as another account's on a Mac,
/// none of this account's listed, it is refused, exit 1, in words that say
/// what stopped it and the fix, and nothing is lent; so is one that listens
/// only on a LAN address, and a UDP one on IPv6 alone (SHARE_BINDING_REVIEWS
/// §6 #2). The home page's Lend says the same words (svrngs-node
/// tests/home.rs `what_the_page_lends_is_what_it_showed`). Lent, it says
/// what svrngs checks and when it closes. Planted, each watched failing:
/// `share::approve` not asked (lent, nothing approved); the UDP check of
/// the address skipped (lent at ::1, which its flows can't reach).
#[tokio::test]
async fn a_share_is_kept_to_what_answers_on_its_port_and_says_what_that_is() {
    use svrngs_node::lend::{self, LendName};
    let wire = Wire::default();
    let (sam, _held) = sams_lab(&wire).await;
    let navidrome = LendName::parse("navidrome").unwrap();
    let mut theirs = Lender::of(&sam, SECRET);
    let mut unlisted = answering(1, (1, 1), "navidrome", None).unwrap();
    (unlisted.read.sockets, unlisted.read.unlisted) = (Vec::new(), true);
    theirs.answers = Some(Ok(unlisted));
    let refused = shares(&theirs, &["4533", "--group", "Lab"])
        .await
        .unwrap_err();
    let words = "svrngs can't share port 4533: the program there doesn't run as you, and on a Mac svrngs can keep a share only to programs that run as you. Run it as you, then share it again.";
    assert_eq!(
        (refused.to_string().as_str(), refused.exit_code()),
        (words, 1)
    );
    assert_eq!(
        lend::read(sam.dir.path(), &navidrome).unwrap(),
        None,
        "nothing lent"
    );
    let at = |at: &str| {
        let mut on = answering(1, (1, 1), "navidrome", None).unwrap();
        on.read.sockets[0].at = at.parse().unwrap();
        let mut lender = Lender::of(&sam, SECRET);
        lender.answers = Some(Ok(on));
        lender
    };
    let refused = shares(&at("192.168.1.5"), &["4533", "--group", "Lab"]).await;
    let words = "svrngs can't share port 4533: the program there listens only on 192.168.1.5, and svrngs reaches a share at this computer's own address. Have it listen on 127.0.0.1, ::1 or every address, then share it again.";
    assert_eq!(refused.unwrap_err().to_string(), words);
    let udp = ["4533", "--udp", "--program", "navidrome", "--group", "Lab"];
    let refused = shares(&at("::1"), &udp).await.unwrap_err().to_string();
    let words = "svrngs can't share UDP port 4533: svrngs sends a UDP share's datagrams to 127.0.0.1, and the program there doesn't listen on it. Have it listen on 127.0.0.1, or on every address, then share it again.";
    assert_eq!(refused, words);
    assert_eq!(
        lend::read(sam.dir.path(), &navidrome).unwrap(),
        None,
        "nothing lent"
    );
    let mut kept = Lender::of(&sam, SECRET);
    kept.answers = Some(answering(0x700e, (4321, 77), "navidrome", None));
    let said = shares(&kept, &["4533", "--group", "Lab"])
        .await
        .expect("navidrome is lent");
    assert!(
        said.contains("Lent navidrome: navidrome in ~/dev/navidrome. svrngs checks port 4533 before each connection it carries there. While another program is on the port, it is closed. If the program starts again, navidrome closes until you approve what runs there: `svrngs approve navidrome`."),
        "{said}"
    );
    let lent = lend::read(sam.dir.path(), &navidrome).unwrap().unwrap();
    assert_eq!(
        lent.terms.approved.map(|a| a.sockets),
        Some([0x700e].into())
    );
}

/// **A share is kept to the program its lender chose** (SHARE_BINDING
/// §5.1): `share 4533 --pid 999@1`, no socket on the port held by 999 or a
/// child, is refused, exit 1, in the doc's words, and nothing is lent;
/// `--pid 4321@76`, its start since another's, is refused as stopped;
/// `--pid 4321@77` is lent. `svrngs approve` approves what answers now,
/// with `--pid` as `share` takes it, its other fields kept, and says only
/// what it approved: a program on a LAN address of the port, which the door
/// never dials, is neither approved nor said. Planted, each watched failing:
/// the holder check skipped (999 lent); every socket's entry said (kept to
/// navidrome and other).
#[tokio::test]
async fn a_share_is_kept_to_the_program_its_lender_chose() {
    use svrngs_node::lend::{self, LendName};
    let wire = Wire::default();
    let (sam, _held) = sams_lab(&wire).await;
    let navidrome = LendName::parse("navidrome").unwrap();
    let mut lender = Lender::of(&sam, SECRET);
    lender.answers = Some(answering(0x700e, (4321, 77), "navidrome", Some(1)));
    let chose = |pid| ["4533", "--group", "Lab", "--pid", pid];
    let refused = shares(&lender, &chose("999@1")).await.unwrap_err();
    let words = "svrngs can't share port 4533: another program also listens on port 4533. Stop it, or choose that one, then share it again.";
    assert_eq!(
        (refused.to_string().as_str(), refused.exit_code()),
        (words, 1)
    );
    lender.answers = Some(answering(0x700e, (4321, 77), "navidrome", Some(77)));
    let stale = shares(&lender, &chose("4321@76")).await.unwrap_err();
    let words = "svrngs can't share port 4533: that program stopped; choose it again.";
    assert_eq!(stale.to_string(), words);
    assert_eq!(
        lend::read(sam.dir.path(), &navidrome).unwrap(),
        None,
        "nothing lent"
    );
    shares(&lender, &chose("4321@77"))
        .await
        .expect("navidrome is lent");
    let lent = lend::read(sam.dir.path(), &navidrome).unwrap().unwrap();
    assert_eq!(
        lent.terms.approved.map(|a| a.sockets),
        Some([0x700e].into())
    );
    let mut beside = answering(0x800e, (5000, 90), "navidrome", Some(90)).unwrap();
    let lan = answering(0x900e, (6000, 1), "other", None).unwrap();
    let mut elsewhere = lan.read.sockets[0].clone();
    elsewhere.at = "192.168.1.5".parse().unwrap();
    beside.read.sockets.push(elsewhere);
    beside.shown.extend(lan.shown);
    lender.answers = Some(Ok(beside));
    (lender.terminal, lender.typed) = (true, "y");
    let approve =
        cli::parse(["svrngs", "approve", "navidrome", "--pid", "5000@90"].map(String::from));
    let said = cli::answer(approve.unwrap(), &lender, &no_relay())
        .await
        .unwrap();
    assert_eq!(
        said,
        "navidrome is kept to navidrome in ~/dev/navidrome now.\n"
    );
    let again = lend::read(sam.dir.path(), &navidrome).unwrap().unwrap();
    assert_eq!(
        again.terms.approved.clone().map(|a| a.sockets),
        Some([0x800e].into())
    );
    assert_eq!(
        (again.name, again.kind, again.terms.audience),
        (lent.name, lent.kind, lent.terms.audience)
    );
}

/// **`svrngs approve` shows what answers before it approves** (SHARE_BINDING
/// §5.4; SHARE_BINDING_REVIEWS §9 #3): with no terminal and no token it is
/// refused, exit 1, in words that say where to approve, nothing approved.
/// At a terminal it says what answers now and asks: anything but y approves
/// nothing; y approves what it said, by that read's token, so a program that
/// took the port while its person read is refused as changed; with
/// `--token` from a lend's list it approves what that list showed, asking
/// nothing, and a token for what answered before is refused. Planted, each
/// watched failing: no look first (approved with no terminal); a y assumed
/// (approved on n); the token of what was said not passed (the program that
/// took the port approved).
#[tokio::test]
async fn svrngs_approve_shows_what_answers_before_it_approves() {
    use svrngs_node::lend::{self, LendName};
    let wire = Wire::default();
    let (sam, _held) = sams_lab(&wire).await;
    let navidrome = LendName::parse("navidrome").unwrap();
    let mut lender = Lender::of(&sam, SECRET);
    lender.answers = Some(answering(0x700e, (4321, 77), "navidrome", None));
    shares(&lender, &["4533", "--group", "Lab"]).await.unwrap();
    let kept = || lend::read(sam.dir.path(), &navidrome).unwrap().unwrap();
    let sockets = || kept().terms.approved.map(|a| a.sockets);
    let other = answering(0x800e, (5000, 90), "other", None);
    let approve = |args: &[&str]| {
        let argv = ["svrngs", "approve", "navidrome"].iter().chain(args);
        cli::parse(argv.map(|s| s.to_string())).unwrap()
    };
    lender.answers = Some(other.clone());
    let refused = cli::answer(approve(&[]), &lender, &no_relay())
        .await
        .unwrap_err();
    let words = "svrngs approve shows you what answers on navidrome's port before it keeps sharing navidrome, and there's no terminal here to ask you: run it in a terminal, or press Keep sharing it on your home page";
    assert_eq!(
        (refused.to_string().as_str(), refused.exit_code()),
        (words, 1)
    );
    assert_eq!(sockets(), Some([0x700e].into()), "nothing approved unseen");
    lender.terminal = true;
    lender.typed = "n";
    let said = cli::answer(approve(&[]), &lender, &no_relay())
        .await
        .unwrap();
    assert_eq!(said, "Nothing approved: navidrome stays closed.\n");
    assert_eq!(
        lender.asked.borrow().last().map(String::as_str),
        Some("Now answering on navidrome's port: other in ~/dev/other. Keep sharing navidrome with it? Type y, then press Enter: ")
    );
    assert_eq!(sockets(), Some([0x700e].into()), "nothing approved on n");
    lender.typed = "y";
    lender.asked.borrow_mut().clear();
    lender.changed = Some(answering(0x900e, (6000, 1), "third", None));
    let refused = cli::answer(approve(&[]), &lender, &no_relay())
        .await
        .unwrap_err();
    let words = "svrngs can't share port 4533: what answers on port 4533 changed; choose it again.";
    assert_eq!(refused.to_string(), words);
    assert_eq!(
        sockets(),
        Some([0x700e].into()),
        "what took the port meanwhile not approved"
    );
    lender.changed = None;
    let said = cli::answer(approve(&[]), &lender, &no_relay())
        .await
        .unwrap();
    assert_eq!(said, "navidrome is kept to other in ~/dev/other now.\n");
    assert_eq!(sockets(), Some([0x800e].into()));
    let listed = svrngs_node::share::approve(
        (4533.try_into().unwrap(), lend::Raw::Tcp),
        other,
        None,
        None,
    );
    let token = svrngs_node::share::token(&listed.unwrap().0);
    let asked = lender.asked.borrow().len();
    lender.terminal = false;
    lender.answers = Some(answering(0xa00e, (7000, 2), "fourth", None));
    let refused = cli::answer(approve(&["--token", &token]), &lender, &no_relay())
        .await
        .unwrap_err();
    assert_eq!(
        refused.to_string(),
        "svrngs can't share port 4533: what answers on port 4533 changed; choose it again."
    );
    lender.answers = Some(answering(0x800e, (5000, 90), "other", None));
    let said = cli::answer(approve(&["--token", &token]), &lender, &no_relay())
        .await
        .unwrap();
    assert_eq!(said, "navidrome is kept to other in ~/dev/other now.\n");
    assert_eq!(lender.asked.borrow().len(), asked, "a token asks nothing");
}

/// **A name shared from another port moves only when asked** (SHARE_BINDING
/// §5.8; SHARE_BINDING_REVIEWS §6 #10): `share 4534 --name navidrome`, the
/// name shared from 4533, is refused with the fix; with `--replace`, it is
/// shared from 4534; shared as UDP from 4534, refused the same; its file
/// unreadable, refused as that, never taken for none. Planted, each watched
/// failing: `share::renamed` not asked (moved unasked); the port alone
/// compared (moved to UDP unasked); an unreadable file taken for none
/// (shared over it unasked).
#[tokio::test]
async fn a_name_shared_from_another_port_moves_only_when_asked() {
    use svrngs_node::lend::{self, LendName};
    let wire = Wire::default();
    let (sam, _held) = sams_lab(&wire).await;
    let lender = Lender::of(&sam, SECRET);
    shares(&lender, &["4533", "--group", "Lab"]).await.unwrap();
    let moved = [
        "4534",
        "--group",
        "Lab",
        "--program",
        "navidrome",
        "--name",
        "navidrome",
    ];
    let refused = shares(&lender, &moved).await.unwrap_err();
    let words = "navidrome is already shared from port 4533: give this one another name, or add --replace to share navidrome from here instead.";
    assert_eq!(
        (refused.to_string().as_str(), refused.exit_code()),
        (words, 1)
    );
    let replaced = [&moved[..], &["--replace"]].concat();
    shares(&lender, &replaced).await.expect("moved when asked");
    let lent = lend::read(sam.dir.path(), &LendName::parse("navidrome").unwrap()).unwrap();
    assert_eq!(lent.and_then(|l| l.port()).map(|p| p.get()), Some(4534));
    let udp = [&moved[1..], &["4534", "--udp"]].concat();
    let refused = shares(&lender, &udp).await.unwrap_err().to_string();
    let words = "navidrome is already shared from port 4534: give this one another name, or add --replace to share navidrome from here instead.";
    assert_eq!(refused, words);
    let file = lend::lends_dir(sam.dir.path()).join("navidrome.json");
    std::fs::write(&file, "not a lend").unwrap();
    let refused = shares(&lender, &moved).await.unwrap_err().to_string();
    let unkept = "navidrome can't be changed on this computer: ";
    assert!(refused.starts_with(unkept), "{refused}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "not a lend");
}
