//! Mutator for numeric binary operations `+`, `-`, `/`, `*`.

use std::convert::TryFrom;
use std::ops::Deref;
use std::ops::{Add, Div, Mul, Sub};

use proc_macro2::Span;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{BinOp, Expr};

use crate::comm::Mutation;
use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::TransformContext;

use crate::MutagenRuntimeConfig;

pub fn run_add<L: Add<R>, R>(
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
pub fn run_sub<L: Sub<R>, R>(
    mutator_id: usize,
    left: L,
    right: R,
    runtime: impl Deref<Target = MutagenRuntimeConfig>,
) -> <L as Sub<R>>::Output {
    runtime.covered(mutator_id);
    if runtime.is_mutation_active(mutator_id) {
        left.may_add(right)
    } else {
        left - right
    }
}
pub fn run_mul<L: Mul<R>, R>(
    mutator_id: usize,
    left: L,
    right: R,
    runtime: impl Deref<Target = MutagenRuntimeConfig>,
) -> <L as Mul<R>>::Output {
    runtime.covered(mutator_id);
    if runtime.is_mutation_active(mutator_id) {
        left.may_div(right)
    } else {
        left * right
    }
}
pub fn run_div<L: Div<R>, R>(
    mutator_id: usize,
    left: L,
    right: R,
    runtime: impl Deref<Target = MutagenRuntimeConfig>,
) -> <L as Div<R>>::Output {
    runtime.covered(mutator_id);
    if runtime.is_mutation_active(mutator_id) {
        left.may_mul(right)
    } else {
        left / right
    }
}

pub fn transform(
    e: Expr,
    transform_info: &SharedTransformInfo,
    context: &TransformContext,
) -> Expr {
    let e = match ExprBinopNum::try_from(e) {
        Ok(e) => e,
        Err(e) => return e,
    };

    let mutator_id = transform_info.add_mutations(
        MutationBinopNum::possible_mutations(e.op)
            .iter()
            .map(|m| m.to_mutation(&e, context)),
    );

    let left = &e.left;
    let right = &e.right;
    let run_fn = match e.op {
        BinopNum::Add => quote_spanned! {e.span()=> run_add},
        BinopNum::Sub => quote_spanned! {e.span()=> run_sub},
        BinopNum::Mul => quote_spanned! {e.span()=> run_mul},
        BinopNum::Div => quote_spanned! {e.span()=> run_div},
    };
    let op_token = e.op_token;
    let tmp_var = transform_info.get_next_tmp_var(op_token.span());
    syn::parse2(quote_spanned! {e.span()=>
        {
            let #tmp_var = #left;
            if false {#tmp_var #op_token #right} else {
                ::mutagen::mutator::mutator_binop_num::#run_fn(
                    #mutator_id,
                    #tmp_var,
                    #right,
                    ::mutagen::MutagenRuntimeConfig::get_default()
                )
            }
        }
    })
    .expect("transformed code invalid")
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct MutationBinopNum {
    op: BinopNum,
}

impl MutationBinopNum {
    fn possible_mutations(original_op: BinopNum) -> Vec<Self> {
        match original_op {
            BinopNum::Add => vec![MutationBinopNum { op: BinopNum::Sub }],
            BinopNum::Sub => vec![MutationBinopNum { op: BinopNum::Add }],
            BinopNum::Mul => vec![MutationBinopNum { op: BinopNum::Div }],
            BinopNum::Div => vec![MutationBinopNum { op: BinopNum::Mul }],
        }
    }

    fn to_mutation(self, original_expr: &ExprBinopNum, context: &TransformContext) -> Mutation {
        Mutation::new_spanned(
            &context,
            "binop_num".to_owned(),
            format!("{}", original_expr.op),
            format!("{}", self.op),
            original_expr.span(),
        )
    }
}

#[derive(Clone, Debug)]
struct ExprBinopNum {
    op: BinopNum,
    left: Expr,
    right: Expr,
    op_token: syn::BinOp,
}

impl TryFrom<Expr> for ExprBinopNum {
    type Error = Expr;
    fn try_from(expr: Expr) -> Result<Self, Expr> {
        match expr {
            Expr::Binary(expr) => match expr.op {
                BinOp::Add(_) => Ok(ExprBinopNum {
                    op: BinopNum::Add,
                    left: *expr.left,
                    right: *expr.right,
                    op_token: expr.op,
                }),
                BinOp::Sub(_) => Ok(ExprBinopNum {
                    op: BinopNum::Sub,
                    left: *expr.left,
                    right: *expr.right,
                    op_token: expr.op,
                }),
                BinOp::Mul(_) => Ok(ExprBinopNum {
                    op: BinopNum::Mul,
                    left: *expr.left,
                    right: *expr.right,
                    op_token: expr.op,
                }),
                BinOp::Div(_) => Ok(ExprBinopNum {
                    op: BinopNum::Div,
                    left: *expr.left,
                    right: *expr.right,
                    op_token: expr.op,
                }),
                _ => Err(Expr::Binary(expr)),
            },
            _ => Err(expr),
        }
    }
}

impl syn::spanned::Spanned for ExprBinopNum {
    fn span(&self) -> Span {
        self.op_token.span()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum BinopNum {
    Add,
    Sub,
    Mul,
    Div,
}

use std::fmt;

impl fmt::Display for BinopNum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinopNum::Add => write!(f, "+"),
            BinopNum::Sub => write!(f, "-"),
            BinopNum::Mul => write!(f, "*"),
            BinopNum::Div => write!(f, "/"),
        }
    }
}

// specification of the traits `AddToSub`, `SubToAdd`, ...
//
// These traits consist of a function `max_x` that panics if the operation `x`
// cannot be performed due to type constraints
macro_rules! binary_x_to_y {
    { $($may_ty:ident, $may_fn:ident, $t1:ident, $t2:ident, $t2_op:tt,)* } => {
        $(
            trait $may_ty<R> {
                type Output;
                fn $may_fn(self, r: R) -> Self::Output;
            }

            impl <L, R> $may_ty<R> for L where L: $t1<R> {
                type Output = <L as $t1<R>>::Output;
                default fn $may_fn(self, _r: R) -> <L as $t1<R>>::Output {
                    MutagenRuntimeConfig::get_default().optimistic_assmuption_failed();
                }
            }

            impl<L, R> $may_ty<R> for L
            where
                L: $t1<R>,
                L: $t2<R>,
                <L as $t2<R>>::Output: Into<<L as $t1<R>>::Output>,
            {
                fn $may_fn(self, r: R) -> Self::Output {
                    (self $t2_op r).into()
                }
            }
        )*

    }
}

binary_x_to_y!(
    AddToSub, may_sub, Add, Sub, -,
    SubToAdd, may_add, Sub, Add, +,
    MulToDiv, may_div, Mul, Div, /,
    DivToMul, may_mul, Div, Mul, *,
);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn sum_inactive() {
        let result = run_add(1, 5, 4, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, 9);
    }
    #[test]
    fn sum_active() {
        let result = run_add(1, 5, 4, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 1);
    }

    #[test]
    fn str_add_inactive() {
        let result = run_add(
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
        run_add(
            1,
            "x".to_string(),
            "y",
            &MutagenRuntimeConfig::with_mutation_id(1),
        );
    }
}
