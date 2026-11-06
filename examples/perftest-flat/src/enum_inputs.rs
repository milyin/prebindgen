//! C collection enum decoding and constructors run once, after alias checks.
use crate::{owned_resource_id, OwnedResource, ResultFailure};
use prebindgen_proc_macro::prebindgen;
use std::sync::atomic::{AtomicUsize, Ordering};

static CONSTRUCTORS: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);

// Intentionally neither Copy nor Clone.
#[prebindgen]
pub enum InputTag {
    Low = -3,
    Normal = 0,
    High = 17,
}
fn score(tag: &InputTag) -> i64 {
    match tag {
        InputTag::Low => -3,
        InputTag::Normal => 0,
        InputTag::High => 17,
    }
}
#[prebindgen]
pub fn enum_input_owned(resource: OwnedResource, tag: InputTag) -> i64 {
    CONSTRUCTORS.fetch_add(1, Ordering::SeqCst);
    assert_ne!(
        owned_resource_id(&resource),
        -2,
        "enum row constructor panic"
    );
    owned_resource_id(&resource) + score(&tag)
}
#[prebindgen]
pub fn enum_input_borrowed(resource: &OwnedResource, tag: Option<&InputTag>) -> i64 {
    CONSTRUCTORS.fetch_add(1, Ordering::SeqCst);
    owned_resource_id(resource) + tag.map_or(1, score)
}
#[prebindgen]
pub fn enum_input_optional(resource: OwnedResource, tag: Option<InputTag>) -> i64 {
    CONSTRUCTORS.fetch_add(1, Ordering::SeqCst);
    owned_resource_id(&resource) + tag.as_ref().map_or(1, score)
}
#[prebindgen]
pub fn enum_inputs_take(rows: Vec<i64>) -> Result<i64, ResultFailure> {
    CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(rows.iter().sum())
}
#[prebindgen]
pub fn enum_inputs_borrow(
    receiver: &mut OwnedResource,
    rows: Vec<i64>,
) -> Result<i64, ResultFailure> {
    CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(owned_resource_id(receiver) + rows.iter().sum::<i64>())
}
#[prebindgen]
pub fn enum_inputs_optional(rows: Option<Vec<Option<i64>>>) -> Result<i64, ResultFailure> {
    CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(rows.map_or(-1, |rows| rows.iter().flatten().sum()))
}
#[prebindgen]
pub fn enum_inputs_alternatives(rows: Vec<i64>) -> Result<i64, ResultFailure> {
    CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(rows.iter().sum())
}
#[prebindgen]
pub fn enum_inputs_reset() {
    CONSTRUCTORS.store(0, Ordering::SeqCst);
    CALLS.store(0, Ordering::SeqCst);
}
#[prebindgen]
pub fn enum_inputs_constructors() -> usize {
    CONSTRUCTORS.load(Ordering::SeqCst)
}
#[prebindgen]
pub fn enum_inputs_calls() -> usize {
    CALLS.load(Ordering::SeqCst)
}
