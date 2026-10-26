//! A group made on the home page (start-group; DESIGN §3, "The home
//! page"): on a computer whose svrngs runs as install.sh keeps it — the
//! bare `svrngs`, this build's, in a `HOME` of its own, where `svrngs new`
//! is refused while it runs (demo/runs/start/2026-10-13-home/NEEDS_HUMAN.md)
//! — the page, in Playwright's headless Chromium and in its WebKit, makes
//! a group by its name and the person's; its folder is placed at
//! `~/svrngs/<name>`, where `join` puts everyone's; a link the page makes
//! for it is redeemed by another computer, which then stands in it; and a
//! name already held makes a second group, said as it is typed and once
//! made, by the one rule `svrngs new` asks too (#138). The group keeps the
//! page this computer's svrngs came from, as install.sh keeps it, so the
//! link is an https link on it. Neither computer is
//! restarted. Dee's computer holds a first group, Lab, made by `svrngs new
//! --relay` at a relay on 127.0.0.1, so it is met there alone, as Bo's
//! computer is: nothing leaves this computer. The browser half is
//! `web/tests/home.mjs`, which fails naming the install command when
//! Playwright or its browsers are absent — it never skips.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;
mod common;
#[path = "common/home_page.rs"]
mod home_page;

use std::path::Path;
use std::time::Duration;

use home_page::home_page;

use serde_json::{json, Value};
use svrngs::cli;
use svrngs::group::Name;
use svrngs_node::link::Link;
use svrngs_node::{actor_of, SigningKey};
use tokio::io::AsyncBufReadExt;

