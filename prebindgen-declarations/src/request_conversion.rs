//! Embed request-free trees without hiding reverse-direction subtrees.
use crate::{DeclInput, DeclInputRequest, DeclOutput, DeclOutputRequest};
impl DeclInput {
    /// Embed all nodes, including arrow domains, into a preparation tree.
    pub fn into_preparation(self) -> DeclInput<DeclInputRequest, DeclOutputRequest> {
        self.map_requests(&mut |never| match never {}, &mut |never| match never {})
    }
}
impl DeclOutput {
    /// Embed all nodes, including arrow domains, into a preparation tree.
    pub fn into_preparation(self) -> DeclOutput<DeclInputRequest, DeclOutputRequest> {
        self.map_requests(&mut |never| match never {}, &mut |never| match never {})
    }
}
