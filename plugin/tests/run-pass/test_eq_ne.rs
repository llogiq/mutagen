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

    let x1 = X {f: 2u32};
    let x2 = X {f: 9u32};

    if x1 == x2 {
        //Do something
    }
}

struct X {
    f: u32,
}

impl PartialEq for X {
    fn eq(&self, other: &X) -> bool {
        self.f == other.f
    }
}