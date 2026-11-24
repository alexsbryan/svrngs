//! What a copy does with a change that would bring a row back when the set
//! that re-added the row is missing (svrn-docs APP_STATE.md §8, question 9):
//! today's rule, upstream cr-sqlite's, and the three candidates slice B
//! decided among (MAGNETS_BUILD.md §3, "Slice B"). It chose N (svrn-docs
//! `rings/current/app-state-diligence/q9-rules-2026-11-22/`), and the
//! fork's engine keeps it from the pin `change.rs` names. A model copy is built with one
//! (`ModelCopy::ruled`), and the reference judges its copies by the same one
//! (`reference::under`). The others stay as variants: today's is the plant
//! N's checks are watched failing on, and H and P are their record.
//!
//! The moment they differ on: a column change at a longer odd causal length
//! than a copy holds a row at brings the row back first
//! (`changes_vtab_write.rs:620-645`). Under today's rule, a copy that held
//! the row alive keeps its earlier life's values, each at col_version 0
//! (`tableinfo.rs:440-454`, the `INSERT OR IGNORE` at `:391-406`); a copy
//! without it makes the row from its key alone; and a change at the shorter
//! length that arrives later is dropped (`changes_vtab_write.rs:511-513`).
//! So where the set that re-added the row is missing, what a copy ends on
//! depends on what it held. Under N it doesn't (`:621-625`, `:744-803`).

/// A copy's rule for that moment. `Rule::default()` is the one svrngs keeps.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Rule {
    /// Upstream cr-sqlite, and the fork before its patch (6a59c477). The
    /// reference is the join of every admissible set.
    Today,
    /// N, "a new life starts empty", the rule svrngs keeps: the column change
    /// brings the row back as a copy that never held it makes it, its other
    /// columns carrying nothing from its earlier life. The reference is the
    /// join, unchanged; the fork's engine changed to match it.
    #[default]
    NewLifeEmpty,
    /// H, "a column change waits for its row's life": a set is held while any
    /// of its column changes is at a causal length past the copy's for which
    /// neither the copy nor the set holds the row's row-level change. A held
    /// set is kept, relayed and judged again at every take, and applies once
    /// that change arrives. The reference joins the sets the same rule admits,
    /// as a fixpoint over the admissible ones.
    WaitForLife,
    /// P, contiguous per device (APP_STATE's first suggestion): a device's set
    /// applies only once every earlier set of the device is counted in this
    /// copy's mark for it. A newcomer, and a member after a full pull from
    /// another, adopt the source's marks (APP_STATE §4, "What travels"). The
    /// reference joins each device's admissible sets below its first missing
    /// one.
    Contiguous,
}

impl Rule {
    pub const ALL: [Rule; 4] = [Rule::Today, Rule::NewLifeEmpty, Rule::WaitForLife, Rule::Contiguous];

    /// The rule's name in the brief and the report: today, N, H or P.
    pub fn name(self) -> &'static str {
        match self {
            Rule::Today => "today",
            Rule::NewLifeEmpty => "N",
            Rule::WaitForLife => "H",
            Rule::Contiguous => "P",
        }
    }

    /// A rule by its name, as `W1_RULES` lists them.
    pub fn named(name: &str) -> Rule {
        Rule::ALL
            .into_iter()
            .find(|r| r.name() == name)
            .unwrap_or_else(|| panic!("no rule {name}: today, N, H or P"))
    }
}
