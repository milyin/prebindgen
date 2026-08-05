//! Generator-level boundary invariants, asserted against the *generated code*
//! rather than against any one declaration's expected text.
//!
//! Every other test in this module pins what one shape lowers to. These two pin
//! what **no** shape may lower to, over a corpus that declares one of every
//! supported category at once — so a new position added to the adapter is
//! covered the moment it appears in that corpus, without anyone remembering to
//! extend a checklist.
//!
//! - **No C extern input materialises a restricted-validity Rust type directly
//!   from caller bytes** (#170, #158). A C caller can put any bit pattern in an
//!   inbound slot, so a slot whose Rust type accepts only *some* patterns —
//!   `bool`, a declared `enum_type` — is undefined behaviour before any
//!   generated code runs. Such a slot must cross behind `MaybeUninit`, which
//!   holds any byte legally and is spelled the same in C.
//! - **No raw pointer is turned into a `Box` or a reference without a null
//!   check first** (#154 review).

use quote::ToTokens;

use super::*;

/// One source item set + declaration set exercising every input category the
/// adapter supports, so the invariants below run over a realistic surface
/// rather than a single shape.
///
/// Deliberately absent: a `repr_c_struct` with a restricted-validity field.
/// That is the one position with no per-value hook, and it is rejected at
/// declaration time unless the binding acknowledges it with
/// `.assume_c_field_validity()` — see `restricted_validity_*` in `structs.rs`.
/// Keeping it out of this corpus is what lets these invariants be absolute.
fn every_input_category() -> String {
    let loc = SourceLocation::default();

    let items: Vec<syn::Item> = vec![
        syn::parse_quote!(
            pub enum Operation {
                Add = 0,
                Sub = 1,
            }
        ),
        syn::parse_quote!(
            pub enum Shape {
                Empty,
                Circle(f64),
                Rect { width: f64, height: f64 },
                Labeled(String, Operation),
                Flagged(bool),
                Captioned(Caption),
            }
        ),
        syn::parse_quote!(
            pub struct Caption {
                pub id: u64,
                pub text: String,
                pub emphatic: bool,
            }
        ),
        syn::parse_quote!(
            pub struct Handle {
                pub inner: u64,
            }
        ),
        // Every input position, one function each.
        syn::parse_quote!(
            pub fn take_scalars(a: i32, b: f64, c: bool) -> bool {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn take_enum(op: Operation) -> i32 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn take_union(s: Shape) -> f64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn take_data_struct(c: Caption) -> u64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn take_handle(h: Handle) -> u64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn borrow_handle(h: &Handle) -> u64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn borrow_handle_mut(h: &mut Handle) -> u64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn take_optional_handle(h: Option<Handle>) -> u64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn take_str(s: &str) -> u64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn take_string(s: String) -> u64 {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn take_scalar_slice(xs: &[u8]) -> u64 {
                unimplemented!()
            }
        ),
        // Every output position that has a matching input wire, so the file
        // also contains the encoders the invariants must not trip over.
        syn::parse_quote!(
            pub fn make_union(flag: bool) -> Shape {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn make_caption(emphatic: bool) -> Caption {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn make_handle() -> Handle {
                unimplemented!()
            }
        ),
        syn::parse_quote!(
            pub fn each_handle(f: impl Fn(&Handle) + Send + Sync + 'static) {
                unimplemented!()
            }
        ),
    ];

    let registry = crate::test_util::reg_from_items(declare_referenced(
        items.into_iter().map(|i| (i, loc.clone())),
    ))
    .expect("index items");

    let mut cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(example_flat))
        .free_memory_function("example_free")
        .mangle_type_name(|base| format!("{base}_t"))
        .mangle_destructor(|base| format!("{base}_drop"))
        .mangle_callback(|bases| format!("closure_{}_t", bases.join("_")))
        .mangle_function(|n| n.to_string())
        .opaque_ptr(syn::parse_quote!(Handle))
        .enum_type(syn::parse_quote!(Operation))
        .tagged_union(syn::parse_quote!(Shape))
        .data_struct(syn::parse_quote!(Caption))
        .callback(syn::parse_quote!(impl Fn(&Handle) + Send + Sync + 'static));

    for f in [
        "take_scalars",
        "take_enum",
        "take_union",
        "take_data_struct",
        "take_handle",
        "borrow_handle",
        "borrow_handle_mut",
        "take_optional_handle",
        "take_str",
        "take_string",
        "take_scalar_slice",
        "make_union",
        "make_caption",
        "make_handle",
        "each_handle",
    ] {
        let ident = format_ident!("{f}");
        cbindgen = cbindgen.function(syn::parse_quote!(#ident)).panic();
    }

    write(cbindgen, registry, "boundary_invariants")
}

/// The generated `#[repr(C)]` mirror structs, by name — needed to follow a
/// by-value struct parameter down into its fields.
fn mirror_structs(file: &syn::File) -> HashMap<String, syn::ItemStruct> {
    file.items
        .iter()
        .filter_map(|i| match i {
            syn::Item::Struct(s) => Some((s.ident.to_string(), s.clone())),
            _ => None,
        })
        .collect()
}

/// The generated `#[repr(C)]` mirror enums, by name. A mirror enum is a
/// restricted-validity type in exactly the way the source enum is: only its
/// declared discriminants are valid, and a C caller writes an `int`.
fn mirror_enums(file: &syn::File) -> HashSet<String> {
    file.items
        .iter()
        .filter_map(|i| match i {
            syn::Item::Enum(e) => Some(e.ident.to_string()),
            _ => None,
        })
        .collect()
}

/// Why `ty`, appearing in an **inbound** slot, would materialise a value whose
/// Rust type rejects some of the bit patterns C can write — or `None` when
/// every pattern is valid.
///
/// `MaybeUninit<T>` terminates the walk: that is the wire whose whole point is
/// to hold C's bytes without interpreting them, and whatever is inside it is
/// the generated converter's problem, checked by the converter's own tests.
/// Raw pointers terminate too — holding a garbage pointer is sound; whether it
/// may be dereferenced is the second invariant's business.
fn restricted_inbound(
    ty: &syn::Type,
    structs: &HashMap<String, syn::ItemStruct>,
    enums: &HashSet<String>,
    seen: &mut HashSet<String>,
) -> Option<String> {
    let name = ty.to_token_stream().to_string();
    if matches!(ty, syn::Type::Ptr(_)) {
        return None;
    }
    if let syn::Type::Reference(r) = ty {
        return restricted_inbound(&r.elem, structs, enums, seen);
    }
    let tail = type_path_tail(ty)?.to_string();
    if tail == "MaybeUninit" {
        return None;
    }
    if tail == "bool" {
        return Some(format!("`{name}` is a `bool` (only `0`/`1` are valid)"));
    }
    if enums.contains(&tail) {
        return Some(format!(
            "`{name}` is a mirror enum (only its declared discriminants are valid)"
        ));
    }
    if let Some(s) = structs.get(&tail) {
        // A by-value mirror struct is only as safe as its fields.
        if !seen.insert(tail.clone()) {
            return None;
        }
        for f in &s.fields {
            if let Some(why) = restricted_inbound(&f.ty, structs, enums, seen) {
                let fname = f
                    .ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "_".to_string());
                return Some(format!("`{name}`'s field `{fname}`: {why}"));
            }
        }
    }
    None
}

/// Every generated function that C can call or that decodes C's bytes: the
/// `#[no_mangle] extern "C"` wrappers and destructors, plus the `__cbg_in_*`
/// input converters they call.
fn inbound_fns(file: &syn::File) -> Vec<syn::ItemFn> {
    file.items
        .iter()
        .filter_map(|i| match i {
            syn::Item::Fn(f) => Some(f.clone()),
            _ => None,
        })
        .filter(|f| f.sig.abi.is_some() || f.sig.ident.to_string().starts_with("__cbg_in_"))
        .collect()
}

/// #170 / #158, as a property of the generator rather than of one declaration:
/// **no inbound slot may have a Rust type that rejects bit patterns C can
/// write.** Restricted-validity values cross behind `MaybeUninit` and are
/// normalised (`bool`) or validated (an enum discriminant) before the Rust
/// value exists.
///
/// Returns are exempt by construction — Rust writes them, so they are valid by
/// definition, and that is why `-> bool` is still a bare `bool`.
#[test]
fn no_c_extern_input_materialises_a_restricted_validity_type() {
    let src = every_input_category();
    let file: syn::File = syn::parse_file(&src).expect("generated file parses");
    let structs = mirror_structs(&file);
    let enums = mirror_enums(&file);

    let mut violations: Vec<String> = Vec::new();
    for f in inbound_fns(&file) {
        for input in &f.sig.inputs {
            let syn::FnArg::Typed(pt) = input else {
                continue;
            };
            let mut seen = HashSet::new();
            if let Some(why) = restricted_inbound(&pt.ty, &structs, &enums, &mut seen) {
                violations.push(format!(
                    "  {}({}): {why}",
                    f.sig.ident,
                    pt.pat.to_token_stream(),
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "inbound slots materialise restricted-validity Rust types from caller bytes:\n{}\n\n{src}",
        violations.join("\n"),
    );

    // The corpus has to actually contain the shapes this is checking, or an
    // empty violation list would mean nothing. `bool` must reach the boundary
    // as a parameter, as a data-struct field and as a union payload; an enum
    // must reach it by value.
    let compact: String = src.split_whitespace().collect();
    for expected in [
        "c:::core::mem::MaybeUninit<bool>",           // parameter
        "pubemphatic:::core::mem::MaybeUninit<bool>", // data_struct field
        "Flagged(::core::mem::MaybeUninit<bool>)",    // union payload
        "op:::core::mem::MaybeUninit<operation_t>",   // enum by value
        "s:::core::mem::MaybeUninit<shape_t>",        // union by value
    ] {
        assert!(
            compact.contains(expected),
            "corpus no longer covers `{expected}` — the invariant above would pass vacuously\n{src}"
        );
    }
}

/// #154's review finding, as a property: **a raw pointer is never turned into a
/// `Box` or a reference without being null-checked first.**
///
/// Checked per function body, which is the granularity the generator emits at:
/// every converter that reconstitutes an owned `Box`, or reborrows C's memory,
/// null-checks in the same body — either returning a named error on its
/// `Result` path or treating NULL as the empty/`None` case.
#[test]
fn no_raw_pointer_is_dereferenced_without_a_null_check() {
    let src = every_input_category();
    let file: syn::File = syn::parse_file(&src).expect("generated file parses");

    let mut violations: Vec<String> = Vec::new();
    let mut checked = 0usize;
    for f in file.items.iter().filter_map(|i| match i {
        syn::Item::Fn(f) => Some(f),
        _ => None,
    }) {
        let body: String = f.block.to_token_stream().to_string();
        let compact: String = body.split_whitespace().collect();
        // `from_raw_parts` is the slice lowering, which reconstitutes memory
        // from a pointer just as `Box::from_raw` does — same rule.
        let reconstitutes = compact.contains("::from_raw(")
            || compact.contains("::from_raw_parts(")
            || compact.contains("&*(")
            || compact.contains("&mut*(");
        if !reconstitutes {
            continue;
        }
        checked += 1;
        if !compact.contains(".is_null()") {
            violations.push(format!("  {}", f.sig.ident));
        }
    }

    assert!(
        violations.is_empty(),
        "these generated fns reconstitute memory from a raw pointer with no null check:\n{}\n\n{src}",
        violations.join("\n"),
    );
    // Same discrimination guard: an empty corpus would pass silently.
    assert!(
        checked >= 5,
        "expected the corpus to exercise several pointer-reconstituting fns, saw {checked}\n{src}"
    );
}
