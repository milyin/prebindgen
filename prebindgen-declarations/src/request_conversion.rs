//! Embed request-free trees without hiding reverse-direction subtrees.
use crate::{Input, InputRequest, Output, OutputRequest};
impl Input {
    /// Embed all nodes, including arrow domains, into a preparation tree.
    pub fn into_preparation(self) -> Input<InputRequest, OutputRequest> {
        self.map_requests(&mut |never| match never {}, &mut |never| match never {})
    }
}
impl Output {
    /// Embed all nodes, including arrow domains, into a preparation tree.
    pub fn into_preparation(self) -> Output<OutputRequest, InputRequest> {
        self.map_requests(&mut |never| match never {}, &mut |never| match never {})
    }
}
