//! C alias policy, including callbacks retained beyond their registration call.
use crate::{owned_resource_id, OwnedResource, ResultFailure};
use prebindgen_proc_macro::prebindgen;

#[prebindgen]
pub fn alias_fixed(left: &mut OwnedResource, right: &OwnedResource) -> i64 {
    owned_resource_id(left) + owned_resource_id(right)
}
#[prebindgen]
pub struct AliasPair {
    pub first: OwnedResource,
    pub second: OwnedResource,
}
#[prebindgen]
pub fn alias_pair_new(first: OwnedResource, second: OwnedResource) -> AliasPair {
    AliasPair { first, second }
}
#[prebindgen]
pub fn alias_nested_sum(values: Vec<Vec<OwnedResource>>) -> i64 {
    values.iter().flatten().map(owned_resource_id).sum()
}
pub struct AliasHandlerImpl(Box<dyn Fn() -> AliasPair + Send + Sync>);
#[prebindgen]
pub type AliasHandler = AliasHandlerImpl;
#[prebindgen]
pub fn alias_handler_new(callback: impl Fn() -> AliasPair + Send + Sync + 'static) -> AliasHandler {
    AliasHandlerImpl(Box::new(callback))
}
#[prebindgen]
pub fn alias_handler_call(handler: &AliasHandler) -> Result<i64, ResultFailure> {
    let pair = (handler.0)();
    Ok(owned_resource_id(&pair.first) + owned_resource_id(&pair.second))
}

#[prebindgen]
pub fn alias_callback_rows(
    callback: impl Fn() -> Vec<OwnedResource>,
) -> Result<i64, ResultFailure> {
    Ok(callback().iter().map(owned_resource_id).sum())
}
