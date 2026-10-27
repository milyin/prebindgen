//! Callback declaration defaults and local conversion overrides.
use prebindgen_proc_macro::prebindgen;
use std::sync::Arc;

#[prebindgen]
pub type CallbackConfigValue = ConfigValue;
pub struct ConfigValue(i64);
#[prebindgen]
pub fn callback_config_number(value: &CallbackConfigValue) -> i64 {
    value.0
}
#[prebindgen]
pub fn callback_config_positive(value: &CallbackConfigValue) -> bool {
    value.0 > 0
}
#[prebindgen]
pub fn callback_config_from_number(number: i64) -> CallbackConfigValue {
    ConfigValue(number)
}
#[prebindgen]
pub fn callback_config_from_flag(accept: bool) -> CallbackConfigValue {
    ConfigValue(if accept { 42 } else { -1 })
}

#[prebindgen]
pub fn callback_config_default(
    callback: impl Fn(CallbackConfigValue, i64) -> CallbackConfigValue + Send + Sync + 'static,
) -> i64 {
    callback(ConfigValue(21), 7).0
}
#[prebindgen]
pub fn callback_config_type_default(callback: impl Fn(CallbackConfigValue) -> i64) -> i64 {
    callback(ConfigValue(21))
}
// The signature can be declared without exporting this function to JNI.
#[prebindgen]
pub fn callback_config_unbound(callback: impl Fn(bool) -> i16) -> i16 {
    callback(true)
}
#[prebindgen]
pub fn callback_config_argument(
    callback: impl Fn(CallbackConfigValue, i64) -> CallbackConfigValue,
) -> i64 {
    callback(ConfigValue(21), 7).0
}
#[prebindgen]
pub fn callback_config_return(
    callback: impl Fn(CallbackConfigValue, i64) -> CallbackConfigValue,
) -> i64 {
    callback(ConfigValue(21), 7).0
}
#[prebindgen]
pub fn callback_config_pair(
    first: impl Fn(CallbackConfigValue, i64) -> CallbackConfigValue,
    second: impl Fn(CallbackConfigValue, i64) -> CallbackConfigValue,
) -> i64 {
    first(ConfigValue(21), 7).0 + second(ConfigValue(21), 7).0
}

#[prebindgen]
pub type CallbackConfigHandler =
    Arc<dyn Fn(CallbackConfigValue, i64) -> CallbackConfigValue + Send + Sync>;
#[prebindgen]
pub fn callback_config_handler(
    callback: impl Fn(CallbackConfigValue, i64) -> CallbackConfigValue + Send + Sync + 'static,
) -> CallbackConfigHandler {
    Arc::new(callback)
}
#[prebindgen]
pub fn callback_config_nested(handlers: Vec<CallbackConfigHandler>) -> i64 {
    std::thread::spawn(move || {
        handlers
            .iter()
            .map(|handler| handler(ConfigValue(21), 7).0)
            .sum()
    })
    .join()
    .unwrap()
}
#[prebindgen]
pub fn callback_config_optional(handler: Option<CallbackConfigHandler>) -> i64 {
    handler.map_or(-1, |handler| handler(ConfigValue(21), 7).0)
}
#[prebindgen]
pub fn callback_config_ordinary(value: CallbackConfigValue) -> i64 {
    value.0
}
