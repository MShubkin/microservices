use asez2_tables::traits::{HasId, HasPlanStatusId, HasUuid};

/// For now we will use the static rules lawyer trait, where the rules
/// for each structure are determined beforehand in the function itself.
///
/// Basically this is a mock.
pub trait RulesLawyer:
    HasPlanStatusId + HasId + HasUuid + PartialEq + Sized + Sync
{
}
