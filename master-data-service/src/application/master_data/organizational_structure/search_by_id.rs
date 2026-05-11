use asez2_shared_db::{db_item::Select, DbAdaptor};
use shared_essential::{
    domain::{OrganizationalStructure, OrganizationalStructureRep},
    presentation::dto::master_data::error::MasterDataResult,
};
use sqlx::PgPool;

const FIELDS: &[&str] = &[
    OrganizationalStructure::uuid,
    OrganizationalStructure::id,
    OrganizationalStructure::text,
    OrganizationalStructure::text_short,
    OrganizationalStructure::level,
    OrganizationalStructure::parent_id,
    OrganizationalStructure::dep_type,
    OrganizationalStructure::created_at,
    OrganizationalStructure::created_by,
    OrganizationalStructure::changed_by,
    OrganizationalStructure::changed_at,
];

pub(crate) async fn search_by_id<'a, I>(
    ids: I,
    pool: &PgPool,
) -> MasterDataResult<Vec<OrganizationalStructureRep>>
where
    I: IntoIterator<Item = &'a i32>,
{
    let items = OrganizationalStructureRep::select(
        &Select::with_fields(FIELDS)
            .eq(OrganizationalStructure::is_removed, false)
            .eq(OrganizationalStructure::is_specialized_department, true)
            .in_any(OrganizationalStructure::id, ids),
        pool,
    )
    .await?;

    Ok(items)
}
