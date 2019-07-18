//! types and traits for optimistic mutators

mod add_to_sub;
mod not_to_none;

pub use add_to_sub::AddToSub;
pub use not_to_none::NotToNone;

#[cfg(feature = "self_test")]
pub use not_to_none::optimistc_types::{TypeWithNotOtherOutput, TypeWithNotTarget};
