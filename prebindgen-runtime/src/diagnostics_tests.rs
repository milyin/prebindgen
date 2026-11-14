use super::*;
use std::{
    collections::BTreeMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};
use tracing::{
    field::{Field, Visit},
    span::{Attributes, Id, Record},
    Event, Metadata, Subscriber,
};

#[derive(Clone, Default)]
struct Capture {
    events: Arc<Mutex<Vec<BTreeMap<String, String>>>>,
    panic: bool,
    filtered: bool,
}
struct Fields(BTreeMap<String, String>);
impl Visit for Fields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.0.insert(field.name().into(), value.into());
    }
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.0.insert(field.name().into(), format!("{value:?}"));
    }
}
impl Subscriber for Capture {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        !self.filtered
    }
    fn new_span(&self, _: &Attributes<'_>) -> Id {
        Id::from_u64(1)
    }
    fn record(&self, _: &Id, _: &Record<'_>) {}
    fn record_follows_from(&self, _: &Id, _: &Id) {}
    fn enter(&self, _: &Id) {}
    fn exit(&self, _: &Id) {}
    fn event(&self, event: &Event<'_>) {
        assert!(!self.panic, "expected subscriber failure");
        let mut fields = Fields(BTreeMap::new());
        event.record(&mut fields);
        fields
            .0
            .insert("target".into(), event.metadata().target().into());
        fields
            .0
            .insert("level".into(), event.metadata().level().to_string());
        self.events.lock().unwrap().push(fields.0);
    }
}

#[test]
fn callback_failures_have_structured_context() {
    let capture = Capture::default();
    tracing::subscriber::with_default(capture.clone(), || {
        report_callback_error("example.Handler", "(J)V", "invalid Unicode scalar");
    });
    let events = capture.events.lock().unwrap();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event["target"], "prebindgen::callback");
    assert_eq!(event["level"], "ERROR");
    assert_eq!(event["callback"], "example.Handler");
    assert_eq!(event["signature"], "(J)V");
    assert_eq!(event["error"], "invalid Unicode scalar");
}

#[test]
fn a_panicking_subscriber_cannot_escape_callback_containment() {
    tracing::subscriber::with_default(
        Capture {
            panic: true,
            ..Default::default()
        },
        || {
            report_callback_error("example.Handler", "()V", "expected failure");
        },
    );
}

#[test]
fn fallback_preserves_visibility_and_respects_subscriber_filters() {
    const MODE: &str = "PREBINDGEN_CALLBACK_DIAGNOSTIC_TEST_MODE";
    if let Ok(mode) = std::env::var(MODE) {
        let report =
            || report_callback_error("example.Handler", "(J)V", "callback visibility probe");
        if mode == "filtered" {
            tracing::subscriber::with_default(
                Capture {
                    filtered: true,
                    ..Default::default()
                },
                report,
            );
        } else {
            report();
        }
        return;
    }
    for mode in ["absent", "filtered"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "diagnostics_tests::fallback_preserves_visibility_and_respects_subscriber_filters",
                "--nocapture",
            ])
            .env(MODE, mode)
            .output()
            .unwrap();
        assert!(output.status.success());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(
            stderr.contains("callback visibility probe"),
            mode == "absent",
            "{stderr}"
        );
    }
}
