use asez2_shared_db::db_item::AsezTimestamp;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{error::MonolithHttpError, MonolithHttpDriver, MonolithHttpProperties};
use crate::dto::dictionary::{
    CommonDictionaryList, DictionaryKind, GetUpdatesJsonResponse,
};
use crate::dto::files_count::{GetFilesCountRequest, GetFilesCountResponse};
use crate::dto::organization::Organization;
use crate::dto::CommonDictionaries;
use crate::{
    dto::{
        attachment::*,
        customer::MonolithCustomer,
        dictionary::{
            CommonDictionaryKind, DictionaryListRes, DictionaryRequestItem,
            GetUpdatesResponse, SearchRequest,
        },
        okpd::Okpd,
        okved::Okved,
        user::MonolithUser,
        vat::Vat,
        MonolithResponse,
    },
    MonolithDriver, MonolithService,
};
use reqwest::multipart::{Form, Part};

impl MonolithService<MonolithHttpDriver> {
    /// Получение записей по их `id`
    ///
    /// Айди описан как общий `T` дженерик, потому что монолит по разным
    /// сущностям принимает айди в строке или числе
    pub async fn search_by_id<I, T, R>(
        &self,
        dictionary: DictionaryKind,
        ids: I,
        token: String,
        user_id: i32,
    ) -> Result<Vec<R>, MonolithHttpError>
    where
        I: IntoIterator<Item = T>,
        T: ToString + Serialize + Send + Sync,
        R: for<'a> Deserialize<'a>,
    {
        let props = MonolithHttpProperties::default()
            .with_token(token)
            .with_user_id(user_id)
            .with_path(dictionary.as_search_by_id_endpoint())
            .with_method(Method::POST);
        let res = match dictionary {
            DictionaryKind::Users | DictionaryKind::Organization => {
                self.driver
                    .request::<_, MonolithResponse<DictionaryListRes<R>>>(
                        &ids.into_iter()
                            .map(|id| DictionaryRequestItem { id: id.to_string() })
                            .collect::<Vec<_>>(),
                        props,
                    )
                    .await?
            }
            _ => {
                self.driver
                    .request::<_, MonolithResponse<DictionaryListRes<R>>>(
                        &ids.into_iter()
                            .map(|id| DictionaryRequestItem { id })
                            .collect::<Vec<_>>(),
                        props,
                    )
                    .await?
            }
        };

        Ok(res.data.value)
    }

    pub async fn search<R>(
        &self,
        dictionary: DictionaryKind,
        req: SearchRequest,
        token: String,
        user_id: i32,
    ) -> Result<Vec<R>, MonolithHttpError>
    where
        R: for<'a> Deserialize<'a>,
    {
        let res = self
            .driver
            .request::<_, MonolithResponse<DictionaryListRes<R>>>(
                &req,
                MonolithHttpProperties::default()
                    .with_token(token)
                    .with_user_id(user_id)
                    .with_path(dictionary.as_search_endpoint())
                    .with_method(Method::POST),
            )
            .await?;

        Ok(res.data.value)
    }

    /// Получение абсолютно всех обновленных справочников монолита
    pub async fn get_updates(
        &self,
        token: String,
    ) -> Result<GetUpdatesResponse, MonolithHttpError> {
        let res = self
            .driver
            .request::<_, MonolithResponse<_>>(
                &(),
                MonolithHttpProperties::default()
                    .with_path("/api/json/master_data/get_updates/0/")
                    .with_token(token)
                    .with_method(Method::POST),
            )
            .await?;

        Ok(res.data)
    }

    /// Получение абсолютно всех обновленных справочников монолита, с обобщенным типом результата
    pub async fn get_updates_json<T>(
        &self,
        timestamp: AsezTimestamp,
        token: String,
    ) -> Result<GetUpdatesJsonResponse<T>, MonolithHttpError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let res = self
            .driver
            .request::<_, MonolithResponse<_>>(
                &(),
                MonolithHttpProperties::default()
                    .with_path(&format!(
                        "/api/json/master_data/get_updates/{}/",
                        timestamp.unix_timestamp()
                    ))
                    .with_token(token)
                    .with_method(Method::POST),
            )
            .await?;

