
// calculates fibonacci numbers
#[cfg_attr(feature="mutationtest", ::mutagen::mutate)]
pub fn fib(n: u32) -> u32 {
    if n == 0 || n == 1 {
        return n;
    }
    fib(n-1) + fib(n-2)

}

#[cfg(test)]
mod tests {

    use super::fib;

    #[test]
    fn fib_0() {
        assert_eq!(0, fib(0))
    }

    #[test]
    fn fib_1() {
        assert_eq!(1, fib(1))
    }

    #[test]
    fn fib_2() {
        assert_eq!(1, fib(2))
    }

    #[test]
    fn fib_3() {
        assert_eq!(2, fib(3))
    }

    #[test]
    fn fib_4() {
        assert_eq!(3, fib(4))
    }
    #[test]
    fn fib_5() {
        assert_eq!(5, fib(5))
    }

}
