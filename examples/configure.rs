use prebindgen_flat::{Flat,Result};
use prebindgen_tools::{Input as I,Output as O,Slot};

/// Both adapters consume exactly this model, with converters pinned to occurrences.
fn configure(flat: &mut Flat<Slot>) -> Result<()> {
    let key=I::selector([I::Handle,I::constructor("keyexpr_from_string",[I::Text])]);
    let payload=I::selector([
        I::Handle,
        I::fields([key.clone(),I::Handle]),
        I::constructor("payload_from_string",[key.clone(),I::Text]),
        I::constructor("payload_from_slice",[key.clone(),I::Bytes]),
    ]);
    let f=flat.function_mut("foo")?;
    f.params[0].ty.slot=Slot::Input(key.clone());
    f.params[1].ty.slot=Slot::Input(payload);
    // Same source type, different occurrence: y only accepts a handle.
    f.params[2].ty.slot=Slot::Input(I::Handle);
    f.ret.slot=Slot::Output(O::combine([
        O::Handle,
        O::fields([
            O::combine([O::ClonedHandle,O::getter("keyexpr_to_string",O::Text)]),
            O::ClonedHandle,
        ]),
    ]));
    for (name,inputs,output) in [
        ("keyexpr_from_string",vec![I::Text],O::Handle),
        ("keyexpr_text",vec![I::Handle],O::Text),
        ("bytes_from_slice",vec![I::Bytes],O::Handle),
        ("bytes_read",vec![I::Handle],O::Bytes),
        ("payload_from_string",vec![key.clone(),I::Text],O::Handle),
        ("payload_from_slice",vec![key,I::Bytes],O::Handle),
        ("payload_text",vec![I::Handle],O::Text),
        ("noop",vec![],O::fields([])),
    ]{
        let f=flat.function_mut(name)?;
        for (p,input) in f.params.iter_mut().zip(inputs){p.ty.slot=Slot::Input(input);}
        f.ret.slot=Slot::Output(output);
    }
    Ok(())
}
