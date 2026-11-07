// SPDX-License-Identifier: AGPL-3.0-or-later
//! A version that cuts the last one off names, in a file here, who upgrades
//! first. `version::STEPS` makes the choice; this holds the note to it, so a
//! raise cannot point at an old note. Watched failing: DESIGN and
//! LENDING_SCOPE named neither version.
use svrngs_node::version::{Behind, STEPS, VERSION};

#[test]
fn a_version_that_cuts_off_the_last_names_who_upgrades_first() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert_eq!(VERSION as usize, STEPS.len());
    for (at, step) in STEPS.iter().enumerate() {
        if let Behind::FlagDay(note) = step {
            let said = std::fs::read_to_string(repo.join(note)).unwrap();
            let named = format!("wire version {}", at + 1);
            assert!(said.contains(&named), "{note} does not name {named}");
        }
    }
}
