// SPDX-License-Identifier: AGPL-3.0-or-later
//! Jellyfin's middleware (demo/runs/mw/2026-10-08/README.md): the lender
//! runs it on loopback in front of their Jellyfin and lends its port.
//!
//! This first commit lands upstream's `provision` as it was (DESIGN §4):
//! nothing calls it yet, so its items are allowed to be unused.
#![allow(dead_code)]

mod viewer;

fn main() {}
