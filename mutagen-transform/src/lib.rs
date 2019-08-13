#![feature(box_syntax)]
#![feature(vec_remove_item)]
#![feature(specialization)]
// proc-macro-span feature is required because `proc_macro2` does not export the api for getting source files for spans
#![feature(proc_macro_span)]

extern crate proc_macro;
use syn::{parse_macro_input, ItemFn};

use mutagen_core::do_transform_item_fn;

#[proc_macro_attribute]
pub fn mutate(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    do_transform_item_fn(attr.into(), input).into()
}
