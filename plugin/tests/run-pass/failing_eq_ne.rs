#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]
extern crate mutagen;

#[mutate]
fn eq_with_early_return() -> usize {
    let a = 'a' == if true { 'b' } else { return 42 };

    24322
}

fn main() {}