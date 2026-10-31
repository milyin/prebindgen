//! Borrowed accessor views, including non-Clone owners and intermediate values.
use crate::CPanicOutput;
use prebindgen_proc_macro::prebindgen;
use std::sync::atomic::{AtomicUsize, Ordering};

static OWNER_DROPS: AtomicUsize = AtomicUsize::new(0);
static LEAF_CLONES: AtomicUsize = AtomicUsize::new(0);
static LEAF_DROPS: AtomicUsize = AtomicUsize::new(0);
static CHILD_CALLS: AtomicUsize = AtomicUsize::new(0);
static NAME_CALLS: AtomicUsize = AtomicUsize::new(0);

#[prebindgen]
pub type GetterOwner = GetterOwnerImpl;
pub struct GetterOwnerImpl {
    child: GetterChild,
    present: bool,
    leaf: GetterLeaf,
    optional: Option<GetterDetails>,
    rows: Vec<GetterDetails>,
}
impl Drop for GetterOwnerImpl {
    fn drop(&mut self) {
        OWNER_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub type GetterChild = GetterChildImpl;
pub struct GetterChildImpl {
    number: i64,
    name: String,
}
#[prebindgen]
pub type GetterLeaf = GetterLeafImpl;
pub struct GetterLeafImpl(i64);
impl Clone for GetterLeafImpl {
    fn clone(&self) -> Self {
        LEAF_CLONES.fetch_add(1, Ordering::SeqCst);
        Self(self.0)
    }
}
impl Drop for GetterLeafImpl {
    fn drop(&mut self) {
        LEAF_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub struct GetterDetails {
    pub number: i64,
    pub text: String,
}

#[prebindgen]
pub fn getter_owner_new(present: bool) -> GetterOwner {
    GetterOwnerImpl {
        child: GetterChildImpl {
            number: 71,
            name: "héllo\0世界😀".into(),
        },
        present,
        leaf: GetterLeafImpl(19),
        optional: present.then(|| GetterDetails {
            number: 23,
            text: "optional".into(),
        }),
        rows: vec![
            GetterDetails {
                number: 31,
                text: "first".into(),
            },
            GetterDetails {
                number: 37,
                text: "second".into(),
            },
        ],
    }
}
#[prebindgen]
pub fn getter_owner_id(value: &GetterOwner) -> i64 {
    value.child.number
}
#[prebindgen]
pub fn getter_owner_child(value: &GetterOwner) -> &GetterChild {
    CHILD_CALLS.fetch_add(1, Ordering::SeqCst);
    &value.child
}
#[prebindgen]
pub fn getter_owner_optional(value: &GetterOwner) -> Option<&GetterChild> {
    value.present.then_some(&value.child)
}
#[prebindgen]
pub fn getter_owner_optional_pointer(value: &GetterOwner) -> Option<&GetterChild> {
    getter_owner_optional(value)
}
#[prebindgen]
pub fn getter_owner_child_matches(owner: &GetterOwner, child: &GetterChild) -> bool {
    std::ptr::eq(&owner.child, child)
}
#[prebindgen]
pub fn getter_owner_child_mut(owner: &mut GetterOwner) -> &mut GetterChild {
    &mut owner.child
}
#[prebindgen]
pub fn getter_owner_optional_mut(owner: &mut GetterOwner) -> Option<&mut GetterChild> {
    owner.present.then_some(&mut owner.child)
}
#[prebindgen]
pub fn getter_owner_try_child_mut(
    owner: &mut GetterOwner,
    fail: bool,
) -> Result<&mut GetterChild, String> {
    if fail {
        Err("mutable child error".into())
    } else {
        Ok(&mut owner.child)
    }
}
#[prebindgen]
pub fn getter_owner_try_optional_mut(
    owner: &mut GetterOwner,
    mode: i32,
) -> Result<Option<&mut GetterChild>, String> {
    match mode {
        0 => Ok(None),
        1 => Ok(Some(&mut owner.child)),
        _ => Err("optional mutable child error".into()),
    }
}
#[prebindgen]
pub fn getter_child_set_number(child: &mut GetterChild, number: i64) {
    child.number = number;
}
#[prebindgen]
pub fn getter_owner_try_child(value: &GetterOwner, fail: bool) -> Result<&GetterChild, String> {
    if fail {
        Err("borrowed child error".into())
    } else {
        Ok(&value.child)
    }
}
#[prebindgen]
pub fn getter_owner_try_optional_child(
    value: &GetterOwner,
    mode: i32,
) -> Result<Option<&GetterChild>, String> {
    match mode {
        0 => Ok(None),
        1 => Ok(Some(&value.child)),
        _ => Err("optional borrowed child error".into()),
    }
}
#[prebindgen]
pub fn getter_static_child(details: &String) -> &'static GetterChild {
    let _ = details.len();
    static CHILD: std::sync::OnceLock<GetterChild> = std::sync::OnceLock::new();
    CHILD.get_or_init(|| GetterChildImpl {
        number: 91,
        name: "static child".into(),
    })
}
#[prebindgen]
pub fn getter_static_leaf() -> &'static GetterLeaf {
    static LEAF: std::sync::OnceLock<GetterLeaf> = std::sync::OnceLock::new();
    LEAF.get_or_init(|| GetterLeafImpl(101))
}
#[prebindgen]
pub fn getter_owner_container(value: &GetterOwner) -> &Option<GetterDetails> {
    &value.optional
}
#[prebindgen]
pub fn getter_owner_rows(value: &GetterOwner) -> &[GetterDetails] {
    &value.rows
}
#[prebindgen]
pub fn getter_owner_leaf(value: &GetterOwner) -> &GetterLeaf {
    &value.leaf
}
#[prebindgen]
pub fn getter_leaf_id(value: &GetterLeaf) -> i64 {
    value.0
}
#[prebindgen]
pub fn getter_child_number(value: &GetterChild) -> &i64 {
    &value.number
}
#[prebindgen]
pub fn getter_child_name(value: &GetterChild) -> &str {
    NAME_CALLS.fetch_add(1, Ordering::SeqCst);
    &value.name
}
#[prebindgen]
pub fn getter_owner_fail(value: &GetterOwner) -> &str {
    assert!(!value.present, "expected borrowed getter failure");
    "valid"
}
#[prebindgen]
pub fn getter_owner_into_id(value: GetterOwner) -> i64 {
    value.child.number
}
#[prebindgen]
pub fn getter_counts() -> Vec<usize> {
    [
        &OWNER_DROPS,
        &LEAF_CLONES,
        &LEAF_DROPS,
        &CHILD_CALLS,
        &NAME_CALLS,
    ]
    .map(|counter| counter.load(Ordering::SeqCst))
    .into()
}
#[prebindgen]
pub fn getter_callback(callback: impl Fn(&GetterOwner) + Send + Sync + 'static, present: bool) {
    let owner = getter_owner_new(present);
    callback(&owner);
    assert_eq!(owner.child.number, 71);
}
#[prebindgen]
pub fn getter_owner_read(value: &GetterOwner) -> &GetterChild {
    &value.child
}
#[prebindgen]
pub fn getter_owner_failure() -> GetterOwner {
    getter_owner_new(true)
}
#[prebindgen]
pub fn getter_owner_details(value: &GetterOwner) -> &GetterDetails {
    &value.rows[0]
}
#[prebindgen]
pub fn getter_fields_new() -> GetterOwner {
    getter_owner_new(false)
}
#[prebindgen]
pub fn getter_two_clones_new() -> GetterOwner {
    getter_owner_new(false)
}

// Owned outputs with borrowed views must preserve the final owner's address.
#[prebindgen]
pub fn owner_view_shared(present: bool) -> GetterOwner {
    getter_owner_new(present)
}
#[prebindgen]
pub fn owner_view_reordered(present: bool) -> GetterOwner {
    getter_owner_new(present)
}
#[prebindgen]
pub fn owner_view_mutable(present: bool) -> GetterOwner {
    getter_owner_new(present)
}
#[prebindgen]
pub fn owner_view_optional(present: bool) -> GetterOwner {
    getter_owner_new(present)
}
#[prebindgen]
pub fn owner_view_optional_mut(present: bool) -> GetterOwner {
    getter_owner_new(present)
}
#[prebindgen]
pub fn owner_view_maybe(present: bool) -> Option<GetterOwner> {
    present.then(|| getter_owner_new(true))
}
#[prebindgen]
pub fn owner_view_from_temporary(_text: &String) -> GetterOwner {
    getter_owner_new(false)
}
#[prebindgen]
pub fn owner_view_checked(fail: bool) -> Result<GetterOwner, String> {
    Ok(getter_owner_new(fail))
}
#[prebindgen]
pub fn owner_view_check(owner: &GetterOwner) -> i64 {
    assert!(!owner.present, "owner view conversion failure");
    0
}
#[prebindgen]
pub fn owner_view_rollback() -> Result<GetterOwner, String> {
    Ok(getter_owner_new(false))
}
#[prebindgen]
pub fn owner_view_late(_owner: &GetterOwner) -> CPanicOutput {
    crate::c_panic_output_new()
}
