#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]

extern crate mutagen;

#[mutate]
fn something() -> bool {
    let n = "test".as_bytes();
    *n == *b"test"
}

#[mutate]
fn owned_eq() -> Option<String> {
    let x = "Hello".to_string();
    if x == "Hello" {
        Some(x)
    } else {
        None
    }
}

fn main() {
    something();
}