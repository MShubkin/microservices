use asez2_shared_db::db_item::AsezTimestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub enum DocumentRequest {
    Save(SaveDocumentReq),
    Get(GetDocumentReq),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum DocumentResponse {
    Save(SaveDocumentResponse),
    Get(GetDocumentResponse),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SaveDocumentReq {
    pub document_name: String,
    pub content: String,
    pub request_id: Uuid,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetDocumentReq {
    pub document_id: Uuid,
    pub document_name: String,
    pub request_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SaveDocumentResponse {
    pub document_id: uuid::Uuid,
    pub request_id: Uuid,
    pub timestamp: AsezTimestamp,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetDocumentResponse {
    pub request_id: Uuid,
    pub content: String,
    pub timestamp: AsezTimestamp,
}

impl From<SaveDocumentResponse> for DocumentResponse {
    fn from(response: SaveDocumentResponse) -> Self {
        DocumentResponse::Save(response)
    }
}

impl From<GetDocumentResponse> for DocumentResponse {
    fn from(response: GetDocumentResponse) -> Self {
        DocumentResponse::Get(response)
    }
}
