#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]
extern crate mutagen;

fn main() {}

struct Test {}

impl Test {
    #[mutate]
    //~^ cannot find value `__COVERAGE1` in this scope [E0425]
    pub fn method(&self) -> i32 {
        5
    }
}