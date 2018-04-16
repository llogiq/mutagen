#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]
extern crate mutagen;

#[mutate]
fn clones(ref mut a: &mut String, b: &mut String) {
    a.push('a');
    b.push('!');
}

fn main() {
    let mut x = String::from("Hi");
    let mut y = String::from("there");

    clones(&mut x, &mut y);
    println!("{} {}", x, y);
}