        Ok(res.data)
    }

    /// Получение справочника "Заказчики"
    pub async fn get_customer_updates(
        &self,
        token: String,
    ) -> Result<Vec<MonolithCustomer>, MonolithHttpError> {
        let res = self.get_updates(token).await?;

        let customers = res
            .entities
            .into_iter()
            .find_map(|item| {
                if let CommonDictionaryList::Customer(records) = item.items {
                    Some(records)
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Ok(customers)
    }

    /// Получение справочника Vat
    pub async fn get_vat_updates(
        &self,
        token: String,
    ) -> Result<Vec<Vat>, MonolithHttpError> {
        let res = self.get_updates(token).await?;

        let vats = res
            .entities
            .into_iter()
            .find_map(|item| {
                if let CommonDictionaryList::Vat(records) = item.items {
                    Some(records)
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Ok(vats)
    }

    pub async fn get_common_dictionaries(
        &self,
        token: String,
    ) -> Result<CommonDictionaries, MonolithHttpError> {
        let res = self.get_updates(token).await?;

        let mut common_dictionaries = CommonDictionaries::default();

        for record in res.entities {
            match record.items {
                CommonDictionaryList::Customer(records) => {
                    common_dictionaries.customers = records;
                }
                CommonDictionaryList::Unit(records) => {
                    common_dictionaries.units = records;
                }
                CommonDictionaryList::Currency(records) => {
                    common_dictionaries.currencies = records;
                }
                CommonDictionaryList::PurchasingTrend(records) => {
                    common_dictionaries.purchasing_trends = records;
                }
                CommonDictionaryList::Category(records) => {
                    common_dictionaries.categories = records;
                }
                _ => {}
            }
        }

        if common_dictionaries.customers.is_empty() {
            return Err(MonolithHttpError::NotFoundDictionary(
                CommonDictionaryKind::Customer,
            ));
        } else if common_dictionaries.units.is_empty() {
            return Err(MonolithHttpError::NotFoundDictionary(
                CommonDictionaryKind::Unit,
            ));
        } else if common_dictionaries.currencies.is_empty() {
            return Err(MonolithHttpError::NotFoundDictionary(
                CommonDictionaryKind::Currency,
            ));
        } else if common_dictionaries.purchasing_trends.is_empty() {
            return Err(MonolithHttpError::NotFoundDictionary(
                CommonDictionaryKind::PurchasingTrend,
            ));
        }

        Ok(common_dictionaries)
    }

    /// Получение пользователей по их id
    pub async fn search_users_by_id<T, I>(
        &self,
        ids: I,
        token: String,
        user_id: i32,
    ) -> Result<Vec<MonolithUser>, MonolithHttpError>
    where
        T: ToString + Serialize + Send + Sync,
        I: IntoIterator<Item = T>,
    {
        self.search_by_id(DictionaryKind::Users, ids, token, user_id).await
    }

    /// Получение организаций по их id
    pub async fn search_organization_by_id<T, I>(
        &self,
        ids: I,
        token: String,
        user_id: i32,
    ) -> Result<Vec<Organization>, MonolithHttpError>
    where
        T: ToString + Serialize + Send + Sync,
        I: IntoIterator<Item = T>,
    {
        self.search_by_id(DictionaryKind::Organization, ids, token, user_id)
            .await
    }

    /// Получение ОКВЭД2 по их id
    pub async fn search_okved_by_id<I>(
        &self,
        ids: I,
        token: String,
        user_id: i32,
    ) -> Result<Vec<Okved>, MonolithHttpError>
    where
        I: IntoIterator<Item = i32>,
    {
        self.search_by_id(DictionaryKind::Okved2, ids, token, user_id).await
    }

    /// Получение ОКПД2 по их id
    pub async fn search_okpd_by_id<I>(
        &self,
        ids: I,
        token: String,
        user_id: i32,
    ) -> Result<Vec<Okpd>, MonolithHttpError>
    where
        I: IntoIterator<Item = i32>,
    {
        self.search_by_id(DictionaryKind::Okpd2, ids, token, user_id).await
    }

    /// Обновление и получение структуры иерархии директорий по UUID иерархии
    ///
    /// Возвращает массив сгенерированных UUID в иерархии
    pub async fn update_hierarchy(
        &self,
        dto: UpdateHierarchyReq,
        token: String,
        user_id: i32,
    ) -> Result<MonolithResponse<UpdateHierarchyResponseData>, MonolithHttpError>
    {
        let res = self
            .driver
            .request::<_, MonolithResponse<UpdateHierarchyResponseData>>(
                &dto,
                MonolithHttpProperties::default()
                    .with_path("/rest/folders/v1/update/hierarchy/")
                    .with_token(token)
                    .with_user_id(user_id)
                    .with_method(Method::POST),
            )
            .await?;

        Ok(res)
    }

    /// Получение количества прикреплённых документов к ППЗ/ДС
    pub async fn get_files_count(
        &self,
        dto: GetFilesCountRequest,
        token: String,
        user_id: i32,
    ) -> Result<MonolithResponse<GetFilesCountResponse>, MonolithHttpError> {
        let res = self
            .driver
            .request::<_, MonolithResponse<GetFilesCountResponse>>(
                &dto,
                MonolithHttpProperties::default()
                    .with_path("/rest/planning/v1/files_count/")
                    .with_token(token)
                    .with_user_id(user_id)
                    .with_method(Method::POST),
            )
            .await?;

        Ok(res)
    }

    /// Получение структуры иерархии директорий по UUID иерархии
    pub async fn get_hierarchy(
        &self,
        dto: GetHierarchyReq,
        token: String,
        user_id: i32,
    ) -> Result<MonolithResponse<GetHierarchyResponseData>, MonolithHttpError> {
        let res = self
            .driver
            .request::<_, MonolithResponse<GetHierarchyResponseData>>(
                &dto,
                MonolithHttpProperties::default()
                    .with_path("/rest/folders/v1/get/hierarchy/")
                    .with_token(token)
                    .with_user_id(user_id)
                    .with_method(Method::POST),
            )
            .await?;

        Ok(res)
    }

    /// Получение структуры иерархии директорий по идентификатору шаблона
    pub async fn get_hierarchy_template(
        &self,
        dto: GetHierarchyTemplateReq,
        token: String,
        user_id: i32,
    ) -> Result<MonolithResponse<GetHierarchyTemplateResponseData>, MonolithHttpError>
    {
        let res = self
            .driver
            .request::<_, MonolithResponse<GetHierarchyTemplateResponseData>>(
                &dto,
                MonolithHttpProperties::default()
                    .with_path("/rest/folders/v1/get/hierarchy_template/")
                    .with_token(token)
                    .with_user_id(user_id)
                    .with_method(Method::POST),
            )
            .await?;

        Ok(res)
    }

    /// Скачивание одного файла по UUID
    pub async fn download_attachment(
        &self,
        uuid: Uuid,
        token: String,
        user_id: i32,
    ) -> Result<Vec<u8>, MonolithHttpError> {
        let path = format!("/rest/folders/v1/download/{}/", uuid);

        self.driver
            .request_blob(
                &(),
                MonolithHttpProperties::default()
                    .with_path(&path)
                    .with_token(token)
                    .with_user_id(user_id)
                    .with_method(Method::GET),
            )
            .await
    }

    /// Загрузка одного или нескольких файлов на сервер
    pub async fn upload_attachments(
        &self,
        data: UploadFileReq,
        token: String,
        user_id: i32,
    ) -> Result<UploadFileResponse, MonolithHttpError> {
        let mut form = Form::new();

        for file in &data.files {
            let part = Part::bytes(file.bytes.clone())
                .file_name(file.name.clone())
                .mime_str(&file.r#type)
                .map_err(|e| MonolithHttpError::Multipart(e.to_string()))?;
            form = form.part("item_list", part);
        }

        self.driver
            .request_multipart(
                form,
                MonolithHttpProperties::default()
                    .with_path("/rest/folders/v1/upload/file/")
                    .with_method(Method::POST)
                    .with_token(token)
                    .with_user_id(user_id),
            )
            .await
    }
}

impl DictionaryKind {
    fn as_search_by_id_endpoint(&self) -> &'static str {
        match self {
            DictionaryKind::Users => "/api/json/users/search_by_id/",
            DictionaryKind::Organization => "/api/json/organization/search_by_id/",
            DictionaryKind::Okpd2 => "/api/json/okpd2/search_by_id/",
            DictionaryKind::Okved2 => "/api/json/okved2/search_by_id/",
            DictionaryKind::Okato => "/api/json/okato/search_by_id/",
        }
    }

    fn as_search_endpoint(&self) -> &'static str {
        match self {
            DictionaryKind::Organization => "/api/json/organization/search/",
            _ => "/undefined/",
        }
    }
}
