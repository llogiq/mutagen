
#[macro_use]
extern crate lazy_static;

use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A helper trait to select a value from a same-typed tuple
pub trait Selector<T> {
    fn get(self, n: usize) -> T;
}

impl<T> Selector<T> for (T, T) {
    fn get(self, n: usize) -> T {
        if n == 0 { self.1 } else { self.0 }
    }
}

impl<T> Selector<T> for (T, T, T) {
    fn get(self, n: usize) -> T {
        match n {
            0 => self.1,
            1 => self.2,
            _ => self.0
        }
    }
}

impl<T> Selector<T> for (T, T, T, T) {
    fn get(self, n: usize) -> T {
        match n {
            0 => self.1,
            1 => self.2,
            2 => self.3,
            _ => self.0
        }
    }
}

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
pub struct Mutagen {
    x: AtomicUsize
}

impl Mutagen {
    /// get the current mutation count
    #[inline]
    pub fn get(&self) -> usize {
        self.x.load(Ordering::Relaxed)
    }

    /// increment the mutation count
    #[inline]
    pub fn next(&self) {
        //TODO: clamp at maximum, return Option<usize>, impl Iterator?
        self.x.fetch_add(1, Ordering::SeqCst);
    }

    /// reset the mutation count to 0
    #[inline]
    pub fn reset(&self) {
        self.x.store(0, Ordering::SeqCst);
    }
}

/// get the current mutation count
#[inline]
pub fn get() -> usize {
    MU.get()
}

/// increment the mutation count
#[inline]
pub fn next() {
    MU.next()
}

/// reset the mutation count to 0 (for testing)
#[inline]
pub fn reset() {
    MU.reset()
}

/// check if the argument matches the current mutation count
///
/// this simplifies operations like `if mutagen::MU.now(42) { .. }`
#[inline]
pub fn now(n: usize) -> bool {
    MU.get() == n
}

/// get the wrapping difference between the given count and the current
/// mutation count
#[inline]
pub fn diff(n: usize) -> usize {
    MU.get().wrapping_sub(n)
}

/// insert the original or an alternate value, e.g. `MU.select(&[2, 0], 42)`
#[inline]
pub fn select<T, S: Selector<T>>(selector: S, n: usize) -> T {
    selector.get(diff(n))
}

/// use with if expressions, e.g. `if MU.t(..) { .. } else { .. }`
#[inline]
pub fn t(t: bool, n: usize) -> bool {
    match diff(n) {
        0 => true,
        1 => false,
        2 => !t,
        _ => t
    }
}

/// use with while expressions, e.g. `while Mu.w(..) { .. }`
///
/// this never leads to infinite loops
#[inline]
pub fn w(t: bool, n: usize) -> bool {
    if now(n) { false } else { t }
}

/// use instead of `&&`
///
/// upholds the invariant that g() is not called unless f() == true
#[inline]
pub fn and<X, Y>(f: X, g: Y, n: usize) -> bool
where
    X: FnOnce() -> bool,
    Y: FnOnce() -> bool,
{
    match diff(n) {
        0 => false,
        1 => true,
        2 => f(),
        3 => !f(),
        4 => f() && !g(),
        _ => f() && g()
    }
}

/// use instead of `||`
///
/// upholds the invariant that g() is not called unless f() == false
#[inline]
pub fn or<X, Y>(f: X, g: Y, n: usize) -> bool
where
    X: FnOnce() -> bool,
    Y: FnOnce() -> bool,
{
    match diff(n) {
        0 => false,
        1 => true,
        2 => f(),
        3 => !f(),
        4 => f() || !g(),
        _ => f() || g()
    }
}

/// use instead of `==`
#[inline]
pub fn eq<X, Y, T: PartialEq>(x: X, y: Y, n: usize) -> bool
    where X: FnOnce() -> T, Y: FnOnce() -> T {
    match diff(n) {
        0 => true,
        1 => false,
        2 => x() != y(),
        _ => x() == y()
    }
}

/// use instead of `!=`
#[inline]
pub fn ne<X, Y, T: PartialEq>(x: X, y: Y, n: usize) -> bool
    where X: FnOnce() -> T, Y: FnOnce() -> T {
    match diff(n) {
        0 => true,
        1 => false,
        2 => x() == y(),
        _ => x() != y()
    }
}

/// use instead of `>` (or, switching operand order `<`)
#[inline]
pub fn gt<X, Y, T: PartialOrd>(x: X, y: Y, n: usize) -> bool
    where X: FnOnce() -> T, Y: FnOnce() -> T {
    match diff(n) {
        0 => false,
        1 => true,
        2 => x() < y(),
        3 => x() <= y(),
        4 => x() >= y(),
        5 => x() == y(),
        6 => x() != y(),
        _ => x() > y()
    }
}

/// use instead of `>=` (or, switching operand order `<=`)
#[inline]
pub fn ge<X, Y, T: PartialOrd>(x: X, y: Y, n: usize) -> bool
    where X: FnOnce() -> T, Y: FnOnce() -> T {
    match diff(n) {
        0 => false,
        1 => true,
        2 => x() < y(),
        3 => x() <= y(),
        4 => x() > y(),
        5 => x() == y(),
        6 => x() != y(),
        _ => x() >= y()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_mutation() {
        reset();
        // Always true
        assert_eq!(true, eq(|| 1, || 0, 0));

        // Always false
        next();
        assert_eq!(false, eq(|| 1, || 0, 0));

        // Checks inequality
        next();
        assert_eq!(true, eq(|| 0, || 1, 0));
        assert_eq!(false, eq(|| 1, || 1, 0));

        // Checks equality
        next();
        assert_eq!(true, eq(|| 0, || 0, 0));
        assert_eq!(false, eq(|| 1, || 0, 0));
    }

    #[test]
    fn ne_mutation() {
        reset();
        // Always true
        assert_eq!(true, ne(|| 1, || 0, 0));

        // Always false
        next();
        assert_eq!(false, ne(|| 1, || 0, 0));

        // Checks equality
        next();
        assert_eq!(false, ne(|| 0, || 1, 0));
        assert_eq!(true, ne(|| 1, || 1, 0));

        // Checks inequality
        next();
        assert_eq!(false, ne(|| 0, || 0, 0));
        assert_eq!(true, ne(|| 1, || 0, 0));
    }
}
