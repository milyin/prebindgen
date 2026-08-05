//! Cached primitive-boxing helpers for erased-`Object` deliveries.
//!
//! Leaves delivered through an erased Kotlin function type (`FunctionN.invoke`,
//! all parameters `Object`) must box their primitives. Boxing with
//! `JNIEnv::new_object("java/lang/Integer", "(I)V", …)` resolves the class
//! (`FindClass`) and the constructor (`GetMethodID`) on **every** call — three
//! JNI round-trips per boxed leaf, multiplied by the leaf count on every
//! callback delivery (the subscriber hot path). These helpers resolve each box
//! class and its static `valueOf` once per process and afterwards box with a
//! single `CallStaticObjectMethod`. `valueOf` also returns the JVM's interned
//! boxes where they exist (`Boolean`, small `Integer`/`Long`/`Short`/`Byte`/
//! `Character` values), so a typical enum-ordinal leaf allocates nothing.

use std::sync::OnceLock;

use jni::{
    objects::{GlobalRef, JObject, JStaticMethodID},
    signature::ReturnType,
    sys::jvalue,
    JNIEnv,
};

/// A `java.lang.*` box class pinned by a process-wide `GlobalRef`, plus its
/// static `valueOf` method ID. The pin keeps the class from unloading, which
/// is what keeps the cached method ID valid.
struct BoxClass {
    class: GlobalRef,
    value_of: JStaticMethodID,
}

fn cached<'c>(
    env: &mut JNIEnv,
    cell: &'c OnceLock<BoxClass>,
    class_name: &str,
    value_of_sig: &str,
) -> Result<&'c BoxClass, String> {
    if let Some(b) = cell.get() {
        return Ok(b);
    }
    let class = env
        .find_class(class_name)
        .map_err(|e| format!("find box class {class_name}: {e}"))?;
    let value_of = env
        .get_static_method_id(&class, "valueOf", value_of_sig)
        .map_err(|e| format!("resolve {class_name}.valueOf: {e}"))?;
    let class = env
        .new_global_ref(&class)
        .map_err(|e| format!("global-ref box class {class_name}: {e}"))?;
    // A concurrent first call may already have filled the cell; both values
    // are equivalent, keep the winner.
    let _ = cell.set(BoxClass { class, value_of });
    Ok(cell.get().expect("cell was just set"))
}

macro_rules! box_helper {
    ($name:ident, $prim:ty, $field:ident, $class:literal, $sig:literal) => {
        #[doc = concat!("Box a `", stringify!($prim), "` into `", $class, "` via cached `valueOf`.")]
        pub fn $name<'local>(
            env: &mut JNIEnv<'local>,
            v: $prim,
        ) -> Result<JObject<'local>, String> {
            static CELL: OnceLock<BoxClass> = OnceLock::new();
            let b = cached(env, &CELL, $class, $sig)?;
            // SAFETY: `value_of` was resolved on this exact class with this
            // exact `valueOf` signature, and the `GlobalRef` pins the class.
            unsafe {
                env.call_static_method_unchecked(
                    &b.class,
                    b.value_of,
                    ReturnType::Object,
                    &[jvalue { $field: v }],
                )
            }
            .and_then(|r| r.l())
            .map_err(|e| format!("box {}: {}", $class, e))
        }
    };
}

box_helper!(
    box_jboolean,
    jni::sys::jboolean,
    z,
    "java/lang/Boolean",
    "(Z)Ljava/lang/Boolean;"
);
box_helper!(
    box_jbyte,
    jni::sys::jbyte,
    b,
    "java/lang/Byte",
    "(B)Ljava/lang/Byte;"
);
box_helper!(
    box_jchar,
    jni::sys::jchar,
    c,
    "java/lang/Character",
    "(C)Ljava/lang/Character;"
);
box_helper!(
    box_jshort,
    jni::sys::jshort,
    s,
    "java/lang/Short",
    "(S)Ljava/lang/Short;"
);
box_helper!(
    box_jint,
    jni::sys::jint,
    i,
    "java/lang/Integer",
    "(I)Ljava/lang/Integer;"
);
box_helper!(
    box_jlong,
    jni::sys::jlong,
    j,
    "java/lang/Long",
    "(J)Ljava/lang/Long;"
);
box_helper!(
    box_jfloat,
    jni::sys::jfloat,
    f,
    "java/lang/Float",
    "(F)Ljava/lang/Float;"
);
box_helper!(
    box_jdouble,
    jni::sys::jdouble,
    d,
    "java/lang/Double",
    "(D)Ljava/lang/Double;"
);
