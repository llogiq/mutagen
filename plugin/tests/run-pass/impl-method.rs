extern crate mutagen;
extern crate mutagen_plugin;

use mutagen_plugin::mutate;

fn main() {}

struct Test {}

impl Test {
    #[mutate]
    pub fn method(&self) -> i32 {
        5
    }

    #[mutate]
    pub fn nested(&self) -> i32 {
        let a = {
            43
        };

        5
    }
}
