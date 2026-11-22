//! The flags that take a value (#147): each reads the next argument as its
//! value, and nothing else does.

use svrngs::cli;

fn argv(args: &[&str]) -> Vec<String> {
    std::iter::once("svrngs")
        .chain(args.iter().copied())
        .map(String::from)
        .collect()
}

/// **Each flag taking a value wants one, and an argument that is no flag
/// takes none** (#147): every one of the fourteen, last on the line, is
/// refused as wanting a value; and an empty argument before a word leaves
/// the word unexpected, never read as the empty argument's value. Red on
/// the plant the finding names, a doubled space in the space-joined string
/// the flags were kept in: the empty string among the flags, so `""` took
/// `x` as its value.
#[test]
fn each_flag_taking_a_value_wants_one_and_no_other_argument_takes_one() {
    let valued = [
        "--as",
        "--origin",
        "--name",
        "--program",
        "--kind",
        "--limit",
        "--refuse",
        "--upload",
        "--reads",
        "--sign-in",
        "--folder",
        "--group",
        "--ask",
        "--pid",
    ];
    for flag in valued {
        let said = cli::parse(argv(&["share", "8080", flag]));
        assert_eq!(said.err(), Some(format!("{flag} wants a value")), "{flag}");
    }
    let said = cli::parse(argv(&["share", "", "x"]));
    assert_eq!(said.err(), Some("unexpected argument `x`".to_string()));
}

#[test]
fn update_is_a_verb_of_its_own_and_says_which_verdict_it_reached() {
    assert_eq!(cli::parse(argv(&["update"])), Ok(cli::Verb::Update));
    assert_eq!(cli::parse(argv(&["restart"])), Ok(cli::Verb::Restart));
    use svrngs_node::update::UpdateError;
    assert_eq!(cli::Refusal::Update(UpdateError::NoPage).exit_code(), 3);
    assert_eq!(cli::Refusal::Update(UpdateError::Mismatch).exit_code(), 1);
    // A release refused for its signature is a refusal; a page with no such
    // file, a precondition absent.
    let unsigned = UpdateError::Unsigned("group.svrnme.sh".into());
    assert_eq!(cli::Refusal::Update(unsigned).exit_code(), 1);
    assert_eq!(cli::Refusal::Update(UpdateError::Missing).exit_code(), 3);
}

/// **Each refusal exits as it always has: 1 refused, 3 could-not-judge**
/// (cli.rs's module docs, "Exit codes"): every one but `Update`, whose
/// verdict `update_is_a_verb_of_its_own_and_says_which_verdict_it_reached`
/// pins. Red on the plant of any one moved to the other code.
#[test]
fn every_refusal_keeps_its_exit_code() {
    use cli::{Refused as R, Unjudged as U};
    use std::path::PathBuf;
    let port = std::num::NonZeroU16::new(8080).unwrap();
    let name = || svrngs::group::Name::parse("house").unwrap();
    let lend = || svrngs_node::lend::LendName::parse("wiki").unwrap();
    let io = || std::io::Error::other("planted");
    let path = || PathBuf::from("/home/sam/.svrngs/lock");
    let at = std::net::SocketAddr::from(([127, 0, 0, 1], 9000));
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(svrngs::identity::NODE_KEY_FILE), b"short").unwrap();
    let key = svrngs::identity::load_or_generate_node_key(dir.path()).unwrap_err();
    let refused = [
        R::NoName(name()),
        R::NoNameToJoin,
        R::LinkRefused,
        R::NameTaken {
            holder: name(),
            group: name(),
        },
        R::Left(name()),
        R::NoGroup("work".into()),
        R::MadeOlder(name()),
        R::NoAudience(port),
        R::Ambiguous {
            name: "house".into(),
            ids: vec!["3f2a91c0".into()],
        },
        R::NotIn(name()),
        R::Share(svrngs::share::ShareRefusal::NoKind(port)),
        R::NotLent {
            name: lend(),
            lent: vec![],
        },
        R::NoAppAddress("https://svrnme.sh".into()),
        R::Room("no".into()),
        R::Folder("no".into()),
        R::Unfit("no".into()),
        R::Removal("no".into()),
        R::Meet("no".into()),
        R::RelayForNobody,
        R::NotKept,
        R::Older,
        R::Readers(lend(), "no".into()),
        R::Terms("no".into()),
        R::Busy(path()),
        R::Unproven("http://127.0.0.1:1".into()),
        R::Behind(path()),
        R::Unrestarted("http://127.0.0.1:1".into()),
    ];
    let unjudged = [
        U::NoDataDir,
        U::DataDir(path(), io()),
        U::Key(key),
        U::Start,
        U::Taken(at, path()),
        U::Api(svrngs_node::api::ApiError::NoAddress(path())),
        U::IssuerAway,
        U::Prompt(io()),
        U::SignIn(svrngs_node::oidc::KeyFailure::Store(path(), io())),
        U::CannotCheck("https://svrnme.sh".into()),
        U::NotRunning,
        U::Unopened("no".into()),
        U::Unanswered("http://127.0.0.1:1".into()),
        U::Unchanged(name()),
        U::RelayStart("no".into()),
        U::GroupsUnread("no".into()),
        U::Keep("house".into()),
    ];
    let refused = refused.map(cli::Refusal::from);
    let unjudged = unjudged.map(cli::Refusal::from);
    for (code, each) in [(1, Vec::from(refused)), (3, Vec::from(unjudged))] {
        for r in each {
            assert_eq!(r.exit_code(), code, "{r:?}");
        }
    }
}
