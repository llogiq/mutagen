#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]
extern crate mutagen;

#[mutate]
fn matches() {
    match 2 {
        1...3 => (),
        a if 3 > 4 => (),
        _ => (),
    };

    match 'a' {
        'a'...'z' => (),
        _ => (),
    };
}

fn main() {

}