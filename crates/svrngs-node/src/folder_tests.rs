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
