//! Constructor-slot boundary and mixed typed/legacy JNI collection rows.
use crate::{callback_row_resource_new, CallbackJob, CallbackRow, CallbackRowResource};
use prebindgen_proc_macro::prebindgen;

#[prebindgen]
pub struct RowLeaf {
    pub value: i64,
}
impl RowLeaf {
    fn new() -> Self {
        Self { value: 1_000_000 }
    }
}
macro_rules! level {
    ($name:ident, $child:ident) => {
        #[prebindgen]
        pub struct $name {
            pub left: $child,
            pub right: $child,
        }
        impl $name {
            fn new() -> Self {
                Self {
                    left: $child::new(),
                    right: $child::new(),
                }
            }
        }
    };
}
level!(Row2, RowLeaf);
level!(Row4, Row2);
level!(Row8, Row4);
level!(Row16, Row8);
level!(Row32, Row16);
level!(Row64, Row32);
#[prebindgen]
pub struct Row127 {
    pub a: Row64,
    pub b: Row32,
    pub c: Row16,
    pub d: Row8,
    pub e: Row4,
    pub f: Row2,
    pub g: RowLeaf,
}
#[prebindgen]
pub struct WideRow {
    pub left: Row64,
    pub right: Row64,
    pub owner: CallbackRowResource,
}
#[prebindgen]
pub fn typed_rows_limit() -> Vec<Row127> {
    vec![Row127 {
        a: Row64::new(),
        b: Row32::new(),
        c: Row16::new(),
        d: Row8::new(),
        e: Row4::new(),
        f: Row2::new(),
        g: RowLeaf::new(),
    }]
}
#[prebindgen]
pub fn typed_rows_wide(count: usize) -> Vec<WideRow> {
    (0..count)
        .map(|i| WideRow {
            left: Row64::new(),
            right: Row64::new(),
            owner: callback_row_resource_new(i as i64),
        })
        .collect()
}
#[prebindgen]
pub fn typed_rows_mixed() -> Vec<Vec<WideRow>> {
    vec![typed_rows_wide(2), vec![], typed_rows_wide(1)]
}

/// Exercise typed JVM rows on Rust-created threads without class-name lookup.
#[prebindgen]
pub fn typed_rows_worker(
    callback: impl Fn(Vec<CallbackRow>) + Send + Sync + 'static,
) -> CallbackJob {
    crate::callback_rows_spawn(callback, 100)
}
