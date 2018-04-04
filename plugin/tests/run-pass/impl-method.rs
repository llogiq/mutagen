#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]
extern crate mutagen;

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