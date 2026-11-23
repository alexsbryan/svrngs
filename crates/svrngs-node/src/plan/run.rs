// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Done, Method, Plan};
use crate::{
    lend::{self, Audience, Lend, Source},
    share::Granted,
};
use std::path::Path;

/// The access step of a confirmed plan. The caller supplies the binding and
/// grant verified by the access owner, after takeover if one was needed.
pub fn publish(dir: &Path, plan: &Plan<'_>, mut lend: Lend, (approved, keep): Granted) -> Done {
    let (nature, port) = (plan.nature, plan.nature.port);
    lend.name = nature.name.clone();
    lend.source = nature
        .raw
        .map_or(Source::Port(port), |raw| Source::Raw(raw, port));
    lend.terms.approved = Some(approved);
    lend.terms.keep = keep.filter(|_| !matches!(plan.method, Method::ThisRun));
    lend.terms.audience = Audience::Selected(plan.terms.groups.keys().cloned().collect());
    lend.terms.limit = plan.terms.limit;
    let saved = lend::publish(dir, lend, plan.if_rev);
    tracing::debug!(target: "plan", lend = %nature.name, ?saved, "plan: conditional share publication");
    match saved {
        Ok(rev) => Done::Published { rev },
        Err(e) => Done::NotPublished { why: e.to_string() },
    }
}
