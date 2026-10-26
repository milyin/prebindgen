//! Retained source callbacks exercised by the Kotlin/JNI covertest.
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
