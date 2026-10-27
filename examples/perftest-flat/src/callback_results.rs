//! Result arguments deliver success values and contain failed invocations.
use crate::{CallbackConfigValue, CallbackReturnResource};
use prebindgen_proc_macro::prebindgen;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[prebindgen]
pub fn callback_result_values(callback: impl Fn(Result<i64, String>)) {
    callback(Ok(11));
    callback(Err("expected callback argument failure".into()));
    callback(Ok(13));
}
#[prebindgen]
pub fn callback_result_unit(callback: impl Fn(Result<(), String>)) {
    callback(Ok(()));
    callback(Err("expected unit argument failure".into()));
    callback(Ok(()));
}
#[prebindgen]
pub fn callback_result_return(callback: impl Fn(Result<i64, String>) -> i64) -> i64 {
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback(Err(
            "no result".into()
        ))))
        .is_err()
    );
    callback(Ok(21))
}
#[prebindgen]
pub fn callback_result_handles(
    callback: impl Fn(CallbackReturnResource, Result<i64, String>, CallbackReturnResource),
) {
    for result in [Err("rollback before upcall".into()), Ok(7)] {
        callback(
            crate::callback_return_resource_new(11),
            result,
            crate::callback_return_resource_new(13),
        );
        assert_eq!(crate::callback_return_resource_live(), 0);
    }
}

static ERROR_DROPS: AtomicUsize = AtomicUsize::new(0);
#[prebindgen]
pub type CallbackResultError = ResultError;
pub struct ResultError(u8);
impl Drop for ResultError {
    fn drop(&mut self) {
        ERROR_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub fn callback_result_error_message(value: &CallbackResultError) -> String {
    format!("mapped callback error {}", value.0)
}
#[prebindgen]
pub fn callback_result_error_panics(_value: &CallbackResultError) -> String {
    panic!("expected callback error mapper panic")
}
#[prebindgen]
pub fn callback_result_mapped(callback: impl Fn(Result<i64, CallbackResultError>)) {
    let before = ERROR_DROPS.load(Ordering::SeqCst);
    callback(Err(ResultError(17)));
    assert_eq!(ERROR_DROPS.load(Ordering::SeqCst), before + 1);
    callback(Ok(17));
}
#[prebindgen]
pub fn callback_result_mapper_panic(callback: impl Fn(Result<i64, CallbackResultError>)) {
    callback_result_mapped(callback);
}
#[prebindgen]
pub type CallbackResultHandler = Arc<dyn Fn(Result<i64, String>) + Send + Sync>;
#[prebindgen]
pub fn callback_result_register(
    callback: impl Fn(Result<i64, String>) + Send + Sync + 'static,
) -> CallbackResultHandler {
    Arc::new(callback)
}
#[prebindgen]
pub fn callback_result_worker(handler: &CallbackResultHandler) {
    let handler = handler.clone();
    std::thread::spawn(move || {
        handler(Err("worker callback argument failure".into()));
        handler(Ok(59));
    })
    .join()
    .unwrap();
}
#[prebindgen]
pub fn callback_result_configured(callback: impl Fn(Result<CallbackConfigValue, String>)) {
    callback(Err("configured callback failure".into()));
    callback(Ok(crate::callback_config_from_number(21)));
}
