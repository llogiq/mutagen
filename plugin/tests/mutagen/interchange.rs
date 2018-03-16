#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]
extern crate mutagen;

#[mutate]
fn simple_interchange(x: i32, y: i32, z: i32) -> i32 {
    x + y - z
}

#[mutate]
fn no_interchange(x: i32, y: u32) {}

struct ComplexStruct {
    x: i32,
    y: u32,
}

#[mutate]
fn interchange_tuple((a, b): (i32, u32), c: i32, (d, e): (u32, i32), ComplexStruct{x: f, y: g}: ComplexStruct) {
    // Note that a and f, won't be exchanged with current code
}

#[mutate]
fn interchange_struct(ComplexStruct{x: a, y: b}: ComplexStruct, ComplexStruct{x: c, y: d}: ComplexStruct)
{

}

fn main() {}