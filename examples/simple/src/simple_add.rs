use mutagen::mutate;

#[cfg_attr(test, mutate)]
pub fn simple_add() -> i32 {
    1 + 2
}

#[cfg_attr(test, mutate)]
pub fn simple_add_u8() -> u8 {
    1 + 2
}

#[cfg_attr(test, mutate)]
pub fn add_repeated_u8() -> u8 {
    1 + 2 + 3 * 2
}

#[cfg_attr(test, mutate)]
pub fn add_two_u8(x: u8) -> u8 {
    x + 2
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

    #[test]
    fn test_add_two_u8() {
        assert_eq!(add_two_u8(1), 3);
    }

    #[test]
    fn test_add_repeated_u8() {
        assert_eq!(add_repeated_u8(), 9);
    }
}
