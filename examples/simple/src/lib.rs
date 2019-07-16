use mutagen::mutate;

#[mutate]
pub fn simple_add() -> u8 {
    1u8 + 2
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_foobar() {
        assert_eq!(simple_add(), 3);
    }
}
