//! Reproduce the current generation restriction without changing the generator.
use prebindgen_c::{Bindings, CCodec, Cbindgen};
use prebindgen_declarations::{fun, Codec, Condition, InputLeaf};
use prebindgen_flat::Result;
use quote::format_ident;

fn model(source: &str) -> Result<Bindings> {
    let source = syn::parse_file(source).map_err(|e| e.to_string())?;
    let flat = Bindings::parse(
        source
            .items
            .into_iter()
            .map(|item| (item, Default::default())),
    )?;
    Cbindgen::new(flat)
        .opaque_ptr("item", syn::parse_quote!(Item))
        .function(fun!(use_both))
        .panic()
        .prepare()
}
fn replace_with_leaf(flat: &mut Bindings, index: usize) -> Result<()> {
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
fn describe(label: &str, flat: &Bindings) -> Result<()> {
    print!("{label} resources:");
    for p in &flat.function("use_both")?.params {
        let plan = p.ty.slot.as_ref().ok_or("missing input")?;
        for r in &plan.resources {
            print!(
                " {}={:?}/registered:{}/self_counted:{}",
                p.name, r.access, r.registered, r.records_consumption
            );
        }
    }
    println!();
    Ok(())
}
fn run_case(label: &str, source: &str, raw: &[usize], reject: bool) -> Result<()> {
    let mut flat = model(source)?;
    for &index in raw {
        replace_with_leaf(&mut flat, index)?;
    }
    describe(label, &flat)?;
    match prebindgen_c::generate(&flat) {
        Ok(generated) if !reject => {
            syn::parse_file(&generated.rust).map_err(|e| e.to_string())?;
            println!("{label}: generation succeeds");
        }
        Err(error)
            if reject
                && error.contains(
                    "cannot mix deferred resource conversions with eagerly constructed inputs",
                ) =>
        {
            println!("{label}: expected generation rejection: {error}");
        }
        Err(error) => return Err(format!("{label}: unexpected error: {error}")),
        Ok(_) => {
            return Err(format!(
                "{label}: the expected restriction is no longer present; revisit the investigation"
            ))
        }
    }
    Ok(())
}
fn main() -> Result<()> {
    let owned = "pub struct Item {pub value:i64} pub fn use_both(first:&Item,second:Item){}";
    let borrowed = "pub struct Item {pub value:i64} pub fn use_both(first:&Item,second:&Item){}";
    let scalar = "pub struct Item {pub value:i64} pub fn use_both(first:&Item,second:i64){}";
    run_case("compiled inputs", owned, &[], false)?;
    run_case("deferred shared + compiled move", owned, &[0], true)?;
    run_case("compiled shared + deferred move", owned, &[1], true)?;
    run_case("deferred inputs", owned, &[0, 1], false)?;
    run_case("mixed shared borrows", borrowed, &[0], true)?;
    run_case("deferred borrow + compiled scalar", scalar, &[0], false)?;
    Ok(())
}
