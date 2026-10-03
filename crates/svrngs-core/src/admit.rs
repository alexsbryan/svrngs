//! Order and void (DESIGN C1, "Admit"): one order for every copy,
//! `(ts, actor, seq, id)`, and the void set built from every `Correct` at once,
//! so it is a function of the set and nothing comes back to life.

use std::collections::BTreeSet;

use crate::chain::{ActId, Body, OnBehalfOf, Role, VerifiedAct};
use crate::ingest::Journal;

/// One act in the fold's order.
#[derive(Debug, Clone, Copy)]
pub struct Folded<'a> {
    pub act: &'a VerifiedAct,
    /// A `Correct` of the same app names it. Membership acts are never voided.
    pub voided: bool,
    /// Whom the act speaks for: only from a Bridge-role key (DESIGN C2).
    pub speaker: Option<&'a OnBehalfOf>,
}

/// Every held act, in order.
pub fn fold(journal: &Journal) -> Vec<Folded<'_>> {
    let voided: BTreeSet<&ActId> = journal
        .acts()
        .filter_map(|a| match a.body() {
            Body::Correct(c) => Some(c),
            _ => None,
        })
        .filter(|c| {
            journal
                .get(&c.corrects)
                .is_some_and(|t| app_of(t.body()) == Some(&c.app))
        })
        .map(|c| &c.corrects)
        .collect();
    let mut order: Vec<&VerifiedAct> = journal.acts().collect();
    order.sort_by_key(|a| (a.ts(), *a.actor(), a.seq(), *a.id()));
    order
        .into_iter()
        .map(|act| Folded {
            act,
            voided: voided.contains(act.id()),
            speaker: act
                .body()
                .on_behalf_of()
                .filter(|_| journal.standing(act.actor()).is_some_and(|s| s.role == Role::Bridge)),
        })
        .collect()
}

fn app_of(body: &Body) -> Option<&crate::chain::AppId> {
    match body {
        Body::Record(r) => Some(&r.app),
        Body::Correct(c) => Some(&c.app),
        Body::Admit(_) => None,
    }
}
