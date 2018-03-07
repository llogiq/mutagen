#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]

extern crate mutagen;

fn foo() -> usize { 2 }
fn bar() -> usize { 23 }

#[mutate]
//~^ ERROR capture of possibly uninitialized variable: `value` [E0381]
//~^^ ERROR cannot borrow `value` as mutable more than once at a time [E0499]
fn blubb() -> usize {
    let mut value;
    if ({ value = foo(); value > 5 }) || ({ value = bar(); value < 5 }) {
        value + 1 //~ ERROR use of possibly uninitialized variable: `value` [E0381]
    } else {
        42
    }
}

fn main() {
    println!("{}", blubb());
}