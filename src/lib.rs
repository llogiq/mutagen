#![feature(specialization, atomic_min_max, try_from, assoc_unix_epoch)]
/// Welcome to the mutagen crate. Your entry point will probably be cargo-mutagen, so install it
/// right away.
#[macro_use]
extern crate lazy_static;

use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};

include!(concat!(env!("OUT_DIR"), "/ops.rs"));

pub mod bounded_loop;
mod iterators;

mod coverage;
pub use coverage::report_coverage;

pub trait Defaulter: Sized {
    fn get_default(count: usize, flag: &AtomicUsize, mask: usize) -> Option<Self>;
}

impl<X> Defaulter for X {
    default fn get_default(_count: usize, _flag: &AtomicUsize, _mask: usize) -> Option<X> {
        None
    }
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

lazy_static! {
    static ref MU: Mutagen = {
        let count = env::var("MUTATION_COUNT")
            .map(|s| s.parse().unwrap_or(0))
            .unwrap_or(0);
        Mutagen {
            x: AtomicUsize::new(count),
        }
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

    /// use instead of `>` (or, switching operand order `<`)
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

    /// use instead of `>=` (or, switching operand order `<=`)
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

    pub fn forloop<'a, I: Iterator + 'a>(
        &self,
        i: I,
        n: usize,
    ) -> Box<Iterator<Item = I::Item> + 'a> {
        match self.diff(n) {
            0 => Box::new(iterators::NoopIterator { inner: i }),
            1 => Box::new(i.skip(1)),
            2 => Box::new(iterators::SkipLast::new(i)),
            3 => Box::new(iterators::SkipLast::new(i.skip(1))),
            _ => Box::new(i),
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

pub fn forloop<'a, I: Iterator + 'a>(
    i: I,
    n: usize,
    flag: &AtomicUsize,
    mask: usize,
) -> Box<Iterator<Item = I::Item> + 'a> {
    report_coverage(n..(n + 4), flag, mask);
    MU.forloop(i, n)
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
