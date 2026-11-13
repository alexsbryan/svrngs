//! A group's name held already, said in one wording by `svrngs new` and
//! the home page (#157). The home page's make and join, once driven here
//! in a browser against a bare svrngs, are on the wire in
//! tests/verbs/home.rs and in web/tests/pages/home.test.mjs's twin
//! (docs/seams.tsv); the home page in a real browser is the browser
//! step's contained test.

#![allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[path = "../../svrngs-node/tests/browser/harness.rs"]
#[macro_use]
#[allow(unused_macros, dead_code)]
mod harness;

use serde_json::json;
use svrngs::cli;

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
