#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]

extern crate mutagen;

#[mutate]
fn main() {
    let mut count = 0;
    count += 1;
    println!("{}", count * 2);
}
