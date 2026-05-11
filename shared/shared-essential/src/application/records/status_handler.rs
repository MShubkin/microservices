use crate::presentation::dto::response_request::Messages;

use super::rules_lawyer::RulesLawyer;

/// TODO
#[async_trait::async_trait]
pub trait StatusHandler {
    type Error: std::error::Error + 'static;

    async fn check_insert<T: RulesLawyer>(
        &self,
        new: &[T],
        messages: &mut Messages,
    ) -> Result<bool, Self::Error>;
    async fn check_update<T: RulesLawyer>(
        &self,
        fields_to_update: &[&str],
        new: &[T],
        old: &[T],
        messages: &mut Messages,
    ) -> Result<bool, Self::Error>;
    async fn check_upsert<T: RulesLawyer>(
        &self,
        fields_to_update: &[&str],
        new: &[T],
        old: &[T],
        messages: &mut Messages,
    ) -> Result<bool, Self::Error>;
}
