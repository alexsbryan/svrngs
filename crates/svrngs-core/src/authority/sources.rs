//! Native wire aliases. Eudoxia owns community authority meaning.
use crate::{ContentHash, Device, OpId, Person};
use eudoxia_community as model;
pub use model::{GroupPolicy, GuestSlot, SourceFault};
pub type AuthorityRef = model::AuthorityRef<OpId>;
pub type PeopleSnapshot = model::PeopleSnapshot<OpId, Person>;
pub type SourceMember = model::SourceMember<OpId, Person, Device>;
pub type AuthorityChange = model::AuthorityChange<OpId, Person, Device, ContentHash>;
pub type OfficeDispute = model::OfficeDispute<OpId>;
pub type RecoveryChoice = model::RecoveryChoice<OpId, Person>;
pub type LegacyAdoption = model::LegacyAdoption<OpId, Person, Device, ContentHash>;
pub type LegacyState = model::LegacyState<OpId>;
pub type SourceFailure = model::SourceFailure<OpId>;
pub type SourceOffice = model::SourceOffice<OpId>;
pub type OfficeConflict = model::OfficeConflict<OpId>;
