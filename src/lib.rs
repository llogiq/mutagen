#![feature(specialization, atomic_min_max, try_from, assoc_unix_epoch)]
/// Welcome to the mutagen crate. Your entry point will probably be cargo-mutagen, so install it
/// right away.
#[macro_use]
extern crate lazy_static;

use std::{env, fmt, io};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::error::Error;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};

mod ops;
pub use ops::*;
mod iterators;
pub mod bounded_loop;

mod coverage;
pub use coverage::report_coverage;

lazy_static! {
    static ref MU: Mutagen = {
        let count = env::var("MUTATION_COUNT").map(|s|s.parse().unwrap_or(0)).unwrap_or(0);
        Mutagen { x: AtomicUsize::new(count) }
    };
}

/// Our very own error type
#[derive(Debug)]
pub struct MutagenError;

/// Be an error
impl Error for MutagenError {
    /// We always return the same description
    ///
    /// # Examples
    ///
    /// ```rust
    ///# use {std::error::Error, mutagen::MutagenError};
    /// assert_eq!("mutated by mutagen", MutagenError.description());
    /// ```
    fn description(&self) -> &str { "mutated by mutagen" }

    /// We don't need a cause to err :-)
    ///
    /// # Examples
    /// ```rust
    ///# use {std::error::Error, mutagen::MutagenError};
    /// assert!(MutagenError.cause().is_none());
    /// ```
    fn cause(&self) -> Option<&Error> { None }
}

impl std::fmt::Display for MutagenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mutated by mutagen")
    }
}

