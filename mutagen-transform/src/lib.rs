extern crate proc_macro;

use mutagen_core::do_transform_item_fn;

#[proc_macro_attribute]
pub fn mutate(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    do_transform_item_fn(attr.into(), input.into()).into()
}
