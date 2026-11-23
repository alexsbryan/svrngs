//! A phone's sets between svrngs-db's copies, by hand (svrn-docs
//! `rings/current/APP_STATE.md` §4, "Phones"; §11, W1, "the simulator
//! drives computers and phones"): each call and carry held to the
//! reference's bookkeeping, as `db.rs` holds every one (`DbCopy::called`).
//! A phone's site vouched for by its home (`app-state-diligence/
//! sync-simplify-2026-11-22/`, change 4, "How W1 proves it"), and a set
//! whose pass doesn't hold dropped by every copy and counted by none. The
//! seeded runs with phones are `db.rs`'s. Each test names the plant it was
//! watched failing on.

use std::collections::BTreeSet;

use crate::cases::cat;
use crate::change::{ChangeSet, SetId, Site, Val};
use crate::db::{DbCopy, phoned, watched};
use crate::live::named;
use crate::reference;
use crate::replica::{Op, Replica};
use crate::sim::TABLES;
use crate::sync::Variant;
use crate::syncs::Net;

const ANA: usize = 0;
const BEN: usize = 1;
const CY: usize = 2;
const ALL: &[usize] = &[ANA, BEN, CY];

/// Bo's phone, the run's one.
const BO: u8 = 0;

fn placed(pos: i64) -> Vec<Op> {
    vec![Op::insert("magnets", cat(), vec![("pos", Val::Integer(pos))])]
}

fn moved(pos: i64) -> Vec<Op> {
    let set = vec![("pos", Val::Integer(pos))];
    vec![Op::Update {
        table: "magnets",
        key: cat(),
        set,
    }]
}

/// Ana's, Ben's and Cy's copies, each of its own computer, the record
/// binding Bo's phone too, Ana's carrying the board: she places the cat at
/// 0, and the three meet, so each is established before any phone writes.
/// A copy not yet synced takes a synced member's marks on its word, as a
/// newcomer trusts its source, and the home's word would go untested. Ana's
/// set too.
fn three() -> (Net<DbCopy>, ChangeSet) {
    let sync = Variant::Vouched.sync().expect("svrngs-db's adoption");
    let mut net = Net::new(sync, 3, &phoned(0, 1));
    let ana = net.write(ANA, &placed(0), "Ana places the cat");
    net.meet(ALL);
    let synced = ALL.iter().all(|&c| net.copies[c].synced());
    assert!(synced, "the case needs every copy established");
    (net, ana)
}

/// Each copy's mark for `site` and what it counts above its marks, by
/// svrngs-db's own ask and by the reference's book, said.
fn marks(net: &Net<DbCopy>, site: Site) -> String {
    let each = ALL.iter().map(|&c| {
        let (marks, haves) = net.copies[c].book();
        let book = &net.books[c];
        let above = |ids: &BTreeSet<SetId>| ids.iter().map(|id| named(*id)).collect::<Vec<_>>();
        format!(
            "{}: svrngs-db's mark {:?} above {:?}, the reference's {:?} above {:?}",
            ["Ana", "Ben", "Cy"][c],
            marks.get(&site),
            above(&haves),
            book.marks().get(&site),
            above(book.above())
        )
    });
    each.collect::<Vec<_>>().join("; ")
}

/// Whether copy `c` keeps the set `id`.
fn keeps(net: &Net<DbCopy>, c: usize, id: SetId) -> bool {
    net.copies[c].kept().iter().any(|s| s.id() == id)
}

