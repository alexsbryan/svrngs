// SPDX-License-Identifier: AGPL-3.0-or-later
//! Original native source interpretation and scoped owned effects through
//! Eudoxia. Group standing is independently established by the native fold.
use crate::{
    grant::Why,
    pass::{carried_name, Label, Pass, PassFault, Whose},
    ContentHash, Op, OpId, RailAct, SignedOp,
};
use alloc::{collections::BTreeMap, format, vec::Vec};
use eudoxia::*;
mod evidence;
mod sources;
pub(crate) use evidence::interpreted;
pub use evidence::{Native, NativeAuthority, SourceRoots};
pub use sources::{
    AuthorityChange, AuthorityRef, GroupPolicy, GuestSlot, OfficeConflict, OfficeDispute,
    PeopleSnapshot, RecoveryChoice, SourceFailure, SourceFault, SourceMember, SourceOffice,
};

pub type Lines<'a> = BTreeMap<OpId, &'a Op<SignedOp>>;
pub(crate) type Judged<T> = Result<T, SourceFailure>;
fn id(s: &str) -> Id {
    Id::new(s).expect("fixed profile name or native content ID")
}
fn resource(tag: &str, label: &Label) -> Resource {
    Resource::new(format!("{tag}/{label}")).unwrap()
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Use<'a> {
    Reach(&'a Label),
    Record(&'a Label),
    Decide,
}
impl Use<'_> {
    fn operation(self) -> Operation {
        Operation::new(match self {
            Self::Reach(_) => "reach",
            Self::Record(_) => "record",
            Self::Decide => "decide",
        })
        .unwrap()
    }
    fn resource(self, pass: &Pass) -> Resource {
        match self {
            Self::Reach(lend) => {
                let tag = if *lend == pass.app { "own" } else { "read" };
                resource(tag, lend)
            }
            Self::Record(part) => resource("record", part),
            Self::Decide => resource("record", &pass.app),
        }
    }
}
fn source(original: &impl serde::Serialize) -> Fact {
    Fact {
        source: SourceRef {
            namespace: id("svrngs-native-pass-json-v1"),
            algorithm: id("blake3"),
            digest: ContentHash::of(&serde_json::to_vec(original).unwrap())
                .as_bytes()
                .to_vec(),
        },
        state: FactState::Established,
    }
}
fn session(issuer: Principal, raw: &str) -> Actor {
    Actor::Session {
        issuer,
        id: AuthorityId::session(issuer, &id("svrngs-native-pass-v1"), raw.as_bytes()),
    }
}
/// Original authenticated evidence, cached once per distinct native Pass. It
/// establishes only this key's delegated scope, never group standing.
pub struct NativeScope<'a> {
    pass: &'a Pass,
    profile: PersonalProfile,
}
impl<'a> NativeScope<'a> {
    pub fn new(pass: &'a Pass) -> Result<Self, Why> {
        if !pass.signed() {
            return Err(Why::Unsigned);
        }
        let issuer =
            Principal::new(*ContentHash::from_hex(&pass.person).unwrap().as_bytes()).unwrap();
        let scope = Scope {
            person: Some(issuer),
            application: None,
            coverage: Coverage::Set(
                core::iter::once(resource("own", &pass.app))
                    .chain(core::iter::once(resource("record", &pass.app)))
                    .chain(pass.reads.iter().map(|label| resource("read", label)))
                    .collect(),
            ),
            operations: [
                Use::Reach(&pass.app).operation(),
                Use::Record(&pass.app).operation(),
            ]
            .into(),
            interval: match pass.until {
                i64::MIN => Interval::empty(i64::MIN),
                until => Interval::new(i64::MIN, until).unwrap(),
            },
            redelegations: 1,
        };
        let profile = PersonalProfile::interpreted(
            id("svrngs-native-pass-v1"),
            i64::MIN,
            source(pass),
            scope,
            session(issuer, &pass.holder),
        )
        .unwrap();
        Ok(Self { pass, profile })
    }
    fn run<T>(
        &self,
        holder: &str,
        use_: Use,
        at: i64,
        purpose: Purpose,
        body: Vec<u8>,
        apply: impl FnOnce() -> T,
    ) -> Result<T, Why> {
        let actor = session(self.profile.principal(), holder);
        let result = self
            .profile
            .operation(at, actor, use_.operation(), use_.resource(self.pass))
            .commit(purpose, body, apply);
        tracing::debug!(target: "rail:authority", holder, at, allowed = result.is_ok(), error = ?result.as_ref().err(), "native: original scope through Eudoxia and owned commit");
        result.map_err(|_| {
            if !self.profile.actor_matches(actor) {
                Why::NotTheHolder
            } else if !self.profile.scope().interval.contains(at) {
                Why::Lapsed
            } else if use_ == Use::Decide {
                Why::Decides
            } else {
                Why::OtherKind
            }
        })
    }
    pub fn reach<T>(
        &self,
        holder: &str,
        lend: &Label,
        at: i64,
        apply: impl FnOnce() -> T,
    ) -> Result<T, PassFault> {
        self.run(
            holder,
            Use::Reach(lend),
            at,
            Purpose::Live,
            lend.as_str().as_bytes().to_vec(),
            apply,
        )
        .map_err(pass_fault)
    }
    pub fn select<'line>(
        &self,
        id: &OpId,
        line: &'line Op<SignedOp>,
        group: &str,
        held: &Lines<'line>,
        out: &mut Lines<'line>,
    ) -> Result<(), Why> {
        let approved = self.pass.op.as_ref();
        if approved.is_some_and(|id| *id != crate::grant::bare_id(line, group)) {
            return Err(Why::Unapproved);
        }
        let kind = match &line.kind.act {
            RailAct::Record { payload } => Some(payload.app()),
            RailAct::Correct {
                corrects,
                replacement,
            } => {
                let target = held.get(corrects).ok_or(Why::Unheld)?;
                let kind = target.kind.act.payload().map_or("", crate::Payload::app);
                if replacement.as_ref().is_some_and(|p| p.app() != kind) {
                    return Err(Why::OtherKind);
                }
                Some(kind)
            }
            _ => None,
        };
        let target = self.part(kind, group);
        let use_ = if kind.is_some() {
            Use::Record(&target)
        } else {
            Use::Decide
        };
        self.run(
            &line.actor,
            use_,
            line.ts_unix,
            Purpose::HistoricalAdmission,
            id.as_bytes().to_vec(),
            || {
                out.insert(id.clone(), line);
            },
        )
    }
    pub fn approving(&self, line: &Op<SignedOp>, group: &str, now: i64) -> Option<Pass> {
        if self.pass.op.is_some() || self.pass.approver.is_none() || line.ts_unix >= self.pass.until
        {
            return None;
        }
        if !matches!(
            line.kind.act,
            RailAct::Record { .. } | RailAct::Correct { .. }
        ) {
            return None;
        }
        let kind = line.kind.act.payload().map(crate::Payload::app);
        let target = self.part(kind, group);
        let body = crate::grant::bare_id(line, group);
        self.run(
            &line.actor,
            Use::Record(&target),
            now,
            Purpose::Live,
            body.as_bytes().to_vec(),
            || Pass {
                op: Some(body),
                ..self.pass.clone()
            },
        )
        .ok()
    }
    /// Source authentication for a carrier storing a line, not record scope or
    /// group standing. The fold separately decides whether that source counts.
    pub fn held_by(&self, holder: &str) -> bool {
        self.profile
            .actor_matches(session(self.profile.principal(), holder))
    }
    fn part(&self, kind: Option<&str>, group: &str) -> Label {
        kind.map_or_else(
            || self.pass.app.clone(),
            |kind| match self.pass.app.whose() {
                Whose::Carried => Label::carried(kind, group),
                Whose::Computer(key) => Label::of(&key, &carried_name(kind, group)),
            },
        )
    }
}
fn pass_fault(why: Why) -> PassFault {
    match why {
        Why::Unsigned => PassFault::Unsigned,
        Why::NotTheHolder => PassFault::NotTheHolder,
        Why::Lapsed => PassFault::Lapsed,
        _ => PassFault::NotNamed,
    }
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
    if let Some(link) = link {
        return carries(pass, link, holder, lend, now, apply);
    }
    reaching(pass)?.reach(holder, lend, now, apply)
}
/// `pass`'s scope for reach: an approval of one entry reaches nothing.
fn reaching(pass: &Pass) -> Result<NativeScope<'_>, PassFault> {
    if pass.op.is_some() {
        return Err(PassFault::NotReach);
    }
    NativeScope::new(pass).map_err(pass_fault)
}
pub fn carries<T>(
    pass: &Pass,
    link: &Pass,
    holder: &str,
    lend: &Label,
    now: i64,
    apply: impl FnOnce() -> T,
) -> Result<T, PassFault> {
    let child = reaching(link)?;
    child.reach(holder, lend, now, || ())?;
    let parent = reaching(pass)?;
    parent.reach(&link.person, lend, now, || ())?;
    let issuer = parent.profile.principal();
    let mut scope = child.profile.scope().clone();
    scope.person = Some(issuer);
    scope.redelegations = 0;
    let profile = parent
        .profile
        .attenuated(source(&(pass, link)), scope, session(issuer, &link.holder))
        .map_err(|_| PassFault::Wider)?;
    NativeScope { pass, profile }.reach(holder, lend, now, apply)
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum ApprovalFault {
    #[error("no write grant")]
    Missing,
    #[error("this app has no live authority here")]
    Open,
    #[error("this entry was not approved")]
    Entry,
}
pub fn approve_in_open(
    approver: &str,
    group: &str,
    reach: &Pass,
    issued: &Pass,
    line: &Op<SignedOp>,
    now: i64,
) -> Result<Pass, ApprovalFault> {
    let grant = line.kind.grant.as_ref().ok_or(ApprovalFault::Missing)?;
    let named = |p: &Pass| p.approver.as_deref() == Some(approver);
    let ours = named(reach)
        && named(grant)
        && grant == issued
        && reach.app.whose() == grant.app.whose()
        && now.abs_diff(line.ts_unix) <= 300;
    if !ours || check(reach, &line.actor, &reach.app, now).is_err() {
        return Err(ApprovalFault::Open);
    }
    NativeScope::new(grant)
        .ok()
        .and_then(|scope| scope.approving(line, group, now))
        .ok_or(ApprovalFault::Entry)
}
