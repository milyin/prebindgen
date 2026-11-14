//! Generated Kotlin preflight across independent owning allocation types.
use prebindgen_proc_macro::prebindgen;
use std::sync::atomic::{AtomicI64, Ordering};
static LIVE: AtomicI64 = AtomicI64::new(0);
static CONVERSIONS: AtomicI64 = AtomicI64::new(0);
macro_rules! resource {
    ($ty:ident,$new:ident,$read:ident,$write:ident) => {
        #[prebindgen]
        pub struct $ty {
            pub value: i64,
        }
        impl Drop for $ty {
            fn drop(&mut self) {
                LIVE.fetch_sub(1, Ordering::SeqCst);
            }
        }
        #[prebindgen]
        pub fn $new(value: i64) -> $ty {
            LIVE.fetch_add(1, Ordering::SeqCst);
            $ty { value }
        }
        #[prebindgen]
        pub fn $read(value: &$ty) -> i64 {
            CONVERSIONS.fetch_add(1, Ordering::SeqCst);
            value.value
        }
        #[prebindgen]
        pub fn $write(value: &mut $ty) -> i64 {
            CONVERSIONS.fetch_add(1, Ordering::SeqCst);
            value.value
        }
    };
}
resource!(
    TypeInputA,
    type_input_a_new,
    type_input_a_read,
    type_input_a_mut
);
resource!(
    TypeInputB,
    type_input_b_new,
    type_input_b_read,
    type_input_b_mut
);
resource!(
    TypeInputControl,
    type_input_control_new,
    type_input_control_read,
    type_input_control_mut
);
#[prebindgen]
pub fn type_input_live() -> i64 {
    LIVE.load(Ordering::SeqCst)
}
#[prebindgen]
pub fn type_input_conversions() -> i64 {
    CONVERSIONS.load(Ordering::SeqCst)
}
#[prebindgen]
pub fn type_input_fixed(
    a: &TypeInputA,
    b: &TypeInputB,
    c: &TypeInputA,
    d: &TypeInputB,
    probe: impl Fn() -> bool,
) -> i64 {
    assert!(probe());
    a.value + b.value + c.value + d.value
}
#[prebindgen]
pub fn type_input_fixed_mut(a: &TypeInputA, b: &mut TypeInputA, other: &TypeInputB) -> i64 {
    a.value + b.value + other.value
}
#[prebindgen]
pub fn type_input_shared_lists(a: Vec<i64>, b: Vec<i64>, control: &mut TypeInputControl) -> i64 {
    a.into_iter().chain(b).sum::<i64>() + control.value
}
#[prebindgen]
pub fn type_input_mut_lists(a: Vec<i64>, b: Vec<i64>) -> i64 {
    a.into_iter().chain(b).sum()
}
#[prebindgen]
pub fn type_input_mixed(reads: Vec<i64>, writes: Vec<i64>) -> i64 {
    reads.into_iter().chain(writes).sum()
}
#[prebindgen]
pub struct TypeInputRow {
    pub a: TypeInputA,
    pub b: TypeInputB,
}
#[prebindgen]
pub fn type_input_row(value: TypeInputRow) -> i64 {
    value.a.value + value.b.value
}
/// Covertest counterpart that receives the record through separate foreign fields.
#[prebindgen]
pub fn type_input_row_fields(value: TypeInputRow) -> i64 {
    type_input_row(value)
}
fn sum(values: Vec<TypeInputRow>) -> i64 {
    values.iter().map(|r| r.a.value + r.b.value).sum()
}
#[prebindgen]
pub fn type_input_rows(values: Vec<TypeInputRow>) -> i64 {
    sum(values)
}
#[prebindgen]
pub fn type_input_rows_plain(values: Vec<TypeInputRow>) -> i64 {
    sum(values)
}
#[prebindgen]
pub fn type_input_nested(values: Vec<Vec<TypeInputRow>>) -> i64 {
    values.into_iter().map(sum).sum()
}
#[prebindgen]
pub fn type_input_callback(callback: impl Fn() -> Vec<TypeInputRow>) -> i64 {
    sum(callback())
}
#[prebindgen]
pub fn type_input_nested_callback(callback: impl Fn() -> Vec<Vec<TypeInputRow>>) -> i64 {
    callback().into_iter().map(sum).sum()
}
#[prebindgen]
pub fn type_input_optional(values: Option<Vec<TypeInputA>>) -> i64 {
    values.map_or(-1, |v| v.iter().map(|a| a.value).sum())
}
#[prebindgen]
pub fn type_input_array(values: [TypeInputA; 2]) -> i64 {
    values.iter().map(|a| a.value).sum()
}

#[prebindgen]
pub struct TypeInputHolder {
    pub row: Option<TypeInputRow>,
}
#[prebindgen]
pub fn type_input_holder(value: TypeInputHolder) -> i64 {
    value.row.map_or(-1, |r| r.a.value + r.b.value)
}
