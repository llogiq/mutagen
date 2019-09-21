//! types and traits for optimistic mutators

mod not_to_none;

pub use not_to_none::NotToNone;

#[cfg(any(test, feature = "self_test"))]
pub use not_to_none::optimistc_types::{TypeWithNotOtherOutput, TypeWithNotTarget};
