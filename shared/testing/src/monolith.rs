use actix_web::{
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use serde::Serialize;
use serde_json::Value;

use asez2_shared_db::ahash::AHashMap;
use monolith_service::{
    dto::{
        attachment::{
            GetHierarchyResponseData, GetHierarchyResponseItem,
            UpdateHierarchyResponseData, UpdateHierarchyResponseItem,
        },
        dictionary::DictionaryListRes,
        files_count::{FilesCount, GetFilesCountResponse},
        user::MonolithUser,
        Messages, MonolithResponse, Status,
    },
    http::{error::MonolithHttpError, MonolithHttpDriver, MonolithHttpService},
    MonolithService,
};

/// Тестовый мок монолита для тестов с взаимодействием с ним
pub struct MockMonolithService {
    /// Данные по каждой ручке
    handlers: HandlerData,
}

/// Хендлер для завершения работы мока и освобождения сетевых ресурсов
pub struct MockMonolithServiceHandle {
    tx: Option<tokio::sync::oneshot::Sender<()>>,
}

type HandlerData = AHashMap<String, Value>;

impl MockMonolithService {
    pub fn new() -> Self {
        Self {
            handlers: Default::default(),
        }
    }

    pub fn run(
        self,
    ) -> Result<(MockMonolithServiceHandle, MonolithHttpService), MonolithHttpError>
    {
        let (tx, rx) = tokio::sync::oneshot::channel();

        let server = HttpServer::new(move || {
            App::new()
                .default_service(web::to(uni_route))
                .app_data(Data::new(self.handlers.clone()))
        })
        .shutdown_timeout(0)
        // Ось выберет порт
        .bind(("localhost", 0))
        .unwrap();

        let addr = *server.addrs().first().expect("Должен быть адрес");
        let server = server.run();
        let handle = server.handle();

        println!("Мок монолита по {} адресу начал работу", addr);
        tokio::spawn(async move {
            tokio::select! {
                server_res = server => {
                    if let Err(err) = server_res {
                        println!("Мок монолита приостановил работу из за ошибки: {}", err)
                    }
                }
                // Ошибка не должна возникнуть, так как tx всегда отправит сообщение при дропе
                _ = rx => {
                    println!("Мок монолита по {} адресу был приостановлен", addr);
                    handle.stop(false).await;
                }
            }
        });

        let service = MonolithService::new(MonolithHttpDriver::basic_driver(
            format!("http://{addr}/").parse().unwrap(),
        )?);
        let handle = MockMonolithServiceHandle { tx: Some(tx) };
        Ok((handle, service))
    }

    pub fn with_handler<T>(mut self, path: &str, data: T) -> Self
    where
        T: Serialize,
    {
        self.handlers
            .insert(path.to_owned(), serde_json::to_value(data).unwrap());
        self
    }

    pub fn search_users_by_id(self, users: Vec<MonolithUser>) -> Self {
        self.with_handler(
            "/api/json/users/search_by_id/",
            DictionaryListRes { value: users },
        )
    }

    pub fn get_file_count(self, files_count: Vec<FilesCount>) -> Self {
        self.with_handler(
            "/rest/planning/v1/files_count/",
            GetFilesCountResponse { value: files_count },
        )
    }

    pub fn update_hierarchy(
        self,
        hierarchy_list: Vec<UpdateHierarchyResponseItem>,
    ) -> Self {
        self.with_handler(
            "/rest/folders/v1/update/hierarchy/",
            UpdateHierarchyResponseData { hierarchy_list },
        )
    }

    pub fn get_hierarchy(
        self,
        hierarchy_list: Vec<GetHierarchyResponseItem>,
    ) -> Self {
        self.with_handler(
            "/rest/folders/v1/get/hierarchy/",
            GetHierarchyResponseData { hierarchy_list },
        )
    }
}

impl Drop for MockMonolithServiceHandle {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            tx.send(()).unwrap();
        }
    }
}

impl Default for MockMonolithService {
    fn default() -> Self {
        Self::new()
    }
}

async fn uni_route(
    request: HttpRequest,
    handler_data: Data<HandlerData>,
) -> impl Responder {
    if let Some(data) = handler_data.get(request.path()) {
        HttpResponse::Ok().json(MonolithResponse {
            data,
            messages: Messages::default(),
            status: Status::Ok,
        })
    } else {
        HttpResponse::InternalServerError()
            .body(format!("По {} пути не установлены данные", request.path()))
    }
}
