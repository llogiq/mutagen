#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]

extern crate mutagen;

fn main() {}

#[mutate]
fn simple() {
    fn t() -> u32 {
        5
    }

    if (42 == t()) {
        // Do something
    }

    if (2 != 3) {
        // Do something
    }
}