/// `svrngs <args>`, this build's, with `home` its `HOME` and nothing else
/// of this computer's environment but its `PATH`, and the group's traces
/// at debug on its stderr: a name's one rule (#138).
fn svrngs(home: &Path, args: &[&str]) -> tokio::process::Command {
    let mut c = tokio::process::Command::new(env!("CARGO_BIN_EXE_svrngs"));
    c.args(args)
        .env_clear()
        .env("HOME", home)
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("RUST_LOG", "group=debug")
        .env("NO_COLOR", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    c
}

/// `svrngs new <name> --as Dee --relay <at>` in `home`, run until it says
/// all it says, as a terminal's Ctrl-C then stops it: what it printed, and
/// what it traced.
async fn new(home: &Path, name: &str, at: &str) -> (Vec<String>, String) {
    let mut new = svrngs(home, &["new", name, "--as", "Dee", "--relay", at])
        .spawn()
        .unwrap();
    let mut out = tokio::io::BufReader::new(new.stdout.take().unwrap()).lines();
    let said = tokio::time::timeout(Duration::from_secs(60), async {
        let mut said = Vec::new();
        while let Ok(Some(line)) = out.next_line().await {
            let all = line.starts_with("Leave this running");
            said.push(line);
            if all {
                break;
            }
        }
        said
    })
    .await
    .expect("svrngs new printed all it says");
    new.kill().await.unwrap();
    (said, traced_by(new).await)
}

/// What `child`, stopped, traced on its stderr.
async fn traced_by(mut child: tokio::process::Child) -> String {
    use tokio::io::AsyncReadExt;
    let mut traced = String::new();
    let err = child.stderr.as_mut().unwrap();
    err.read_to_string(&mut traced).await.unwrap();
    traced
}

/// The rule's one trace for a name held `held` times already.
fn ruled(traced: &str, name: &str, held: usize) -> bool {
    traced.lines().any(|l| {
        l.contains("group: a name's one rule")
            && l.contains(&format!("name={name}"))
            && l.contains(&format!("held={held}"))
    })
}

/// svrngs as login keeps it in `home`, the bare command left running,
/// and its home page's address once it answers.
async fn run(home: &Path) -> (tokio::process::Child, String) {
    let running = svrngs(home, &[]).spawn().unwrap();
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let api = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let api = std::fs::read_to_string(home.join(".svrngs/api")).unwrap_or_default();
            let api = api.trim().to_string();
            let token = std::fs::read_to_string(home.join(".svrngs/token")).unwrap_or_default();
            let asked = http.get(format!("{api}/v1/groups")).bearer_auth(token);
            if asked.send().await.is_ok() {
                break api;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("svrngs answers at the API it wrote");
    let port = api.rsplit(':').next().unwrap();
    (running, format!("http://svrngs.localhost:{port}/"))
}

in_both_engines!(
    /// **A group made on the home page, where svrngs runs from login**
    /// (start-group (c)). Planted, watched failing: the page as 99c8638
    /// left it, with no Make a group.
    the_home_page_makes_a_group_another_computer_joins_with_no_restart
);
async fn the_home_page_makes_a_group_another_computer_joins_with_no_restart(
    engine: harness::Engine,
) {
    let (relay, _ours, _bo_dir, bo) = common::sams_computer(4).await;
    let home = tempfile::tempdir().unwrap();
    let at = harness::relay_url(&relay);

    // Lab, Dee's first group, met at the relay here; then another called
    // lab, said beside the first by whose it is and since when, by the
    // name's one rule (#138).
    let (said, traced) = new(home.path(), "Lab", &at).await;
    assert!(ruled(&traced, "Lab", 0), "{said:#?}\n{traced}");
    let (said, traced) = new(home.path(), "lab", &at).await;
    let today = chrono::Local::now().format("%-d %b");
    let second =
        format!("You’re already in Lab (yours, since {today}). This is a second group called lab.");
    assert!(said.contains(&second), "{said:#?}");
    // Its folder its own. On a disk that ignores case (macOS's), Lab's
    // folder answers to LAB, a spelling neither group has, and lab's place
    // is Lab's, so lab goes beside it, its id's start after its name; on
    // one that keeps case (Linux's), lab's place is free, and lab's.
    let held = svrngs::group::groups(&home.path().join(".svrngs"));
    let lab = held
        .iter()
        .find(|s| s.name().as_str() == "lab")
        .expect("lab held");
    let id = &lab.id()[..8];
    assert!(home.path().join("svrngs/Lab").is_dir(), "Lab's folder");
    let ignores_case = home.path().join("svrngs/LAB").exists();
    let apart = match ignores_case {
        true => home.path().join(format!("svrngs/lab-{id}")),
        false => home.path().join("svrngs/lab"),
    };
    assert!(
        (said.iter()).any(|l| l.starts_with(&format!("lab's folder is {}: ", apart.display()))),
        "{said:#?}"
    );
    assert!(ruled(&traced, "lab", 1), "{traced}");

    // The page this computer's svrngs came from, as install.sh keeps it
    // in the data dir: a group made on the home page keeps it, so its
    // links are https links on it.
    let data = home.path().join(".svrngs");
    let came_from = "http://127.0.0.1:9/";
    std::fs::write(data.join("origin"), came_from).unwrap();

    let (mut running, page) = run(home.path()).await;

    let all = "#home section h2, #home section p, #home > p, #home summary";
    let seen = home_page(
        engine,
        &page,
        &std::fs::read_to_string(home.path().join(".svrngs/token")).unwrap(),
        json!({
            "all": all,
            "do": [
                ["#make input[name=name]", "Trip"],
                ["#make input[name=person]", "Dee"],
                ["#make button:has-text('Make a group')", null],
                ["#home section button:has-text('Make a link to invite someone to Trip')", null, true],
                ["#make input[name=name]", "trip", true],
                ["#make input[name=person]", "Dee"],
                ["#make button:has-text('Make a group')", null],
            ],
        }),
    )
    .await;
    eprintln!("the home page's run: {seen:#}");

    // Made, and said so; its folder placed where everyone's is.
    let place = home.path().join("svrngs/Trip");
    let made = seen["after"][0].to_string();
    assert!(made.contains("You made Trip."), "{seen:#}");
    assert!(
        made.contains(&format!(
            "Trip’s folder is on this computer at {}",
            place.display()
        )),
        "{seen:#}"
    );
    assert!(place.is_dir(), "{} not made", place.display());

    // A link the page made for it, on the page svrngs came from,
    // redeemed by Bo's computer.
    let shown = seen["after"][1].to_string();
    let https = format!("Send this link to the person you are inviting: {came_from}#");
    assert!(shown.contains(&https), "{seen:#}");
    let at_link = shown.find(&format!("{came_from}#")).expect("a link shown");
    let raw = shown[at_link..].split([' ', '"']).next().unwrap();
    let link = Link::parse(raw).expect("the link shown is a link");
    let bo_key = SigningKey::from_bytes(&[4; 32]);
    let joined = cli::join_group(&bo, &bo_key, &link, &Name::parse("Bo").unwrap()).await;
    let Ok(cli::Said::Joined { seed, .. }) = joined else {
        panic!("Bo is let in: {joined:?}");
    };
    assert_eq!(seed.name().as_str(), "Trip");
    let standing = bo.rail().standing(&seed.id()).unwrap();
    assert!(standing.names(&actor_of(&bo_key)), "Bo stands in Trip");

    // A name already held: said as it is typed, then made, and said again,
    // by the rule `svrngs new` asks (#138); its words to the letter the
    // command's, pinned apart (`the_command_and_the_page_say_…`, #157).
    let before = seen["after"][2].to_string();
    assert!(
        before.contains("You’re already in Trip (yours, since "),
        "{seen:#}"
    );
    assert!(
        before.contains("). This makes a second group called trip."),
        "{seen:#}"
    );
    let after = seen["after"][3].to_string();
    assert!(
        after.contains("You made trip. You’re already in Trip (yours, since "),
        "{seen:#}"
    );
    assert!(
        after.contains("). This is a second group called trip."),
        "{seen:#}"
    );
    running.kill().await.unwrap();
    let traced = traced_by(running).await;
    assert!(
        ruled(&traced, "trip", 1),
        "the page's make, by the one rule: {traced}"
    );
    bo.endpoint().close().await;
}

in_both_engines!(
    /// **Another group joined from its link on the home page, while
    /// svrngs runs** (arch-command (c)): Dee's computer, in Lab and its
    /// svrngs running as login keeps it, where `svrngs join` is refused,
    /// is sent a link to Sam's Trip. Pasted on the home page, the page asks
    /// the computer that sent it what it is for and says so on the button,
    /// "Join Trip as Dee", before it is pressed; pressed, the svrngs
    /// running there joins as `svrngs join` joins (join.rs, the one join),
    /// the page says so and draws Trip beside Lab, its folder placed where
    /// a joiner's is, and Sam's computer has let Dee in. Planted, watched
    /// failing: the page as 87d8e802 left it, with no join.
    the_home_page_joins_another_group_from_its_link_while_svrngs_runs
);
async fn the_home_page_joins_another_group_from_its_link_while_svrngs_runs(
    engine: harness::Engine,
) {
    let (relay, ours, _sam_dir, sam) = common::sams_computer(6).await;
    let (trip, _) = cli::new_group(
        &sam,
        Name::parse("Trip").unwrap(),
        Name::parse("Sam").unwrap(),
        None,
        None,
        &ours,
    )
    .await
    .expect("Sam makes Trip");
    let made = (sam.host().link(&trip.id(), false, None, None)).expect("Sam makes a link to Trip");
    let link = svrngs_node::link::client_link(&made.token);

    let home = tempfile::tempdir().unwrap();
    new(home.path(), "Lab", &harness::relay_url(&relay)).await;
    let (mut running, page) = run(home.path()).await;
    let all = "#home section h2, #home section p, #home > p, #home summary";
    let seen = home_page(
        engine,
        &page,
        &std::fs::read_to_string(home.path().join(".svrngs/token")).unwrap(),
        json!({
            "all": all,
            "do": [
                ["#join input[name=link]", link, true],
                ["#join input[name=person]", "Dee"],
                ["#join button:has-text('Join Trip as Dee')", null],
            ],
        }),
    )
    .await;
    eprintln!("the home page's run: {seen:#}");

    // What the link is for, said on the button before it is pressed.
    let asked = seen["after"][0].to_string();
    assert!(asked.contains("Join Trip"), "{seen:#}");
    // Joined, said so, and Trip drawn beside Lab, its folder placed.
    let joined = seen["after"][1].to_string();
    assert!(joined.contains("You’re in Trip, as Dee."), "{seen:#}");
    let heads: Vec<_> = (seen["all"].as_array().into_iter().flatten())
        .chain(seen["after"][1].as_array().into_iter().flatten())
        .filter_map(Value::as_str)
        .collect();
    for group in ["Trip (Sam’s", "Lab (yours"] {
        assert!(
            heads.iter().any(|h| h.starts_with(group)),
            "{group}: {seen:#}"
        );
    }
    // Just joined Trip, its shelf is folded as "Bring more to Trip"; Lab's,
    // a group Dee was in already, is not (the operator, 2026-10-26;
    // demo/xenia/findings/after-join-reads-as-permissions.md).
    let summaries: Vec<_> = (seen["after"][1].as_array().into_iter().flatten())
        .filter_map(Value::as_str)
        .filter(|h| h.starts_with("Bring more to"))
        .collect();
    assert_eq!(summaries, ["Bring more to Trip"], "{seen:#}");
    let place = home.path().join("svrngs/Trip");
    assert!(
        joined.contains(&format!(
            "Trip’s folder is on this computer at {}",
            place.display()
        )),
        "{seen:#}"
    );
    assert!(place.is_dir(), "{} not made", place.display());
    // Sam's computer let Dee in.
    let register = sam.rail().register(&trip.id()).unwrap();
    assert!(
        (register.members.values()).any(|m| m.person.as_str() == "Dee"),
        "Dee stands in Trip on Sam's computer"
    );
    running.kill().await.unwrap();
    sam.endpoint().close().await;
}

/// **A group's namesakes, one wording for the command and the page**
/// (v1ui-findings (e); #157): the sentence `svrngs new` prints when a
/// name is held already (cli.rs `namesakes_said`) and the one the home
/// page says (question.js `namesakes`, run by node), for one namesake and
/// for two, one of them someone else's: the same words to the letter,
/// the apostrophe and the day included. Planted, watched failing: the
/// two as v1ui-door left them — "Sam's" and "Sam’s", chrono's "3 Oct"
/// and the locale's "Oct 3".
#[test]
fn the_command_and_the_page_say_a_name_held_already_in_one_wording() {
    use svrngs::group::Namesake;
    let trip = |name: &str, maker: Option<&str>, since| Namesake {
        name: name.into(),
        maker: maker.map(Into::into),
        since,
    };
    let source = harness::repo().join("crates/svrngs-node/src/home/question.js");
    let script = format!(
        "const q = await import({:?}); const [n, name] = JSON.parse(process.argv[1]); console.log(q.namesakes(n, name, 'This is'));",
        format!("file://{}", source.display())
    );
    let one = vec![trip("Trip", Some("sam"), Some(1_791_000_000))];
    let two = vec![one[0].clone(), trip("trip", None, Some(1_759_500_000))];
    for beside in [one, two] {
        let out = std::process::Command::new("node")
            .args(["--input-type=module", "-e", &script])
            .arg(json!([beside, "TRIP"]).to_string())
            .output()
            .expect("node runs");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let page = String::from_utf8_lossy(&out.stdout).trim().to_string();
        assert_eq!(cli::namesakes_said(&beside, "TRIP"), page);
    }
}
