//! Resource-sensitive callback registration and conversion boundaries.
use crate::{callback_owned_drops, callback_owned_new, CallbackOwned};
use prebindgen_proc_macro::prebindgen;

#[prebindgen]
pub type CallbackBoundaryHandler = BoundaryHandler;
pub struct BoundaryHandler(Box<dyn Fn(i64) + Send + Sync>);
#[prebindgen]
pub struct CallbackBoundaryError {
    pub code: i64,
    pub message: String,
}
#[prebindgen]
pub fn callback_boundary_error_into(error: CallbackBoundaryError) -> CallbackBoundaryError {
    error
}
#[prebindgen]
pub fn callback_boundary_register(
    callback: impl Fn(i64) + Send + Sync + 'static,
    fail: bool,
) -> Result<CallbackBoundaryHandler, CallbackBoundaryError> {
    if fail {
        drop(callback);
        Err(CallbackBoundaryError {
            code: 17,
            message: "registration rejected".into(),
        })
    } else {
        Ok(BoundaryHandler(Box::new(callback)))
    }
}
#[prebindgen]
pub fn callback_boundary_fire(handler: &CallbackBoundaryHandler, value: i64) {
    (handler.0)(value);
}
#[prebindgen]
pub fn callback_boundary_fold(
    callback: impl Fn(i64) + Send + Sync + 'static,
    count: i64,
) -> Vec<i64> {
    for index in 0..count {
        callback(index);
    }
    (0..count).collect()
}
#[prebindgen]
pub fn callback_boundary_faults(callback: impl Fn(Vec<CallbackOwned>) + Send + Sync + 'static) {
    for _ in 0..2 {
        let drops = callback_owned_drops();
        callback(vec![callback_owned_new(11), callback_owned_new(13)]);
        assert_eq!(
            callback_owned_drops(),
            drops + 2,
            "callback failed to reclaim its owned arguments"
        );
    }
}
#[prebindgen]
pub type CallbackBoundaryPanic = BoundaryPanic;
pub struct BoundaryPanic(i64);
#[prebindgen]
pub fn callback_boundary_view(value: &CallbackBoundaryPanic) -> i64 {
    assert!(value.0 >= 0, "expected callback conversion panic");
    value.0
}
#[prebindgen]
pub fn callback_boundary_conversion(
    callback: impl Fn(CallbackOwned, CallbackBoundaryPanic, CallbackOwned) + Send + Sync + 'static,
) {
    for value in [-1, 7] {
        let drops = callback_owned_drops();
        callback(
            callback_owned_new(11),
            BoundaryPanic(value),
            callback_owned_new(13),
        );
        assert_eq!(callback_owned_drops(), drops + 2);
    }
}
