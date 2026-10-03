//! svrngs-core: a group's record (DESIGN C1), as a pure, synchronous state
//! machine. It never touches a socket, a disk, a clock or a key: the caller
//! hands it bytes, the time and signatures, and it hands back acts, outcomes
//! and the fold.
//!
//! - [`chain`]: the act on the wire, its id and signature, and
//!   [`VerifiedAct::parse`], the one way bytes become an act.
//! - [`ingest`]: [`Journal::decide`] and [`Journal::apply`] — what a copy holds.
//! - [`admit`]: [`fold`] — order and void.
//! - [`guests`]: the guest book — whom the bridge may act for.

pub mod admit;
pub mod chain;
pub mod guests;
pub mod ingest;
pub mod payload;

pub use admit::{Folded, fold};
pub use chain::{
    ActId, Actor, Admit, AppId, Body, Correct, Draft, Genesis, GroupId, Guest, GuestEvent, Interaction, Name, Nonce,
    OnBehalfOf, ParseError, Record, Role, Sig, Speaker, VerifiedAct, admit_message,
};
pub use guests::{GuestBook, GuestChange, GuestEntry};
pub use ingest::{Holdable, Ingest, Journal, MAX_SKEW_SECS, Standing, Why};
pub use payload::{MAX_PAYLOAD_BYTES, Payload, PayloadError};

#[cfg(test)]
mod tests;
