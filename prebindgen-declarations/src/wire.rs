//! Direction-specific metadata selected by the codec's prepared converters.
use crate::Codec;
use prebindgen_conversions::{Conversion, WireNode};
/// Input wire metadata declared by the prepared input converter.
pub type CodecInputWire<C> = <<C as Codec>::Input as Conversion>::Wire;
/// Input chain metadata declared by the prepared input converter.
pub type CodecInputChain<C> = <<C as Codec>::Input as Conversion>::Chain;
/// Input branch metadata declared by the prepared input converter.
pub type CodecInputBranch<C> = <<C as Codec>::Input as Conversion>::Branch;
/// Prepared input wire tree, independent of the opposite direction.
pub type CodecInputNode<C> = WireNode<CodecInputWire<C>, CodecInputChain<C>, CodecInputBranch<C>>;
/// Output wire metadata declared by the prepared output converter.
pub type CodecOutputWire<C> = <<C as Codec>::Output as Conversion>::Wire;
/// Output chain metadata declared by the prepared output converter.
pub type CodecOutputChain<C> = <<C as Codec>::Output as Conversion>::Chain;
/// Output branch metadata declared by the prepared output converter.
pub type CodecOutputBranch<C> = <<C as Codec>::Output as Conversion>::Branch;
/// Prepared output wire tree, independent of the opposite direction.
pub type CodecOutputNode<C> =
    WireNode<CodecOutputWire<C>, CodecOutputChain<C>, CodecOutputBranch<C>>;
