#[cfg_attr(all(test, feature = "with_mutagen"), ::mutagen::mutate)]
pub fn foo() -> i32 {
    1 + 2
}

#[cfg(test)]
mod tests {
    use super::foo;

    #[test]
    fn test_foo() {
        assert_eq!(foo(), 3);
    }
}
