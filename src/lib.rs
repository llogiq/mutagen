#![feature(specialization)]
/// Welcome to the mutagen crate. Your entry point will probably be cargo-mutagen, so install it
/// right away.
#[macro_use]
extern crate lazy_static;

use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};

mod ops;
pub use ops::*;

mod coverage;
pub use coverage::report_coverage;

/// A helper trait to select a value from a same-typed tuple
#[doc(hidden)]
pub trait Selector<T> {
    fn get(self, n: usize) -> T;
}

impl<T> Selector<T> for (T, T) {
    fn get(self, n: usize) -> T {
        if n == 0 {
            self.1
        } else {
            self.0
        }
    }
}

impl<T> Selector<T> for (T, T, T) {
    fn get(self, n: usize) -> T {
        match n {
            0 => self.1,
            1 => self.2,
            _ => self.0,
        }
    }
}

impl<T> Selector<T> for (T, T, T, T) {
    fn get(self, n: usize) -> T {
        match n {
            0 => self.1,
            1 => self.2,
            2 => self.3,
            _ => self.0,
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

    /// insert the original or an alternate value, e.g. `MU.select(&[2, 0], 42)`
    pub fn select<T, S: Selector<T>>(&self, selector: S, n: usize) -> T {
        selector.get(self.diff(n))
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

    /// use instead of `>` (or, switching operand order `<`)
    pub fn gt<R, T: PartialOrd<R>>(&self, x: T, y: R, n: usize) -> bool {
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

    /// use instead of `>=` (or, switching operand order `<=`)
    pub fn ge<R, T: PartialOrd<R>>(&self, x: T, y: R, n: usize) -> bool {
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
}

/// get the current mutation count
pub fn get() -> usize {
    MU.get()
}

/// check if the argument matches the current mutation count
///
/// this simplifies operations like `if mutagen::MU.now(42) { .. }`
pub fn now(n: usize) -> bool {
    MU.now(n)
}

/// get the unsigned wrapping difference between the current mutation count and the given count
pub fn diff(n: usize) -> usize {
    MU.diff(n)
}

/// increment the mutation count
pub fn next() {
    MU.next()
}

/// insert the original or an alternate value, e.g. `MU.select(&[2, 0], 42)`
pub fn select<T, S: Selector<T>>(selector: S, n: usize) -> T {
    MU.select(selector, n)
}

/// use with if expressions, e.g. `if MU.t(..) { .. } else { .. }`
pub fn t(t: bool, n: usize) -> bool {
    MU.t(t, n)
}

/// use with while expressions, e.g. `while Mu.w(..) { .. }`
///
/// this never leads to infinite loops
pub fn w(t: bool, n: usize) -> bool {
    MU.w(t, n)
}

/// use instead of `==`
pub fn eq<R, T: PartialEq<R>>(x: T, y: R, n: usize) -> bool {
    MU.eq(x, y, n)
}

/// use instead of `!=`
pub fn ne<R, T: PartialEq<R>>(x: T, y: R, n: usize) -> bool {
    MU.ne(x, y, n)
}

/// use instead of `>` (or, switching operand order `<`)
pub fn gt<R, T: PartialOrd<R>>(x: T, y: R, n: usize) -> bool {
    MU.gt(x, y, n)
}

/// use instead of `>=` (or, switching operand order `<=`)
pub fn ge<R, T: PartialOrd<R>>(x: T, y: R, n: usize) -> bool {
    MU.ge(x, y, n)
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
        assert_eq!(true, mu.eq(1, 0, 0));

        // Always false
        mu.next();
        assert_eq!(false, mu.eq(1, 0, 0));

        // Checks inequality
        mu.next();
        assert_eq!(true, mu.eq(0, 1, 0));
        assert_eq!(false, mu.eq(1, 1, 0));

        // Checks equality
        mu.next();
        assert_eq!(true, mu.eq(0, 0, 0));
        assert_eq!(false, mu.eq(1, 0, 0));
    }

    #[test]
    fn ne_mutation() {
        let mu = Mutagen {
            x: AtomicUsize::new(0),
        };

        // Always true
        assert_eq!(true, mu.ne(1, 0, 0));

        // Always false
        mu.next();
        assert_eq!(false, mu.ne(1, 0, 0));

        // Checks equality
        mu.next();
        assert_eq!(false, mu.ne(0, 1, 0));
        assert_eq!(true, mu.ne(1, 1, 0));

        // Checks inequality
        mu.next();
        assert_eq!(false, mu.ne(0, 0, 0));
        assert_eq!(true, mu.ne(1, 0, 0));
    }
}
