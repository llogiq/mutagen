use std::ops::{Add, Sub};

/// trait that is used to optimistically change `+` to `-`.
///
/// This trait provides a function `may_sub` that can replace any occurence of `+`.
/// If the type `Sub` is not implemented for the appropriate types, the function panics.
pub trait AddToSub<R> {
    type Output;
    fn may_sub(self, r: R) -> Self::Output;
}

impl<L, R> AddToSub<R> for L
where
    L: Add<R>,
{
    type Output = <L as Add<R>>::Output;

    default fn may_sub(self, _r: R) -> <L as Add<R>>::Output {
        panic!("not sub");
    }
}

impl<L, R> AddToSub<R> for L
where
    L: Add<R>,
    L: Sub<R>,
    <L as Sub<R>>::Output: Into<<L as Add<R>>::Output>,
{
    fn may_sub(self, r: R) -> Self::Output {
        (self - r).into()
    }
}
