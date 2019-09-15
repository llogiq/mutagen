//! Mutator for comparison operations `<`, `<=`, `=>`, `>`

use std::convert::TryFrom;
use std::ops::Deref;

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use syn::spanned::Spanned;
use syn::{BinOp, Expr};

use crate::comm::Mutation;
use crate::transformer::transform_info::SharedTransformInfo;
use crate::transformer::TransformContext;

use crate::MutagenRuntimeConfig;

pub fn run<L: PartialOrd<R>, R>(
    mutator_id: usize,
    left: L,
    right: R,
    original_op: BinopCmp,
    runtime: impl Deref<Target = MutagenRuntimeConfig>,
) -> bool {
    runtime.covered(mutator_id);
    let mutations = MutationBinopCmp::possible_mutations(original_op);
    if let Some(m) = runtime.get_mutation(mutator_id, &mutations) {
        m.mutate(left, right)
    } else {
        original_op.cmp(left, right)
    }
}

pub fn transform(
    e: Expr,
    transform_info: &SharedTransformInfo,
    context: &TransformContext,
) -> Expr {
    let e = match ExprBinopCmp::try_from(e) {
        Ok(e) => e,
        Err(e) => return e,
    };

    let mutator_id = transform_info.add_mutations(
        MutationBinopCmp::possible_mutations(e.op)
            .iter()
            .map(|m| m.to_mutation(&e, context)),
    );

    let left = &e.left;
    let right = &e.right;
    let op = e.op_tokens();

    syn::parse2(quote_spanned! {e.span=>
        ::mutagen::mutator::mutator_binop_cmp::run(
                #mutator_id,
                #left,
                #right,
                #op,
                ::mutagen::MutagenRuntimeConfig::get_default()
            )
    })
    .expect("transformed code invalid")
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct MutationBinopCmp {
    op: BinopCmp,
}

impl MutationBinopCmp {
    fn possible_mutations(original_op: BinopCmp) -> Vec<Self> {
        [BinopCmp::Lt, BinopCmp::Le, BinopCmp::Ge, BinopCmp::Gt]
            .iter()
            .copied()
            .filter(|&op| op != original_op)
            .map(|op| MutationBinopCmp { op })
            .collect()
    }

    fn mutate<L: PartialOrd<R>, R>(self, left: L, right: R) -> bool {
        self.op.cmp(left, right)
    }

    fn to_mutation(self, original_op: &ExprBinopCmp, context: &TransformContext) -> Mutation {
        Mutation::new_spanned(
            &context,
            "binop_cmp".to_owned(),
            format!("{}", original_op.op),
            format!("{}", self.op),
            original_op.span,
        )
    }
}

#[derive(Clone, Debug)]
struct ExprBinopCmp {
    op: BinopCmp,
    left: Expr,
    right: Expr,
    span: Span,
}

impl TryFrom<Expr> for ExprBinopCmp {
    type Error = Expr;
    fn try_from(expr: Expr) -> Result<Self, Expr> {
        match expr {
            Expr::Binary(expr) => match expr.op {
                BinOp::Lt(t) => Ok(ExprBinopCmp {
                    op: BinopCmp::Lt,
                    left: *expr.left,
                    right: *expr.right,
                    span: t.span(),
                }),
                BinOp::Le(t) => Ok(ExprBinopCmp {
                    op: BinopCmp::Le,
                    left: *expr.left,
                    right: *expr.right,
                    span: t.span(),
                }),
                BinOp::Ge(t) => Ok(ExprBinopCmp {
                    op: BinopCmp::Ge,
                    left: *expr.left,
                    right: *expr.right,
                    span: t.span(),
                }),
                BinOp::Gt(t) => Ok(ExprBinopCmp {
                    op: BinopCmp::Gt,
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

impl ExprBinopCmp {
    fn op_tokens(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        tokens.extend(quote_spanned!(self.span=>
            ::mutagen::mutator::mutator_binop_cmp::BinopCmp::));
        tokens.extend(match self.op {
            BinopCmp::Lt => quote_spanned!(self.span=> Lt),
            BinopCmp::Le => quote_spanned!(self.span=> Le),
            BinopCmp::Ge => quote_spanned!(self.span=> Ge),
            BinopCmp::Gt => quote_spanned!(self.span=> Gt),
        });
        tokens
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BinopCmp {
    Lt,
    Le,
    Ge,
    Gt,
}

impl BinopCmp {
    fn cmp<L: PartialOrd<R>, R>(self, left: L, right: R) -> bool {
        match self {
            BinopCmp::Lt => left < right,
            BinopCmp::Le => left <= right,
            BinopCmp::Ge => left >= right,
            BinopCmp::Gt => left > right,
        }
    }
}

use std::fmt;

impl fmt::Display for BinopCmp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinopCmp::Lt => write!(f, "<"),
            BinopCmp::Le => write!(f, "<="),
            BinopCmp::Ge => write!(f, ">="),
            BinopCmp::Gt => write!(f, ">"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn possible_mutations_le() {
        assert_eq!(
            MutationBinopCmp::possible_mutations(BinopCmp::Le),
            vec![
                MutationBinopCmp { op: BinopCmp::Lt },
                MutationBinopCmp { op: BinopCmp::Ge },
                MutationBinopCmp { op: BinopCmp::Gt },
            ]
        )
    }

    #[test]
    fn possible_mutations_gt() {
        assert_eq!(
            MutationBinopCmp::possible_mutations(BinopCmp::Gt),
            vec![
                MutationBinopCmp { op: BinopCmp::Lt },
                MutationBinopCmp { op: BinopCmp::Le },
                MutationBinopCmp { op: BinopCmp::Ge },
            ]
        )
    }

    #[test]
    fn cmp_lt() {
        assert_eq!(BinopCmp::Lt.cmp(1, 2), true);
        assert_eq!(BinopCmp::Lt.cmp(3, 3), false);
        assert_eq!(BinopCmp::Lt.cmp(5, 4), false);
    }

    #[test]
    fn cmp_le() {
        assert_eq!(BinopCmp::Le.cmp(1, 2), true);
        assert_eq!(BinopCmp::Le.cmp(3, 3), true);
        assert_eq!(BinopCmp::Le.cmp(5, 4), false);
    }

    #[test]
    fn cmp_ge() {
        assert_eq!(BinopCmp::Ge.cmp(1, 2), false);
        assert_eq!(BinopCmp::Ge.cmp(3, 3), true);
        assert_eq!(BinopCmp::Ge.cmp(5, 4), true);
    }

    #[test]
    fn cmp_gt() {
        assert_eq!(BinopCmp::Gt.cmp(1, 2), false);
        assert_eq!(BinopCmp::Gt.cmp(3, 3), false);
        assert_eq!(BinopCmp::Gt.cmp(5, 4), true);
    }

    use crate::MutagenRuntimeConfig;

    #[test]
    fn mutator_cmp_gt_inactive() {
        assert_eq!(
            run(
                1,
                1,
                2,
                BinopCmp::Gt,
                &MutagenRuntimeConfig::without_mutation()
            ),
            false
        );
        assert_eq!(
            run(
                1,
                5,
                4,
                BinopCmp::Gt,
                &MutagenRuntimeConfig::without_mutation()
            ),
            true
        );
    }
    #[test]
    fn mutator_cmp_gt_active1() {
        assert_eq!(
            run(
                1,
                1,
                2,
                BinopCmp::Gt,
                &MutagenRuntimeConfig::with_mutation_id(1)
            ),
            true
        );
        assert_eq!(
            run(
                1,
                3,
                3,
                BinopCmp::Gt,
                &MutagenRuntimeConfig::with_mutation_id(1)
            ),
            false
        );
    }
}
