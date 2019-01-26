extern crate mutagen;
extern crate mutagen_plugin;

use mutagen_plugin::mutate;

#[mutate]
fn main() {
    // Mutate range
    for i in 0..5 {

    }

    // Mutate vector
    let v = vec![0, 1, 2, 3];
    for i in v.iter() {

    }

    // Slice with mutable iterator
    let mut slice = &mut [1, 2, 3];
    for element in slice.iter_mut() {

    }
}
