use std::{ffi::OsStr, os::unix::prelude::OsStrExt};

use env_setup::*;
use lettre::Address;

macro_rules! set_vars {
    ($($var_name: ident = $val: expr),*) => {
        $(std::env::set_var($var_name, $val.to_string()));*
    };
}

const VARS: &[&str] = &[
    LOGGER_MODE,
    LOGGER_DIR,
    LOGGER_FILE,
    RABBITMQ_HOST,
    RABBITMQ_VHOST,
    RABBITMQ_PORT,
    RABBITMQ_USERNAME,
    RABBITMQ_PASSWORD,
    RABBITMQ_RETRIES,
    RABBITMQ_INTERVAL_MS,
    PLANDB_SRV_INNER_ADDR,
    PLANDB_SRV_INNER_PORT,
    SRV_HOST,
    SRV_PORT,
    SRV_WORKERS,
    SRV_PAYLOAD_LIMIT,
    SRV_JSON_LIMIT,
    SRV_MAX_CONN_PER_WORKER,
    SRV_BLOCKING_THREADS_PER_WORKER,
    SRV_THREAD_COUNT,
    POSTGRES_VHOST,
    POSTGRES_PORT,
    POSTGRES_DB,
    POSTGRES_USER,
    POSTGRES_PASSWORD,
    POSTGRES_MIN_CONNECTIONS,
    POSTGRES_MAX_CONNECTIONS,
    POSTGRES_TIMEOUT_S,
    POSTGRES_CONNECTION_REFRESH_S,
    EMAIL_LOGIN,
    EMAIL_SENDER,
    EMAIL_PASSWORD,
    EMAIL_SMTP_PORT,
    EMAIL_SMTP_SERVER,
];

fn cancel_vars() {
    for x in VARS {
        std::env::remove_var(x);
        assert!(std::env::var(x).is_err());
    }
}

