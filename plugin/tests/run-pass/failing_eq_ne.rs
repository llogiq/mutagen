extern crate mutagen;
extern crate mutagen_plugin;

use mutagen_plugin::mutate;

#[mutate]
fn eq_with_early_return() -> usize {
    let a = 'a' == if true { 'b' } else { return 42 };

    24322
}

fn main() {}
