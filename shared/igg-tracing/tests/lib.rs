use igg_tracing::ServiceDescription;
use tracing_subscriber::prelude::*;

fn retrieve_log<F>(trace_fn: F) -> Vec<String>
where
    F: Fn(),
{
    std::env::set_var("RUST_TRACE", "trace");

    let dir = tempfile::tempdir().expect("We can make a temporary directory");
    let file_name = "MyLog.log";
    let tz = time::UtcOffset::from_hms(3, 0, 0).expect("Moscow exists.");

    let service = ServiceDescription {
        vendor: "Super Vendor".to_owned(),
        name: "Супер Сервис".to_owned(),
        version: "2.4.3".to_owned(),
        host: "127.0.0.1".parse().unwrap(),
    };
    {
        // Guard must be named. `_` will result in an instant drop and nothing will be written.
        let (cef_layer, _guard) = service.into_guarded_cef_layer_file(
            dir.path().join(file_name),
            ["insert"].as_slice(),
            tz,
        );
        let subscriber = tracing_subscriber::Registry::default().with(cef_layer);
        let _guard = subscriber.set_default();

        let span = tracing::span!(
            tracing::Level::TRACE,
            "test-span",
            "uri" = "/v1/insert",
            "user_agent" = "Bond",
            "user_code" = "007",
            "source_ip" = "localhost",
            // If you don't specify `u16`, it will go to debug.
            "source_port" = 3000u16,
            "request_id" = "some-uuid",
        );
        let _span_guard = span.enter();
        trace_fn();
    }

    let path = std::path::PathBuf::from(dir.path()).join(file_name);
    let log = std::fs::read_to_string(path).expect("We should be able to read log");

    // We need the regex to replace the date.
    let rex_date =
        regex::Regex::new("[\\w]{3} [\\d]{2} [\\d]{2}:[\\d]{2}:[\\d]{2}")
            .expect("This is valid regex.");
    let rex_pid = regex::Regex::new("dpid=[\\d]+").expect("This is valid regex.");

    log.lines()
        .map(|entry| {
            let entry = rex_date.replace(entry, "[date]");
            rex_pid.replace(&entry, "[dpid]").into_owned()
        })
        .collect::<Vec<String>>()
}

#[test]
fn trace_with_id() {
    let trace_fn = || tracing::trace!(kind = "insert", id = "Im cool ID");
    let expected = vec!["[date] 127.0.0.1 CEF:0|Super Vendor|Супер Сервис|2.4.3|default|\
    Im cool ID|1|suser=007 src=localhost spt=3000 request=/v1/insert requestClientApplication=Bond [dpid]".to_string()];

    let log = retrieve_log(trace_fn);
    assert_eq!(log, expected)
}

#[test]
fn trace_with_message() {
    let trace_fn = || tracing::trace!(kind = "insert", "Im cool {}", "MESSAGE");
    let expected = vec![
        "[date] 127.0.0.1 CEF:0|Super Vendor|Супер Сервис|2.4.3|default|Im cool MESSAGE|1|\
        suser=007 src=localhost spt=3000 request=/v1/insert requestClientApplication=Bond [dpid]"
            .to_string(),
    ];

    let log = retrieve_log(trace_fn);
    assert_eq!(log, expected)
}

#[test]
fn trace_with_id_higher_priority() {
    let trace_fn = || {
        tracing::trace!(
            kind = "insert",
            id = "ID is cooler than MESSAGE",
            "Im cool {}",
            "MESSAGE"
        )
    };
    let expected =
        vec!["[date] 127.0.0.1 CEF:0|Super Vendor|Супер Сервис|2.4.3|default|\
    ID is cooler than MESSAGE|1|suser=007 src=localhost spt=3000 \
    request=/v1/insert requestClientApplication=Bond [dpid]"
            .to_string()];

    let log = retrieve_log(trace_fn);
    assert_eq!(log, expected)
}

#[test]
fn trace_with_many_messages() {
    let trace_fn = || {
        tracing::trace!(kind = "insert", "Im cool {}", "MESSAGE1");
        tracing::trace!(kind = "insert", "Im worried {}", "MESSAGE2");
        tracing::trace!(kind = "insert", "Im ohno {}", "MESSAGE3");
    };
    let expected = vec![
        "[date] 127.0.0.1 CEF:0|Super Vendor|Супер Сервис|2.4.3|default|Im cool MESSAGE1|1|suser=007 src=localhost spt=3000 request=/v1/insert requestClientApplication=Bond [dpid]"
            .to_string(),
        "[date] 127.0.0.1 CEF:0|Super Vendor|Супер Сервис|2.4.3|default|Im worried MESSAGE2|1|suser=007 src=localhost spt=3000 request=/v1/insert requestClientApplication=Bond [dpid]"
            .to_string(),
        "[date] 127.0.0.1 CEF:0|Super Vendor|Супер Сервис|2.4.3|default|Im ohno MESSAGE3|1|suser=007 src=localhost spt=3000 request=/v1/insert requestClientApplication=Bond [dpid]"
            .to_string(),
    ];

    let log = retrieve_log(trace_fn);
    assert_eq!(log, expected)
}
