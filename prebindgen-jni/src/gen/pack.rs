//! Packed parameters: one array per JNI type instead of one argument per
//! leaf — for values whose leaves would exceed the JVM's 255-slot method
//! argument limit (`data_class!(..).jobject_input()`, or any extern that
//! would otherwise overflow).

use prebindgen_tools::{Input, Wire};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::leaf::{join, Leaf, LeafTy, Prim};

const ORDER: [Prim; 8] = [
    Prim::Z,
    Prim::B,
    Prim::C,
    Prim::S,
    Prim::I,
    Prim::J,
    Prim::F,
    Prim::D,
];

/// The groups: each primitive kind present, then objects; with the leaf
/// indices in each.
fn groups(leaves: &[Leaf]) -> Vec<(Option<Prim>, Vec<usize>)> {
    let mut out = Vec::new();
    for p in ORDER {
        let idx: Vec<usize> = leaves
            .iter()
            .enumerate()
            .filter(|(_, l)| l.prim() == Some(p))
            .map(|(i, _)| i)
            .collect();
        if !idx.is_empty() {
            out.push((Some(p), idx));
        }
    }
    let objs: Vec<usize> = leaves
        .iter()
        .enumerate()
        .filter(|(_, l)| l.is_obj())
        .map(|(i, _)| i)
        .collect();
    if !objs.is_empty() {
        out.push((None, objs));
    }
    out
}

fn group_name(root: &str, p: Option<Prim>) -> String {
    join(
        root,
        &format!("_pack{}", p.map(|p| p.desc()).unwrap_or("L")),
    )
}

/// The packed leaves standing for `leaves` (rooted at `root`).
pub(crate) fn packed_leaves(root: &str, leaves: &[Leaf]) -> Vec<Leaf> {
    groups(leaves)
        .into_iter()
        .map(|(p, _)| Leaf {
            name: group_name(root, p),
            ty: match p {
                Some(p) => LeafTy::PrimArray(p),
                None => LeafTy::ObjArray,
            },
            nullable: false,
        })
        .collect()
}

/// Kotlin expressions for the packed leaves, from the unpacked ones.
pub(crate) fn pack_kotlin(leaves: &[Leaf], exprs: &[String]) -> Vec<String> {
    groups(leaves)
        .into_iter()
        .map(|(p, idx)| {
            let items = idx
                .iter()
                .map(|&i| exprs[i].clone())
                .collect::<Vec<_>>()
                .join(", ");
            match p {
                Some(p) => format!("{}Of({items})", lower_first(&p.kt_array())),
                None => format!("arrayOf<Any?>({items})"),
            }
        })
        .collect()
}

fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().chain(c).collect(),
        None => String::new(),
    }
}

/// The Rust input over the packed wires: unpack every leaf into the local
/// the unpacked input reads, then evaluate it.
pub(crate) fn pack_input(root: &str, leaves: &[Leaf], input: Input) -> Input {
    let rt = quote!(::prebindgen_jni_runtime);
    let mut wires = Vec::new();
    let mut stmts: Vec<TokenStream> = Vec::new();
    for (p, idx) in groups(leaves) {
        let packed = prebindgen_tools::names::ident(&group_name(root, p));
        let leaf = Leaf {
            name: String::new(),
            ty: match p {
                Some(p) => LeafTy::PrimArray(p),
                None => LeafTy::ObjArray,
            },
            nullable: false,
        };
        wires.push(Wire::new(packed.clone(), leaf.rs()));
        match p {
            Some(p) => {
                let (read, _) = p.array_helpers();
                let read = format_ident!("{}", read);
                let buf = format_ident!("__{}", packed);
                stmts.push(quote!(let #buf = #rt::#read(env, &#packed)?;));
                for (k, &i) in idx.iter().enumerate() {
                    let n = &input.wires[i].name;
                    stmts.push(quote!(let #n = #buf[#k];));
                }
            }
            None => {
                for (k, &i) in idx.iter().enumerate() {
                    let n = &input.wires[i].name;
                    stmts.push(quote!(let #n = #rt::object_array_get(env, &#packed, #k)?;));
                }
            }
        }
    }
    let e = input.expr;
    Input {
        wires,
        expr: quote!({ #(#stmts)* #e }),
        fallible: true,
        pass: input.pass,
    }
}
