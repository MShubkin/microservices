use std::sync::Arc;

use asez2_shared_db::{db_item::Select, DbItem};
use shared_essential::{
    domain::PartnerTypeCommission,
    presentation::dto::{
        processing::{
            GetPartnersReq, GetPartnersResponseData, GetPartnersResponseItem,
        },
        response_request::ApiResponse,
    },
};
use sqlx::PgPool;

use crate::common::Result;

const PARTNER_FIELDS: &[&str] = &[
    PartnerTypeCommission::uuid,
    PartnerTypeCommission::user_id,
    PartnerTypeCommission::role_id,
];

pub(crate) async fn get_partners(
    req: GetPartnersReq,
    db_pool: Arc<PgPool>,
) -> Result<ApiResponse<GetPartnersResponseData, ()>> {
    let GetPartnersReq { protocol_type_id } = req;

    let partner_select = Select::with_fields(PARTNER_FIELDS)
        .eq(PartnerTypeCommission::protocol_type_id, protocol_type_id);
    let partners =
        PartnerTypeCommission::select(&partner_select, db_pool.as_ref()).await?;

    let item_list = partners
        .into_iter()
        .map(|p| GetPartnersResponseItem {
            commission_role_id: p.role_id,
            user_id: p.user_id,
            uuid: p.uuid,
        })
        .collect();
    Ok(ApiResponse::default().with_data(GetPartnersResponseData { item_list }))
}
