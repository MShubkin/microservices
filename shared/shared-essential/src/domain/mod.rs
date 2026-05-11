/// Доменный уровень - это уровень с бизнес-логикой.
/// В идеале, доменный уровень не знает о существовании других уровней.
/// Здесь содержатся специальные, не совсем dto, которые используются для таблиц и переноса.
pub use asez2_tables as tables;
pub use asez2_tables::PlanOrAmendment;
pub use master_data::*;
pub use tables::traits::*;
pub use tables::*;

pub mod enums;
pub mod master_data;
