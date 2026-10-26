use prebindgen_proc_macro::{features, prebindgen, prebindgen_out_dir};
pub const PREBINDGEN_OUT_DIR: &str = prebindgen_out_dir!();
pub const FEATURES: &str = features!();
#[prebindgen]
pub type ScalarHandler = ScalarHandlerImpl;
pub struct ScalarHandlerImpl(Box<dyn Fn(i64) + Send + Sync>);
#[prebindgen]
pub fn scalar_handler_new(callback: impl Fn(i64) + Send + Sync + 'static) -> ScalarHandler {
    ScalarHandlerImpl(Box::new(callback))
}
#[prebindgen]
pub fn scalar_fire(h: &ScalarHandler, n: i64) -> i64 {
    let start = std::time::Instant::now();
    for i in 0..n {
        (h.0)(i);
    }
    start.elapsed().as_nanos() as i64
}
#[prebindgen]
#[derive(Clone)]
pub struct Record {
    pub x: i64,
    pub y: i64,
}
#[prebindgen]
pub type RecordHandler = RecordHandlerImpl;
pub struct RecordHandlerImpl(Box<dyn Fn(&Record) + Send + Sync>);
#[prebindgen]
pub fn record_handler_new(callback: impl Fn(&Record) + Send + Sync + 'static) -> RecordHandler {
    RecordHandlerImpl(Box::new(callback))
}
#[prebindgen]
pub fn record_fire(h: &RecordHandler, n: i64) -> i64 {
    let start = std::time::Instant::now();
    for i in 0..n {
        (h.0)(&Record { x: i, y: i + 1 });
    }
    start.elapsed().as_nanos() as i64
}
#[prebindgen]
pub type Owned = OwnedImpl;
#[derive(Clone)]
pub struct OwnedImpl(i64);
#[prebindgen]
pub fn owned_new(value: i64) -> Owned {
    OwnedImpl(value)
}
#[prebindgen]
pub fn owned_value(value: &Owned) -> i64 {
    value.0
}
#[prebindgen]
pub type OwnedHandler = OwnedHandlerImpl;
pub struct OwnedHandlerImpl(Box<dyn Fn(Owned) + Send + Sync>);
#[prebindgen]
pub fn owned_handler_new(callback: impl Fn(Owned) + Send + Sync + 'static) -> OwnedHandler {
    OwnedHandlerImpl(Box::new(callback))
}
#[prebindgen]
pub fn owned_fire(h: &OwnedHandler, n: i64) -> i64 {
    let start = std::time::Instant::now();
    for i in 0..n {
        (h.0)(OwnedImpl(i));
    }
    start.elapsed().as_nanos() as i64
}
