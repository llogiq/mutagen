// proc-macro-span feature is required because `proc_macro2` does not export the api for getting source files for spans
#![feature(proc_macro_span)]
#![feature(specialization)]

mod runtime_config;
mod transformer;

pub mod comm;
pub mod mutator;
pub mod optimistic;

pub use runtime_config::CoverageHit;
pub use runtime_config::MutagenRuntimeConfig;

pub use transformer::do_transform_item_fn;
