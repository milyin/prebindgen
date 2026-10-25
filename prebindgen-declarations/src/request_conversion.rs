//! Embed explicit operations into preparation trees.
use crate::{Input, InputRequest, Output, OutputRequest};

impl Input {
    /// Embed this subtree in preparation. This keeps its explicit operations;
    /// source-dependent normalization still runs when the tree is resolved.
    pub fn into_preparation(self) -> Input<InputRequest> {
        match self {
            Input::Request(never) => match never {},
            Input::Leaf => Input::Leaf,
            Input::Optional(child) => Input::Optional(Box::new(child.into_preparation())),
            Input::Sequence(child) => Input::Sequence(Box::new(child.into_preparation())),
            Input::Fields(children) => {
                Input::Fields(children.into_iter().map(Input::into_preparation).collect())
            }
            Input::Record(children) => {
                Input::Record(children.into_iter().map(Input::into_preparation).collect())
            }
            Input::Selector(children) => {
                Input::Selector(children.into_iter().map(Input::into_preparation).collect())
            }
            Input::Constructor { function, args } => Input::Constructor {
                function,
                args: args.into_iter().map(Input::into_preparation).collect(),
            },
        }
    }
}
impl Output {
    /// Embed this subtree in preparation. This keeps its explicit operations;
    /// source-dependent normalization still runs when the tree is resolved.
    pub fn into_preparation(self) -> Output<OutputRequest> {
        match self {
            Output::Request(never) => match never {},
            Output::Leaf => Output::Leaf,
            Output::Discard => Output::Discard,
            Output::Projected(policy) => Output::Projected(policy),
            Output::Optional(child) => Output::Optional(Box::new(child.into_preparation())),
            Output::Named { name, output } => Output::Named {
                name,
                output: Box::new(output.into_preparation()),
            },
            Output::Sequence {
                element,
                mode,
                delivery,
            } => Output::Sequence {
                element: Box::new(element.into_preparation()),
                mode,
                delivery,
            },
            Output::Result { ok, error } => Output::Result {
                ok: Box::new(ok.into_preparation()),
                error,
            },
            Output::Fields(children) => {
                Output::Fields(children.into_iter().map(Output::into_preparation).collect())
            }
            Output::OwnedFields(children) => {
                Output::OwnedFields(children.into_iter().map(Output::into_preparation).collect())
            }
            Output::Combine(children) => {
                Output::Combine(children.into_iter().map(Output::into_preparation).collect())
            }
            Output::Record { fields, mode } => Output::Record {
                fields: fields.into_iter().map(Output::into_preparation).collect(),
                mode,
            },
            Output::Getter { function, output } => Output::Getter {
                function,
                output: Box::new(output.into_preparation()),
            },
            Output::Into { function, output } => Output::Into {
                function,
                output: Box::new(output.into_preparation()),
            },
        }
    }
}
