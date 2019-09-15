//! Mutator for binary operation `+`.

use std::convert::TryFrom;
use std::ops::Add;
use std::ops::Deref;

use proc_macro2::Span;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{BinOp, Expr};

use crate::comm::Mutation;
use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::TransformContext;

use crate::optimistic::AddToSub;
use crate::MutagenRuntimeConfig;

pub fn run<L: Add<R>, R>(
    mutator_id: usize,
    left: L,
    right: R,
    runtime: impl Deref<Target = MutagenRuntimeConfig>,
) -> <L as Add<R>>::Output {
    runtime.covered(mutator_id);
    if runtime.is_mutation_active(mutator_id) {
        left.may_sub(right)
    } else {
        left + right
    }
}

pub fn run_native_num<I: Add<I, Output = I>>(
    mutator_id: usize,
    left: I,
    right: I,
    runtime: impl Deref<Target = MutagenRuntimeConfig>,
) -> I {
    runtime.covered(mutator_id);
    if runtime.is_mutation_active(mutator_id) {
        left.may_sub(right)
    } else {
        left + right
    }
}

pub fn transform(
    e: Expr,
    transform_info: &SharedTransformInfo,
    context: &TransformContext,
) -> Expr {
    let e = match ExprBinopAdd::try_from(e) {
        Ok(e) => e,
        Err(e) => return e,
    };

    let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
        &context,
        "binop_add".to_owned(),
        "+".to_owned(),
        "-".to_owned(),
        e.span,
    ));

    let left = &e.left;
    let right = &e.right;

    // if the current expression is based on numbers, use the function `run_native_num` instead
    let run_fn = if context.is_num_expr() {
        quote_spanned! {e.span=> run_native_num}
    } else {
        quote_spanned! {e.span=> run}
    };

    syn::parse2(quote_spanned! {e.span=>
        ::mutagen::mutator::mutator_binop_add::#run_fn(
                #mutator_id,
                #left,
                #right,
                ::mutagen::MutagenRuntimeConfig::get_default()
            )
    })
    .expect("transformed code invalid")
}

#[derive(Clone, Debug)]
pub struct ExprBinopAdd {
    pub left: Expr,
    pub right: Expr,
    pub span: Span,
}

impl TryFrom<Expr> for ExprBinopAdd {
    type Error = Expr;
    fn try_from(expr: Expr) -> Result<Self, Expr> {
        match expr {
            Expr::Binary(expr) => match expr.op {
                BinOp::Add(t) => Ok(ExprBinopAdd {
                    left: *expr.left,
                    right: *expr.right,
                    span: t.span(),
                }),
                _ => Err(Expr::Binary(expr)),
            },
            _ => Err(expr),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn sum_inactive() {
        let result = run(1, 5, 4, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, 9);
    }
    #[test]
    fn sum_active() {
        let result = run(1, 5, 4, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 1);
    }

    #[test]
    fn str_add_inactive() {
        let result = run(
            1,
            "x".to_string(),
            "y",
            &MutagenRuntimeConfig::without_mutation(),
        );
        assert_eq!(&result, "xy");
    }
    #[test]
    #[should_panic]
    fn str_add_active() {
        run(
            1,
            "x".to_string(),
            "y",
            &MutagenRuntimeConfig::with_mutation_id(1),
        );
    }

    #[test]
    fn sum_native_inactive() {
        let result =
            run_native_num(1, 5, 4, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, 9);
    }

    #[test]
    fn sum_native_active() {
        let result =
            run_native_num(1, 5, 4, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 1);
    }
}
