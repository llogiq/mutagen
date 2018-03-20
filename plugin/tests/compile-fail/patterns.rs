#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]
extern crate mutagen;

#[mutate]
//~^ ERROR arbitrary expressions
fn matches() {
    match 2 {
        1...3 => (),    //~ ERROR [E0029]
        _ => (),
    };

    match 'a' {
        'a'...'z' => (),
        _ => (),
    };
}

fn main() {

}