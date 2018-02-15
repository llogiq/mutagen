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

/// A global helper struct to keep a mutation count
///
/// This is used by the Mutagen crate to simplify the code it inserts.
/// You should have no business using it manually.
pub struct Mutagen {
    x: AtomicUsize
}

/// The global mutation count
pub static MU: Mutagen = Mutagen { x: AtomicUsize::new(0) };

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

    /// increment the mutation count
    pub fn next(&self) {
        //TODO: clamp at maximum, return Option<usize>, impl Iterator?
        self.x.fetch_add(1, Ordering::SeqCst);
    }

    /// insert the original or an alternate value, e.g. `MU.select(&[2, 0], 42)`
    pub fn select<T, S: Selector<T>>(&self, selector: S, n: usize) -> T {
        selector.get(n.wrapping_sub(self.get()))
    }

    /// use with if expressions, e.g. `if MU.t(..) { .. } else { .. }`
    pub fn t(&self, t: bool, n: usize) -> bool {
        match n.wrapping_sub(self.get()) {
            0 => true,
            1 => false,
            2 => !t,
            _ => t
        }
    }

    /// use with while expressions, e.g. `while Mu.w(..) { .. }`
    ///
    /// this never leads to infinite loops
    pub fn w(&self, t: bool, n: usize) -> bool {
        if self.get() == n { false } else { t }
    }

    /// use instead of `&&`
    ///
    /// upholds the invariant that g() is not called unless f() == true
    pub fn and<X, Y>(&self, f: X, g: Y, n: usize) -> bool
        where X: FnOnce() -> bool, Y: FnOnce() -> bool {
        match n.wrapping_sub(self.get()) {
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
    pub fn or<X, Y>(&self, f: X, g: Y, n: usize) -> bool
        where X: FnOnce() -> bool, Y: FnOnce() -> bool {
        match n.wrapping_sub(self.get()) {
            0 => false,
            1 => true,
            2 => f(),
            3 => !f(),
            4 => f() || !g(),
            _ => f() || g()
        }
    }

    /// use instead of `==`
    pub fn eq<T: PartialEq>(&self, x: T, y: T, n: usize) -> bool {
        match n.wrapping_sub(self.get()) {
            0 => true,
            1 => false,
            2 => x != y,
            _ => x == y
        }
    }

    /// use instead of `!=`
    pub fn ne<T: PartialEq>(&self, x: T, y: T, n: usize) -> bool {
        match n.wrapping_sub(self.get()) {
            0 => true,
            1 => false,
            2 => x == y,
            _ => x != y
        }
    }

    /// use instead of `>` (or, switching operand order `<`)
    pub fn gt<T: PartialOrd>(&self, x: T, y: T, n: usize) -> bool {
        match n.wrapping_sub(self.get()) {
            0 => false,
            1 => true,
            2 => x < y,
            3 => x <= y,
            4 => x >= y,
            5 => x <= y && x >= y, // == with PartialOrd
            6 => x < y || x > y, // != with PartialOrd
            _ => x > y
        }
    }

    /// use instead of `>=` (or, switching operand order `<=`)
    pub fn ge<T: PartialOrd>(&self, x: T, y: T, n: usize) -> bool {
        match n.wrapping_sub(self.get()) {
            0 => false,
            1 => true,
            2 => x < y,
            3 => x <= y,
            4 => x > y,
            5 => x <= y && x >= y, // == with PartialOrd
            6 => x < y || x > y, // != with PartialOrd
            _ => x >= y
        }
    }
}
