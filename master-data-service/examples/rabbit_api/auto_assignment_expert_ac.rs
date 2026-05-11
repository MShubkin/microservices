use amqprs::channel::BasicPublishArguments;
use amqprs::connection::OpenConnectionArguments;
use amqprs::BasicProperties;

use broker::rabbit::RabbitAdapter;
use broker::{BrokerAdapter, Consumer, Publisher};
use shared_essential::domain::enums::master_data::DirectoryType;
use shared_essential::domain::master_data::routes::Route;
use shared_essential::presentation::dto::master_data::request::{
    MasterDataSearchRequest, MasterDataSearchType,
};
use shared_essential::presentation::dto::master_data::response::MasterDataSearchResponse;
use shared_essential::presentation::dto::AsezResult;

pub const dictionary: &str = "dictionary";

#[tokio::main]
async fn main() {
    // Подключение к RabbitMQ серверу
    let connection_args =
        OpenConnectionArguments::new("localhost", 5672, "guest", "guest");
    let rabbit_adapter =
        RabbitAdapter::connect(connection_args, Default::default())
            .await
            .expect("Не удалось подключиться к RabbitMQ серверу");

    let basic_props = BasicProperties::default()
        .with_content_type("application/json")
        .with_persistence(false)
        .finish();
    let publish_args = BasicPublishArguments::new("", dictionary);
    let (publisher, mut return_consumer) = rabbit_adapter
        .direct_reply(basic_props, publish_args, "direct_reply_one_return_consumer")
        .await
        .expect("Не настроить direct_reply механизм");

    // Публикация запроса для сервиса НСИ
    let request = MasterDataSearchRequest {
        search_type: MasterDataSearchType::GetFullDirectory(vec![
            DirectoryType::AutoAssignmentExpertPa,
        ]),
    };
    publisher.publish(&request).await.expect("Ошибка публикации запроса");

    // Получение ответа ответа от сервиса НСИ
    let received_message = return_consumer.consume().await.unwrap();

    let content: AsezResult<MasterDataSearchResponse> = received_message.content;

    let directory_search_response = content.expect("Ошибка сервиса НСИ");

    // Результаты поиска
    let result_records: Vec<Route> = directory_search_response
        .records
        .auto_assignment_expert_ac
        .unwrap_or_default();

    for route in result_records {
        // Заголовок
        let _header = route.route_header;
        // Критерии
        let _crit = route.route_crit;
        // Пользователи
        let _crit = route.route_users;
    }
}
