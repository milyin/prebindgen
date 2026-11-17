#![allow(non_camel_case_types, unused_variables)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
#[cfg(test)]
mod owner_views;

#[cfg(test)]
mod conversion_allocations;
#[cfg(test)]
mod output_owners;

#[cfg(test)]
static GETTER_TESTS: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
static INLINE_OWNER_TESTS: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[allow(non_camel_case_types, unused_variables)]
pub mod inline_owners {
    include!(concat!(env!("OUT_DIR"), "/owners.rs"));
    include!("inline_owner_tests.rs");
}
