#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]
extern crate mutagen;

#[mutate]
fn mutated_function() {
    let ord = 0x60;

    if (ord < 0x41 || ord > 0x5A) && (ord < 0x71 || ord > 0x7A) {
        // Do something
    }

    if (2 == 3) {
        // Do something
    }

    if (true && false) {
        // Do something
    }

    if (2 != 3) {
        // Do something
    }

    if (ord < 2) {
        // Do something
    }

    if ord == 2 {
        // Some
    }
}

fn main() {}