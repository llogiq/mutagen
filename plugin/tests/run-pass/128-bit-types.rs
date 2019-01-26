extern crate mutagen;
extern crate mutagen_plugin;

use mutagen_plugin::mutate;

#[mutate]
fn main() {
    let u_zero = 0u128;
    let u_max = std::u128::MAX;
    let u_min = std::u128::MIN;
    println!("{}, {}, {}", u_zero, u_max, u_min);
    let i_zero = 0i128;
    let i_max = std::i128::MAX;
    let i_min = std::i128::MIN;
    println!("{}, {}, {}", i_zero, i_max, i_min);
}
