// SPDX-License-Identifier: AGPL-3.0-or-later
//! A holder's pass and line, read for Eudoxia's community profile, which
//! decides what the pass lets its holder do by rules (svrn-docs
//! rings/current/EUDOXIA_JUDGE.md §3, R1); and a group's lines as the
//! evidence its interpreter reads (R2). svrngs keeps the bytes: signatures,
//! ids and labels. Group standing is the interpreter's, never a pass's.
use crate::{
    grant::bare_id,
    pass::{carried_name, Label, Pass, Whose},
    Op, OpId, Payload, RailAct, SignedOp,
};
use alloc::{collections::BTreeMap, string::ToString};
use eudoxia_community::pass as r1;
mod evidence;
mod sources;
pub(crate) use evidence::interpreted;
pub use evidence::{Native, NativeAuthority, SourceRoots};
pub use r1::{ApprovalFault, PassFault, Why};
pub use sources::{
    AuthorityChange, AuthorityRef, GroupPolicy, OfficeConflict, OfficeDispute, PeopleSnapshot,
    RecoveryChoice, SourceFailure, SourceFault, SourceMember, SourceOffice,
};

pub type Lines<'a> = BTreeMap<OpId, &'a Op<SignedOp>>;
pub(crate) type Judged<T> = Result<T, SourceFailure>;

/// `pass` as the profile reads it, its signatures verified here.
pub fn read(pass: &Pass) -> r1::Pass<'_> {
    r1::Pass {
        person: &pass.person,
        key: hex::decode(&pass.person)
            .ok()
            .and_then(|k| k.try_into().ok()),
        holder: &pass.holder,
        app: pass.app.as_str(),
        reads: pass.reads.iter().map(Label::as_str).collect(),
        until: pass.until,
        approver: pass.approver.as_deref(),
        approves: pass.op.as_ref().map(ToString::to_string),
        signed: pass.signed(),
        lender: match pass.app.whose() {
            Whose::Carried => None,
            Whose::Computer(key) => Some(key),
        },
    }
}

/// `line` as the profile reads it: the part each kind it names writes, under
/// `pass`'s app, and the corrected line's where `held` holds it.
pub(crate) fn entry<'l>(
    pass: &r1::Pass,
    line: &'l Op<SignedOp>,
    group: &str,
    content: &'l str,
    held: &Lines,
) -> r1::Entry<'l> {
    let part = |kind: &str| r1::Part {
        kind: kind.into(),
        label: match pass.lender {
            None => Label::carried(kind, group),
            Some(key) => Label::of(&key, &carried_name(kind, group)),
        }
        .to_string(),
    };
    let act = match &line.kind.act {
        RailAct::Record { payload } => r1::Act::Write(part(payload.app())),
        RailAct::Correct {
            corrects,
            replacement,
        } => r1::Act::Correct {
            target: held
                .get(corrects)
                .map(|t| part(t.kind.act.payload().map_or("", Payload::app))),
            replacement: replacement.as_ref().map(|p| part(p.app())),
        },
        _ => r1::Act::Decide,
    };
    r1::Entry {
        actor: &line.actor,
        written: line.ts_unix,
        content,
        act,
    }
}
/// `line`, a holder's under `pass` (already [`read`]), counted into `out`
/// only if the profile's rules let it stand in its person's place.
pub fn select<'line>(
    pass: &r1::Pass,
    id: &OpId,
    line: &'line Op<SignedOp>,
    group: &str,
    held: &Lines<'line>,
    out: &mut Lines<'line>,
) -> Result<(), Why> {
    let content = bare_id(line, group).to_string();
    let entry = entry(pass, line, group, &content, held);
    let selected = r1::entry(pass, &entry, || {
        out.insert(id.clone(), line);
    });
    tracing::debug!(target: "rail:authority", %id, actor = %line.actor, error = ?selected.err(), "pass: a holder's line, by the community profile's rules");
    selected
}

pub fn check(pass: &Pass, holder: &str, lend: &Label, now: i64) -> Result<(), PassFault> {
    resolve(pass, None, holder, lend, now, || ())
}
pub fn resolve<T>(
    pass: &Pass,
    link: Option<&Pass>,
    holder: &str,
    lend: &Label,
    now: i64,
    apply: impl FnOnce() -> T,
) -> Result<T, PassFault> {
    let reached = r1::reach(
        &read(pass),
        link.map(read).as_ref(),
        holder,
        lend.as_str(),
        now,
        apply,
    );
    tracing::debug!(target: "rail:authority", holder, %lend, now, error = ?reached.as_ref().err(), "pass: reach, by the community profile's rules");
    reached
}

/// Source authentication for a carrier storing a line, not record scope or
/// group standing. The fold separately decides whether that source counts.
pub fn held(pass: &Pass, holder: &str) -> bool {
    r1::held(&read(pass), holder)
}

/// The Wasm signer's approval of `line` in `group` for the open whose pass
/// is `reach` and which issued `issued`, by `approver`, at `now`.
pub fn approve_in_open(
    approver: &str,
    group: &str,
    reach: &Pass,
    issued: &Pass,
    line: &Op<SignedOp>,
    now: i64,
) -> Result<Pass, ApprovalFault> {
    let grant = line.kind.grant.as_ref();
    let (opener, writes) = (read(reach), read(grant.unwrap_or(issued)));
    let content = bare_id(line, group);
    let text = content.to_string();
    let entry = entry(&writes, line, group, &text, &Lines::new());
    let opened = r1::open(
        approver,
        &opener,
        grant.map(|g| (&writes, g == issued)),
        &entry,
        now,
    );
    tracing::debug!(target: "rail:authority", %approver, error = ?opened.err(), "pass: an open's approval, by the community profile's rules");
    opened?;
    let grant = grant.ok_or(ApprovalFault::Missing)?;
    Ok(Pass {
        op: Some(content),
        ..grant.clone()
    })
}
