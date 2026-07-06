//! Destination-language **code generators** — self-contained declaration
//! models + renderers, independent of the binding pipeline (`api::core`)
//! and the language adapters (`api::lang`). Adapters build these models;
//! the generators render formatted source text and write files.

pub(crate) mod kotlin;
