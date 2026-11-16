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
pub type CodecInputNode<C> = WireNode<
    CodecInputWire<C>,
    CodecInputChain<C>,
    CodecInputBranch<C>,
    CodecInputAnnotation<C>,
    CodecInputContainer<C>,
>;
/// Output wire metadata declared by the prepared output converter.
pub type CodecOutputWire<C> = <<C as Codec>::Output as Conversion>::Wire;
/// Output chain metadata declared by the prepared output converter.
pub type CodecOutputChain<C> = <<C as Codec>::Output as Conversion>::Chain;
/// Output branch metadata declared by the prepared output converter.
pub type CodecOutputBranch<C> = <<C as Codec>::Output as Conversion>::Branch;
/// Prepared output wire tree, independent of the opposite direction.
pub type CodecOutputNode<C> = WireNode<
    CodecOutputWire<C>,
    CodecOutputChain<C>,
    CodecOutputBranch<C>,
    CodecOutputAnnotation<C>,
    CodecOutputContainer<C>,
>;

/// Input metadata for wire-free annotations.
pub type CodecInputAnnotation<C> = <<C as Codec>::Input as Conversion>::Annotation;
/// Input metadata for a container's internal representation.
pub type CodecInputContainer<C> = <<C as Codec>::Input as Conversion>::Container;

/// Output metadata for wire-free annotations.
pub type CodecOutputAnnotation<C> = <<C as Codec>::Output as Conversion>::Annotation;
/// Output metadata for a container's internal representation.
pub type CodecOutputContainer<C> = <<C as Codec>::Output as Conversion>::Container;
