#![feature(proc_macro_span)]
#![feature(box_syntax)]
#![feature(vec_remove_item)]
#![feature(specialization)]

pub mod mutagen_file;
mod mutation;
mod runtime_config;
mod transformer;

pub mod mutate_args;
pub mod mutator;
pub mod optimistic;

pub use mutation::{BakedMutation, Mutation};
pub use runtime_config::MutagenRuntimeConfig;
