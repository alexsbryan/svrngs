// SPDX-License-Identifier: AGPL-3.0-or-later
//! Adoption (svrn-docs `rings/current/EUDOXIA_JUDGE.md` §2e): which decider a
//! group is on, and the check its keeper's Adopt passes before the
//! interpreter takes the pinned old-fold result as its roots. The closure is
//! every line on the `prev` chains of the Adopt's heads and of its own
//! `prev`, so lawfulness is a function of the Adopt and that closure alone:
//! nothing held later changes it, and once lawful the group never reverts.
use super::{GroupPolicy, LegacyAdoption, LegacyState, Lines, SourceMember, SourceRoots};
use crate::{ActSet, ContentHash, Decided, Device, Journaled, OpId, RailAct, RailGap};
use crate::{RingVerifier, Roster};
use alloc::{collections::BTreeSet, string::String, vec::Vec};
use eudoxia_community::Roots;

/// Why an Adopt is not lawful, traced at debug where it is found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unlawful {
    /// A line of its closure is not held here: it may be, once synced.
    Missing,
    /// Its closure holds a line of the new kinds that is no Ready, or a
    /// cut through a line outside it.
    Closure,
    /// The old fold over the closure did not settle.
    Unsettled,
    /// Its author is not the old fold's keeper over the closure.
    NotKeeper,
    /// The keeper's key does not stand as a computer.
    Computer,
    /// Its author handed the office on since it last became keeper.
    HandedOn,
    /// Its author's chain forks inside the closure.
    Fork,
    /// A computer standing in the closure's result signed no Ready there
    /// naming the author, with every tip it held inside the closure.
    Ready,
    /// What it pins is not what this computer's old fold gives.
    Pin,
}

/// The group's one decider over its signature-checked lines (§2e §2), with
/// an adopted group's roots. An Adopt whose author never held the office,
/// neither the seed's key nor a hand-on's, is dropped unread. Of lawful
/// Adopts the least by id is the epoch, so no arrival order picks; `known`,
/// an Adopt this computer found lawful before, skips the old fold.
pub(crate) fn decider(
    roster: &Roster,
    held: &Lines<'_>,
    ns: &str,
    verifier: &dyn RingVerifier,
    known: Option<&OpId>,
) -> (Decided, Option<super::Judged<SourceRoots>>, Option<Vec<u8>>) {
    if roster.policy.is_some() {
        let roots = SourceRoots::seeded(roster, ns);
        return (Decided::Founded, Some(roots), None);
    }
    let seeded = roster.members.values().flatten().next();
    let to = |act: &RailAct, k: &str| matches!(act, RailAct::HandOn { key } if key == k);
    let handed = |k: &str| held.values().any(|op| to(&op.kind.act, k));
    let mut waits = None;
    for (id, op) in held {
        let Some(adoption) = op.kind.act.adoption() else {
            continue;
        };
        if seeded != Some(&op.actor) && !handed(&op.actor) {
            tracing::debug!(target: "rail:adopt", %id, actor = %op.actor, "adoption: dropped unread, its author never held the office");
            continue;
        }
        let mut bytes = None;
        let lawful = closure(held, adoption.heads.iter().chain(&op.kind.prev)).and_then(|lines| {
            let namespace = ns.into();
            if known != Some(id) {
                let at = (op.actor.as_str(), op.kind.prev.as_ref());
                let (pinned, result) = pin(&lines, roster, ns, verifier, at, &adoption.heads)?;
                (pinned == *adoption).then_some(()).ok_or(Unlawful::Pin)?;
                bytes = Some(result);
            }
            let closure = lines.into_keys().collect();
            let model =
                Roots::adopted(id.clone(), adoption, closure).map_err(|_| Unlawful::Computer)?;
            Ok(SourceRoots { namespace, model })
        });
        tracing::debug!(target: "rail:adopt", %id, known = known == Some(id), unlawful = ?lawful.as_ref().err(), "adoption: an Adopt judged over its closure alone");
        match lawful {
            Ok(roots) => return (Decided::Adopted(id.clone()), Some(Ok(roots)), bytes),
            Err(Unlawful::Missing) => drop(waits.get_or_insert_with(|| id.clone())),
            Err(_) => {}
        }
    }
    (waits.map_or(Decided::Legacy, Decided::Waiting), None, None)
}

/// Every line on the `prev` chains from `heads`, each held here and, by its
/// kind and never by the unsigned `v`, below line version 15 but a Ready.
fn closure<'a, 'h>(
    held: &Lines<'a>,
    heads: impl Iterator<Item = &'h OpId>,
) -> Result<Lines<'a>, Unlawful> {
    let mut pending: Vec<&OpId> = heads.collect();
    let mut out = Lines::new();
    while let Some(id) = pending.pop() {
        if out.contains_key(id) {
            continue;
        }
        let Some(op) = held.get(id) else {
            tracing::debug!(target: "rail:adopt", %id, "adoption: a closure line is not held here");
            return Err(Unlawful::Missing);
        };
        if op.kind.line_version() >= crate::AUTHORITY_LINE_VERSION && op.kind.act.ready().is_none()
        {
            tracing::debug!(target: "rail:adopt", %id, "adoption: the closure holds a line of the new kinds");
            return Err(Unlawful::Closure);
        }
        out.insert(id.clone(), *op);
        pending.extend(&op.kind.prev);
    }
    Ok(out)
}

