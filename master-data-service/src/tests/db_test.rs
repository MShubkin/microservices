#![cfg(test)]

use std::sync::Arc;

use asez2_shared_db::DbItem;
use shared_essential::domain::{
    favorites::FavoriteList,
    plan_reasons_cancel::{PlanReasonCancelCustomer, PlanReasonCancelHeader},
    routes::{RouteCrit, RouteData, RouteHeader, RouteType},
    scheduler_calendar::scheduler_update_catalog_request::SchedulerRequestUpdateCatalog,
    OrganizationalStructure, OrganizationalUserAssignment,
};
use sqlx::PgPool;

/// Таблицы НСИ, которые должны чиститься перед каждым тестом.
const NSI_TRANSIENT_TABLES: &[&str] = &[
    FavoriteList::TABLE,
    RouteHeader::TABLE,
    RouteCrit::TABLE,
    RouteData::TABLE,
    RouteType::TABLE,
    PlanReasonCancelHeader::TABLE,
    PlanReasonCancelCustomer::TABLE,
    OrganizationalUserAssignment::TABLE,
    OrganizationalStructure::TABLE,
    SchedulerRequestUpdateCatalog::TABLE,
];

pub(super) async fn run_db_test<F, FutFn>(
    extra_migs_files: &'static [&'static str],
    run: FutFn,
) where
    F: futures::Future<Output = ()>,
    FutFn: FnOnce(Arc<PgPool>) -> F + 'static,
{
    testing::BaseMigPath::MigrationsHome
        .run_test_with_migrations(
            "src/tests/extra_migrations",
            extra_migs_files,
            NSI_TRANSIENT_TABLES,
            run,
        )
        .await
}
