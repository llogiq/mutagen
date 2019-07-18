use mutagen::mutate;

#[cfg_attr(test, mutate)]
pub fn simple_add() -> u8 {
    1u8 + 2
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_simple_add() {
        assert_eq!(simple_add(), 3);
    }

}
