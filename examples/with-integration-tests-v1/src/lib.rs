// calculates fibonacci numbers
#[cfg_attr(test, ::mutagen::mutate)]
pub fn fib(n: u32) -> u32 {
    if n == 0 || n == 1 {
        return n;
    }
    fib(n - 1) + fib(n - 2)
}
