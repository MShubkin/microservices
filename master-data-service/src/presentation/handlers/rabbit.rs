use tracing::info;

use shared_essential::presentation::dto::master_data::{
    error::MasterDataResult,
    request::{MasterDataSearchRequest, MasterDataSearchType},
    response::{DirectoryRecords, MasterDataSearchResponse},
};

use crate::application::master_data::get_master_data;
use crate::application::{
    directory_get_full_data, directory_search_by_id, directory_search_by_user_input,
};

pub(crate) async fn process_dictionary_request(
    search_request: MasterDataSearchRequest,
) -> MasterDataResult<MasterDataSearchResponse> {
    let master_data = get_master_data()?;

    info!(
        kind = "master_data",
        "process_dictionary_request. message: {:?}", &search_request
    );

    let search_type = search_request.search_type;
    match search_type {
        MasterDataSearchType::SearchById(data, directory_type) => {
            let (messages, record) =
                directory_search_by_id(master_data, directory_type, &data).await?;
            Ok(MasterDataSearchResponse {
                messages,
                records: DirectoryRecords::from_iter([record.value]),
            })
        }
        MasterDataSearchType::SearchByUserInput(data, directory_type) => {
            let (messages, record) =
                directory_search_by_user_input(master_data, directory_type, &data)
                    .await?;
            Ok(MasterDataSearchResponse {
                messages,
                records: DirectoryRecords::from_iter([record.value]),
            })
        }
        MasterDataSearchType::GetFullDirectory(directory_vec) => {
            directory_get_full_data(master_data, directory_vec).await
        }
    }
}
