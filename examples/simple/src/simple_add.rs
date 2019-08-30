use mutagen::mutate;

#[cfg_attr(test, mutate)]
pub fn simple_add() -> i32 {
    1 + 2
}

#[cfg_attr(test, mutate)]
pub fn simple_add_u8() -> u8 {
    1u8 + 2
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_simple_add() {
        assert_eq!(simple_add(), 3);
    }

    #[test]
    fn test_simple_add_u8() {
        assert_eq!(simple_add_u8(), 3);
    }
}
