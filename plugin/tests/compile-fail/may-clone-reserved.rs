#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]

extern crate mutagen;

fn main() {}

struct SomeStruct{}

impl SomeStruct {
    #[mutate]
    //~^ expected unit struct/variant or constant, found local variable `self` [E0424]
    pub fn mutate_self(&mut self) {

    }
}