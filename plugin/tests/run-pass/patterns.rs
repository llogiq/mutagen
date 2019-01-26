extern crate mutagen;
extern crate mutagen_plugin;

use mutagen_plugin::mutate;

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
