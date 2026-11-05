// SPDX-License-Identifier: AGPL-3.0-or-later
//! This build's version, and the one check made of it (arch-version; AUDIT
//! §2.5, "one version check instead of compatibility per field"). Every
//! answer a page reads of a computer carries it: the people's lends a page
//! asks the computer it reaches (`node.rs` `people_lends`), and each
//! computer's own lends, which that computer asks of the others
//! (`door.rs` `/lends`, `node.rs` `ask_lends`). One whose answer carries a
//! lower version, or none, runs an older svrngs: said once, by whose
//! computer it is, and never read field by field. The phone's page compiles
//! this file by `#[path]` (`web/src/lib.rs` `older`), so a page and a
//! computer ask one rule.
//!
//! Raised when an answer a page reads changes what it carries. 1 is the
//! first: no build that left this host carries one
//! (`ralph/next/arch/arch-version/SURVEY.md`), and the operator let those
//! go (2026-10-15, "Let them go; V1's publish replaces them"). 2 is the
//! first whose door takes a pass carried by its holder's link
//! (capability-signed; `door.rs` `LINK_HEADER`): a page opening a lend at
//! a computer at 1 hands it a holder as bytes, as before (DESIGN §3), and
//! every answer at 1 is read as it was. 3 names each person whose computer
//! did not say what they lend by their first key beside their name
//! (keys-names; `node.rs` `Named`), where 2 named them by name: a page reads
//! a name alone at 2 as before (question.js `door`). 4 is the holders'
//! flag day (svrn-docs rings/current/HOLDER_APPROVAL.md): an app's entry
//! from a phone counts only with its open's approver's approval of its exact
//! content, which a computer below 4 neither asks for nor reads (line
//! version 13), so an answer below 4 is older and not read; the group's
//! keeper upgrades first (docs/releases/). 5 is selected-group lending
//! (LENDING_SCOPE): a lend's list answer carries the stable group IDs it is
//! lent to, and a list or lend request from a caller that knows to send
//! [`crate::version::VERSION`] in [`SCOPE_HEADER`] may read them. An old
//! consumer below 5 accepts a 5 answer, so [`older`] alone does not fence
//! it: a lender refuses an unmarked discovery request, and a consumer
//! refuses an unscoped answer, through the scope marker beside the version.

/// This build's version, as its answers carry it.
pub const VERSION: u32 = 5;

/// The header a caller sends to declare it speaks selected-group scope
/// (LENDING_SCOPE §"Updates, migration and mixed versions"): its value is
/// [`VERSION`]. Compatibility data, never authorization — a caller that
/// sends it still passes the lender's actual audience and membership
/// checks. A lender refuses a discovery request without it, and a consumer
/// refuses a discovery answer whose messages do not carry scope.
pub const SCOPE_HEADER: &str = "x-svrngs-version";

/// Whether an answer carrying `version`, none when it carries none, comes
/// from a computer running an svrngs older than any this one reads.
pub fn older(version: Option<u32>) -> bool {
    version.is_none_or(|v| v < VERSION)
}

/// Whether a computer whose answer carries `version` takes a pass carried
/// by a link (capability-signed).
pub fn links(version: Option<u32>) -> bool {
    version.is_some_and(|v| v >= 2)
}
