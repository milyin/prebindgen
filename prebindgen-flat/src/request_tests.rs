//! A generator with its own request enums and no staged-conversion dependency.
use super::*;
use proc_macro2::TokenStream;
use quote::quote;

struct Input {
    nodes: Vec<crate::WireNode<syn::Type>>,
}
enum Read<'a> {
    Decode { bias: &'a i32 },
    Noop,
}
enum InputWire {
    Active,
    Value,
}
impl Conversion for Input {
    type Wire = syn::Type;
    type Group = ();
    type SlotFragment<'a> = Read<'a>;
    fn wire_nodes(&self) -> &[crate::WireNode<syn::Type>] {
        &self.nodes
    }
    fn code_fragment_slot(&self, mode: Read<'_>, expected: &syn::Type) -> Result<TokenStream> {
        match mode {
            Read::Decode { bias } => {
                if !same_type(expected, &syn::parse_quote!(i32)) {
                    return Err("expected i32".into());
                }
                let wire = &crate::wire_definitions(self.nodes.iter())
                    .next()
                    .expect("logical wire index out of range")
                    .name;
                Ok(quote!(i32::from(#wire) + #bias))
            }
            Read::Noop => {
                if !same_type(expected, &syn::parse_quote!(())) {
                    return Err("expected unit".into());
                }
                Ok(quote!(()))
            }
        }
    }
}
impl Input {
    fn code_fragment_wire(
        &self,
        pos: usize,
        mode: InputWire,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        let wire = crate::wire_definitions(self.nodes.iter())
            .nth(pos)
            .ok_or("wire out of range")?;
        match mode {
            InputWire::Active => {
                if !same_type(expected, &syn::parse_quote!(Option<bool>)) {
                    return Err("expected activity".into());
                }
                Ok(quote!(None))
            }
            InputWire::Value => {
                if !same_type(expected, &wire.rust_type()) {
                    return Err("expected wire type".into());
                }
                let name = &wire.name;
                Ok(quote!(#name))
            }
        }
    }
}
struct Output {
    nodes: Vec<crate::WireNode<syn::Type>>,
}
impl Conversion for Output {
    type Wire = syn::Type;
    type Group = ();
    // Deliberately different from the input: no shared context or phase vocabulary.
    type SlotFragment<'a> = &'a syn::Ident;
    fn wire_nodes(&self) -> &[crate::WireNode<syn::Type>] {
        &self.nodes
    }
    fn code_fragment_slot(&self, source: &syn::Ident, expected: &syn::Type) -> Result<TokenStream> {
        if !same_type(expected, &syn::parse_quote!(i64)) {
            return Err("expected i64".into());
        }
        Ok(quote!(i64::from(#source)))
    }
}
impl Output {
    fn code_fragment_wire(
        &self,
        pos: usize,
        source: &syn::Ident,
        expected: &syn::Type,
    ) -> Result<TokenStream> {
        crate::wire_definitions(self.nodes.iter())
            .nth(pos)
            .ok_or("wire out of range")?;
        self.code_fragment_slot(source, expected)
    }
}

#[test]
fn independent_protocols_compose_into_compilable_expressions() {
    let input = Input {
        nodes: vec![crate::WireNode::Wire(Wire {
            name: syn::parse_quote!(raw),
            kind: syn::parse_quote!(u8),
        })],
    };
    let output = Output {
        nodes: vec![crate::WireNode::Wire(Wire {
            name: syn::parse_quote!(out),
            kind: syn::parse_quote!(i64),
        })],
    };
    let decoded = syn::parse_quote!(decoded);
    let bias = 7;
    let value = input
        .code_fragment_slot(Read::Decode { bias: &bias }, &syn::parse_quote!(i32))
        .unwrap();
    let binding = bind_fragment(&decoded, &syn::parse_quote!(i32), value).unwrap();
    let result = output
        .code_fragment_wire(0, &decoded, &syn::parse_quote!(i64))
        .unwrap();
    let noop = input
        .code_fragment_slot(Read::Noop, &syn::parse_quote!(()))
        .unwrap();
    let active = input
        .code_fragment_wire(0, InputWire::Active, &syn::parse_quote!(Option<bool>))
        .unwrap();
    let raw = input
        .code_fragment_wire(0, InputWire::Value, &syn::parse_quote!(u8))
        .unwrap();
    let source = quote! {
        fn wrapper(raw: u8) -> i64 {
            let _: () = #noop;
            let active: Option<bool> = #active;
            assert!(active.is_none());
            assert_eq!(#raw, raw);
            #binding
            #result
        }
        fn main() { assert_eq!(wrapper(5), 12); }
    };
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("requests.rs");
    let executable = dir.path().join("requests");
    std::fs::write(&file, source.to_string()).unwrap();
    let compiled = std::process::Command::new("rustc")
        .arg(&file)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert!(std::process::Command::new(executable)
        .status()
        .unwrap()
        .success());
    assert!(input
        .code_fragment_slot(Read::Decode { bias: &bias }, &syn::parse_quote!(String))
        .is_err());
    assert!(input
        .code_fragment_wire(1, InputWire::Active, &syn::parse_quote!(Option<bool>))
        .is_err());
    assert!(output
        .code_fragment_wire(1, &decoded, &syn::parse_quote!(i64))
        .is_err());
}

#[test]
fn binding_helper_requires_one_expression() {
    let name = syn::parse_quote!(answer);
    let expected = syn::parse_quote!(i32);
    assert!(bind_fragment(&name, &expected, quote!()).is_err());
    assert!(bind_fragment(&name, &expected, quote!(let x = 1;)).is_err());
    assert!(bind_fragment(&name, &expected, quote!(1; 2)).is_err());
    assert!(bind_fragment(
        &name,
        &expected,
        quote!({
            let x = 1;
            x + 2
        })
    )
    .is_ok());
    assert!(bind_fragment(&name, &syn::parse_quote!(()), quote!(())).is_ok());
}

#[test]
fn one_logical_sequence_supports_packed_and_callback_readers() {
    enum Delivery<'a> {
        Pack(&'a syn::Ident),
        Visit {
            source: &'a syn::Ident,
            callback: &'a syn::Ident,
        },
    }
    struct Sequence {
        schema: Vec<crate::WireNode<syn::Type>>,
    }
    impl Conversion for Sequence {
        type Wire = syn::Type;
        type Group = ();
        type SlotFragment<'a> = Delivery<'a>;
        fn wire_nodes(&self) -> &[crate::WireNode<syn::Type>] {
            &self.schema
        }
        fn code_fragment_slot(&self, mode: Delivery<'_>, _: &syn::Type) -> Result<TokenStream> {
            Ok(match mode {
                Delivery::Pack(source) => {
                    quote!(NativeRows {data:#source.as_ptr(),len:#source.len()})
                }
                Delivery::Visit { source, callback } => {
                    quote!({for row in #source {#callback(row.x,row.y);}})
                }
            })
        }
    }
    let mut schema: Vec<crate::WireNode<syn::Type>> = ["x", "y"]
        .into_iter()
        .map(|name| Wire {
            name: syn::parse_str(name).unwrap(),
            kind: syn::parse_quote!(i64),
        })
        .map(crate::WireNode::Wire)
        .collect::<Vec<_>>();
    schema = vec![crate::WireNode::Sequence {
        metadata: None,
        element: std::mem::take(&mut schema),
    }];
    let conversion = Sequence { schema };
    assert_eq!(
        crate::wire_definitions(conversion.wire_nodes().iter()).count(),
        2
    );
    let source = syn::parse_quote!(values);
    let callback = syn::parse_quote!(callback);
    let pack = conversion
        .code_fragment_slot(Delivery::Pack(&source), &syn::parse_quote!(NativeRows))
        .unwrap();
    let visit = conversion
        .code_fragment_slot(
            Delivery::Visit {
                source: &source,
                callback: &callback,
            },
            &syn::parse_quote!(()),
        )
        .unwrap();
    let code = quote! {
        #[repr(C)] struct Row {x:i64,y:i64}
        #[repr(C)] struct NativeRows {data:*const Row,len:usize}
        fn pack(values:&[Row])->NativeRows {#pack}
        fn visit(values:&[Row],mut callback:impl FnMut(i64,i64)) {#visit}
        fn main() {
            let values=[Row {x:1,y:2},Row {x:3,y:4}];
            let rows=pack(&values);assert_eq!(rows.len,2);
            let packed=unsafe {std::slice::from_raw_parts(rows.data,rows.len)};
            let mut visited=vec![];visit(&values,|x,y|visited.push((x,y)));
            assert_eq!(visited,packed.iter().map(|row|(row.x,row.y)).collect::<Vec<_>>());
        }
    };
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("readers.rs");
    let binary = dir.path().join("readers");
    std::fs::write(&file, code.to_string()).unwrap();
    let build = std::process::Command::new("rustc")
        .arg(&file)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(std::process::Command::new(binary)
        .status()
        .unwrap()
        .success());
}
