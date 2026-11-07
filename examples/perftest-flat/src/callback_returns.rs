//! Source APIs exercising values returned from foreign callbacks.
use prebindgen_proc_macro::prebindgen;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
#[prebindgen]
pub fn callback_return_i64(callback: impl Fn(i64) -> i64 + Send + Sync + 'static) -> i64 {
    callback(21)
}
#[prebindgen]
pub fn callback_return_zero(callback: impl Fn() -> i64 + Send + Sync + 'static) -> i64 {
    callback()
}
#[prebindgen]
pub fn callback_return_bool(callback: impl Fn(bool) -> bool + Send + Sync + 'static) -> bool {
    callback(true)
}
#[prebindgen]
pub fn callback_return_char(callback: impl Fn(char) -> char + Send + Sync + 'static) -> char {
    callback('🦀')
}
#[prebindgen]
pub fn callback_return_float(callback: impl Fn(f64) -> f64 + Send + Sync + 'static) -> f64 {
    callback(1.25)
}
#[prebindgen]
pub fn callback_return_unsigned(callback: impl Fn(u64) -> u64 + Send + Sync + 'static) -> u64 {
    callback(u64::MAX)
}
#[prebindgen]
pub fn callback_return_wide(callback: impl Fn(i128) -> i128 + Send + Sync + 'static) -> i128 {
    callback(i128::MAX)
}
#[prebindgen]
pub fn callback_return_text(callback: impl Fn(String) -> String + Send + Sync + 'static) -> String {
    callback("héllo\0世界".into())
}
#[prebindgen]
pub fn callback_return_bytes(
    callback: impl Fn(Vec<u8>) -> Vec<u8> + Send + Sync + 'static,
) -> Vec<u8> {
    callback(vec![0, 128, 255])
}
#[prebindgen]
pub fn callback_return_optional(
    callback: impl Fn(Option<i64>) -> Option<i64> + Send + Sync + 'static,
    present: bool,
) -> Option<i64> {
    callback(present.then_some(21))
}
#[prebindgen]
pub fn callback_return_vector(
    callback: impl Fn(Vec<i64>) -> Vec<i64> + Send + Sync + 'static,
) -> Vec<i64> {
    callback(vec![1, 2, 3])
}
#[prebindgen]
pub struct CallbackReturnRecord {
    pub number: i64,
    pub label: String,
}
#[prebindgen]
pub fn callback_return_record(
    callback: impl Fn(CallbackReturnRecord) -> CallbackReturnRecord + Send + Sync + 'static,
) -> i64 {
    let value = callback(CallbackReturnRecord {
        number: 21,
        label: "row".into(),
    });
    assert_eq!(value.label, "answer");
    value.number
}
#[prebindgen]
pub enum CallbackReturnChoice {
    First,
    Second,
}
#[prebindgen]
pub fn callback_return_enum(
    callback: impl Fn(CallbackReturnChoice) -> CallbackReturnChoice + Send + Sync + 'static,
) -> bool {
    matches!(
        callback(CallbackReturnChoice::First),
        CallbackReturnChoice::Second
    )
}
static RESOURCE_LIVE: AtomicUsize = AtomicUsize::new(0);
#[prebindgen]
pub type CallbackReturnResource = CallbackReturnResourceImpl;
pub struct CallbackReturnResourceImpl(i64);
impl Drop for CallbackReturnResourceImpl {
    fn drop(&mut self) {
        RESOURCE_LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub fn callback_return_resource_new(value: i64) -> CallbackReturnResource {
    RESOURCE_LIVE.fetch_add(1, Ordering::SeqCst);
    CallbackReturnResourceImpl(value)
}
#[prebindgen]
pub fn callback_return_resource_live() -> u64 {
    RESOURCE_LIVE.load(Ordering::SeqCst) as u64
}
#[prebindgen]
pub fn callback_return_owned(
    callback: impl Fn() -> CallbackReturnResource + Send + Sync + 'static,
) -> i64 {
    callback().0
}
#[prebindgen]
pub fn callback_return_owned_optional(
    callback: impl Fn() -> Option<CallbackReturnResource> + Send + Sync + 'static,
) -> Option<i64> {
    callback().map(|value| value.0)
}
#[prebindgen]
pub fn callback_return_owned_vector(
    callback: impl Fn() -> Vec<CallbackReturnResource> + Send + Sync + 'static,
) -> i64 {
    callback().into_iter().map(|v| v.0).sum()
}
#[prebindgen]
pub type CallbackReturnJob = CallbackReturnJobImpl;
pub struct CallbackReturnJobImpl(std::thread::JoinHandle<i64>);
#[prebindgen]
pub fn callback_return_spawn(
    callback: impl Fn(i64) -> i64 + Send + Sync + 'static,
) -> CallbackReturnJob {
    let callback = Arc::new(callback);
    CallbackReturnJobImpl(std::thread::spawn(move || {
        std::thread::scope(|scope| {
            let a = callback.clone();
            let a = scope.spawn(move || (0..100).map(|i| a(i)).sum::<i64>());
            let b = scope.spawn(move || (0..100).map(|i| callback(i)).sum::<i64>());
            a.join().unwrap() + b.join().unwrap()
        })
    }))
}
#[prebindgen]
pub fn callback_return_join(job: CallbackReturnJob) -> i64 {
    job.0.join().unwrap()
}
#[prebindgen]
pub type CallbackReturnHandler = CallbackReturnHandlerImpl;
pub struct CallbackReturnHandlerImpl(Box<dyn Fn(i64) -> i64 + Send + Sync>);
#[prebindgen]
pub fn callback_return_handler_new(
    callback: impl Fn(i64) -> i64 + Send + Sync + 'static,
    fail: bool,
) -> Result<CallbackReturnHandler, String> {
    if fail {
        drop(callback);
        Err("callback registration rejected".into())
    } else {
        Ok(CallbackReturnHandlerImpl(Box::new(callback)))
    }
}
#[prebindgen]
pub fn callback_return_handler_call(handler: &CallbackReturnHandler, value: i64) -> i64 {
    (handler.0)(value)
}
#[prebindgen]
pub fn callback_return_catch(callback: impl Fn(i64) -> i64 + Send + Sync + 'static) -> i64 {
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| callback(0))).is_err());
    callback(21)
}