/// What an Adopt by `author` over `closure` pins, from the old fold over it;
/// or why it may not be signed. The one decider of both the Adopt a
/// keeper's computer writes and every computer's check of it.
fn pin(
    closure: &Lines<'_>,
    roster: &Roster,
    ns: &str,
    verifier: &dyn RingVerifier,
    (author, at): (&str, Option<&OpId>),
    heads: &BTreeSet<OpId>,
) -> Result<(LegacyAdoption, Vec<u8>), Unlawful> {
    let acts = ActSet::of(closure.values().copied());
    let fold = crate::admit(&acts, &[], roster, ns, verifier);
    let m = fold.membership();
    // The old fold's own test (`keeps`): an office act at the Adopt's place,
    // after `at`, counts only if no hand-on of the author's comes before it.
    let chain: Vec<_> = closure.iter().map(|(id, op)| (id, *op)).collect();
    let handed = at.is_some_and(|at| crate::membership::spent(&chain).contains(at));
    let ready = |key: &String| {
        let at = |t: &OpId| closure.contains_key(t);
        let named =
            |(tips, keeper): (&BTreeSet<OpId>, &str)| keeper == author && tips.iter().all(at);
        (closure.values()).any(|op| op.actor == *key && op.kind.act.ready().is_some_and(named))
    };
    let unready = !(m.standing.iter()).all(|k| m.device(k) != Some(Device::Computer) || ready(k));
    let gap = |g: &RailGap| match g {
        RailGap::Unsettled => Some(Unlawful::Unsettled),
        RailGap::UnheldCut { .. } => Some(Unlawful::Closure),
        RailGap::SequenceFork { actor, .. } if actor == author => Some(Unlawful::Fork),
        _ => None,
    };
    let phone = m.device(author) != Some(Device::Computer);
    let checks = [
        // A lone leaver stays the old fold's keeper; it stands nowhere (B3).
        (
            m.keeper.as_deref() != Some(author) || !m.standing.contains(author),
            Unlawful::NotKeeper,
        ),
        (phone, Unlawful::Computer),
        (handed, Unlawful::HandedOn),
        (unready, Unlawful::Ready),
    ];
    let failed = checks.iter().find(|(fails, _)| *fails).map(|(_, why)| *why);
    if let Some(why) = fold.gaps().iter().find_map(gap).or(failed) {
        tracing::debug!(target: "rail:adopt", author, ?why, "adoption: not lawful over its closure");
        return Err(why);
    }
    let admitted: BTreeSet<&OpId> = fold.ops().iter().map(|o| &o.id).collect();
    let uncounted = closure.keys().filter(|id| !admitted.contains(id));
    let member = |(key, person): (&String, &crate::Person)| {
        let first = m.devices.get(key).unwrap_or(key).clone();
        // A cut key never stands, so it is held by nothing now.
        let member = SourceMember::root(first, person.clone(), m.device(key).unwrap_or_default());
        (key.clone(), member)
    };
    // The old fold's exact bytes an Adopt pins: its membership, and the
    // ids it admitted.
    let result = serde_json::json!({ "membership": m, "admitted": admitted });
    let bytes = serde_json::to_vec(&result).expect("a fold serialises");
    let state = LegacyState {
        joined: m.admitted_at.clone(),
        meets: m.meets.clone(),
        signers: m.signers.clone(),
        holders: m.holders.clone(),
        visits: m
            .visits
            .iter()
            .map(|(id, g)| (id.clone(), g.stopped))
            .collect(),
    };
    let adoption = LegacyAdoption {
        policy: GroupPolicy::SourcesV1,
        heads: heads.clone(),
        uncounted: uncounted.cloned().collect(),
        result: legacy_result_pin(ns, &bytes),
        members: m.bindings.iter().map(member).collect(),
        out: m
            .cut
            .iter()
            .map(|(k, c)| (k.clone(), (c.by.clone(), c.left)))
            .collect(),
        office: author.into(),
        state,
    };
    Ok((adoption, bytes))
}

const LEGACY_RESULT_DOMAIN: &[u8] = b"svrngs/legacy-result/v1\0";

/// An adoption pins the old fold's exact bytes in its group: the domain,
/// `namespace`, which holds no NUL, then NUL, then the bytes. The pin
/// proves those bytes; they alone give no authority.
pub fn legacy_result_pin(namespace: &str, bytes: &[u8]) -> ContentHash {
    ContentHash::of(&[LEGACY_RESULT_DOMAIN, namespace.as_bytes(), b"\0", bytes].concat())
}

/// The Adopt this computer's key `author` may sign over every line it holds,
/// its heads every Ready naming it, their tips and its own lines; or why not.
pub fn adoption(
    held: &ActSet,
    roster: &Roster,
    ns: &str,
    verifier: &dyn RingVerifier,
    author: &str,
) -> Result<LegacyAdoption, Unlawful> {
    let (lines, _) = super::evidence::originals(ns, held, verifier);
    let mut heads = BTreeSet::new();
    for (id, op) in &lines {
        match op.kind.act.ready() {
            Some((tips, keeper)) if keeper == author => {
                heads.extend(tips.iter().chain([id]).cloned())
            }
            _ if op.actor == author => drop(heads.insert(id.clone())),
            _ => {}
        }
    }
    let at = (lines.iter())
        .filter(|(_, op)| op.actor == author)
        .max_by_key(|(_, op)| op.kind.seq);
    tracing::debug!(target: "rail:adopt", author, heads = heads.len(), "adoption: the heads an Adopt would name");
    let closure = closure(&lines, heads.iter())?;
    pin(
        &closure,
        roster,
        ns,
        verifier,
        (author, at.map(|(id, _)| id)),
        &heads,
    )
    .map(|(a, _)| a)
}
