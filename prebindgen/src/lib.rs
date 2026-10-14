//! Source capture and build configuration shared by the binding pipeline.

/// File name for storing the crate name
const CRATE_NAME_FILE: &str = "crate_name.txt";

/// File name for storing enabled Cargo features collected in build.rs
const FEATURES_FILE: &str = "features.txt";

/// Default group name for items without explicit group name
pub const DEFAULT_GROUP_NAME: &str = "default";

pub(crate) mod api;
pub(crate) mod codegen;

pub use crate::api::{
    buildrs::{
        get_all_features, get_enabled_features, get_prebindgen_out_dir, init_prebindgen_out_dir,
        is_feature_enabled,
    },
    record::SourceLocation,
    source::Source,
    utils::{edition::RustEdition, target_triple::TargetTriple},
};

/// Utilities shared with consumers of captured source records.
pub mod utils {
    #[doc(hidden)]
    pub use crate::api::utils::jsonl::{read_jsonl_file, write_to_jsonl_file};
    pub use crate::api::utils::target_triple::TargetTriple;
}

#[doc(hidden)]
pub use crate::api::record::Record;
#[doc(hidden)]
pub use crate::api::record::RecordKind;
