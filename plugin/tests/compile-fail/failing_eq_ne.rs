#![feature(plugin)] //~ ERROR mismatched types [E0308]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]
extern crate mutagen;

#[mutate] //~ ERROR mismatched types [E0308]
fn eq_with_early_return() -> usize {
    let a = 'a' == if true { 'b' } else { return 42 };

    24322
}

fn main() {}