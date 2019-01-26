extern crate mutagen;
extern crate mutagen_plugin;

use mutagen_plugin::mutate;

#[mutate]
fn main() {
    let mut count = 0;
    count += 1;
    println!("{}", count * 2);
}
