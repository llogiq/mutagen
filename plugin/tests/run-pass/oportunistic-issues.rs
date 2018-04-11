#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]
extern crate mutagen;

#[mutate]
fn main() {
    let _x = 1u32 << 2 * 3;
}