fn restore_env_vars(env_vars: Vec<(&'static str, String)>) {
    for (k, v) in env_vars {
        std::env::set_var(k, v);
    }
}

fn save_env_vars() -> Vec<(&'static str, String)> {
    VARS.iter()
        .copied()
        .filter_map(|x| match std::env::var(x) {
            Ok(v) => Some((x, v)),
            Err(_e) => None,
        })
        .collect()
}

fn setup_and_cancel_test<T>(run: fn() -> T) -> T {
    let env_vars = save_env_vars();
    cancel_vars();
    let res = { run() };
    restore_env_vars(env_vars);
    res
}

fn get_rabbit() {
    let (actual, expected) = setup_and_cancel_test(|| {
        let host = String::from("b");
        let vhost = String::from("/");
        let port = 5672;
        let user = String::from("b");
        let pw = String::from("b");
        let retries = 10;
        let retry_interval_ms = 500;

        set_vars!(
            RABBITMQ_HOST = host,
            RABBITMQ_VHOST = vhost,
            RABBITMQ_PORT = port,
            RABBITMQ_USERNAME = user,
            RABBITMQ_PASSWORD = pw,
            RABBITMQ_RETRIES = retries,
            RABBITMQ_INTERVAL_MS = retry_interval_ms
        );

        let actual = RabbitCfg::from_env().unwrap();
        let expected = RabbitCfg {
            host,
            vhost,
            port,
            user,
            pw,
            retries,
            retry_interval_ms,
        };

        (actual, expected)
    });

    assert_eq!(expected, actual)
}

fn get_plan() {
    let (expected, actual) = setup_and_cancel_test(|| {
        let host = "127.0.0.1".to_string();
        let port = 3000;

        set_vars!(PLANDB_SRV_INNER_ADDR = host, PLANDB_SRV_INNER_PORT = port);

        let actual = PlanAddress::from_env();
        let expected = PlanAddress { host, port };

        (actual.unwrap(), expected)
    });
    assert_eq!(expected, actual)
}

fn get_srv() {
    let (expected, actual) = setup_and_cancel_test(|| {
        let host = SRV_HOST_DEFAULT_VALUE.to_string();
        let port = SRV_PORT_DEFAULT_VALUE.parse::<u16>().unwrap();
        let workers = 2;
        let max_conn_per_worker = 1000;
        let blocking_threads_per_worker = 2;
        let server_thread_count = 7;

        set_vars!(
            SRV_MAX_CONN_PER_WORKER = max_conn_per_worker,
            SRV_BLOCKING_THREADS_PER_WORKER = blocking_threads_per_worker,
            SRV_PORT = port,
            SRV_HOST = host,
            SRV_THREAD_COUNT = server_thread_count,
            SRV_WORKERS = workers
        );

        let actual = ServerCfg::from_env();
        let expected = ServerCfg {
            host,
            port,
            workers,
            max_conn_per_worker,
            blocking_threads_per_worker,
            server_thread_count,
        };

        (actual.unwrap(), expected)
    });
    assert_eq!(expected, actual)
}

fn get_pg() {
    let (expected, actual) = setup_and_cancel_test(|| {
        let host = "rcdevstand.inlinegroup.ru".to_string();
        let port = 5632;
        let user = "postgres".to_string();
        let pw = "changeme".to_string();
        let db_name = "asez2_unit_test_db".to_string();
        let min_connections = 1;
        let max_connections = 4;
        let conn_timeout_s = 10;
        let conn_refresh_interval_s = 3600;

        set_vars!(
            POSTGRES_VHOST = host,
            POSTGRES_PORT = port,
            POSTGRES_USER = user,
            POSTGRES_PASSWORD = pw,
            POSTGRES_DB = db_name,
            POSTGRES_MIN_CONNECTIONS = min_connections,
            POSTGRES_MAX_CONNECTIONS = max_connections,
            POSTGRES_TIMEOUT_S = conn_timeout_s,
            POSTGRES_CONNECTION_REFRESH_S = conn_refresh_interval_s
        );

        let actual = PostgresCfg::from_env();
        let expected = PostgresCfg {
            host,
            port,
            user,
            pw,
            db_name,
            min_connections,
            max_connections,
            conn_timeout_s,
            conn_refresh_interval_s,
        };

        (actual.unwrap(), expected)
    });
    assert_eq!(expected, actual)
}

fn get_email() {
    let (expected, actual) = setup_and_cancel_test(|| {
        let login = "stage@inlinegroup.loc".to_string();
        let sender = "stage@inlinegroup.ru".parse::<Address>().unwrap();
        let pw = "stage".to_string();
        let stmp_server = "rcdevstand.inlinegroup.ru".to_string();
        let port = 465;

        set_vars!(
            EMAIL_SENDER = sender,
            EMAIL_LOGIN = login,
            EMAIL_PASSWORD = pw,
            EMAIL_SMTP_SERVER = stmp_server,
            EMAIL_SMTP_PORT = port
        );

        let actual = MailerCfg::from_env();
        let expected = MailerCfg {
            login,
            sender,
            port,
            pw,
            stmp_server,
        };

        (actual.unwrap(), expected)
    });
    assert_eq!(expected, actual)
}

fn get_tracing1() {
    let (expected, actual) = setup_and_cancel_test(|| {
        let logger_mode = 1;

        set_vars!(LOGGER_MODE = logger_mode);

        let actual = TracingCfg::from_env();
        let expected = TracingCfg {
            tracing_kind: trace_setup::TracingKind::Cef,
        };

        (actual.unwrap(), expected)
    });
    assert_eq!(expected, actual)
}

fn get_tracing_bad() {
    let (expected, actual) = setup_and_cancel_test(|| {
        let logger_mode = 0;

        set_vars!(LOGGER_MODE = logger_mode);

        let actual = TracingCfg::from_env();
        let expected = TracingCfg {
            tracing_kind: trace_setup::TracingKind::None,
        };

        (actual.unwrap(), expected)
    });
    assert_eq!(expected, actual)
}

fn get_bad_fail4() {
    let (expected, actual) = setup_and_cancel_test(|| {
        let port = 3000;

        set_vars!(PLANDB_SRV_INNER_PORT = port);

        let actual = PlanAddress::from_env();
        let expected = EnvError::MissingVar(PLANDB_SRV_INNER_ADDR.to_string());

        (actual, expected)
    });

    assert_eq!(expected.unwrap_err().to_string(), actual.to_string())
}

fn get_bad_fail5() {
    let (expected, actual) = setup_and_cancel_test(|| {
        let host = String::from("b");
        let vhost = String::from("/");
        let port = 5672;
        let user = String::from("b");
        let retries = 10;
        let retry_interval_ms = 500;

        set_vars!(
            RABBITMQ_HOST = host,
            RABBITMQ_VHOST = vhost,
            RABBITMQ_PORT = port,
            RABBITMQ_USERNAME = user,
            RABBITMQ_RETRIES = retries,
            RABBITMQ_INTERVAL_MS = retry_interval_ms
        );

        let actual = RabbitCfg::from_env();
        let expected = EnvError::MissingVar(RABBITMQ_PASSWORD.to_string());

        (actual, expected)
    });
    assert_eq!(expected.unwrap_err().to_string(), actual.to_string())
}

fn json_payload() {
    let actual = setup_and_cancel_test(JsonConfig::from_env);
    assert_eq!(actual.unwrap(), JsonConfig::default());

    const LIMIT: usize = 123;
    let actual = setup_and_cancel_test(|| {
        set_vars!(SRV_JSON_LIMIT = LIMIT);
        JsonConfig::from_env()
    });
    assert_eq!(actual.unwrap(), JsonConfig::new(LIMIT));
}

fn try_get() {
    let actual =
        setup_and_cancel_test(|| env_setup::try_get::<u16>("RABBITMQ_PORT", 12345));
    assert_eq!(actual.unwrap(), 12345);

    let actual = setup_and_cancel_test(|| {
        set_vars!(RABBITMQ_PORT = 2345);
        env_setup::try_get::<u16>(RABBITMQ_PORT, 12345)
    });
    assert_eq!(actual.unwrap(), 2345);

    let actual = setup_and_cancel_test(|| {
        set_vars!(RABBITMQ_PORT = "qwerty");
        env_setup::try_get::<u16>(RABBITMQ_PORT, 12345)
    });
    assert!(matches!(actual.unwrap_err(), EnvError::ParseInt(_)));

    let actual = setup_and_cancel_test(|| {
        std::env::set_var(RABBITMQ_PORT, OsStr::from_bytes(&[0xff]));
        env_setup::try_get::<u16>(RABBITMQ_PORT, 12345)
    });
    assert!(matches!(actual.unwrap_err(), EnvError::InvalidVar(..)));
}

fn try_get_maybe() {
    let actual =
        setup_and_cancel_test(|| env_setup::try_get_maybe::<u16>("RABBITMQ_PORT"));
    assert_eq!(actual.unwrap(), None);

    let actual = setup_and_cancel_test(|| {
        set_vars!(RABBITMQ_PORT = 2345);
        env_setup::try_get_maybe::<u16>(RABBITMQ_PORT)
    });
    assert_eq!(actual.unwrap(), Some(2345));

    let actual = setup_and_cancel_test(|| {
        set_vars!(RABBITMQ_PORT = "qwerty");
        env_setup::try_get_maybe::<u16>(RABBITMQ_PORT)
    });
    assert!(matches!(actual.unwrap_err(), EnvError::ParseInt(_)));
}

/// Ранятся тесты все в одной функции, потому что они должны идти последовательно
#[test]
fn env_setup_tests() {
    get_rabbit();
    get_plan();
    get_srv();
    get_pg();
    get_email();
    get_tracing1();
    get_tracing_bad();
    get_bad_fail4();
    get_bad_fail5();
    json_payload();
    try_get();
    try_get_maybe();
}
