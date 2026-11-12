//! Embed request-free trees without hiding reverse-direction subtrees.
use crate::{DeclarationInput, DeclarationOutput, InputRequest, OutputRequest};
impl DeclarationInput {
    /// Embed all nodes, including arrow domains, into a preparation tree.
    pub fn into_preparation(self) -> DeclarationInput<InputRequest, OutputRequest> {
        self.map_requests(&mut |never| match never {}, &mut |never| match never {})
    }
}
impl DeclarationOutput {
    /// Embed all nodes, including arrow domains, into a preparation tree.
    pub fn into_preparation(self) -> DeclarationOutput<OutputRequest, InputRequest> {
        self.map_requests(&mut |never| match never {}, &mut |never| match never {})
    }
}
