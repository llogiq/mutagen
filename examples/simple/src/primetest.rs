use mutagen::mutate;

#[cfg_attr(test, mutate)]
pub fn primetest(n: u32) -> bool {
    if n % 2 == 0 {
        return n == 2;
    }
    if n == 1 {
        return false;
    }
    let mut k = 3;
    while k * k <= n {
        if n % k == 0 {
            return false;
        }
        k += 2;
    }
    true
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_primetest_0() {
        assert!(!primetest(0))
    }
    #[test]
    fn test_primetest_1() {
        assert!(!primetest(1))
    }
    #[test]
    fn test_primetest_2() {
        assert!(primetest(2))
    }
    #[test]
    fn test_primetest_3() {
        assert!(primetest(3))
    }
    #[test]
    fn test_primetest_4() {
        assert!(!primetest(4))
    }
    #[test]
    fn test_primetest_25() {
        assert!(!primetest(25))
    }
    #[test]
    fn test_primetest_29() {
        assert!(primetest(29))
    }
    #[test]
    fn test_primetest_31() {
        assert!(primetest(31))
    }
}
