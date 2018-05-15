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

    // While loop
    i = 0;
    while i < 10 {
        i += 1;
    }

    // Nested loops
    i = 0;
    loop {
        while i < 10 {
            i += 1;
        }

        break;
    }

    // While let
    let pat = Some(0usize);
    while let Some(_) = pat {
        // Block code
        break;
    }

    // For loop
    let v = [1, 2, 3];
    for i in v.iter() {
        println!("Block code");
    }
}
