//! Runtime half of the JNI binding generator.
//!
//! `prebindgen`'s `lang::JniGen` adapter *generates* JNI binding code at
//! build time; the helpers in this crate are what that generated code
//! *calls* at run time. A shipped binding library depends on this crate,
//! not on the generator.

mod box_helpers;
mod byte_array_helpers;
mod class_lookup;
mod iface_method;
mod jni_binding_error;
mod leaves;
mod string_helpers;

pub use box_helpers::{
    box_jboolean, box_jbyte, box_jchar, box_jdouble, box_jfloat, box_jint, box_jlong, box_jshort,
};
pub use byte_array_helpers::{decode_byte_array, encode_byte_array, null_byte_array};
pub use class_lookup::{find_class, init_class_loader};
pub use iface_method::CachedIfaceMethod;
pub use jni_binding_error::JniBindingError;
pub use leaves::{
    drop_local, fixed, new_object_array, new_string, object_array_get, object_array_len,
    object_array_set, read_booleans, read_bytes, read_chars, read_doubles, read_floats, read_ints,
    read_longs, read_shorts, read_string, read_u8s, report_callback_error, write_booleans,
    write_bytes, write_chars, write_doubles, write_floats, write_ints, write_longs, write_shorts,
    write_u8s, MaybeOwned, Upcall, new_handle, take_handle, borrow_handle, borrow_handle_mut, free_handle, check_domain,
};
#[doc(hidden)]
pub use jni;
pub use string_helpers::{decode_string, encode_string, null_string};
