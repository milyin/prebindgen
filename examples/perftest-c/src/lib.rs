#![allow(non_camel_case_types, unused_variables)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
#[cfg(test)]
mod owner_views;

#[cfg(test)]
mod alias_allocations;

#[cfg(test)]
static GETTER_TESTS: std::sync::Mutex<()> = std::sync::Mutex::new(());
