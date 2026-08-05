//! Runtime half of the JNI binding generator.
//!
//! `prebindgen`'s `lang::JniGen` adapter *generates* JNI binding code at
//! build time; the helpers in this crate are what that generated code
//! *calls* at run time. A shipped binding library depends on this crate,
//! not on the generator.

mod box_helpers;
mod byte_array_helpers;
mod iface_method;
mod jni_binding_error;
mod string_helpers;

pub use box_helpers::{
    box_jboolean, box_jbyte, box_jchar, box_jdouble, box_jfloat, box_jint, box_jlong, box_jshort,
};
pub use byte_array_helpers::{decode_byte_array, encode_byte_array, null_byte_array};
pub use iface_method::CachedIfaceMethod;
pub use jni_binding_error::JniBindingError;
pub use string_helpers::{decode_string, encode_string, null_string};
