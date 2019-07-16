use std::ops::Not;

/// trait that is used to optimistically remove a negation `!` from an expression
///
/// This trait provides a function `may_none` that passes the input value unchanged
/// If the value cannot be converted to the output type of the negation using `Into`, the function panics.
pub trait NotToNone {
    type Output;
    // do nothing
    fn may_none(self) -> Self::Output;
}

impl<T> NotToNone for T
where
    T: Not,
{
    type Output = <T as Not>::Output;

    default fn may_none(self) -> <T as Not>::Output {
        panic!("optimistic type mismatch: negation output is different type");
    }
}

impl<T> NotToNone for T
where
    T: Not,
    T: Into<<T as Not>::Output>,
{
    fn may_none(self) -> Self::Output {
        self.into()
    }
}