#[prebindgen]
pub fn callback_return_error_message(error: &String) -> String {
    error.clone()
}

#[prebindgen]
pub fn callback_return_handler_wrap(
    callback: impl Fn(i64) -> i64 + Send + Sync + 'static,
) -> CallbackReturnHandler {
    CallbackReturnHandlerImpl(Box::new(callback))
}
#[prebindgen]
pub fn callback_return_handler_value(handler: CallbackReturnHandler) -> i64 {
    (handler.0)(21)
}
#[prebindgen]
pub fn callback_return_record_new(number: i64, label: String) -> CallbackReturnRecord {
    CallbackReturnRecord { number, label }
}
#[prebindgen]
pub fn callback_return_char_catch(callback: impl Fn() -> char + Send + Sync + 'static) -> char {
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(&callback)).is_err());
    callback()
}
#[prebindgen]
pub fn callback_return_owned_vector_catch(
    callback: impl Fn() -> Vec<CallbackReturnResource> + Send + Sync + 'static,
) -> i64 {
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(&callback)).is_err());
    callback().into_iter().map(|v| v.0).sum()
}
#[prebindgen]
pub type CallbackReturnPair = CallbackReturnPairImpl;
pub struct CallbackReturnPairImpl {
    resource: CallbackReturnResource,
    character: char,
}
#[prebindgen]
pub fn callback_return_pair_new(
    resource: CallbackReturnResource,
    character: char,
) -> CallbackReturnPair {
    CallbackReturnPairImpl {
        resource,
        character,
    }
}
#[prebindgen]
pub fn callback_return_pair_catch(
    callback: impl Fn() -> CallbackReturnPair + Send + Sync + 'static,
) -> i64 {
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(&callback)).is_err());
    let pair = callback();
    assert_eq!(pair.character, 'c');
    pair.resource.0
}
#[prebindgen]
pub fn callback_return_unit(callback: impl Fn(i64) + Send + Sync + 'static) {
    callback(21)
}
#[prebindgen]
pub fn callback_return_record_into(row: CallbackReturnRecord) -> CallbackReturnRecord {
    row
}

#[prebindgen]
pub fn callback_return_handler_many(handlers: Vec<CallbackReturnHandler>) -> i64 {
    handlers.into_iter().map(|handler| (handler.0)(21)).sum()
}
#[prebindgen]
pub fn callback_return_resource_id(value: &CallbackReturnResource) -> i64 {
    value.0
}
#[prebindgen]
pub type CallbackReturnBorrowedPair = CallbackReturnBorrowedPairImpl;
pub struct CallbackReturnBorrowedPairImpl(i64);
#[prebindgen]
pub fn callback_return_borrowed_pair_new(
    resource: &mut CallbackReturnResource,
    character: char,
) -> CallbackReturnBorrowedPair {
    assert_eq!(character, 'c');
    resource.0 += 1;
    CallbackReturnBorrowedPairImpl(resource.0)
}
#[prebindgen]
pub fn callback_return_borrowed_pair_catch(
    callback: impl Fn() -> CallbackReturnBorrowedPair + Send + Sync + 'static,
) -> i64 {
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(&callback)).is_err());
    callback().0
}
#[prebindgen]
pub fn callback_return_c_result(
    callback: impl Fn(char) -> char + Send + Sync + 'static,
) -> Result<char, String> {
    Ok(callback('c'))
}
#[prebindgen]
pub fn callback_return_take(
    callback: impl Fn(CallbackReturnResource) -> CallbackReturnResource + Send + Sync + 'static,
) -> i64 {
    callback(callback_return_resource_new(91)).0
}
#[prebindgen]
pub fn callback_return_i8(callback: impl Fn(i8) -> i8 + Send + Sync + 'static) -> i8 {
    callback(-8)
}
#[prebindgen]
pub fn callback_return_i16(callback: impl Fn(i16) -> i16 + Send + Sync + 'static) -> i16 {
    callback(-1600)
}
#[prebindgen]
pub fn callback_return_f32(callback: impl Fn(f32) -> f32 + Send + Sync + 'static) -> f32 {
    callback(1.25)
}
#[prebindgen]
pub fn callback_return_u128(callback: impl Fn(u128) -> u128 + Send + Sync + 'static) -> u128 {
    callback(u128::MAX)
}

#[prebindgen]
pub struct CallbackWireRow {
    pub resource: CallbackReturnResource,
    pub character: char,
}
#[prebindgen]
pub fn callback_return_wire_row_catch(
    callback: impl Fn() -> CallbackWireRow + Send + Sync + 'static,
) -> i64 {
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(&callback)).is_err());
    let row = callback();
    assert_eq!(row.character, 'c');
    row.resource.0
}

#[prebindgen]
pub fn callback_return_wire_rows_catch(
    callback: impl Fn() -> Vec<CallbackWireRow> + Send + Sync + 'static,
) -> i64 {
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(&callback)).is_err());
    callback()
        .into_iter()
        .map(|row| {
            assert_eq!(row.character, 'c');
            row.resource.0
        })
        .sum()
}
