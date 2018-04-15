#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]

extern crate mutagen;

#[mutate]
fn main() {
    let mut i = 0;

    loop {
        if i == 5 {
            break;
        }

        i += 1;
    }

    // Multi-threaded
    let handle = ::std::thread::spawn(|| {
        let mut i = 0;
        loop {
            i += 1;

            if i == 10 {
                break;
            }
        }
    });

    handle.join();
}
