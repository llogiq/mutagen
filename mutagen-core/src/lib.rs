#![feature(proc_macro_span, specialization)]

mod mutagen_file;
mod mutation;
pub mod mutator;
pub mod optimistic;
mod runtime_config;

pub use mutagen_file::get_mutations_file;
pub use mutation::{BakedMutation, Mutation};
pub use runtime_config::MutagenRuntimeConfig;
