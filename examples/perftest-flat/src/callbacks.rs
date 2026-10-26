//! Retained source callbacks exercised by the Kotlin/JNI covertest.
use crate::CallbackReturnResource;
use crate::Priority;
use prebindgen_proc_macro::prebindgen;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[prebindgen]
#[derive(Clone)]
pub struct CallbackRecord {
    pub number: i64,
    pub text: String,
}

#[prebindgen]
pub type CallbackSet = CallbackSetImpl;
pub struct CallbackSetImpl {
    value: Arc<dyn Fn(i64, &CallbackRecord) + Send + Sync>,
    empty: Arc<dyn Fn() + Send + Sync>,
}
#[prebindgen]
pub fn callback_set_new(
    value: impl Fn(i64, &CallbackRecord) + Send + Sync + 'static,
    empty: impl Fn() + Send + Sync + 'static,
) -> CallbackSet {
    CallbackSetImpl {
        value: Arc::new(value),
        empty: Arc::new(empty),
    }
}
#[prebindgen]
pub fn callback_set_fire(callbacks: &CallbackSet, count: i64) {
    for i in 0..count {
        (callbacks.value)(
            i,
            &CallbackRecord {
                number: i + 10,
                text: "héllo\0世界".into(),
            },
        );
        (callbacks.empty)();
    }
}
#[prebindgen]
pub type CallbackJob = CallbackJobImpl;
pub struct CallbackJobImpl(std::thread::JoinHandle<()>);
#[prebindgen]
pub fn callback_set_spawn(callbacks: &CallbackSet, count: i64) -> CallbackJob {
    let value = callbacks.value.clone();
    let empty = callbacks.empty.clone();
    CallbackJobImpl(std::thread::spawn(move || {
        for i in 0..count {
            value(
                i,
                &CallbackRecord {
                    number: i + 10,
                    text: "héllo\0世界".into(),
                },
            );
            empty();
        }
    }))
}
#[prebindgen]
pub fn callback_job_join(job: CallbackJob) {
    job.0.join().unwrap();
}

