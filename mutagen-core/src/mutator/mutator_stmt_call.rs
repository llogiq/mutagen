//! Mutator for removing statements that only consist of a method or function call.

use std::convert::TryFrom;
use std::ops::Deref;

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Expr, Stmt};

use crate::comm::Mutation;
use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::TransformContext;

use crate::MutagenRuntimeConfig;

pub fn should_run(mutator_id: usize, runtime: impl Deref<Target = MutagenRuntimeConfig>) -> bool {
    runtime.covered(mutator_id);
    // should run if mutation is inactive
    !runtime.is_mutation_active(mutator_id)
}

pub fn transform(
    s: Stmt,
    transform_info: &SharedTransformInfo,
    context: &TransformContext,
) -> Stmt {
    let s = match StmtCall::try_from(s) {
        Ok(s) => s,
        Err(s) => return s,
    };

    let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
        context,
        "stmt_call".to_owned(),
        context
            .original_stmt
            .to_token_stream()
            .to_string()
            .replace("\n", " "),
        "".to_owned(),
        s.span,
    ));

    let call = &s.call;

    syn::parse2(quote_spanned! {s.span=>
        if ::mutagen::mutator::mutator_stmt_call::should_run(
                #mutator_id,
                ::mutagen::MutagenRuntimeConfig::get_default()
            )
        {
            #call;
        } else {
            ::mutagen::mutator::mutator_stmt_call::stmt_call_to_none()
        }
    })
    .expect("transformed code invalid")
}

#[derive(Debug, Clone)]
struct StmtCall {
    call: TokenStream,
    span: Span,
}

impl TryFrom<Stmt> for StmtCall {
    type Error = Stmt;
    fn try_from(stmt: Stmt) -> Result<Self, Stmt> {
        match stmt {
            Stmt::Semi(Expr::MethodCall(call), _) => Ok(StmtCall {
                span: call.span(),
                call: call.into_token_stream(),
            }),
            Stmt::Semi(Expr::Call(call), _) => Ok(StmtCall {
                span: call.span(),
                call: call.into_token_stream(),
            }),
            _ => Err(stmt),
        }
    }
}

/// a trait for optimistically removing a statement containing a method- or function call.
///
/// This operation is optimistic, since the statement could have the type `!` and can be used in surprising contexts:
///
/// * `let x = {f(return y);}`
/// * `let x = {std::process::abort();}`
///
/// Above examples compile and it is not possible to remove the statements without introducing compiler errors.
pub trait StmtCallToNone {
    fn stmt_call_to_none() -> Self;
}

impl<T> StmtCallToNone for T {
    default fn stmt_call_to_none() -> Self {
        MutagenRuntimeConfig::get_default().optimistic_assumption_failed();
    }
}

impl StmtCallToNone for () {
    fn stmt_call_to_none() {}
}

pub fn stmt_call_to_none<T: StmtCallToNone>() -> T {
    <T as StmtCallToNone>::stmt_call_to_none()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn stmt_inactive() {
        let result = should_run(1, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, true);
    }
    #[test]
    fn stmt_active() {
        let result = should_run(1, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, false);
    }
}