/// A trait to allow mutating `if let'/`while let` statements
///
/// # Examples
///
/// ```rust
///# use {mutagen::l, std::env::VarError, std::io};
///# let (coverage, mask) = (::std::sync::atomic::AtomicUsize::new(0), 1);
/// l(Some("works on Option<T>"), 0, &coverage, mask); // -> None
/// let x: Result<usize, ()> = Ok(1); l(x, 0, &coverage, mask); // -> Err(())
/// let x: io::Result<()> = Ok(()); l(x, 0, &coverage, mask); // -> Err(Other)
/// let x: Result<&str, VarError> = Ok(""); l(x, 0, &coverage, mask); // -> Err(Unknown)
/// ```
pub trait Letter {
    fn l(self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self;
}

impl<T> Letter for T {
    default fn l(self, _count: usize, _cov: &AtomicUsize, _mask: usize) -> Self { self }
}

impl<T> Letter for Option<T> {
    fn l(self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self {
        report_coverage(mutation_count..mutation_count + 1, coverage, mask);
        if now(mutation_count) { None } else { self }
    }
}

macro_rules! l_impl {
    { $x: expr; $($tt:tt)*} => {
        #[doc = $x] $($tt)*
    };
    ($err_type: ty, $err: expr) => {
        impl<T> Letter for Result<T, $err_type> {
            l_impl! {
                concat!(
            "```
#![feature(try_from, assoc_unix_epoch)]
use std::{cell::RefCell, convert::TryFrom, env, error::*, fmt, io, str::FromStr};
let _x : ", stringify!($err_type), " = ", stringify!($err), ";
```");
                fn l(self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self {
                    report_coverage(mutation_count..mutation_count + 1, coverage, mask);
                    if now(mutation_count) { Result::Err($err) } else { self }
                }
            }
        }
    };
}

l_impl!((), ());
l_impl!(io::Error, io::ErrorKind::Other.into());
l_impl!(fmt::Error, fmt::Error);
l_impl!(std::env::VarError, std::env::VarError::NotPresent);
l_impl!(std::str::ParseBoolError, bool::from_str("").unwrap_err());
l_impl!(std::num::ParseIntError, i8::from_str_radix("", 2).unwrap_err());
l_impl!(std::num::TryFromIntError, i8::try_from(128u8).unwrap_err());
l_impl!(std::num::ParseFloatError, str::parse::<f32>("").unwrap_err());
l_impl!(std::str::Utf8Error, std::str::from_utf8(&[193u8]).unwrap_err());
l_impl!(std::string::FromUtf8Error, String::from_utf8(vec![193u8]).unwrap_err());
l_impl!(std::string::FromUtf16Error, String::from_utf16(&[0xD800]).unwrap_err());
l_impl!(std::char::DecodeUtf16Error,
        std::char::decode_utf16(0xD800u16..0xD801).next().unwrap().unwrap_err());
l_impl!(std::cell::BorrowError, {
    let x;
    {
        let c = RefCell::new(());
        let _r = c.borrow_mut();
        x = c.try_borrow().unwrap_err();
    }
    x
});
l_impl!(std::cell::BorrowMutError, {
    let x;
    {
        let c = RefCell::new(());
        let _r = c.borrow_mut();
        x = c.try_borrow_mut().unwrap_err();
    }
    x
});
l_impl!(std::char::CharTryFromError, char::try_from(0xFFFF_FFFFu32).unwrap_err());
l_impl!(std::char::ParseCharError, char::from_str("").unwrap_err());
l_impl!(std::ffi::NulError, std::ffi::CString::new("\0").unwrap_err());
l_impl!(std::ffi::FromBytesWithNulError, std::ffi::CStr::from_bytes_with_nul(b"\0x").unwrap_err());
l_impl!(std::ffi::IntoStringError,
        std::ffi::CString::new(vec![193]).unwrap().into_string().unwrap_err());
l_impl!(std::net::AddrParseError, "".parse::<std::net::IpAddr>().unwrap_err());
l_impl!(std::path::StripPrefixError, std::path::Path::new("/x").strip_prefix("x").unwrap_err());
l_impl!(std::sync::mpsc::RecvError, std::sync::mpsc::RecvError);
l_impl!(std::sync::mpsc::TryRecvError, std::sync::mpsc::TryRecvError::Empty);
l_impl!(std::sync::mpsc::RecvTimeoutError, std::sync::mpsc::RecvTimeoutError::Timeout);
l_impl!(std::time::SystemTimeError, std::time::SystemTime::UNIX_EPOCH.duration_since(
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1)).unwrap_err());
//TODO std::sync::PoisonError
impl<T, E: Default> Letter for Result<T, std::sync::mpsc::SendError<E>> {
    /// ```
    ///# use std::sync::mpsc::SendError;
    /// let _x : SendError<u8> = SendError(0);
    /// ```
    fn l(self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self {
        report_coverage(mutation_count..mutation_count + 1, coverage, mask);
        if now(mutation_count) {
            Result::Err(std::sync::mpsc::SendError(Default::default()))
        } else { self }
    }
}
impl<T, E: Default> Letter for Result<T, std::sync::mpsc::TrySendError<E>> {
    /// ```
    ///# use std::sync::mpsc::TrySendError;
    /// let _x : TrySendError<u8> = TrySendError::Full(0);
    /// ```
    fn l(self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self {
        report_coverage(mutation_count..mutation_count + 1, coverage, mask);
        if now(mutation_count) {
            Result::Err(std::sync::mpsc::TrySendError::Full(Default::default()))
        } else { self }
    }
}
impl<T> Letter for Result<T, std::sync::TryLockError<T>> {
    /// ```
    ///# use std::sync::TryLockError;
    /// let _x : TryLockError<u8> = TryLockError::WouldBlock;
    /// ```
    fn l(self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self {
        report_coverage(mutation_count..mutation_count + 1, coverage, mask);
        if now(mutation_count) { Result::Err(std::sync::TryLockError::WouldBlock) } else { self }
    }
}

impl<'e, T> Letter for Result<T, Box<Error + 'e>> {
    fn l(self, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self {
        report_coverage(mutation_count..mutation_count + 1, coverage, mask);
        if now(mutation_count) { Result::Err(Box::new(MutagenError)) } else { self }
    }
}

/// use on `if let` or `while let` statements to opportunistically replace Some(_) / Ok(_)
pub fn l<L: Letter>(l: L, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> L {
    l.l(mutation_count, coverage, mask)
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
    pub fn eq<R, T: PartialEq<R>>(&self, x: &T, y: &R, n: usize) -> bool {
        match self.diff(n) {
            0 => true,
            1 => false,
            2 => x != y,
            _ => x == y,
        }
    }

    /// use instead of `!=`
    pub fn ne<R, T: PartialEq<R>>(&self, x: &T, y: &R, n: usize) -> bool {
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

    pub fn forloop<'a, I: Iterator + 'a>(&self, i: I, n: usize) -> Box<Iterator<Item=I::Item> + 'a> {
        match self.diff(n) {
            0 => Box::new(iterators::NoopIterator{inner: i}),
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
pub fn eq<R, T: PartialEq<R>>(x: &T, y: &R, n: usize) -> bool {
    MU.eq(x, y, n)
}

/// use instead of `!=`
pub fn ne<R, T: PartialEq<R>>(x: &T, y: &R, n: usize) -> bool {
    MU.ne(x, y, n)
}

/// use instead of `>` (or, switching operand order `<`)
pub fn gt<R, T: PartialOrd<R>>(x: &T, y: &R, n: usize) -> bool {
    MU.gt(x, y, n)
}

/// use instead of `>=` (or, switching operand order `<=`)
pub fn ge<R, T: PartialOrd<R>>(x: &T, y: &R, n: usize) -> bool {
    MU.ge(x, y, n)
}

pub fn forloop<'a, I: Iterator + 'a>(i: I, n: usize) -> Box<Iterator<Item=I::Item > + 'a> {
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
}
