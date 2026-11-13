//! Source-shape checks and controls for Zenoh's nested expansion requirements.
use prebindgen_jni::prelude::*;
use super::{items, rejected};

use prebindgen_jni::{expand_return,fun,JniOutput};
use prebindgen_flat::{Flat, Result};
use prebindgen_jni::{callback, data_class, package, JniGen};
use quote::ToTokens;
use std::{collections::BTreeMap, path::Path, process::Command};

const ZENOH_SHA: &str = "99a7e7084e00824f85c94cbabfa9573108d53dea";

fn shape(mut item: syn::Item) -> Option<(String, String)> {
    fn fields(fields: &mut syn::Fields) {
        for field in fields {
            field.attrs.clear();
        }
    }
    let name = match &mut item {
        syn::Item::Enum(i) => {
            i.attrs.clear();
            for variant in &mut i.variants {
                variant.attrs.clear();
                fields(&mut variant.fields);
            }
            i.ident.to_string()
        }
        syn::Item::Struct(i) => {
            i.attrs.clear();
            fields(&mut i.fields);
            i.ident.to_string()
        }
        syn::Item::Type(i) => {
            i.attrs.clear();
            i.ident.to_string()
        }
        syn::Item::Const(i) => {
            i.attrs.clear();
            i.ident.to_string()
        }
        _ => return None,
    };
    Some((name, item.into_token_stream().to_string()))
}

// Optional reproducibility check against the real, pinned Git object. This reads
// the object database and does not fetch, check out or modify the consumer repo.
fn verify_snapshot(repo: &Path) -> Result<()> {
    let mut actual = BTreeMap::new();
    for file in [
        "src/lib.rs",
        "src/base/advanced_subscriber/mod.rs",
        "src/base/time/timestamp.rs",
        "src/base/time/timestamp_stack.rs",
        "src/base/config/zenoh_id.rs",
        "src/base/entity.rs",
        "src/base/query/reply.rs",
    ] {
        let output = Command::new("git")
            .args(["show", &format!("{ZENOH_SHA}:{file}")])
            .current_dir(repo)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(format!(
                "cannot read pinned Zenoh source: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let source = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
        actual.extend(
            syn::parse_file(&source)
                .map_err(|e| e.to_string())?
                .items
                .into_iter()
                .filter_map(shape),
        );
    }
    let fixture = syn::parse_file(include_str!("zenoh_shapes.rs")).map_err(|e| e.to_string())?;
    let mut count = 0;
    for (name, expected) in fixture.items.into_iter().filter_map(shape) {
        if actual.get(&name) != Some(&expected) {
            return Err(format!("Zenoh fixture `{name}` differs from pinned source"));
        }
        count += 1;
    }
    println!("Zenoh snapshot: {count} declaration shapes match {ZENOH_SHA}");
    Ok(())
}

pub fn run() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    if let Some(flag) = args.next() {
        if flag != "--zenoh-flat" {
            return Err("expected --zenoh-flat PATH or no arguments".into());
        }
        let path = args.next().ok_or("--zenoh-flat requires a checkout path")?;
        if args.next().is_some() {
            return Err("unexpected probe arguments".into());
        }
        verify_snapshot(Path::new(&path))?;
    }
    let source = include_str!("zenoh_shapes.rs");
    for (name, ty) in [
        ("RecoveryConfig", syn::parse_quote!(RecoveryConfig)),
        (
            "TimestampStackRecord",
            syn::parse_quote!(TimestampStackRecord),
        ),
        ("ReplyStruct", syn::parse_quote!(ReplyStruct)),
    ] {
        let flat = Flat::<()>::parse_selected(items(source), &[], &[ty])?;
        assert!(!flat.payload_enums.is_empty());
        println!("Zenoh {name} source shape: payload dependencies captured");
    }

    let timestamp =
        format!("{source}\npub fn echo_timestamp(value: Timestamp) -> Timestamp {{ value }}");
    let generated = JniGen::builder()
        .items(items(&timestamp))
        .set_package_prefix("example")
        .package(
            package!()
                .class(data_class!(Timestamp))
                .fun(fun!(echo_timestamp)),
        )
        .build()?;
    syn::parse_file(&generated.rust).map_err(|e| e.to_string())?;
    let class = &generated.kotlin["example/Timestamp.kt"];
    assert!(class.contains("ntp64: kotlin.ULong"), "{class}");
    assert!(class.contains("id: kotlin.ByteArray"), "{class}");
    println!("Zenoh Timestamp control: u64 property is ULong, id is ByteArray");

    let id = format!("{source}\npub fn echo_id(value: ZenohId) -> ZenohId {{ value }}");
    let generated = JniGen::builder()
        .items(items(&id))
        .set_package_prefix("example")
        .package(package!().class(data_class!(ZenohId)).fun(fun!(echo_id)))
        .build();
    rejected(
        "Zenoh fixed-array property prerequisite",
        generated,
        "unsupported type",
    );

    // A control without enums isolates today's distinction between explicit
    // field expansion and a declared class with canonical property types.
    // The opaque field is intentionally not declared as a foreign class.
    let source = "pub type NativePayload = String;
        pub struct Envelope { pub payload: NativePayload }
        pub fn payload_size(value: &NativePayload) -> i64 { value.len() as i64 }
        pub fn deliver() -> Envelope { unimplemented!() }
        pub fn register(cb: impl Fn(Envelope)) {}";
    let builder = || {
        JniGen::builder()
            .items(items(source))
            .set_package_prefix("example")
            .expand(expand_return!(NativePayload).field(fun!(payload_size)))
            .expand(expand_return!(Envelope).view(JniOutput::OwnedFields(vec![JniOutput::auto()])))
    };
    let expanded = builder()
        .package(
            package!()
                .class(callback!(impl Fn(Envelope)).name("EnvelopeHandler"))
                .fun(fun!(deliver))
                .fun(fun!(register)),
        )
        .build()?;
    syn::parse_file(&expanded.rust).map_err(|e| e.to_string())?;
    assert!(expanded.rust.contains("crate::payload_size"));
    assert!(!expanded.kotlin.contains_key("example/NativePayload.kt"));
    assert!(!expanded.kotlin.contains_key("example/Envelope.kt"));
    let callback = &expanded.kotlin["example/EnvelopeHandler.kt"];
    assert!(callback.contains("Long"), "{callback}");
    assert!(!callback.contains(": example.Envelope"), "{callback}");
    println!("ungrouped field control: function/callback inherit child expansion without classes");

    let grouped = builder()
        .package(package!().class(data_class!(Envelope)).fun(fun!(deliver)))
        .build();
    let error = grouped
        .err()
        .expect("an ordinary property needs a canonical representation");
    assert!(error.contains("NativePayload"), "{error}");
    println!("grouped field control: {error}");
    Ok(())
}
