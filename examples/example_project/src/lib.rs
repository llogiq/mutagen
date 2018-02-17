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
    if (ord < 0x41 || ord > 0x5A) && (ord < 0x71 || ord > 0x7A) {
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

    use super::count_alphabetic_chars;
    use super::mutagen;

    #[test]
    fn test_count_alphabetic_chars() {
        let string = "AsfwrgebrtSSNNfegerhLLSL4243";
        let mut results = vec![];

        for _ in 0..100 {
            mutagen::next();

            //Actual test
            let result = 3 == count_alphabetic_chars('S', string);

            println!(
                "Test {}: {} == {} => {}",
                mutagen::get(),
                3,
                count_alphabetic_chars('S', string),
                result
            );

            results.push(result);
        }

        // All results should be false
        assert_eq!(true, results.iter().all(|x| !*x));
    }
}
