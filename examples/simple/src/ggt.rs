use mutagen::mutate;

#[cfg_attr(test, mutate)]
pub fn ggt_loop(mut a: u32, mut b: u32) -> u32 {
    loop {
        if a == 0 {
            return b;
        }
        if b == 0 {
            return a;
        }
        if a > b {
            a -= b;
        } else {
            b -= a;
        }
    }
}

#[cfg_attr(test, mutate)]
pub fn ggt_rec(mut a: u32, mut b: u32) -> u32 {
    if a == b || a == 0 || b == 0 {
        return a | b;
    }
    if a > b {
        ggt_rec(a - b, b)
    } else {
        ggt_rec(a, b - a)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_ggt_loop_4_4() {
        assert_eq!(ggt_loop(4, 4), 4)
    }
    #[test]
    fn test_ggt_loop_3_5() {
        assert_eq!(ggt_loop(3, 5), 1)
    }
    #[test]
    fn test_ggt_loop_5_3() {
        assert_eq!(ggt_loop(5, 3), 1)
    }

    #[test]
    fn test_ggt_rec_4_4() {
        assert_eq!(ggt_rec(4, 4), 4)
    }
    #[test]
    fn test_ggt_rec_3_5() {
        assert_eq!(ggt_rec(3, 5), 1)
    }
    #[test]
    fn test_ggt_rec_5_3() {
        assert_eq!(ggt_rec(5, 3), 1)
    }
    #[test]
    fn test_ggt_rec_0_2() {
        assert_eq!(ggt_loop(0, 2), 2)
    }
    #[test]
    fn test_ggt_rec_2_0() {
        assert_eq!(ggt_loop(2, 0), 2)
    }
}
