#![feature(specialization, atomic_min_max, try_from)]
/// Welcome to the mutagen crate. Your entry point will probably be cargo-mutagen, so install it
/// right away.
#[macro_use] extern crate lazy_static;

/// The mutate proc_macro_attribute
use std::env;
use std::ops::*;
use std::sync::atomic::{AtomicUsize, Ordering};

mod iterators;
pub mod bounded_loop;

mod coverage;
pub use coverage::report_coverage;

pub trait Defaulter: Sized {
    fn get_default(count: usize, flag: &AtomicUsize, mask: usize) -> Option<Self>;
}

impl<X> Defaulter for X {
    default fn get_default(_count: usize, _flag: &AtomicUsize, _mask: usize) -> Option<X> { None }
}

impl<X: Default> Defaulter for X {
    fn get_default(count: usize, flag: &AtomicUsize, mask: usize) -> Option<X> {
        if now(count, flag, mask) {
            Some(Default::default())
        } else {
            None
        }
    }
}

/// Here we don't use an `Option<T>` because unlike with `Defaulter`, `T` may
/// be unsized. However, this is all OK because `Clone: Sized`
pub trait MayClone<T> {
    fn may_clone(&self) -> bool;
    fn clone(&self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self;
}

impl<T> MayClone<T> for T {
    default fn may_clone(&self) -> bool { false }
    default fn clone(&self, _mc: usize, _cov: &AtomicUsize, _mask: usize) -> Self { unimplemented!() }
}

impl<T: Clone> MayClone<T> for T {
    fn may_clone(&self) -> bool { true }
    fn clone(&self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self {
        report_coverage(mutation_count..(mutation_count + 1), coverage, mask);
        Clone::clone(&self)
    }
}

pub trait Step {
    fn inc(self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self;
    fn dec(self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self;
}

macro_rules! step_impl {
    ($($ty:ty), *) => {
        $(
            impl Step for $ty {
                fn inc(self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self {
                    match self.checked_add(1) {
                        Some(x) => {
                            if now(mutation_count, coverage, mask) {
                                x
                            } else {
                                self
                            }
                        }
                        None => { self }
                    }
                }
                fn dec(self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self {
                    match self.checked_add(1) {
                        Some(x) => {
                            if now(mutation_count, coverage, mask) {
                                x
                            } else {
                                self
                            }
                        }
                        None => { self }
                    }
                }
            }
        )*
    }
}

step_impl!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

macro_rules! mutate_binop {
    ($op_ty:ident, $op_fn:ident, $found_ty:ident, $found_fn:ident, $repl_ty:ident, $repl_fn:ident) => {
        pub trait $op_ty<Rhs = Self> {
            type Output;
            fn $op_fn(self, _rhs: Rhs, _mutation_count: usize, _coverage: &AtomicUsize, _mask: usize) -> Self::Output;
        }

        impl<T, Rhs> $op_ty<Rhs> for T
        where T: $found_ty<Rhs> {
            type Output = <T as $found_ty<Rhs>>::Output;
            default fn $op_fn(self, rhs: Rhs, _mutation_count: usize, _coverage: &AtomicUsize, _mask: usize) -> Self::Output {
                $found_ty::$found_fn(self, rhs)
            }
        }

        impl<T, Rhs> $op_ty<Rhs> for T
        where T: $found_ty<Rhs>,
              T: $repl_ty<Rhs>,
             <T as $repl_ty<Rhs>>::Output: Into<<T as $found_ty<Rhs>>::Output> {
            fn $op_fn(self, rhs: Rhs, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self::Output {
                if now(mutation_count, coverage, mask) {
                    $repl_ty::$repl_fn(self, rhs).into()
                } else {
                    $found_ty::$found_fn(self, rhs)
                }
            }
        }
    }
}

mutate_binop!(AddSub, add_sub, Add, add, Sub, sub);
mutate_binop!(SubAdd, sub_add, Sub, sub, Add, add);
mutate_binop!(MulDiv, mul_div, Mul, mul, Div, div);
mutate_binop!(DivMul, div_mul, Div, div, Mul, mul);
mutate_binop!(ShlShr, shl_shr, Shl, shl, Shr, shr);
mutate_binop!(ShrShl, shr_shl, Shr, shr, Shl, shl);
mutate_binop!(BitAndBitOr, bitand_bitor, BitAnd, bitand, BitOr, bitor);
mutate_binop!(BitOrBitAnd, bitor_bitand, BitOr, bitor, BitAnd, bitand);

macro_rules! mutate_assignop {
    ($op_ty:ident, $op_fn:ident, $found_ty:ident, $found_fn:ident, $repl_ty:ident, $repl_fn:ident) => {
        pub trait $op_ty<Rhs=Self> {
            fn $op_fn(&mut self, _rhs: Rhs, _mutation_count: usize, _coverage: &AtomicUsize, mask: usize);
        }

        impl<T, R> $op_ty<R> for T where T: $found_ty<R> {
            default fn $op_fn(&mut self, rhs: R, _mutation_count: usize, _coverage: &AtomicUsize, _mask: usize) {
                $found_ty::$found_fn(self, rhs);
            }
        }

        impl<T, R> $op_ty<R> for T
        where T: $found_ty<R>,
              T: $repl_ty<R> {
            fn $op_fn(&mut self, rhs: R, mutation_count: usize, coverage: &AtomicUsize, mask: usize) {
                if now(mutation_count, coverage, mask) {
                    $repl_ty::$repl_fn(self, rhs);
                } else {
                    $found_ty::$found_fn(self, rhs);
                }
            }
        }
    }
}

mutate_assignop!(AddSubAssign, add_sub_assign, AddAssign, add_assign, SubAssign, sub_assign);
mutate_assignop!(SubAddAssign, sub_add_assign, SubAssign, sub_assign, AddAssign, add_assign);
mutate_assignop!(MulDivAssign, mul_div_assign, MulAssign, mul_assign, DivAssign, div_assign);
mutate_assignop!(DivMulAssign, div_mul_assign, DivAssign, div_assign, MulAssign, mul_assign);
mutate_assignop!(ShlShrAssign, shl_shr_assign, ShlAssign, shl_assign, ShrAssign, shr_assign);
mutate_assignop!(ShrShlAssign, shr_shl_assign, ShrAssign, shr_assign, ShlAssign, shl_assign);
mutate_assignop!(BitAndBitOrAssign, bitand_bitor_assign, BitAndAssign, bitand_assign, BitOrAssign, bitor_assign);
mutate_assignop!(BitOrBitAndAssign, bitor_bitand_assign, BitOrAssign, bitor_assign, BitAndAssign, bitand_assign);

macro_rules! mutate_unop {
    ($op_trait:ident, $op_fn:ident, $found_trait:ident, $found_fn:ident) => {
        pub trait $op_trait {
            type Output;
            fn $op_fn(self, _mutation_count: usize, _coverage: &AtomicUsize, _mask: usize) -> Self::Output;
        }

        impl<T> $op_trait for T where T: $found_trait {
            type Output = <T as $found_trait>::Output;
            default fn $op_fn(self, _mutation_count: usize, _cov: &AtomicUsize, _mask: usize) -> Self::Output {
                $found_trait::$found_fn(self)
            }
        }

        impl<T> $op_trait for T where T: $found_trait, T: Into<<T as $found_trait>::Output> {
            fn $op_fn(self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self::Output {
                if now(mutation_count, coverage, mask) {
                    self.into()
                } else {
                    $found_trait::$found_fn(self)
                }
            }
        }
    }
}

mutate_unop!(MayNot, may_not, Not, not);
mutate_unop!(MayNeg, may_neg, Neg, neg);

lazy_static! {
    static ref MU: Mutagen = {
        let count = env::var("MUTATION_COUNT").map(|s|s.parse().unwrap_or(0)).unwrap_or(0);
        Mutagen { x: AtomicUsize::new(count) }
    };
}

/// A global helper struct to keep a mutation count
///
/// This is used by the Mutagen crate to simplify the code it inserts.
/// You should have no business using it manually.
#[doc(hidden)]
pub struct Mutagen {
    x: AtomicUsize,
}

impl Mutagen {
    /// get the current mutation count
    pub fn get(&self) -> usize {
        self.x.load(Ordering::Relaxed)
    }

    /// check if the argument matches the current mutation count
    ///
    /// this simplifies operations like `if mutagen::MU.now(42) { .. }`
    pub fn now(&self, n: usize) -> bool {
        self.get() == n
    }

    pub fn diff(&self, n: usize) -> usize {
        self.get().wrapping_sub(n)
    }

    /// increment the mutation count
    pub fn next(&self) {
        self.x.fetch_add(1, Ordering::SeqCst);
    }

    /// use with if expressions, e.g. `if MU.t(..) { .. } else { .. }`
    pub fn t(&self, t: bool, n: usize) -> bool {
        match self.diff(n) {
            0 => true,
            1 => false,
            2 => !t,
            _ => t,
        }
    }

    /// use with while expressions, e.g. `while Mu.w(..) { .. }`
    ///
    /// this never leads to infinite loops
    pub fn w(&self, t: bool, n: usize) -> bool {
        if self.now(n) {
            false
        } else {
            t
        }
    }

    /// use instead of `==`
    pub fn eq<R, T: PartialEq<R>>(&self, x: T, y: R, n: usize) -> bool {
        match self.diff(n) {
            0 => true,
            1 => false,
            2 => x != y,
            _ => x == y,
        }
    }

    /// use instead of `!=`
    pub fn ne<R, T: PartialEq<R>>(&self, x: T, y: R, n: usize) -> bool {
        match self.diff(n) {
            0 => true,
            1 => false,
            2 => x == y,
            _ => x != y,
        }
    }

    /// use instead of `>`
    pub fn gt<R, T: PartialOrd<R>>(&self, x: &T, y: &R, n: usize) -> bool {
        match self.diff(n) {
            0 => false,
            1 => true,
            2 => x < y,
            3 => x <= y,
            4 => x >= y,
            5 => x == y,
            6 => x != y,
            _ => x > y,
        }
    }

    /// use instead of `>`
    pub fn lt<R, T: PartialOrd<R>>(&self, x: &T, y: &R, n: usize) -> bool {
        match self.diff(n) {
            0 => false,
            1 => true,
            2 => x > y,
            3 => x >= y,
            4 => x <= y,
            5 => x == y,
            6 => x != y,
            _ => x < y,
        }
    }

    /// use instead of `>=`
    pub fn ge<R, T: PartialOrd<R>>(&self, x: &T, y: &R, n: usize) -> bool {
        match self.diff(n) {
            0 => false,
            1 => true,
            2 => x < y,
            3 => x <= y,
            4 => x > y,
            5 => x == y,
            6 => x != y,
            _ => x >= y,
        }
    }

    /// use instead of `>=`
    pub fn le<R, T: PartialOrd<R>>(&self, x: &T, y: &R, n: usize) -> bool {
        match self.diff(n) {
            0 => false,
            1 => true,
            2 => x > y,
            3 => x >= y,
            4 => x < y,
            5 => x == y,
            6 => x != y,
            _ => x <= y,
        }
    }

    pub fn forloop<'a, I: Iterator + 'a>(&self, i: I, n: usize) -> Box<Iterator<Item=I::Item> + 'a> {
        match self.diff(n) {
            0 => Box::new(iterators::NoopIterator{inner: i}),
            1 => Box::new(i.skip(1)),
            2 => Box::new(iterators::SkipLast::new(i)),
            3 => Box::new(iterators::SkipLast::new(i.skip(1))),
            _ => Box::new(i),
        }
    }

    pub fn inc<T: Step>(&self, t: T, n: usize, coverage: &AtomicUsize, mask: usize)-> T {
        t.inc(n, coverage, mask)
    }

    pub fn dec<T: Step>(&self, t: T, n: usize, coverage: &AtomicUsize, mask: usize) -> T {
        t.dec(n, coverage, mask)
    }

    pub fn inc_dec<T: Step>(&self, t: T, n: usize, coverage: &AtomicUsize, mask: usize) -> T {
        match self.diff(n) {
            0 => t.inc(n, coverage, mask),
            1 => t.dec(n, coverage, mask),
            _ => t,
        }
    }
}

/// get the current mutation count
pub fn get() -> usize {
    MU.get()
}

/// check if the argument matches the current mutation count
///
/// this simplifies operations like `if mutagen::MU.now(42) { .. }`
pub fn now(n: usize, flag: &AtomicUsize, mask: usize) -> bool {
    report_coverage(n..(n + 1), flag, mask);
    MU.now(n)
}

/// get the unsigned wrapping difference between the current mutation count and the given count
pub fn diff(n: usize, len: usize, flag: &AtomicUsize, mask: usize) -> usize {
    report_coverage(n..(n + len), flag, mask);
    MU.diff(n)
}

/// increment the mutation count
pub fn next() {
    MU.next()
}

/// use with if expressions, e.g. `if MU.t(..) { .. } else { .. }`
pub fn t(t: bool, n: usize, flag: &AtomicUsize, mask: usize) -> bool {
    report_coverage(n..(n + 3), flag, mask);
    MU.t(t, n)
}

/// use with while expressions, e.g. `while Mu.w(..) { .. }`
///
/// this never leads to infinite loops
pub fn w(t: bool, n: usize, flag: &AtomicUsize, mask: usize) -> bool {
    report_coverage(n..(n + 1), flag, mask);
    MU.w(t, n)
}

/// use instead of `==`
pub fn eq<R, T: PartialEq<R>>(x: T, y: R, n: usize, flag: &AtomicUsize, mask: usize) -> bool {
    report_coverage(n..(n + 3), flag, mask);
    MU.eq(x, y, n)
}

/// use instead of `!=`
pub fn ne<R, T: PartialEq<R>>(x: T, y: R, n: usize, flag: &AtomicUsize, mask: usize) -> bool {
    report_coverage(n..(n + 3), flag, mask);
    MU.ne(x, y, n)
}

/// use instead of `>` (or, switching operand order `<`)
pub fn gt<R, T: PartialOrd<R>>(x: &T, y: &R, n: usize, flag: &AtomicUsize, mask: usize) -> bool {
    report_coverage(n..(n + 7), flag, mask);
    MU.gt(x, y, n)
}

/// use instead of `>=` (or, switching operand order `<=`)
pub fn ge<R, T: PartialOrd<R>>(x: &T, y: &R, n: usize, flag: &AtomicUsize, mask: usize) -> bool {
    report_coverage(n..(n + 7), flag, mask);
    MU.ge(x, y, n)
}

/// use instead of `>` (or, switching operand order `<`)
pub fn lt<R, T: PartialOrd<R>>(x: &T, y: &R, n: usize, flag: &AtomicUsize, mask: usize) -> bool {
    report_coverage(n..(n + 7), flag, mask);
    MU.lt(x, y, n)
}

/// use instead of `>=` (or, switching operand order `<=`)
pub fn le<R, T: PartialOrd<R>>(x: &T, y: &R, n: usize, flag: &AtomicUsize, mask: usize) -> bool {
    report_coverage(n..(n + 7), flag, mask);
    MU.le(x, y, n)
}

pub fn forloop<'a, I: Iterator + 'a>(i: I, n: usize, flag: &AtomicUsize, mask: usize) -> Box<Iterator<Item=I::Item > + 'a> {
    report_coverage(n..(n + 4), flag, mask);
    MU.forloop(i, n)
}

/// increment a literal
pub fn inc<T: Step>(lit: T, n: usize, flag: &AtomicUsize, mask: usize) -> T {
    MU.inc(lit, n, flag, mask)
}

/// decrement a literal
pub fn dec<T: Step>(lit: T, n: usize, flag: &AtomicUsize, mask: usize) -> T {
    MU.dec(lit, n, flag, mask)
}

/// increment or decrement a literal
pub fn inc_dec<T: Step>(lit: T, n: usize, flag: &AtomicUsize, mask: usize) -> T {
    //TODO: make this cover both inc and dec
    MU.inc_dec(lit, n, flag, mask)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_mutation() {
        let mu = Mutagen {
            x: AtomicUsize::new(0),
        };

        // Always true
        assert_eq!(true, mu.eq(&1, &0, 0));

        // Always false
        mu.next();
        assert_eq!(false, mu.eq(&1, &0, 0));

        // Checks inequality
        mu.next();
        assert_eq!(true, mu.eq(&0, &1, 0));
        assert_eq!(false, mu.eq(&1, &1, 0));

        // Checks equality
        mu.next();
        assert_eq!(true, mu.eq(&0, &0, 0));
        assert_eq!(false, mu.eq(&1, &0, 0));
    }

    #[test]
    fn ne_mutation() {
        let mu = Mutagen {
            x: AtomicUsize::new(0),
        };

        // Always true
        assert_eq!(true, mu.ne(&1, &0, 0));

        // Always false
        mu.next();
        assert_eq!(false, mu.ne(&1, &0, 0));

        // Checks equality
        mu.next();
        assert_eq!(false, mu.ne(&0, &1, 0));
        assert_eq!(true, mu.ne(&1, &1, 0));

        // Checks inequality
        mu.next();
        assert_eq!(false, mu.ne(&0, &0, 0));
        assert_eq!(true, mu.ne(&1, &0, 0));
    }

    #[test]
    fn eq_with_references() {
        let n = "test".as_bytes();
        let mu = Mutagen {
            x: AtomicUsize::new(0),
        };

        assert_eq!(true, mu.eq(&*n, &*b"test", 0));
    }
}