/// **A phone's site is vouched for by its home, between computers**
/// (change 4, "How W1 proves it"; tests/phones.rs, the same case at one
/// pair of databases): Ana places the cat, and the three copies meet. Bo's
/// phone moves the cat twice through Ana's computer, its home, to 1 and
/// then 2, at seqs 1 and 2 of his site there, and carries each set to her
/// copy, which prunes the first, outranked, before any round. Then Ana
/// calls Ben, Ben calls Cy, and the three meet by calls. Ben's copy, sent
/// the second set alone, takes Ana's word for the site's mark; Cy's doesn't
/// take Ben's, who isn't its home, and closes on Ana's call. Every copy's
/// mark for the phone's site ends at 2, by svrngs-db's own ask and the
/// reference's book alike; nothing is kept above a gap; every copy equals
/// the reference over the three sets; and each call is the reference's.
/// Plant, watched failing: svrngs-db's `claimed` trusting the site's device
/// in place of its home (sync.rs, `from == Some(&device)`): Ben's and Cy's
/// marks for the phone's site stay short, the second set kept above a gap,
/// and Ben's copy parts from the reference at Ana's call.
#[test]
fn a_phone_s_site_is_vouched_for_by_its_home() {
    let ((said, wrong), parted) = watched("a phone's two quick moves".into(), || {
        let (mut net, ana) = three();
        let one = net.phone((BO, ANA), &moved(1), (false, &[ANA]));
        let two = net.phone((BO, ANA), &moved(2), (false, &[ANA]));
        let bo = two.id().0;
        assert_eq!((one.id(), two.id()), ((bo, 1), (bo, 2)), "seqs 1 and 2 of one site");
        assert_eq!(bo.home(), Site::of(1), "Ana's computer, its home");
        let mut said = vec![format!(
            "  Bo's site {}; Ana keeps {:?}",
            named((bo, 0)),
            ids(&net.copies[ANA].kept())
        )];
        let mut wrong = Vec::new();
        if keeps(&net, ANA, one.id()) {
            wrong.push("Ana's copy keeps the first set: the case needs it pruned".to_owned());
        }
        net.call(ANA, BEN);
        said.push(format!("  after Ana calls Ben: {}", marks(&net, bo)));
        net.call(BEN, CY);
        said.push(format!("  after Ben calls Cy: {}", marks(&net, bo)));
        net.meet(ALL);
        said.push(format!("  after the meet: {}", marks(&net, bo)));
        let want = reference::state(TABLES, [&ana, &one, &two]);
        for (c, name) in [(ANA, "Ana's"), (BEN, "Ben's"), (CY, "Cy's")] {
            let (real, haves) = net.copies[c].book();
            let book = &net.books[c];
            if real.get(&bo) != Some(&2) || book.mark(bo) != 2 {
                wrong.push(format!("{name} mark for Bo's site isn't 2"));
            }
            if !haves.is_empty() || !book.above().is_empty() {
                wrong.push(format!("{name} copy keeps sets above a gap"));
            }
            if net.copies[c].state() != want {
                wrong.push(format!("{name} copy is not the reference's"));
            }
        }
        (said.join("\n"), wrong)
    });
    println!("a phone's two quick moves, svrngs-db's copies:\n{said}");
    assert!(wrong.is_empty() && parted.is_empty(), "{wrong:#?}\n{parted:#?}");
}

/// **A phone's set whose pass doesn't hold is dropped by every copy and
/// counted by none** (the operator's B′, 2026-11-22; wire.rs
/// `Phoned::signer`; tests/phones.rs, the signer's refusal of the same
/// stamp): Ana places the cat, and the three copies meet. Bo's phone moves
/// the cat through Ana's computer, its home, and carries the set with its
/// stamp past its pass's lapse, re-signed by his open's approver, to Ana's
/// copy and then Ben's: each drops it as not its device's
/// (`Took::NotItsDevice`), held at each carry to the reference, which
/// counts nothing of it. His next move, honest, at the next seq of his site
/// there, is carried to Ana's copy and taken, and the three meet by calls.
/// No copy keeps the lapsed set or counts its place, by svrngs-db's own ask
/// or the reference's book, and every copy equals the reference over Ana's
/// set and the honest move. Plant, watched failing: the pass's lapse unread
/// in wire.rs `Phoned::signer` (`self.ts <= pass.until` dropped): Ana's
/// copy takes the lapsed set, where the reference drops it.
#[test]
fn a_phone_s_set_whose_pass_doesn_t_hold_is_counted_by_none() {
    let ((said, wrong), parted) = watched("a lapsed pass".into(), || {
        let (mut net, ana) = three();
        let lapsed = net.phone((BO, ANA), &moved(1), (true, &[ANA, BEN]));
        let honest = net.phone((BO, ANA), &moved(2), (false, &[ANA]));
        let bo = lapsed.id().0;
        assert!(!lapsed.signed() && honest.signed());
        assert_eq!(
            (lapsed.id(), honest.id()),
            ((bo, 1), (bo, 2)),
            "seqs 1 and 2 of one site"
        );
        net.meet(ALL);
        let said = format!("  after the meet: {}", marks(&net, bo));
        let want = reference::state(TABLES, [&ana, &honest]);
        let mut wrong = Vec::new();
        for (c, name) in [(ANA, "Ana's"), (BEN, "Ben's"), (CY, "Cy's")] {
            let (real, haves) = net.copies[c].book();
            let counted = real.get(&bo).is_some_and(|m| *m >= 1) || haves.contains(&lapsed.id());
            if counted || net.books[c].holds(lapsed.id()) || keeps(&net, c, lapsed.id()) {
                wrong.push(format!("{name} copy keeps or counts the lapsed set"));
            }
            if net.copies[c].state() != want {
                wrong.push(format!("{name} copy is not the reference's"));
            }
        }
        (said, wrong)
    });
    println!("a lapsed pass, svrngs-db's copies:\n{said}");
    assert!(wrong.is_empty() && parted.is_empty(), "{wrong:#?}\n{parted:#?}");
}

fn ids(sets: &[ChangeSet]) -> Vec<String> {
    sets.iter().map(|s| named(s.id())).collect()
}
