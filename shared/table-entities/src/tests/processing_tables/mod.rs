use crate::*;

use asez2_shared_db::db_item::AsezDate;
use asez2_shared_db::test_setup::run_db_test;
use asez2_shared_db::{DbAdaptor, DbItem};
use sqlx::FromRow;
use uuid::Uuid;

mod agenda;
mod agenda_item;
mod attachment;
mod ca_version;
mod commission_result;
mod document_approver;
mod field_history;
mod joins;
mod plan_retrospective;
mod plan_version;
mod protocol;
mod protocol_item;
mod rels_agenda_protocol;
mod rels_agenda_protocol_items;
mod route_addep;

const USER1: i32 = 1000034567;
const USER2: i32 = 1000034568;
