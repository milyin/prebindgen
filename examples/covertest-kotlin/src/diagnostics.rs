//! An application-installed tracing subscriber observes generated callback errors.
use prebindgen_jni_runtime::jni::{
    objects::{JClass, JString},
    JNIEnv,
};
use std::{io::Write, sync::Mutex};

static EVENTS: Mutex<Vec<u8>> = Mutex::new(Vec::new());
struct Writer;
impl Write for Writer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        EVENTS.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_CallbackDiagnosticsProbe_install(
    mut env: JNIEnv<'_>,
    _: JClass<'_>,
) {
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::ERROR)
        .with_writer(|| Writer)
        .finish();
    if let Err(error) = tracing::subscriber::set_global_default(subscriber) {
        let _ = env.throw_new("java/lang/IllegalStateException", error.to_string());
    }
}
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_CallbackDiagnosticsProbe_events<'local>(
    env: JNIEnv<'local>,
    _: JClass<'local>,
) -> JString<'local> {
    let events = String::from_utf8(EVENTS.lock().unwrap().clone()).unwrap();
    env.new_string(events)
        .unwrap_or_else(|_| JString::from(prebindgen_jni_runtime::jni::objects::JObject::null()))
}
