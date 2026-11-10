// SPDX-License-Identifier: AGPL-3.0-or-later
//! scripts/surface.txt, the one list of what an app asks of svrngs (svrn-docs
//! `rings/current/EXTENSION_POINT.md` §6.5, §8 "One list"), held to the code
//! and to the guide's reference, [`GUIDE`] as the binary holds it, each way.

use std::collections::BTreeSet;

use svrngs_core::pass::Label;

use crate::door::limit::READS_A_MINUTE;
use crate::door::LendRefusal::{self, *};
use crate::folder::carried::{ACTS, WRITES_AN_HOUR};
use crate::origin::SEND_MAX;
use crate::surface::{AppRoute, GUIDE, LEND_PATH, READS_PATH, STOP_SUFFIX, TO_PATH};

/// The list's lines of `kind`, each after its word.
fn listed(kind: &str) -> BTreeSet<String> {
    let list = include_str!("../../../scripts/surface.txt");
    let lines = list
        .lines()
        .filter_map(|l| l.strip_prefix(kind)?.strip_prefix(' '));
    lines.map(str::to_string).collect()
}

/// How the list spells `route`, none for the app's own files. Every
/// variant is named, so a new one fails to compile here until it is spelled.
fn spelled(route: &AppRoute) -> Option<String> {
    match route {
        AppRoute::Acts => Some(ACTS.into()),
        AppRoute::Reads => Some(READS_PATH.into()),
        AppRoute::Stop(_) => Some(format!("{READS_PATH}/<label>{STOP_SUFFIX}")),
        AppRoute::To => Some(TO_PATH.into()),
        AppRoute::Lend(..) => Some(format!("{LEND_PATH}/<label>/<path>")),
        AppRoute::Own => None,
    }
}

/// Every refusal's word. Every variant is named, so a new one fails to
/// compile here until it is listed.
fn words() -> BTreeSet<String> {
    let every = [
        Away, Pass, Closed, Unlent, Path, Full, Fault, Unasked, Stopped, Removed, Ended, Limit,
    ];
    for r in every {
        match r {
            Away | Pass | Closed | Unlent | Path | Full | Fault | Unasked | Stopped | Removed
            | Ended | Limit => {}
        }
    }
    every
        .into_iter()
        .map(|r: LendRefusal| r.as_str().to_string())
        .collect()
}

/// The limits, `<name> <number> | <the most>`, as the code's numbers say
/// them.
fn limits() -> BTreeSet<String> {
    let (entry, writes, reads) = (
        svrngs_core::MAX_PAYLOAD_BYTES,
        WRITES_AN_HOUR,
        READS_A_MINUTE,
    );
    [
        format!("entry {entry} | {} KiB", entry / 1024),
        format!(
            "writes {writes} | {writes} entries at once, then one every {} seconds",
            3600 / writes
        ),
        format!(
            "reads {reads} | {reads} reads at once, then {} a second",
            reads / 60
        ),
        format!("send {SEND_MAX} | {} MiB", SEND_MAX >> 20),
    ]
    .into_iter()
    .collect()
}

/// The cells of `column` of each row of the guide's reference's tables
/// whose header's first cell is `header`.
fn reference(header: &str, column: usize) -> BTreeSet<String> {
    let section = GUIDE
        .split("\n## The reference\n")
        .nth(1)
        .expect("the guide has a reference");
    let section = section.split("\n## ").next().unwrap_or_default();
    let head = format!("\n| {header} |");
    let tables = section.split(&head).skip(1);
    let rows = tables.flat_map(|t| t.lines().skip(2).take_while(|l| l.starts_with('|')));
    rows.filter_map(|r| Some(r.split('|').nth(column + 1)?.trim().to_string()))
        .collect()
}

/// **The list is the code's, and the code's is the list** (§6.5): each
/// route parses to the variant the list spells it as, and every variant
/// but the app's own files is listed; each limit is the number the code
/// holds, said as the code's numbers say it; each refusal word is one
/// `LendRefusal` says, and every one is listed. Planted, watched failing:
/// `refused ended` taken out of the list; `refused nope` put in; the route
/// `/svrngs-to` taken out; WRITES_AN_HOUR made 300.
#[test]
fn the_list_is_the_code_s_and_the_code_s_is_the_list() {
    let films = Label::of(&[2; 32], "films");
    let every = [
        AppRoute::Acts,
        AppRoute::Reads,
        AppRoute::Stop(films.clone()),
        AppRoute::To,
        AppRoute::Lend(Some(films), "/x".into()),
    ];
    let code: BTreeSet<_> = every.iter().filter_map(spelled).collect();
    assert_eq!(
        listed("route"),
        code,
        "scripts/surface.txt's routes, then the code's"
    );
    for route in listed("route") {
        let asked = route.replace("<label>", &Label::of(&[2; 32], "films").to_string());
        let parsed = AppRoute::of(&asked.replace("<path>", "x"));
        assert_eq!(
            spelled(&parsed).as_ref(),
            Some(&route),
            "{route} parses as {parsed:?}"
        );
    }
    assert_eq!(
        listed("limit"),
        limits(),
        "scripts/surface.txt's limits, then the code's"
    );
    assert_eq!(
        listed("refused"),
        words(),
        "scripts/surface.txt's words, then the code's"
    );
}

/// **The guide's reference is the list, and the list is the reference's**
/// (§6.5, §8 "One page"): the routes its calls name, the most its limits
/// say, and the words its refusals table gives, each the list's and no
/// more, in the guide the binary prints. Planted, watched failing: the
/// `pass` row taken out of the guide's table; a call `GET svrngs-more`
/// put in it; the 64 KiB row said as 64 KB.
#[test]
fn the_guide_s_reference_is_the_list_and_the_list_is_the_reference_s() {
    let calls = reference("Call", 0).into_iter().filter_map(|c| {
        let call = c
            .strip_prefix("`GET ")
            .or_else(|| c.strip_prefix("`POST "))?;
        Some(format!(
            "/{}",
            call.trim_end_matches('`').split('?').next()?
        ))
    });
    assert_eq!(
        listed("route"),
        calls.collect(),
        "the list's routes, then the guide's calls"
    );
    let most = listed("limit")
        .into_iter()
        .filter_map(|l| Some(l.split_once(" | ")?.1.to_string()));
    assert_eq!(
        most.collect::<BTreeSet<_>>(),
        reference("Limit", 1),
        "the list's limits, then the guide's"
    );
    let said = reference("Word", 0)
        .into_iter()
        .map(|w| w.trim_matches('`').to_string());
    assert_eq!(
        listed("refused"),
        said.collect(),
        "the list's words, then the guide's"
    );
}
