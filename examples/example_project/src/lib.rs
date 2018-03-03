//! Simple example library
//!
#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(mutagen_plugin))]
#![feature(custom_attribute)]

#[cfg(test)]
extern crate mutagen;

/// This function counts the number of alphabetic chars
///
/// This example is a bit contrived but that is to different mutagen mutations
///
/// # Examples
///
/// ```
/// use example_project::count_alphabetic_chars;
///
/// assert_eq!(0,    count_alphabetic_chars('A', ""));
/// assert_eq!(1,    count_alphabetic_chars('A', "ABCDE"));
/// assert_eq!(3, count_alphabetic_chars('A', "ABABA"));
///
/// assert_eq!(0, count_alphabetic_chars('1', "BCDE"));
/// assert_eq!(0, count_alphabetic_chars('A', "BCDE"));
/// ```
#[cfg_attr(test, mutate)]
pub fn count_alphabetic_chars(c: char, string: &str) -> usize {
    let mut count = 0;
    let ord = c as u8;
    let chars = string.chars().collect::<Vec<char>>();

    // Check if within bounds
    if (ord < 0x41 || ord > 0x5A) && (ord < 0x61 || ord > 0x7A) {
        return 0;
    }

    for i in 0..chars.len() {
        if chars[i] == c {
            count += 1;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::count_alphabetic_chars;

    #[test]
    fn test_count_alphabetic_chars() {
        let string = "AsfwrgebrtSSNNfegerhLLSL4243";

        let result = count_alphabetic_chars('S', string);
        assert_eq!(3, result);
    }

    #[test]
    fn test_count_lt_a() {
        let string = "AsfwragebrtSSNNfegerhLLSL`4243";

        let result = count_alphabetic_chars('`', string);
        assert_eq!(0, result);
    }

    #[test]
    fn test_count_a() {
        let string = "AsfwragebrtSSNNfegerhLLS1L4243";

        let result = count_alphabetic_chars('a', string);
        assert_eq!(1, result);
    }

    #[test]
    fn test_count_z() {
        let string = "AsfwragebrtSSNNfegezrhLLSL4243";

        let result = count_alphabetic_chars('z', string);
        assert_eq!(1, result);
    }

    #[test]
    fn test_count_gt_z() {
        let string = "AsfwragebrtSSNNfegezrhLLS{L4243";

        let result = count_alphabetic_chars('{', string);
        assert_eq!(0, result);
    }

    #[test]
    fn test_count_lt_A() {
        let string = "AsfwragebrtSSNNfeg@erhLLSL4243";

        let result = count_alphabetic_chars('@', string);
        assert_eq!(0, result);
    }

    #[test]
    fn test_count_A() {
        let string = "AsfwragebrtSSNNfegerhLLSL4243";

        let result = count_alphabetic_chars('A', string);
        assert_eq!(1, result);
    }

    #[test]
    fn test_count_Z() {
        let string = "AsfwragebrtSSNNfegeZzrhLLSL4243";

        let result = count_alphabetic_chars('Z', string);
        assert_eq!(1, result);
    }

    #[test]
    fn test_count_gt_Z() {
        let string = "AsfwragebrtSSNNfegeZz[rhLLSL4243";

        let result = count_alphabetic_chars('[', string);
        assert_eq!(0, result);
    }

    #[test]
    fn test_count_empty() {
        let string = "";

        assert_eq!(0, count_alphabetic_chars('S', string));
    }

    #[test]
    fn test_count_non_ascii() {
        let string = "Adwfwrec DW34542";

        assert_eq!(0, count_alphabetic_chars(' ', string));
    }
}
