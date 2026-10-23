//! Reader-selected ABI access, separate from the common logical schema.
use crate::{TransportLayout, TransportWires};
use prebindgen_flat::{Conversion, Function, Result, Wire, WireKind};
use proc_macro2::TokenStream;

/// Opt-in native transport used by the staged C/JNI writers. Implementing
/// Conversion alone imposes no ABI ordering, packing or delivery strategy.
pub trait NativeConversion: Conversion {
    /// Reader-specific requests concerning a physical transport wire.
    type WireFragment<'a>;
    /// Physical layout chosen by this reader, independent of layout().
    fn transport(&self) -> &TransportLayout<Self::Wire, Self::Group>;
    /// Emit a physical-wire expression in this transport's own index domain.
    fn code_fragment_wire(
        &self,
        pos: usize,
        mode: Self::WireFragment<'_>,
        expected: &syn::Type,
    ) -> Result<TokenStream>;
}
/// Derived native enumeration. A blanket implementation ensures it follows the
/// transport tree and cannot be overridden independently.
pub trait NativeConversionExt: NativeConversion {
    /// Physical wires selected by the staged transport, in its native order.
    fn native_wires(&self) -> TransportWires<'_, Self::Wire, Self::Group> {
        self.transport().iter()
    }
}
impl<T: NativeConversion + ?Sized> NativeConversionExt for T {}
/// Native input enumeration for generators selecting NativeConversion.
pub trait NativeFunction {
    /// Wire description selected by the native reader.
    type Wire: WireKind;
    /// Source input slots lowered into this reader's native parameters.
    /// Generator infrastructure (callbacks, contexts, acknowledgements) may be
    /// added separately by the reader; this is not a common schema guarantee.
    fn native_inputs(&self) -> Result<std::vec::IntoIter<&Wire<Self::Wire>>>;
}
impl<I: NativeConversion, O: Conversion> NativeFunction for Function<I, O> {
    type Wire = I::Wire;
    fn native_inputs(&self) -> Result<std::vec::IntoIter<&Wire<I::Wire>>> {
        Ok(self
            .input_conversions()?
            .into_iter()
            .flat_map(|slot| slot.native_wires())
            .collect::<Vec<_>>()
            .into_iter())
    }
}
