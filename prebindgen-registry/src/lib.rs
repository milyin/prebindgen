//! Compatibility import path for build-script declarations. All implementation is in the new pipeline.
#![warn(missing_docs)]
pub use prebindgen_tools::{
    expand_param, expand_return, fields, fun, ExpandDecl, ExpandParamDecl, ExpandReturnDecl,
    FieldsDecl, FunctionDecl,
};
