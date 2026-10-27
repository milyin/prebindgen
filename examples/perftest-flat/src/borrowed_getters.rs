//! Borrowed accessor views, including non-Clone owners and intermediate values.
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
