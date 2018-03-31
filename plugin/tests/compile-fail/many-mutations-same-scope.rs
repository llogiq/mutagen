#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]

extern crate mutagen;

fn main() {}

#[mutate] //~ trait bound `std::sync::atomic::AtomicUsize: std::marker::Copy` is not satisfied [E0277]
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

    if 2 == 3 || 5 == 6 || 7 == 9 || 9 == 19 ||
        1 == 2 || 3 == 4 || 10 == 11 || 15 == 20 ||
        1 == 2 || 3 == 4 || 10 == 11 || 15 == 20 ||
        1 == 2 || 3 == 4 || 10 == 11 || 15 == 20 {
        // Lot's of mutations on same scope
    }
}