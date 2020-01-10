//! Types and functions for communication between mutagen-processes
//!
//! Mutagen requires several communication-channles to function fully
//!
//! * The procedural macro informs the runner about all baked mutations
//! * The runner informs the test-suite about its mode (mutation or coverage) and additional required information (mutation_id, num_mutations)
//! * The mutators in the test suite inform the runner about coverage-hits
//!
//! Currently, communication from the procedural macro and test-suite is implemented via files in the `target/mutagen` directory.
//! The communication to the test-suite is implemented via environemnt variables
mod coverage;
mod mutagen_files;
mod mutation;
mod report;

pub use coverage::{CoverageCollection, CoverageHit};
pub use mutagen_files::*;
pub use mutation::{BakedMutation, Mutation};
pub use report::{MutagenReport, MutantStatus};
