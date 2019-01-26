extern crate mutagen;
extern crate mutagen_plugin;

use mutagen_plugin::mutate;

#[mutate]
fn main() {
    let _x = 1u32 << 2 * 3;
}
