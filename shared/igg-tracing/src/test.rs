use super::*;

fn rbd_sample() -> ReqBaseData {
    ReqBaseData {
        category: "get".to_string(),
        uri: "https://inlinegroup.ru".to_string(),
        src_ip: "127.0.0.1".to_string(),
        src_port: 80,
        user_agent: "Agent Smith".to_string(),
        user_code: "075EK".to_string(),
        request_id: "some-request-id".to_string(),
        trace_id: "some-trace-id".to_string(),
    }
}

#[test]
fn test_req_base_data_default() {
    let exp = ReqBaseData {
        category: "default".to_string(),
        uri: "".to_string(),
        src_ip: "0.0.0.0".to_string(),
        src_port: 0,
        user_agent: "".to_string(),
        user_code: "unknown".to_string(),
        request_id: "".to_string(),
        trace_id: "".to_string(),
    };
    assert_eq!(exp, ReqBaseData::default())
}

#[test]
fn test_tracing_kind_conversion() {
    let kinds = [
        TracingKind::User as u64,
        TracingKind::Document as u64,
        TracingKind::Expert as u64,
        TracingKind::Something as u64,
    ];
    let exp = [1, 2, 3, 4u64];
    assert_eq!(kinds, exp);
}

#[test]
fn test_event_severity() {
    let events: Vec<u8> =
        [Level::TRACE, Level::DEBUG, Level::INFO, Level::WARN, Level::ERROR]
            .iter()
            .map(event_severity)
            .collect();

    let exp = vec![1, 2, 3, 5, 8];
    assert_eq!(events, exp)
}

#[test]
/// This test used to test the `ReqBaseData::update_from` function.
/// It ensures that the improved `record` functions function the same.
fn test_record_update_reqbasedata1() {
    let mut dest = rbd_sample();
    let src = ReqBaseData {
        category: "".to_string(),
        uri: "".to_string(),
        src_ip: "".to_string(),
        src_port: 0,
        user_agent: "".to_string(),
        user_code: "".to_string(),
        request_id: "".to_string(),
        trace_id: "".to_string(),
    };
    let exp = dest.clone();
    // This block mimics the `ReqBaseData::update_from` function.
    {
        dest.record("category", &src.category);
        dest.record("uri", &src.uri);
        dest.record("source_ip", &src.src_ip);
        dest.record_u16("source_port", src.src_port);
        dest.record("user_agent", &src.user_agent);
        dest.record("user_code", &src.user_code);
        dest.record("request_id", &src.request_id);
        dest.record("trace_id", &src.trace_id);
    }

    assert_eq!(dest, exp);
}

#[test]
/// This test used to test the `ReqBaseData::update_from` function.
/// It ensures that the improved `record` functions function the same.
fn test_record_update_reqbasedata2() {
    // This is the same as `rbd_sample` but it is better to be
    // explicit here.
    let mut dest = ReqBaseData {
        category: "category".to_string(),
        uri: "https://inlinegroup.ru".to_string(),
        src_ip: "127.0.0.1".to_string(),
        src_port: 80,
        user_agent: "Agent Smith".to_string(),
        user_code: "075EK".to_string(),
        request_id: "somerequestid".to_string(),
        trace_id: "some-trace-id".to_string(),
    };
    let src = ReqBaseData {
        category: "category".to_string(),
        uri: "https://inlinegroup.ru".to_string(),
        src_ip: "127.0.0.2".to_string(),
        src_port: 443,
        user_agent: "Agent Orange".to_string(),
        user_code: "".to_string(),
        request_id: "some-other-request-id".to_string(),
        trace_id: "some-other-trace-id".to_string(),
    };
    let exp = ReqBaseData {
        category: "category".to_string(),
        uri: "https://inlinegroup.ru".to_string(),
        src_ip: "127.0.0.2".to_string(),
        src_port: 443,
        user_agent: "Agent Orange".to_string(),
        user_code: "075EK".to_string(),
        request_id: "someotherrequestid".to_string(),
        trace_id: "some-other-trace-id".to_string(),
    };
    // This block mimics the `ReqBaseData::update_from` function.
    {
        dest.record("category", &src.category);
        dest.record("uri", &src.uri);
        dest.record("source_ip", &src.src_ip);
        dest.record_u16("source_port", src.src_port);
        dest.record("user_agent", &src.user_agent);
        dest.record("user_code", &src.user_code);
        dest.record("request_id", &src.request_id);
        dest.record("trace_id", &src.trace_id);
    }

    assert_eq!(dest, exp);
}

#[test]
fn test_rbd_record1() {
    let mut rbd = ReqBaseData::default();

    rbd.record("source_port", "999");
    assert_eq!(rbd, ReqBaseData::default())
}

#[test]
fn test_rbd_record2() {
    let mut rbd = ReqBaseData::default();

    rbd.record("category", "category");
    rbd.record("uri", "https://inlinegroup.ru");
    rbd.record("source_ip", "127.0.0.1");
    rbd.record("user_agent", "Agent Smith");
    rbd.record("user_code", "075EK");
    rbd.record("request_id", "some-request-uuid");
    rbd.record("trace_id", "some-trace-uuid");

    let exp = ReqBaseData {
        category: "category".to_string(),
        uri: "https://inlinegroup.ru".to_string(),
        src_ip: "127.0.0.1".to_string(),
        src_port: 0,
        user_agent: "Agent Smith".to_string(),
        user_code: "075EK".to_string(),
        request_id: "somerequestuuid".to_string(),
        trace_id: "some-trace-uuid".to_string(),
    };
    assert_eq!(rbd, exp)
}
