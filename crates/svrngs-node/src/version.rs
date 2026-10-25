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
//! every answer at 1 is read as it was.

/// This build's version, as its answers carry it.
pub const VERSION: u32 = 2;

/// Whether an answer carrying `version`, none when it carries none, comes
/// from a computer running an svrngs older than any this one reads.
pub fn older(version: Option<u32>) -> bool {
    version.is_none_or(|v| v < 1)
}

/// Whether a computer whose answer carries `version` takes a pass carried
/// by a link (capability-signed).
pub fn links(version: Option<u32>) -> bool {
    version.is_some_and(|v| v >= 2)
}
