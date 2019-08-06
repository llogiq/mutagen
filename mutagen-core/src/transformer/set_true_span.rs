//! sets the span of the generated code to be at the location of the original code.
//!
//! However, the flag `procmacro2_semver_exempt` is required. Otherwise the function `located_at` is not exported. It is required to call the test suite with `RUSTFLAGS='--cfg procmacro2_semver_exempt' cargo test` to enable that feature.

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn;
use syn::Expr;

pub fn set_true_span_expr(expr: Expr, new_span: Span) -> Expr {
    syn::parse2(set_true_span(expr.into_token_stream(), new_span)).unwrap()
}

#[cfg(procmacro2_semver_exempt)]
/// replaces all occurences of the default span with the given one
pub fn set_true_span(stream: TokenStream, new_span: Span) -> TokenStream {
    use proc_macro2::{Group, TokenTree};

    stream
        .into_iter()
        .map(|tt| {
            let mut tt = if let TokenTree::Group(g) = tt {
                let new_stream = set_true_span(g.stream(), new_span);
                TokenTree::Group(Group::new(g.delimiter(), new_stream))
            } else {
                tt
            };
            let current_span = tt.span();
            if Span::call_site().eq(&current_span) {
                // located_at is semver excempt
                tt.set_span(current_span.located_at(new_span));
            } else {
            }
            tt
        })
        .collect()
}

#[cfg(not(procmacro2_semver_exempt))]
/// replaces all occurences of the default span with the given one
pub fn set_true_span(stream: TokenStream, _new_span: Span) -> TokenStream {
    stream
}
