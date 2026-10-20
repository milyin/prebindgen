//! Build-script declaration types and macros, re-exported from `prebindgen_declarations`.
#![warn(missing_docs)]
pub use prebindgen_declarations::{
    expand_param, expand_return, fields, fun, ExpandDecl, ExpandParamDecl, ExpandReturnDecl,
    FieldsDecl, FunctionDecl, Projection, ProjectionMode,
};
