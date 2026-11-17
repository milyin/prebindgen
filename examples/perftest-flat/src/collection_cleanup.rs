//! Callback result ownership in nested/optional foreign row descriptors.
use crate::{owned_resource_id, FixedCleanupRecord, OwnedResource, ResultFailure};
use prebindgen_proc_macro::prebindgen;
fn score(value: &FixedCleanupRecord) -> i64 {
    owned_resource_id(&value.first)
        + value.second.as_ref().map_or(0, owned_resource_id)
        + value.text.len() as i64
}
#[prebindgen]
pub struct CleanupGroup {
    pub before: OwnedResource,
    pub rows: Vec<Vec<FixedCleanupRecord>>,
    pub after: OwnedResource,
}
#[prebindgen]
pub fn cleanup_group_new(
    before: OwnedResource,
    rows: Vec<Vec<FixedCleanupRecord>>,
    after: OwnedResource,
) -> CleanupGroup {
    assert_ne!(owned_resource_id(&before), -7, "group constructor panic");
    CleanupGroup {
        before,
        rows,
        after,
    }
}
#[prebindgen]
pub fn cleanup_group_call(callback: impl Fn() -> CleanupGroup) -> Result<i64, ResultFailure> {
    let group = callback();
    Ok(owned_resource_id(&group.before)
        + owned_resource_id(&group.after)
        + group.rows.iter().flatten().map(score).sum::<i64>())
}
#[prebindgen]
pub fn cleanup_optional_call(
    callback: impl Fn() -> Option<Vec<Option<FixedCleanupRecord>>>,
) -> Result<i64, ResultFailure> {
    Ok(callback()
        .into_iter()
        .flatten()
        .flatten()
        .map(|value| score(&value))
        .sum())
}
