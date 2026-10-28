#![allow(non_camel_case_types, unused_variables)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
#[cfg(test)]
mod owner_views;
