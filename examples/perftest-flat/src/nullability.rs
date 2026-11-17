//! Required/optional Kotlin inputs and reference-valued inactive constructor slots.
use prebindgen_proc_macro::prebindgen;

#[prebindgen]
pub struct NullabilityHandle {
    pub value: i64,
}
#[prebindgen]
pub fn nullability_handle_new(value: i64) -> NullabilityHandle {
    NullabilityHandle { value }
}
#[prebindgen]
pub fn nullability_handle_read(value: &NullabilityHandle) -> i64 {
    value.value
}
#[prebindgen]
pub fn nullability_handle_optional(value: Option<&NullabilityHandle>) -> i64 {
    value.map(|v| v.value).unwrap_or(-1)
}
#[prebindgen]
pub fn nullability_text(value: &str) -> i64 {
    value.len() as i64
}
#[prebindgen]
pub fn nullability_text_optional(value: Option<&str>) -> i64 {
    value.map(|v| v.len() as i64).unwrap_or(-1)
}
#[prebindgen]
pub fn nullability_bytes(value: &[u8]) -> i64 {
    value.len() as i64
}
#[prebindgen]
pub fn nullability_bytes_optional(value: Option<&[u8]>) -> i64 {
    value.map(|v| v.len() as i64).unwrap_or(-1)
}
#[prebindgen]
pub fn nullability_wide(value: i128) -> i128 {
    value
}
#[prebindgen]
pub fn nullability_wide_optional(value: Option<i128>) -> Option<i128> {
    value
}
#[prebindgen]
pub enum NullabilityChoice {
    A,
    B,
}
#[prebindgen]
pub struct NullabilityValue {
    pub text: String,
    pub bytes: Vec<u8>,
    pub wide: i128,
    pub choice: NullabilityChoice,
    pub handle_value: i64,
}
#[prebindgen]
pub fn nullability_value_new(
    text: String,
    bytes: Vec<u8>,
    wide: i128,
    choice: NullabilityChoice,
    handle: &NullabilityHandle,
) -> NullabilityValue {
    NullabilityValue {
        text,
        bytes,
        wide,
        choice,
        handle_value: handle.value,
    }
}
#[prebindgen]
pub fn nullability_value_sum(value: NullabilityValue) -> i64 {
    value.text.len() as i64
        + value.bytes.len() as i64
        + value.wide as i64
        + match value.choice {
            NullabilityChoice::A => 0,
            NullabilityChoice::B => 1,
        }
        + value.handle_value
}
#[prebindgen]
pub fn nullability_value_bulk(value: NullabilityValue) -> i64 {
    nullability_value_sum(value)
}
#[prebindgen]
pub fn nullability_value_optional(value: Option<NullabilityValue>) -> i64 {
    value.map(nullability_value_sum).unwrap_or(-1)
}
#[prebindgen]
pub fn nullability_values(values: Vec<NullabilityValue>) -> i64 {
    values.into_iter().map(nullability_value_sum).sum()
}
#[prebindgen]
pub fn nullability_values_optional(values: Option<Vec<NullabilityValue>>) -> i64 {
    values.map(nullability_values).unwrap_or(-1)
}
#[prebindgen]
pub fn nullability_handles_optional(values: Option<Vec<NullabilityHandle>>) -> i64 {
    values
        .map(|v| v.iter().map(|h| h.value).sum())
        .unwrap_or(-1)
}
#[prebindgen]
pub fn nullability_text_callback(callback: impl Fn() -> String) -> String {
    callback()
}
#[prebindgen]
pub fn nullability_optional_text_callback(callback: impl Fn() -> Option<String>) -> Option<String> {
    callback()
}
#[prebindgen]
pub fn nullability_value_from_callback(callback: impl Fn() -> String) -> NullabilityValue {
    NullabilityValue {
        text: callback(),
        bytes: Vec::new(),
        wide: 0,
        choice: NullabilityChoice::A,
        handle_value: 0,
    }
}
#[prebindgen]
pub fn nullability_callback_choose(value: NullabilityValue) -> i64 {
    nullability_value_sum(value)
}
