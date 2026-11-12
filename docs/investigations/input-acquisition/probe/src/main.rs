//! Regression controls for the former mixed-input generation restriction.
use prebindgen_c::{CBindings, CCodec, Cbindgen};
use prebindgen_declarations::{fun, Codec, Condition, InputLeaf};
use prebindgen_flat::Result;
use quote::format_ident;

fn model(source: &str) -> Result<CBindings> {
    let source = format!("pub struct Packet {{ pub id:u32 }} {source}");
    let source = syn::parse_file(&source).map_err(|e| e.to_string())?;
    let flat = CBindings::parse(
        source
            .items
            .into_iter()
            .map(|item| (item, Default::default())),
    )?;
    Cbindgen::new(flat)
        .opaque_ptr("item", syn::parse_quote!(Item))
        .opaque_ptr("packet", syn::parse_quote!(Packet))
        .function(fun!(use_both))
        .panic()
        .prepare()
}
fn replace_with_leaf(flat: &mut CBindings, index: usize) -> Result<()> {
    let ty = flat.function("use_both")?.params[index].ty.ty.clone();
    let wire = format_ident!("manual_wire_{index}");
    let target = format_ident!("manual_value_{index}");
    let codec = CCodec::raw();
    let mut plan = codec.input_leaf(InputLeaf::new(
        flat,
        &ty,
        &(),
        wire,
        target,
        &Condition::default(),
    ))?;
    // Complete the leaf's normal adapter validation and receipt-channel setup.
    codec.finish_input(&mut plan, &())?;
    codec.validate_input(&plan)?;
    flat.function_mut("use_both")?.params[index].ty.slot = Some(plan);
    Ok(())
}
fn run_case(label: &str, source: &str, raw: &[usize]) -> Result<()> {
    let mut flat = model(source)?;
    for &index in raw {
        replace_with_leaf(&mut flat, index)?;
    }
    let generated = prebindgen_c::generate(&flat)?;
    syn::parse_file(&generated.rust).map_err(|e| e.to_string())?;
    println!("{label}: generation succeeds");
    Ok(())
}
fn main() -> Result<()> {
    let owned = "pub struct Item {pub value:i64} pub fn use_both(first:&Item,second:Item){}";
    let borrowed = "pub struct Item {pub value:i64} pub fn use_both(first:&Item,second:&Item){}";
    let scalar = "pub struct Item {pub value:i64} pub fn use_both(first:&Item,second:i64){}";
    run_case("compiled inputs", owned, &[])?;
    run_case("deferred shared + compiled move", owned, &[0])?;
    run_case("compiled shared + deferred move", owned, &[1])?;
    run_case("deferred inputs", owned, &[0, 1])?;
    run_case("mixed shared borrows", borrowed, &[0])?;
    run_case("deferred borrow + compiled scalar", scalar, &[0])?;
    let unrelated = "pub struct Item {pub value:i64} pub fn use_both(first:&Item,second:Packet){}";
    run_case("unrelated compiled inputs", unrelated, &[])?;
    run_case("unrelated deferred borrow + compiled move", unrelated, &[0])?;
    Ok(())
}
