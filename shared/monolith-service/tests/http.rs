use env_setup::MonolithCfg;
use monolith_service::http::MonolithHttpDriver;
use monolith_service::MonolithService;

use tokio::test;

fn setup_test() -> MonolithService<MonolithHttpDriver> {
    let cfg = MonolithCfg::from_env().unwrap();

    let driver = MonolithHttpDriver::basic_driver(cfg.url)
        .expect("Ошибка при настройке http драйвера");

    MonolithService::new(driver)
}

/// Тест на работоспособность получения пользователей по id
#[test]
async fn get_users_by_id() {
    let service = setup_test();

    let ids = vec!["1"];
    let res = service
        .search_users_by_id(&ids, String::from("token"), 123)
        .await
        .expect("Ошибка при получении данных");

    assert!(!res.is_empty());
}

/// Тест на получение всех обновленных данных по справочникам
#[test]
async fn get_updates() {
    let service = setup_test();

    let res = service
        .get_updates(String::from("token"))
        .await
        .expect("Ошибка при получении данных");

    assert!(!res.entities.is_empty());
}

/// Тест на получение обновленных данных по справочнику Заказчиков
#[test]
async fn get_customer_updates() {
    let service = setup_test();

    let res = service
        .get_customer_updates(String::from("token"))
        .await
        .expect("Ошибка при получении данных");

    assert!(!res.is_empty());
}