static OWNED_DROPS: AtomicUsize = AtomicUsize::new(0);
#[prebindgen]
pub type CallbackOwned = CallbackOwnedImpl;
#[derive(Clone)]
pub struct CallbackOwnedImpl(i64);
impl Drop for CallbackOwnedImpl {
    fn drop(&mut self) {
        OWNED_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub fn callback_owned_new(id: i64) -> CallbackOwned {
    CallbackOwnedImpl(id)
}
#[prebindgen]
pub fn callback_owned_id(value: &CallbackOwned) -> i64 {
    value.0
}
#[prebindgen]
pub fn callback_owned_drops() -> usize {
    OWNED_DROPS.load(Ordering::SeqCst)
}
#[prebindgen]
pub fn callback_owned_fire(callback: impl Fn(CallbackOwned) + Send + Sync + 'static) {
    callback(CallbackOwnedImpl(42));
}
#[prebindgen]
pub fn callback_owned_borrow(
    value: &CallbackOwned,
    callback: impl Fn(&CallbackOwned) + Send + Sync + 'static,
) {
    callback(value);
}

static TRANSFERRED_DROPS: AtomicUsize = AtomicUsize::new(0);
#[prebindgen]
pub type CallbackTransferred = CallbackTransferredImpl;
pub struct CallbackTransferredImpl(i64);
impl Drop for CallbackTransferredImpl {
    fn drop(&mut self) {
        TRANSFERRED_DROPS.fetch_add(1, Ordering::SeqCst);
    }
}
#[prebindgen]
pub fn callback_transferred_id(value: &CallbackTransferred) -> i64 {
    value.0
}
#[prebindgen]
pub fn callback_transferred_drops() -> usize {
    TRANSFERRED_DROPS.load(Ordering::SeqCst)
}
#[prebindgen]
pub fn callback_transferred_fire(
    callback: impl Fn(CallbackTransferred, Option<CallbackTransferred>) + Send + Sync + 'static,
    present: bool,
) {
    callback(
        CallbackTransferredImpl(7),
        present.then(|| CallbackTransferredImpl(8)),
    );
}
#[prebindgen]
pub fn callback_projection_fire(callback: impl Fn(CallbackProjected) + Send + Sync + 'static) {
    callback(CallbackTransferredImpl(9));
}
#[prebindgen]
pub type CallbackProjected = CallbackTransferredImpl;
#[prebindgen]
pub fn callback_projected_id(value: &CallbackProjected) -> i64 {
    value.0
}
#[prebindgen]
pub fn callback_preflight(value: CallbackOwned, callback: impl Fn() + Send + Sync + 'static) {
    callback();
    drop(value);
}

#[prebindgen]
pub fn callback_scalars(callback: impl Fn(i64, i64) + Send + Sync + 'static) {
    callback(17, 29);
}
#[prebindgen]
pub fn callback_texts(callback: impl Fn(&str, &[u8], Vec<i64>) + Send + Sync + 'static) {
    callback("hello\0世界", &[0, 127, 128, 255], vec![1, -2, i64::MAX]);
}

#[prebindgen]
pub fn callback_optional_owned(
    callback: impl Fn(Option<CallbackOwned>) + Send + Sync + 'static,
    present: bool,
) {
    callback(present.then(|| CallbackOwnedImpl(56)));
}
#[prebindgen]
pub struct CallbackEnvelope {
    pub handle: CallbackOwned,
    pub label: String,
}
#[prebindgen]
pub fn callback_envelope(callback: impl Fn(CallbackEnvelope) + Send + Sync + 'static) {
    callback(CallbackEnvelope {
        handle: CallbackOwnedImpl(57),
        label: "scoped record".into(),
    });
}
#[prebindgen]
pub fn callback_primitives(
    callback: impl Fn(bool, char, u8, u16, u32, u64, i128, u128, f32, f64) + Send + Sync + 'static,
) {
    callback(
        true,
        '🦀',
        u8::MAX,
        u16::MAX,
        u32::MAX,
        u64::MAX,
        i128::MIN,
        u128::MAX,
        -0.0,
        f64::INFINITY,
    );
}

#[prebindgen]
pub fn callback_enums(callback: impl Fn(Priority, Option<Priority>) + Send + Sync + 'static) {
    callback(Priority::High, Some(Priority::Low));
}

#[prebindgen]
pub type CallbackConstructor = CallbackConstructorImpl;
pub struct CallbackConstructorImpl(Arc<dyn Fn(i64) + Send + Sync>);
#[prebindgen]
pub fn callback_constructor_new(
    callback: impl Fn(i64) + Send + Sync + 'static,
) -> CallbackConstructor {
    CallbackConstructorImpl(Arc::new(callback))
}
/// A binding-only constructor: used by an expansion, without its own Kotlin function.
#[prebindgen]
pub fn callback_constructor_from(
    callback: impl Fn(i64) + Send + Sync + 'static,
) -> CallbackConstructor {
    callback_constructor_new(callback)
}
#[prebindgen]
pub fn callback_constructor_owned(
    callback: impl Fn(CallbackOwned) + Send + Sync + 'static,
) -> CallbackConstructor {
    callback_constructor_new(move |value| callback(CallbackOwnedImpl(value)))
}
#[prebindgen]
pub fn callback_constructor_owned_fire(handler: CallbackConstructor, value: i64) {
    callback_constructor_fire(handler, value);
}
#[prebindgen]
pub fn callback_constructor_fire(handler: CallbackConstructor, value: i64) {
    (handler.0)(value);
}
#[prebindgen]
pub fn callback_constructor_pair(first: CallbackConstructor, second: CallbackConstructor) {
    (first.0)(31);
    (second.0)(37);
}
#[prebindgen]
pub fn callback_constructor_mixed(
    direct: impl Fn(i64) + Send + Sync + 'static,
    handler: CallbackConstructor,
) {
    direct(61);
    (handler.0)(67);
}
#[prebindgen]
pub fn callback_constructor_set_fire(callbacks: CallbackSet) {
    callback_set_fire(&callbacks, 2);
}
#[prebindgen]
pub fn callback_constructor_selected(handler: CallbackConstructor, value: i64) {
    callback_constructor_fire(handler, value);
}
#[prebindgen]
pub fn callback_constructor_optional(handler: Option<CallbackConstructor>) -> bool {
    if let Some(handler) = handler {
        (handler.0)(41);
        true
    } else {
        false
    }
}
#[prebindgen]
pub fn callback_constructor_list(handlers: Vec<CallbackConstructor>) {
    for (index, handler) in handlers.into_iter().enumerate() {
        (handler.0)(50 + index as i64);
    }
}
#[prebindgen]
pub type CallbackConstructorEnvelope = CallbackConstructorEnvelopeImpl;
pub struct CallbackConstructorEnvelopeImpl(CallbackConstructor);
#[prebindgen]
pub fn callback_constructor_wrap(handler: CallbackConstructor) -> CallbackConstructorEnvelope {
    CallbackConstructorEnvelopeImpl(handler)
}
#[prebindgen]
pub fn callback_constructor_nested(envelope: CallbackConstructorEnvelope) {
    let CallbackConstructorEnvelopeImpl(handler) = envelope;
    (handler.0)(43);
}
#[prebindgen]
pub type CallbackConstructorJob = CallbackConstructorJobImpl;
pub struct CallbackConstructorJobImpl {
    start: std::sync::mpsc::Sender<()>,
    thread: std::thread::JoinHandle<()>,
}
#[prebindgen]
pub fn callback_constructor_spawn(
    handler: CallbackConstructor,
    value: i64,
) -> CallbackConstructorJob {
    let (start, ready) = std::sync::mpsc::channel();
    let thread = std::thread::spawn(move || {
        if ready.recv().is_ok() {
            (handler.0)(value);
        }
    });
    CallbackConstructorJobImpl { start, thread }
}
#[prebindgen]
pub fn callback_constructor_run(job: CallbackConstructorJob) {
    job.start.send(()).unwrap();
    job.thread.join().unwrap();
}
#[prebindgen]
pub fn callback_constructor_checked(handler: CallbackConstructor, value: char) {
    (handler.0)(value as i64);
}

/// Send but not Sync: JNI must continue serializing access between threads.
#[prebindgen]
pub type CallbackReentry = CallbackReentryImpl;
pub struct CallbackReentryImpl(std::cell::RefCell<i64>);
#[prebindgen]
pub fn callback_reentry_new(value: i64) -> CallbackReentry {
    CallbackReentryImpl(std::cell::RefCell::new(value))
}
#[prebindgen]
pub fn callback_reentry_read(value: &CallbackReentry) -> i64 {
    *value.0.borrow()
}
#[prebindgen]
pub fn callback_reentry_write(value: &mut CallbackReentry, new_value: i64) {
    *value.0.get_mut() = new_value;
}
#[prebindgen]
pub fn callback_reentry_take(value: CallbackReentry) -> i64 {
    value.0.into_inner()
}
#[prebindgen]
pub fn callback_reentry_shared(
    value: &CallbackReentry,
    callback: impl Fn() + Send + Sync + 'static,
) -> i64 {
    let borrow = value.0.borrow();
    callback();
    *borrow
}
#[prebindgen]
pub fn callback_reentry_exclusive(
    value: &mut CallbackReentry,
    callback: impl Fn() + Send + Sync + 'static,
) -> i64 {
    let borrow = value.0.get_mut();
    callback();
    *borrow += 1;
    *borrow
}
#[prebindgen]
pub fn callback_reentry_consuming(
    value: CallbackReentry,
    callback: impl Fn() + Send + Sync + 'static,
) -> i64 {
    callback();
    value.0.into_inner()
}
#[prebindgen]
pub fn callback_reentry_pair(
    first: &CallbackReentry,
    second: &CallbackReentry,
    callback: impl Fn() + Send + Sync + 'static,
) -> i64 {
    let a = first.0.borrow();
    let b = second.0.borrow();
    callback();
    *a + *b
}
#[prebindgen]
pub fn callback_reentry_mixed(
    first: &CallbackReentry,
    second: &mut CallbackReentry,
    callback: impl Fn() + Send + Sync + 'static,
) -> i64 {
    let a = first.0.borrow();
    let b = second.0.get_mut();
    callback();
    *b += 1;
    *a + *b
}
#[prebindgen]
pub fn callback_reentry_worker(
    value: &CallbackReentry,
    callback: impl Fn() + Send + Sync + 'static,
) -> i64 {
    let borrow = value.0.borrow();
    std::thread::spawn(callback).join().unwrap();
    *borrow
}
// Does not touch RefCell: a broken lock test can detect overlap without racing it.
#[prebindgen]
pub fn callback_reentry_probe(_value: &CallbackReentry) -> i64 {
    1
}
#[prebindgen]
pub fn callback_reentry_optional(
    value: Option<&CallbackReentry>,
    callback: impl Fn() + Send + Sync + 'static,
) -> i64 {
    let borrow = value.map(|value| value.0.borrow());
    callback();
    borrow.as_deref().copied().unwrap_or(-1)
}
#[prebindgen]
pub fn callback_reentry_clone(value: &CallbackReentry) -> CallbackReentry {
    callback_reentry_new(*value.0.borrow())
}
#[prebindgen]
pub fn callback_reentry_constructed(
    value: CallbackReentry,
    callback: impl Fn() + Send + Sync + 'static,
) -> i64 {
    callback();
    value.0.into_inner()
}

static ROW_RESOURCE_LIVE: AtomicUsize = AtomicUsize::new(0);
static ROW_RESOURCE_DROPS: AtomicUsize = AtomicUsize::new(0);
#[prebindgen]
pub type CallbackRowResource = CallbackRowResourceImpl;
pub struct CallbackRowResourceImpl(i64);
impl Clone for CallbackRowResourceImpl {
    fn clone(&self) -> Self {
        assert!(self.0 >= 0, "callback row clone failure");
        callback_row_resource_new(self.0)
    }
}
impl Drop for CallbackRowResourceImpl {
    fn drop(&mut self) {
        ROW_RESOURCE_LIVE.fetch_sub(1, Ordering::SeqCst);
        ROW_RESOURCE_DROPS.fetch_add(1, Ordering::SeqCst);
        assert_ne!(self.0, 777, "expected callback row drop failure");
    }
}
#[prebindgen]
pub fn callback_row_resource_new(id: i64) -> CallbackRowResource {
    ROW_RESOURCE_LIVE.fetch_add(1, Ordering::SeqCst);
    CallbackRowResourceImpl(id)
}
#[prebindgen]
pub fn callback_row_resource_id(value: &CallbackRowResource) -> i64 {
    value.0
}
#[prebindgen]
pub fn callback_row_resource_live() -> i64 {
    ROW_RESOURCE_LIVE.load(Ordering::SeqCst) as i64
}
#[prebindgen]
pub fn callback_row_resource_drops() -> i64 {
    ROW_RESOURCE_DROPS.load(Ordering::SeqCst) as i64
}
#[prebindgen]
pub fn callback_row_resource_hold(
    _value: &CallbackRowResource,
    callback: impl Fn() + Send + Sync + 'static,
) {
    callback();
}
#[prebindgen]
pub struct CallbackRow {
    pub first: CallbackRowResource,
    pub label: String,
    pub second: CallbackRowResource,
}
fn callback_row(index: i64) -> CallbackRow {
    CallbackRow {
        first: callback_row_resource_new(index * 10 + 1),
        label: format!("row{index}"),
        second: callback_row_resource_new(index * 10 + 2),
    }
}
#[prebindgen]
pub fn callback_rows_owned(callback: impl Fn(Vec<CallbackRow>) + Send + Sync + 'static) {
    callback((0..3).map(callback_row).collect());
}
#[prebindgen]
pub fn callback_rows_borrowed(callback: impl Fn(&[CallbackRow]) + Send + Sync + 'static) {
    let rows = (0..3).map(callback_row).collect::<Vec<_>>();
    callback(&rows);
    // The callback only borrows these original Rust records.
    assert_eq!(rows[1].second.0, 12);
}
#[prebindgen]
pub fn callback_rows_optional(
    callback: impl Fn(Option<Vec<Option<CallbackRow>>>) + Send + Sync + 'static,
    mode: i32,
) {
    callback(match mode {
        0 => None,
        1 => Some(vec![]),
        _ => Some(vec![Some(callback_row(0)), None, Some(callback_row(1))]),
    });
}
#[prebindgen]
pub struct CallbackRowDetail {
    pub resource: Option<CallbackRowResource>,
    pub note: String,
}
#[prebindgen]
pub struct CallbackNestedRow {
    pub row: CallbackRow,
    pub detail: Option<CallbackRowDetail>,
}
#[prebindgen]
pub fn callback_rows_nested(
    callback: impl Fn(Vec<Vec<Option<CallbackNestedRow>>>) + Send + Sync + 'static,
) {
    callback(vec![
        vec![
            Some(CallbackNestedRow {
                row: callback_row(0),
                detail: Some(CallbackRowDetail {
                    resource: Some(callback_row_resource_new(90)),
                    note: "present".into(),
                }),
            }),
            None,
        ],
        vec![],
        vec![
            Some(CallbackNestedRow {
                row: callback_row(1),
                detail: None,
            }),
            Some(CallbackNestedRow {
                row: callback_row(2),
                detail: Some(CallbackRowDetail {
                    resource: None,
                    note: "absent".into(),
                }),
            }),
        ],
    ]);
}
#[prebindgen]
pub fn callback_rows_pair(
    callback: impl Fn(Vec<CallbackRow>, Vec<CallbackRow>) + Send + Sync + 'static,
) {
    callback(
        vec![callback_row(1)],
        vec![callback_row(2), callback_row(3)],
    );
}
#[prebindgen]
pub fn callback_row_handles(callback: impl Fn(Vec<CallbackRowResource>) + Send + Sync + 'static) {
    callback(vec![
        callback_row_resource_new(71),
        callback_row_resource_new(72),
    ]);
}
#[prebindgen]
pub struct CallbackTransferredRow {
    pub first: CallbackRowResource,
    pub label: String,
    pub second: CallbackRowResource,
}
#[prebindgen]
pub fn callback_rows_transferred(
    callback: impl Fn(Vec<CallbackTransferredRow>) + Send + Sync + 'static,
) {
    callback(
        (0..3)
            .map(|i| CallbackTransferredRow {
                first: callback_row_resource_new(i * 10 + 1),
                label: format!("row{i}"),
                second: callback_row_resource_new(i * 10 + 2),
            })
            .collect(),
    );
}
#[prebindgen]
pub fn callback_rows_conversion_failure(
    callback: impl Fn(Vec<CallbackRow>) + Send + Sync + 'static,
) {
    let mut rows = (0..3).map(callback_row).collect::<Vec<_>>();
    rows[1].second.0 = -1;
    callback(rows);
    callback(vec![callback_row(4)]);
}
#[prebindgen]
pub fn callback_rows_spawn(
    callback: impl Fn(Vec<CallbackRow>) + Send + Sync + 'static,
    count: i64,
) -> CallbackJob {
    let callback = Arc::new(callback);
    CallbackJobImpl(std::thread::spawn(move || {
        std::thread::scope(|scope| {
            for _ in 0..2 {
                let callback = callback.clone();
                scope.spawn(move || {
                    for _ in 0..count {
                        callback(vec![callback_row(0), callback_row(1)]);
                    }
                });
            }
        });
    }))
}
#[prebindgen]
pub fn callback_row_job_finished(job: &CallbackJob) -> bool {
    job.0.is_finished()
}
#[prebindgen]
pub fn callback_rows_transferred_pair(
    callback: impl Fn(Vec<CallbackTransferredRow>, Vec<CallbackRow>) + Send + Sync + 'static,
) {
    callback(
        vec![CallbackTransferredRow {
            first: callback_row_resource_new(81),
            label: "transferred".into(),
            second: callback_row_resource_new(82),
        }],
        vec![callback_row(1), callback_row(2)],
    );
}

#[prebindgen]
pub type CallbackUniqueResource = CallbackUniqueResourceImpl;
// Deliberately not Clone: an owned row expansion must move this value.
pub struct CallbackUniqueResourceImpl(CallbackRowResource);
#[prebindgen]
pub fn callback_unique_resource_id(value: &CallbackUniqueResource) -> i64 {
    callback_row_resource_id(&value.0)
}
#[prebindgen]
pub struct CallbackUniqueRow {
    pub resource: CallbackUniqueResource,
    pub label: String,
}
#[prebindgen]
pub fn callback_rows_unique(callback: impl Fn(Vec<CallbackUniqueRow>) + Send + Sync + 'static) {
    callback(vec![
        CallbackUniqueRow {
            resource: CallbackUniqueResourceImpl(callback_row_resource_new(101)),
            label: "first".into(),
        },
        CallbackUniqueRow {
            resource: CallbackUniqueResourceImpl(callback_row_resource_new(102)),
            label: "second".into(),
        },
    ]);
}

#[prebindgen]
pub struct CallbackObservedRow {
    pub resource: CallbackRowResource,
    pub label: String,
}
#[prebindgen]
pub fn callback_rows_ref(callback: impl Fn(Vec<CallbackObservedRow>) + Send + Sync + 'static) {
    callback(vec![CallbackObservedRow {
        resource: callback_row_resource_new(111),
        label: "observed".into(),
    }]);
}

#[prebindgen]
pub fn callback_rows_unique_error(
    callback: impl Fn(Vec<CallbackUniqueRow>) + Send + Sync + 'static,
) {
    callback(vec![
        CallbackUniqueRow {
            resource: CallbackUniqueResourceImpl(callback_row_resource_new(777)),
            label: "drop failure".into(),
        },
        CallbackUniqueRow {
            resource: CallbackUniqueResourceImpl(callback_row_resource_new(102)),
            label: "still cleaned".into(),
        },
    ]);
}

#[prebindgen]
pub fn callback_row_return_cleanup_failure(
    callback: impl Fn(Vec<CallbackUniqueRow>) -> CallbackReturnResource + Send + Sync + 'static,
) {
    drop(callback(vec![CallbackUniqueRow {
        resource: CallbackUniqueResourceImpl(callback_row_resource_new(777)),
        label: "drop failure".into(),
    }]));
}
