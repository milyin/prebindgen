//! Rust-side pieces that are not function wrappers: the prelude, handle
//! destructors, and JNI symbol names.

use prebindgen_tools::names;
use quote::{format_ident, quote};

use super::{Class, Gen};

/// JNI's escaping of one name component: `_` → `_1`, `;` → `_2`, `[` →
/// `_3`, anything outside ASCII alphanumerics → `_0xxxx`.
fn jni_escape(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '_' => out.push_str("_1"),
            ';' => out.push_str("_2"),
            '[' => out.push_str("_3"),
            c if c.is_ascii_alphanumeric() => out.push(c),
            c => {
                let mut buf = [0u16; 2];
                for u in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("_0{u:04x}"));
                }
            }
        }
    }
    out
}

/// The JNI symbol of a native method: `Java_<class>_<method>`.
pub(crate) fn jni_symbol(class_fqn: &str, method: &str) -> String {
    let class = class_fqn
        .split('.')
        .map(jni_escape)
        .collect::<Vec<_>>()
        .join("_");
    format!("Java_{class}_{}", jni_escape(method))
}

impl<'a> Gen<'a> {
    pub(crate) fn rust_prelude(&mut self) {
        let je = format!("{}/JniErrorHandler", self.base_pkg.replace('.', "/"));
        self.rust.push(quote! {
            /// Report a binding error to the Kotlin error sink, unless a
            /// Kotlin exception is already pending.
            #[allow(dead_code)]
            fn __jni_signal(
                env: &mut ::prebindgen_jni_runtime::jni::JNIEnv,
                sink: &::prebindgen_jni_runtime::jni::objects::JObject,
                msg: &str,
            ) {
                static __M: ::prebindgen_jni_runtime::CachedIfaceMethod =
                    ::prebindgen_jni_runtime::CachedIfaceMethod::new();
                if env.exception_check().unwrap_or(false) {
                    return;
                }
                let __s = match env.new_string(msg) {
                    ::core::result::Result::Ok(s) => s,
                    ::core::result::Result::Err(_) => return,
                };
                let _ = __M.call_object(
                    env,
                    #je,
                    "run",
                    "(Ljava/lang/String;)Ljava/lang/Object;",
                    sink,
                    &[::prebindgen_jni_runtime::jni::sys::jvalue { l: __s.as_raw() }],
                );
            }
        });
        self.rust.guards(self.flat);
    }

    /// A handle class's `freePtr`, and the alignment its tag bit relies on.
    pub(crate) fn rust_free_ptr(&mut self, c: &Class) {
        let sym = format_ident!("{}", jni_symbol(&c.fqn(), "freePtr"));
        let t = self.q.path(&names::ident(&c.rust));
        let msg = format!(
            "`{}`: a handle type must have alignment >= 2 (bit 0 is the closed tag)",
            c.rust
        );
        self.rust.push(quote! {
            #[no_mangle]
            #[allow(non_snake_case)]
            pub unsafe extern "system" fn #sym(
                _env: ::prebindgen_jni_runtime::jni::JNIEnv,
                _class: ::prebindgen_jni_runtime::jni::objects::JClass,
                ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
            ) {
                ::prebindgen_jni_runtime::free_handle::<#t>(ptr)
            }
            const _: () = assert!(::core::mem::align_of::<#t>() >= 2, #msg);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbols_escape_underscores() {
        assert_eq!(
            jni_symbol("io.prebindgen.covertest.esc_pkg.Esc_Probe", "freePtr"),
            "Java_io_prebindgen_covertest_esc_1pkg_Esc_1Probe_freePtr"
        );
        assert_eq!(
            jni_symbol("io.x.CovNative", "escape_probe_value"),
            "Java_io_x_CovNative_escape_1probe_1value"
        );
    }
}
