//! The review's case, invariant 6's twin (APP_STATE §7; §4, "A removed
//! device"), run as written: Jo moves a magnet to 2, and every online copy
//! takes it. Sam moves it to 3, and those copies prune Jo's set, now
//! overwritten. An offline computer returns and removes Sam. Every copy then
//! equals the reference over every admissible set, Sam's 3, which merged
//! before his removal, and none falls back to the value before Jo's move.

use crate::change::{ChangeSet, SetId, Site, State, Val, pack};
use crate::reference;
use crate::replica::{ModelCopy, Op, Plant, Replica, Table};

const MAGNETS: &[Table] = &[Table {
    name: "magnets",
    cols: &["pos"],
    defaults: &[],
}];

const JO: usize = 0;
const SAM: usize = 1;
/// A third computer, online throughout.
const MIA: usize = 2;
/// The computer offline from Jo's move until it removes Sam.
const AWAY: usize = 3;
const NAMES: [&str; 4] = ["Jo's", "Sam's", "Mia's", "the offline"];

fn moved(pos: i64) -> Op {
    Op::insert("magnets", vec![Val::text("cat")], vec![("pos", Val::Integer(pos))])
}

fn pos(state: &State) -> Option<i64> {
    match state
        .get(&("magnets".to_owned(), pack(&[Val::text("cat")])))?
        .get("pos")
    {
        Some(Val::Integer(p)) => Some(*p),
        _ => None,
    }
}

/// These copies take `set`.
fn put<R: Replica>(copies: &mut [R], set: &ChangeSet, to: &[usize]) {
    for &c in to {
        copies[c].take(set);
    }
}

/// The case on four copies `make` builds; what breaks it, said.
fn review_case<R: Replica>(make: &dyn Fn(Site) -> R) -> Result<(), String> {
    let mut copies: Vec<R> = (1..=4).map(|i| make(Site::of(i))).collect();
    let first = copies[MIA].write(&[moved(1)]).expect("the magnet placed");
    put(&mut copies, &first, &[JO, SAM, AWAY]);
    // The offline computer is away from here on.
    let jo = copies[JO].write(&[moved(2)]).expect("Jo's move");
    put(&mut copies, &jo, &[SAM, MIA]);
    let sam = copies[SAM].write(&[moved(3)]).expect("Sam's move");
    put(&mut copies, &sam, &[JO, MIA]);
    for c in [JO, SAM, MIA] {
        if copies[c].kept().iter().any(|s| s.id() == jo.id()) {
            return Err(format!(
                "{} copy still keeps Jo's set: the case needs it pruned",
                NAMES[c]
            ));
        }
    }
    // The offline computer returns and removes Sam, and the record tells
    // every copy. Sam's computer is refused by the members' sync from now
    // on; the rest meet, each sending each all it keeps, as a rebuild from a
    // clean source pulls everything again, until no copy changes.
    let sam_s = copies[SAM].site();
    for copy in &mut copies {
        copy.removed(sam_s);
    }
    let meeting = [JO, MIA, AWAY];
    let now = |copies: &[R]| -> Vec<(State, Vec<SetId>)> {
        copies
            .iter()
            .map(|c| (c.state(), c.kept().iter().map(ChangeSet::id).collect()))
            .collect()
    };
    for round in 0.. {
        let before = now(&copies);
        for from in meeting {
            for to in meeting.into_iter().filter(|&to| to != from) {
                for set in copies[from].kept() {
                    put(&mut copies, &set, &[to]);
                }
            }
        }
        if now(&copies) == before {
            break;
        }
        assert!(round < 10, "the meet settles");
    }
    let want = reference::state(MAGNETS, [&first, &jo, &sam]);
    if pos(&want) != Some(3) {
        return Err(format!("the reference is at {:?}, not Sam's 3", pos(&want)));
    }
    for c in meeting {
        let got = pos(&copies[c].state());
        if got == Some(1) {
            return Err(format!(
                "{} copy fell back to 1, the value before Jo's move; the reference is Sam's 3",
                NAMES[c]
            ));
        }
        if copies[c].state() != want {
            return Err(format!("{} copy is at {got:?}; the reference is Sam's 3", NAMES[c]));
        }
    }
    Ok(())
}

#[test]
fn every_copy_ends_at_sam_s_3_and_none_falls_back() {
    review_case(&|site| ModelCopy::new(site, MAGNETS)).unwrap();
}

#[test]
fn undoing_a_removed_device_s_sets_falls_back_to_the_value_before_jo_s_move() {
    let err = review_case(&|site| ModelCopy::planted(site, MAGNETS, Plant::UndoRemoved)).unwrap_err();
    assert!(err.contains("fell back to 1"), "{err}");
}
