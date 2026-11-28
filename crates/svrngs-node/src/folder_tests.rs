// SPDX-License-Identifier: AGPL-3.0-or-later
//! The turns one large send at a time takes ([`super::Sends`]).

use super::*;

const WINDOW: u64 = BLOBS_RECEIVE_WINDOW as u64;
const A: Get = (1, 1);
const B: Get = (2, 1);
const C: Get = (3, 1);

fn sends() -> Sends<&'static str> {
    Sends {
        turn: None,
        waiting: VecDeque::new(),
        sent: BTreeMap::new(),
        looks: 0,
    }
}

/// `get` past its first window, so it asks for a turn.
fn large(s: &mut Sends<&'static str>, get: Get, asker: &'static str) -> Option<&'static str> {
    s.ask(get, WINDOW, "free")
        .expect("a get's first window needs no turn");
    s.ask(get, 1, asker)
}

#[test]
fn a_gets_first_window_goes_without_a_turn() {
    let mut s = sends();
    assert_eq!(large(&mut s, A, "a"), Some("a"));
    assert_eq!(
        s.ask(B, WINDOW, "b"),
        Some("b"),
        "a small file does not wait"
    );
    assert_eq!(s.turn, Some(A));
}

#[test]
fn a_second_large_get_waits_and_the_turn_is_kept() {
    let mut s = sends();
    assert_eq!(large(&mut s, A, "a"), Some("a"));
    assert_eq!(large(&mut s, B, "b"), None);
    assert_eq!(s.ask(A, 1, "a again"), Some("a again"));
}

#[test]
fn a_waiting_get_sends_a_chunk_at_each_look_and_the_turn_passes_when_it_is_up() {
    let mut s = sends();
    large(&mut s, A, "a");
    large(&mut s, B, "b");
    for _ in 1..LOOKS_A_TURN {
        assert_eq!(s.look(), ["b"], "so its fetcher sees it alive");
        assert_eq!(s.turn, Some(A));
        assert_eq!(s.ask(B, 1, "b"), None);
    }
    assert_eq!(s.look(), ["b"], "the turn, handed to who waited");
    assert_eq!(s.turn, Some(B));
    assert_eq!(s.ask(A, 1, "a"), None, "and the last turn's get waits now");
}

#[test]
fn a_get_that_ends_hands_on_its_turn_and_one_that_waited_is_forgotten() {
    let mut s = sends();
    large(&mut s, A, "a");
    large(&mut s, B, "b");
    large(&mut s, C, "c");
    assert_eq!(s.ended(B), None, "it was not its turn");
    assert_eq!(s.ended(A), Some("c"));
    assert_eq!(s.turn, Some(C));
    assert_eq!(s.ended(C), None);
    assert_eq!(s.turn, None, "with none waiting the turn is free");
    assert!(s.sent.is_empty());
}

/// A program with the token passes no budget, so its entries can overdraw
/// one; the app's next write waits one refill, not one for each of them.
/// The rest of the refill is held at the route, tests/carried.rs.
#[test]
fn an_overdrawn_write_budget_waits_one_refill_and_no_more() {
    use super::carried::wait_after;
    let at = 1_000_000;
    assert_eq!(wait_after(std::iter::repeat_n(at, 199), at), 0, "room left");
    assert_eq!(
        wait_after(std::iter::repeat_n(at, 200), at),
        18,
        "just empty"
    );
    assert_eq!(
        wait_after(std::iter::repeat_n(at, 5_000), at),
        18,
        "overdrawn"
    );
    assert_eq!(
        wait_after(std::iter::repeat_n(at, 5_000), at + 18),
        0,
        "refilled"
    );
}

/// Run is told every directory this computer syncs, each group's, the one
/// list `both_ways` keeps (synced.rs `syncing_in`); none where none was
/// ever kept; and a list that can't be read refuses, so nothing is kept
/// past it. Planted, watched failing: one group's directories alone.
#[test]
fn run_is_told_every_synced_directory_or_refuses() {
    let data = tempfile::tempdir().unwrap();
    assert_eq!(synced(data.path()), Ok(vec![]));
    let asks = [("/home/ada/svrngs/house", "g1"), ("/srv/shared", "g2")];
    let asks: Vec<_> = (asks.iter())
        .map(|(dir, group)| FolderAsk {
            dir: dir.into(),
            group: group.to_string(),
        })
        .collect();
    std::fs::write(
        data.path().join("synced.json"),
        serde_json::to_vec(&asks).unwrap(),
    )
    .unwrap();
    let mut said = synced(data.path()).unwrap();
    said.sort();
    assert_eq!(said, ["/home/ada/svrngs/house", "/srv/shared"]);
    std::fs::write(data.path().join("synced.json"), "not a list").unwrap();
    let refused = synced(data.path()).unwrap_err();
    assert!(
        refused.starts_with("svrngs can't read which folders it syncs, so nothing was kept ("),
        "{refused}"
    );
}
