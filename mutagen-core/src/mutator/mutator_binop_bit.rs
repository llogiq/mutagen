//! Mutator for binary bit-operations `|`, `&`, `^`.

use std::convert::TryFrom;
use std::ops::Deref;
use std::ops::{BitAnd, BitOr, BitXor};

use proc_macro2::Span;
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{BinOp, Expr};

use crate::comm::Mutation;
use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::TransformContext;

use crate::MutagenRuntimeConfig;

pub fn run_and<L: BitAnd<R>, R>(
    mutator_id: usize,
    left: L,
    right: R,
    runtime: impl Deref<Target = MutagenRuntimeConfig>,
) -> <L as BitAnd<R>>::Output {
    runtime.covered(mutator_id);
    let mutations = MutationBinopBit::possible_mutations(BinopBit::And);
    if let Some(m) = runtime.get_mutation_for_mutator(mutator_id, &mutations) {
        match m.op {
            BinopBit::Or => left.and_may_or(right),
            BinopBit::Xor => left.and_may_xor(right),
            _ => unreachable!(),
        }
    } else {
        left & right
    }
}
pub fn run_or<L: BitOr<R>, R>(
    mutator_id: usize,
    left: L,
    right: R,
    runtime: impl Deref<Target = MutagenRuntimeConfig>,
) -> <L as BitOr<R>>::Output {
    runtime.covered(mutator_id);
    let mutations = MutationBinopBit::possible_mutations(BinopBit::Or);
    if let Some(m) = runtime.get_mutation_for_mutator(mutator_id, &mutations) {
        match m.op {
            BinopBit::And => left.or_may_and(right),
            BinopBit::Xor => left.or_may_xor(right),
            _ => unreachable!(),
        }
    } else {
        left | right
    }
}
pub fn run_xor<L: BitXor<R>, R>(
    mutator_id: usize,
    left: L,
    right: R,
    runtime: impl Deref<Target = MutagenRuntimeConfig>,
) -> <L as BitXor<R>>::Output {
    runtime.covered(mutator_id);
    let mutations = MutationBinopBit::possible_mutations(BinopBit::Xor);
    if let Some(m) = runtime.get_mutation_for_mutator(mutator_id, &mutations) {
        match m.op {
            BinopBit::And => left.xor_may_and(right),
            BinopBit::Or => left.xor_may_or(right),
            _ => unreachable!(),
        }
    } else {
        left ^ right
    }
}

pub fn transform(
    e: Expr,
    transform_info: &SharedTransformInfo,
    context: &TransformContext,
) -> Expr {
    let e = match ExprBinopBit::try_from(e) {
        Ok(e) => e,
        Err(e) => return e,
    };

    let mutator_id = transform_info.add_mutations(
        MutationBinopBit::possible_mutations(e.op)
            .iter()
            .map(|m| m.to_mutation(&e, context)),
    );

    let left = &e.left;
    let right = &e.right;

    let run_fn = match e.op {
        BinopBit::And => quote_spanned! {e.span()=> run_and},
        BinopBit::Or => quote_spanned! {e.span()=> run_or},
        BinopBit::Xor => quote_spanned! {e.span()=> run_xor},
    };
    let op_token = e.op_token;
    let tmp_var = transform_info.get_next_tmp_var(op_token.span());
    syn::parse2(quote_spanned! {e.span()=>
        {
            let #tmp_var = #left;
            if false {#tmp_var #op_token #right} else {
                ::mutagen::mutator::mutator_binop_bit::#run_fn(
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
struct MutationBinopBit {
    op: BinopBit,
}

impl MutationBinopBit {
    fn possible_mutations(original_op: BinopBit) -> Vec<Self> {
        [BinopBit::And, BinopBit::Or, BinopBit::Xor]
            .iter()
            .copied()
            .filter(|&op| op != original_op)
            .map(|op| MutationBinopBit { op })
            .collect()
    }

    fn to_mutation(self, original_expr: &ExprBinopBit, context: &TransformContext) -> Mutation {
        Mutation::new_spanned(
            context,
            "binop_bit".to_owned(),
            format!("{}", original_expr.op),
            format!("{}", self.op),
            original_expr.span(),
        )
    }
}

#[derive(Clone, Debug)]
struct ExprBinopBit {
    op: BinopBit,
    left: Expr,
    right: Expr,
    op_token: syn::BinOp,
}

impl TryFrom<Expr> for ExprBinopBit {
    type Error = Expr;
    fn try_from(expr: Expr) -> Result<Self, Expr> {
        match expr {
            Expr::Binary(expr) => match expr.op {
                BinOp::BitAnd(_) => Ok(ExprBinopBit {
                    op: BinopBit::And,
                    left: *expr.left,
                    right: *expr.right,
                    op_token: expr.op,
                }),
                BinOp::BitOr(_) => Ok(ExprBinopBit {
                    op: BinopBit::Or,
                    left: *expr.left,
                    right: *expr.right,
                    op_token: expr.op,
                }),
                BinOp::BitXor(_) => Ok(ExprBinopBit {
                    op: BinopBit::Xor,
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

impl syn::spanned::Spanned for ExprBinopBit {
    fn span(&self) -> Span {
        self.op_token.span()
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum BinopBit {
    And,
    Or,
    Xor,
}

use std::fmt;

impl fmt::Display for BinopBit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinopBit::And => write!(f, "&"),
            BinopBit::Or => write!(f, "|"),
            BinopBit::Xor => write!(f, "^"),
        }
    }
}

// specification of the traits `AndToOr`, `OrToAnd`, ...
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
                    MutagenRuntimeConfig::get_default().optimistic_assumption_failed();
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
    AndToOr, and_may_or, BitAnd, BitOr, |,
    AndToXor, and_may_xor, BitAnd, BitXor, ^,
    OrToAnd, or_may_and, BitOr, BitAnd, &,
    OrToXor, or_may_xor, BitOr, BitXor, ^,
    XorToAnd, xor_may_and, BitXor, BitAnd, &,
    XorToOr, xor_may_or, BitXor, BitOr, |,
);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn and_inactive() {
        let result = run_and(1, 0b11, 0b10, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, 0b10);
    }
    #[test]
    fn and_active1() {
        let result = run_and(1, 0b11, 0b10, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 0b11);
    }
    #[test]
    fn and_active2() {
        let result = run_and(1, 0b11, 0b10, &MutagenRuntimeConfig::with_mutation_id(2));
        assert_eq!(result, 0b01);
    }

    #[test]
    fn or_inactive() {
        let result = run_or(1, 0b11, 0b10, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, 0b11);
    }
    #[test]
    fn or_active1() {
        let result = run_or(1, 0b11, 0b10, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 0b10);
    }
    #[test]
    fn or_active2() {
        let result = run_or(1, 0b11, 0b10, &MutagenRuntimeConfig::with_mutation_id(2));
        assert_eq!(result, 0b01);
    }

    #[test]
    fn xor_inactive() {
        let result = run_xor(1, 0b11, 0b10, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, 0b01);
    }
    #[test]
    fn xor_active1() {
        let result = run_xor(1, 0b11, 0b10, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, 0b10);
    }
    #[test]
    fn xor_active2() {
        let result = run_xor(1, 0b11, 0b10, &MutagenRuntimeConfig::with_mutation_id(2));
        assert_eq!(result, 0b11);
    }
}
