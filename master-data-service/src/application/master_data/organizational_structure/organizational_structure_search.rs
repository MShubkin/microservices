use asez2_shared_db::db_item::AdaptorableIter;
use asez2_shared_db::db_item::Select;
use asez2_shared_db::DbItem;
use shared_essential::{
    domain::{DepartmentLevel, OrganizationalStructure},
    presentation::dto::master_data::error::MasterDataResult,
};
use sqlx::PgPool;

use crate::application::master_data::org_user_assignment_search;
use crate::presentation::dto::{
    OrganizationalStructureSearchReqBody, OrganizationalStructureSearchResData,
};

pub(crate) async fn organizational_structure_search(
    user_id: i32,
    request: OrganizationalStructureSearchReqBody,
    db_pool: &PgPool,
) -> MasterDataResult<OrganizationalStructureSearchResData> {
    let OrganizationalStructureSearchReqBody {
        from,
        quantity,
        search,
        mut organization_structure_id,
        level,
        is_specialized_department,
    } = request;

    if organization_structure_id.is_none() {
        organization_structure_id =
            get_user_spec_dep_org_assignment(user_id, db_pool).await?
    };

    let select = Select::full::<OrganizationalStructure>()
        .eq(OrganizationalStructure::is_removed, false)
        .eq_maybe(
            OrganizationalStructure::is_specialized_department,
            is_specialized_department,
        )
        .fields_containing(
            [OrganizationalStructure::text, OrganizationalStructure::text_short],
            format!("(?i){search}"),
        )
        .offset(from as usize)
        .take_n(quantity as usize);

    let select = if let Some(level) = level {
        select.eq(OrganizationalStructure::level, level)
    } else {
        select.greater(OrganizationalStructure::level, DepartmentLevel::Department)
    };

    let result = if let Some(id) = organization_structure_id {
        let initial_select = Select::default().eq(OrganizationalStructure::id, id);
        OrganizationalStructure::select_recursive(
            &initial_select,
            OrganizationalStructure::id,
            OrganizationalStructure::parent_id,
            &select,
            db_pool,
        )
        .await
    } else {
        OrganizationalStructure::select(&select, db_pool).await
    }?;

    // Применяем на результат пагинацию и возвращаем что получилось
    let value = result
        .into_iter()
        .adaptors_with_fields(OrganizationalStructure::FIELDS)
        .collect();
    Ok(OrganizationalStructureSearchResData { value })
}

/// Поиск идентификатора ПД, к которому приписан пользователь.
async fn get_user_spec_dep_org_assignment(
    user_id: i32,
    pool: &PgPool,
) -> MasterDataResult<Option<i32>> {
    // все департаменты (level = 2), к которым приписан пользователь
    let department_ids =
        org_user_assignment_search::organizational_user_assignment_by_id(&[
            user_id,
        ])
        .await?
        .value
        .first()
        .map_or_else(Vec::new, |x| {
            x.organization_structure_ids
                .iter()
                // на втором месте находится идентификатор департамента
                .filter_map(|x| x.get(1).copied())
                .collect()
        });

    if department_ids.is_empty() {
        return Ok(None);
    }

    let org_struct_id = OrganizationalStructure::select_option(
        &Select::with_fields([OrganizationalStructure::id])
            .in_any(OrganizationalStructure::id, department_ids)
            .eq(OrganizationalStructure::is_specialized_department, true),
        pool,
    )
    .await?
    .map(|x| x.id);
    Ok(org_struct_id)
}
