#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]
extern crate mutagen;

#[mutate]
//~^ the trait bound `u32: mutagen::MulDiv<i32>` is not satisfied [E0277]
fn muldiv_issue(offset: u32) -> u32 {
    let size1 = 1u32;
    let size2 = 2u32;

    let val = ((size2 & 0xFF) << 8) | size1 & 0xFF;
    let b = offset + 2 + (val * 2);

    b
}


